//! `parinn mcp` (ADR-2610102211): a stdio MCP server of ParleyInn's own,
//! one per harness session, reading and writing one JSON-RPC message
//! per line through `serde_json`. It speaks `initialize`, `ping`,
//! `tools/list` and `tools/call`, and serves the recall tools of §9.2
//! ([`tools`]): `event_search`, `event_expand`, `event_get`,
//! `landmark_list`, `pin_list` and `stat_list`, each inside the
//! envelope (§9.3), and `mcp_server_get`, which is no recall tool.
//!
//! Recall is pull-only: the server answers what the agent asks and
//! writes nothing to it unasked (I2). A call is served in the recall
//! scope it names, `run` unless it says more ([`scope`], RCL-05). Until
//! the session binding lands the server serves no run, so every event
//! lies outside the current run: a call takes only the explicit scope
//! `room`, for the personal room alone, and shows its run as `other`. A
//! call refused for its scope, or for a gap in the range it asked for,
//! is noted for the caller to count and audit (RCL-04, I6).
//!
//! An event is served only once it checks against the record: its
//! content opens its commitment, its header hashes to its stored hash,
//! and its `prev_hash` names its predecessor. Its origin and its trust
//! level derive from the record and the node's key set
//! (`administration::origin`, PRV-13) and the default trust policy
//! (`administration::trust_policy`, PRV-02), never from a stored value.
//! A refusal is fixed text, naming at most an address it parsed and
//! renders anew; it never repeats what the call passed (RCL-04).
//!
//! Every call that returns an envelope is recorded as a recall event
//! ([`recording`], RCL-07).
//!
//! Once its harness session's session binding is held, each tool call
//! is served under the run its tool-use id names through what the
//! server's ingest read, or refused ([`tool_call`], PRV-14).

pub mod cursor;
mod event_search;
pub mod expand;
mod item;
pub mod recording;
pub mod scope;
mod stats;
pub mod tool_call;
mod tools;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use administration::origin::{KeySet, Origin as Derived};
use administration::quarantine::Quarantine;
use administration::trust_policy::{self, TrustLevel as Level};
use envelope::{Envelope, Origin, TrustLevel};
use key::signing::PublicKey;
use record::event::{EventKind, Provenance};
use record::id::{RoomId, WriterId};
use serde_json::{Value, json};
use store::Store;

pub use recording::{Note, RecallWriter};

/// The MCP protocol versions the server speaks, newest first.
pub const PROTOCOL_VERSIONS: [&str; 3] = ["2025-11-25", "2025-06-18", "2025-03-26"];

/// Why `event_get` refuses the default scope.
pub const OUTSIDE_RUN: &str = "the address lies outside the current run: this server serves no run yet, so pass scope \"room\"";

/// The JSON-RPC error codes the server answers with.
const PARSE_ERROR: i64 = -32700;
const INTERNAL_ERROR: i64 = -32603;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;

/// The refusals the recall tools answer with, each fixed text.
pub(crate) const BAD_ADDRESS: &str = "the address is neither a writer label or id with a seq, such as A1:7 or <writer id>:<seq>, nor parinn:room/<room id>/w/<writer id>/<seq>";
const NO_TOOL: &str = "no such tool: this server offers event_search, event_expand, event_get, landmark_list, pin_list, stat_list and mcp_server_get";
const CANCELLED: &str = "the search ran past its deadline of 2 s and was cancelled";
/// The refusal for a record that cannot be read at an address.
pub(crate) const UNREADABLE: &str = "the record cannot be read at this address: run parinn verify";
const NO_INDEX: &str = "the search index cannot be read: run parinn verify";

/// What a recall tool served: the envelope, and the scope it served in.
#[derive(Debug)]
pub(crate) struct Served {
    pub(crate) envelope: Envelope,
    pub(crate) scope: Option<scope::RecallScope>,
}

/// The server over one store, for one node.
pub struct Server {
    store: Store,
    version: String,
    device_key: PublicKey,
    personal_room: RoomId,
    /// What finds the run of each tool call: the server's live ingest.
    runs: Option<Box<dyn tool_call::Runs>>,
    /// What finding each call's run came to since `take_found`.
    found: RefCell<Vec<tool_call::Found>>,
    /// How long a search may run before it is cancelled (SEC-04).
    deadline: Duration,
    /// How many searches the deadline cancelled since `take_cancelled`.
    cancelled: Cell<u64>,
    /// How many searches the deadline cancelled since this server started.
    timed_out: Cell<u64>,
    /// What records each recall, when this server records (RCL-07).
    recall_writer: Option<RecallWriter>,
    /// The device seat's writer, which assigns the writer labels a range
    /// names its writer by (RCL-08), when this server may assign them.
    labeller: Option<WriterId>,
    /// What the calls since `take_notes` came to, for the caller to log.
    notes: RefCell<Vec<Note>>,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("store", &self.store)
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}

impl Server {
    /// A server answering from `store` for the node whose device key is
    /// `device_key` and whose personal room is `personal_room`, naming
    /// itself as parinn `version`.
    #[must_use]
    pub fn new(store: Store, version: &str, device_key: PublicKey, personal_room: RoomId) -> Self {
        Self {
            store,
            version: version.to_owned(),
            device_key,
            personal_room,
            runs: None,
            found: RefCell::new(Vec::new()),
            deadline: event_search::DEADLINE,
            cancelled: Cell::new(0),
            timed_out: Cell::new(0),
            recall_writer: None,
            labeller: None,
            notes: RefCell::new(Vec::new()),
        }
    }

    /// This server, finding each tool call's run through `runs`
    /// (PRV-14).
    #[must_use]
    pub fn with_runs(mut self, runs: Box<dyn tool_call::Runs>) -> Self {
        self.runs = Some(runs);
        self
    }

    /// This server, recording each recall through `recall_writer` (RCL-07),
    /// and assigning writer labels in the same writer.
    #[must_use]
    pub fn with_recall_writer(mut self, recall_writer: RecallWriter) -> Self {
        self.labeller = Some(recall_writer.writer);
        self.recall_writer = Some(recall_writer);
        self
    }

    /// This server, assigning writer labels in `device`, a device seat's
    /// writer, without recording any recall (RCL-08).
    #[must_use]
    pub fn with_labeller(mut self, device: WriterId) -> Self {
        self.labeller = Some(device);
        self
    }

    /// What finding each call's run came to since this was last asked,
    /// for the caller to log, and to count and audit each refusal
    /// (PRV-14, I6).
    pub fn take_found(&self) -> Vec<tool_call::Found> {
        self.found.take()
    }

    /// This server, its searches cancelled after `deadline`.
    #[must_use]
    pub fn with_query_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    /// How many searches the deadline cancelled since this was last
    /// asked, for the caller to count and audit (SEC-04, I6).
    pub fn take_cancelled(&self) -> u64 {
        self.cancelled.take()
    }

    /// What the calls since this was last asked came to that the caller
    /// logs: a call cut at its cap, one whose scope extends past its
    /// run, one that went unrecorded (RCL-03, RCL-05, RCL-07, I6).
    pub fn take_notes(&self) -> Vec<Note> {
        self.notes.take()
    }

    /// The name of the run of the call whose params are `params`: none
    /// while no session binding is held, before which a call is served
    /// under no run.
    ///
    /// # Errors
    ///
    /// The refusal of a call whose run cannot be found.
    fn run_of(&self, params: &Value) -> Result<Option<String>, &'static str> {
        let Some(runs) = self.runs.as_deref().filter(|r| r.serving()) else {
            return Ok(None);
        };
        let found = tool_call::find(runs, params);
        let run = found.run.clone();
        self.found.borrow_mut().push(found);

        run.map(Some).map_err(|cause| cause.refusal())
    }

    /// The reply to one message, as one line of JSON; none for a
    /// notification or a response.
    pub fn handle(&mut self, line: &str) -> Option<String> {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            return Some(error(&Value::Null, PARSE_ERROR, "parse error"));
        };
        let id = message.get("id");
        let reply = match (message["method"].as_str(), id) {
            // A notification, or a response to a request the server never sends.
            (Some(_), None) | (None, Some(_)) => return None,
            (None, None) => error(&Value::Null, INVALID_REQUEST, "invalid request"),
            (Some(method), Some(id)) => match method {
                "initialize" => result(id, self.initialize(&message["params"])),
                "ping" => result(id, json!({})),
                "tools/list" => result(id, tools::list()),
                "tools/call" => result(id, self.call(&message["params"])),
                _ => error(id, METHOD_NOT_FOUND, "method not found"),
            },
        };

        Some(reply)
    }

    fn initialize(&self, params: &Value) -> Value {
        let asked = params["protocolVersion"].as_str();
        let version = PROTOCOL_VERSIONS
            .into_iter()
            .find(|v| Some(*v) == asked)
            .unwrap_or(PROTOCOL_VERSIONS[0]);

        json!({
            "protocolVersion": version,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "parinn", "version": self.version},
        })
    }

    fn call(&mut self, params: &Value) -> Value {
        // The server's own state is told whether or not a run is found: it
        // is what says why none is.
        if params["name"].as_str() == Some("mcp_server_get") {
            return text_result(&self.mcp_server_get(), false);
        }
        let run = match self.run_of(params) {
            Ok(run) => run,
            Err(refusal) => return text_result(refusal, true),
        };
        let Some(tool) = tools::recall_tool(params["name"].as_str()) else {
            return text_result(NO_TOOL, true);
        };
        let arguments = &params["arguments"];
        match self.serve(tool, arguments, run.as_deref()) {
            Ok(served) => {
                self.record(tool, arguments, run.as_deref(), &served);
                text_result(&served.envelope.render(), false)
            }
            Err(why) => text_result(&why, true),
        }
    }

    /// The recall tool `tool` answering `arguments` for the run `run`.
    fn serve(
        &mut self,
        tool: &'static str,
        arguments: &Value,
        run: Option<&str>,
    ) -> Result<Served, String> {
        match tool {
            "event_get" => self.event_get(arguments, run),
            "event_expand" => self.event_expand(arguments, run),
            "event_search" => self.event_search(arguments, run),
            "landmark_list" => self.landmark_list(arguments, run),
            "pin_list" => self.pin_list(arguments, run),
            _ => self.stat_list(arguments, run),
        }
    }

    /// `mcp_server_get`: this server's own state as closed words and
    /// counts, no recalled content: whether it holds a session binding
    /// (REC-13), whether the search index reads, how many searches the
    /// deadline timed out since it started, and its version.
    fn mcp_server_get(&self) -> String {
        let held = self.runs.as_deref().is_some_and(|r| r.serving());
        let readable = self.store.search_index_unindexed_writers().is_ok();

        json!({
            "session_binding": if held { "held" } else { "not_held" },
            "search_index": if readable { "readable" } else { "unreadable" },
            "searches_timed_out": self.timed_out.get(),
            "version": self.version,
        })
        .to_string()
    }

    /// What a call for the scope `run` is told when no run is served.
    fn no_run(&self, tool: &str) -> &'static str {
        if tool == "event_search" {
            event_search::NEEDS_RUN
        } else {
            OUTSIDE_RUN
        }
    }

    /// The fixed text a refused scope is answered with. A scope the
    /// server does not serve is noted for the caller to count and audit
    /// (RCL-04, I6).
    fn refusal(&self, tool: &'static str, refused: scope::Refused) -> String {
        match refused {
            scope::Refused::Text(text) => {
                self.notes.borrow_mut().push(Note::ScopeRefused(tool));
                text.to_owned()
            }
            scope::Refused::Unreadable => NO_INDEX.to_owned(),
        }
    }

    /// Notes a call whose scope extends past the calling run, for the
    /// caller to log (RCL-05).
    fn note_extended(&self, tool: &'static str, scope: &scope::Scope) {
        if scope.extends() {
            self.notes.borrow_mut().push(Note::Extended {
                tool,
                scope: scope.name.as_str(),
            });
        }
    }

    /// `event_search`: the events whose text matches, best first, each a
    /// hit with its preview and score, in the envelope (RCL-02).
    fn event_search(&mut self, arguments: &Value, run: Option<&str>) -> Result<Served, String> {
        let params = event_search::parse(arguments).map_err(str::to_owned)?;
        let compiled = search::compile(&params.request()).map_err(|e| e.to_string())?;
        let scope = self.scope_of(arguments, run, "event_search")?;
        self.note_extended("event_search", &scope);
        let Some((rooms, writers)) = scope.limits(params.writer) else {
            return Ok(Served {
                envelope: Envelope::default(),
                scope: Some(scope.name),
            });
        };
        let keys = KeySet::load(&self.store, &self.device_key).map_err(|_| NO_INDEX.to_owned())?;
        let quarantine =
            Quarantine::load(&self.store, self.personal_room).map_err(|_| NO_INDEX.to_owned())?;
        let seats = RefCell::new(BTreeMap::new());
        // The trust filter keeps matches before the ranking cuts them; an
        // event whose trust cannot be derived is left out, never kept as
        // if it were what the call asked for. So is a quarantined event,
        // and one whose quarantine cannot be read (RCL-06).
        let keep = |address: record::id::Address| {
            let trusted = params.trust.is_none_or(|wanted| {
                self.trust_at(address, &keys, &mut seats.borrow_mut())
                    .is_ok_and(|level| wanted.as_str() == level.as_str())
            });

            trusted && quarantine.holds(address).is_ok_and(|held| !held)
        };
        let search = params.search(&compiled, &rooms, &writers, self.deadline);
        let found = match self.store.search_where(&search, &keep) {
            Ok(found) => found,
            Err(store::StoreError::Cancelled) => {
                self.cancelled.set(self.cancelled.get() + 1);
                self.timed_out.set(self.timed_out.get() + 1);
                return Err(CANCELLED.to_owned());
            }
            Err(_) => return Err(NO_INDEX.to_owned()),
        };
        let mut items = Vec::new();
        for hit in &found {
            items.push(self.hit_item(hit, &keys, &scope, &mut seats.borrow_mut())?);
        }
        items.truncate(params.k);

        Ok(Served {
            envelope: Envelope {
                items,
                ..Envelope::default()
            },
            scope: Some(scope.name),
        })
    }

    /// The scope a call for `tool` names.
    fn scope_of(
        &self,
        arguments: &Value,
        run: Option<&str>,
        tool: &'static str,
    ) -> Result<scope::Scope, String> {
        let call = scope::Call {
            store: &self.store,
            personal_room: self.personal_room,
            run,
        };

        scope::resolve(&call, arguments, None, self.no_run(tool)).map_err(|r| self.refusal(tool, r))
    }
}

/// Where an event came from and how far to trust it, as its writer's
/// seat says.
#[derive(Debug, Clone)]
pub(crate) struct Seated {
    pub(crate) author: String,
    pub(crate) origin: Origin,
    pub(crate) device: bool,
    pub(crate) room: RoomId,
}

/// The seats a call has looked up so far, by writer.
pub(crate) type Seats = BTreeMap<WriterId, Seated>;

/// The origin the envelope shows for a derived one.
fn shown_origin(origin: Derived) -> Origin {
    match origin {
        Derived::Witnessed => Origin::Witnessed,
        Derived::Ingested => Origin::Ingested,
        Derived::Bundle => Origin::Bundle,
        Derived::Peer => Origin::Peer,
    }
}

/// The derived origin an envelope's one stands for.
fn derived_origin(origin: Origin) -> Derived {
    match origin {
        Origin::Witnessed => Derived::Witnessed,
        Origin::Ingested => Derived::Ingested,
        Origin::Bundle => Derived::Bundle,
        Origin::Peer => Derived::Peer,
    }
}

/// The trust level the default trust policy gives an event (PRV-02). The
/// deployment mode, the prompt match and the attended mark matter only
/// while no run leaves R3 and R8 open, which no run can show before
/// OQ-40, so a `user` event is never trusted here.
fn trust_of(
    provenance: &Provenance,
    kind: EventKind,
    origin: Origin,
    device_seat: bool,
) -> TrustLevel {
    let level = trust_policy::trust_level(&trust_policy::Input {
        provenance,
        kind,
        origin: derived_origin(origin),
        device_seat,
        launcher_signed: false,
        interactive: false,
        prompt_matched: false,
        attended: false,
    });
    match level {
        Level::Trusted => TrustLevel::Trusted,
        Level::Untrusted => TrustLevel::Untrusted,
    }
}

/// A tool call's result: `text` as the one content block.
fn text_result(text: &str, is_error: bool) -> Value {
    json!({
        "content": [{"type": "text", "text": text}],
        "isError": is_error,
    })
}

fn result(id: &Value, result: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string()
}

fn error(id: &Value, code: i64, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}

/// Answers each line `input` delivers on `out` through `handle`, until
/// the input closes. Each wait for a line ends after `idle` and starts
/// again, so no wait is without a deadline (ENG-03). A panic while
/// handling a line stops here: a request is answered with an internal
/// error under its id, `panicked` is told so the caller can audit it,
/// and serving goes on (ENG-04).
///
/// # Errors
///
/// Fails when reading the input or writing a reply fails.
pub fn serve(
    handle: &mut dyn FnMut(&str) -> Option<String>,
    input: &Receiver<io::Result<String>>,
    out: &mut dyn Write,
    idle: Duration,
    panicked: &mut dyn FnMut(),
) -> io::Result<()> {
    loop {
        match input.recv_timeout(idle) {
            Ok(Err(e)) if e.kind() == io::ErrorKind::InvalidData => {
                // A line that is not UTF-8 is a message that does not
                // parse; the next line may.
                writeln!(out, "{}", error(&Value::Null, PARSE_ERROR, "parse error"))?;
                out.flush()?;
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::FileTooLarge => {
                // A request past the input's line limit (SEC-05) was
                // never held whole; the next line may be one.
                let reply = error(&Value::Null, INVALID_REQUEST, "request exceeds 1 MiB");
                writeln!(out, "{reply}")?;
                out.flush()?;
            }
            Ok(line) => {
                let line = line?;
                let reply =
                    panic::catch_unwind(AssertUnwindSafe(|| handle(&line))).unwrap_or_else(|_| {
                        // The panic stops at the entry point, and the
                        // client hears of it (ENG-04).
                        panicked();
                        panic_reply(&line)
                    });
                if let Some(reply) = reply {
                    writeln!(out, "{reply}")?;
                    out.flush()?;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

/// The reply to `line` once handling it panicked: an internal error
/// with the request's id, so the client can match it to its request;
/// none for a notification, which takes no reply; a null id for a line
/// that names no method.
fn panic_reply(line: &str) -> Option<String> {
    let id = match serde_json::from_str::<Value>(line) {
        Ok(message) if message.get("method").is_some() => message.get("id")?.clone(),
        _ => Value::Null,
    };

    Some(error(&id, INTERNAL_ERROR, "internal error"))
}

#[cfg(test)]
mod tests;
