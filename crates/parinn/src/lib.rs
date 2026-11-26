//! The ParleyInn core executable, specified in docs/srs: the hook
//! handlers, the MCP server and the CLI so far.
//!
//! It runs `version`, `ingest`, `mcp`, `verify`, `hook`,
//! `status`, `doctor`, `purge`, `quarantine`, and the principal acts
//! `counter ack`, `configuration accept` and `risk-acceptance`. Every other
//! command in the SRS's §9.5 is a pending scenario under features/, and
//! lands with the plan that makes its scenario pass.

// A lint the core's code would quiet says so with an expectation and a
// reason, which fails once nothing trips it; a bare allow does not
// (SEC-01, ENG-16).
#![deny(clippy::allow_attributes)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use administration::act as act_record;
use administration::home::Home;
use administration::node::Node;
use administration::node_identity::IdentitySource;
use administration::verify::{Report, Shape, VerifyError, verify};
use allowed_root::{Access, AllowedRoots};
use audit::AuditError;
use audit::counters::{Counts, counts};
use audit::entry::{Counter, Entry, Kind, Occurrence};
use audit::log::Log;
use configuration::source::OsUser;
use configuration::{Loaded, SettingsLayer};
use hook::Failure;
use key::key_store::KeyStoreKind;
use key::random::OsEntropy;
use redaction::Redactor;
use serde_json::{Value, json};
use store::Store;

/// The command did what it was asked.
pub const EXIT_OK: u8 = 0;
/// The command failed.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;
/// The record failed an integrity check.
pub const EXIT_INTEGRITY: u8 = 3;

/// How long `parinn mcp` waits for a line before it waits again.
const IDLE: Duration = Duration::from_secs(1);

/// The longest line the process's input may carry, the hook input's
/// limit (SEC-05); the MCP server's requests are held to it too.
pub const INPUT_LINE_LIMIT: usize = hook::HOOK_INPUT_LIMIT;

/// The longest `parinn hook` waits for its whole hook input, however its
/// lines arrive; its hook budget ends the wait sooner (NFR-02).
const HOOK_INPUT_WAIT: Duration = Duration::from_millis(500);

/// What a build stamps into the executable. The release build sets
/// `PARINN_VERSION` and `PARINN_COMMIT` (ENG-01); a local build reports
/// version "dev" and no commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Build<'a> {
    pub version: &'a str,
    pub commit: Option<&'a str>,
}

/// The version a build reports when the release pipeline stamps none.
const BUILD_DEV: &str = "dev";

/// This executable's build.
pub const BUILD: Build<'static> = Build {
    version: match option_env!("PARINN_VERSION") {
        Some(v) => v,
        None => BUILD_DEV,
    },
    commit: option_env!("PARINN_COMMIT"),
};

const USAGE: &str = "usage: parinn <command> [flags]

commands:
  configuration accept [--json]          accept the configuration (widening)
  counter ack [--json]                   acknowledge the counters (neutral)
  doctor [--json]                        fail while a counter rose since the last acknowledgement
  event search WORDS [--room ID] [--k N] find events by their text, as event_search does
  event expand RANGE [--room ID]         read an address range, as event_expand does; --cursor C continues one
  hook <hook>                            run the hook handler on the hook input on stdin
  ingest --all | --path P [--json]       record every transcript under the roots, or the one at P
  licenses                               print the third-party license texts built in
  mcp                                    serve the MCP tools over stdio
  purge --range RANGE [--reason TEXT]    erase the content of an address range (widening)
  quarantine add SELECTOR [--reason T]   hold events out of recall (cut)
  quarantine release SELECTOR            take a quarantine back (widening)
  quarantine list [--json]               show the quarantines in force
  repository show [DIR] [--json]         show the repository identity of DIR, the working directory by default
  repository bind [DIR] [--commit ID]    bind or rebind the repository identity (widening)
  rebuild [--json]                       derive the origins and the search index again from the record
  risk-acceptance show [--json]          show the residual risks open on this node
  risk-acceptance record [--json]        accept the open residual risks (widening)
  risk-acceptance withdraw [--json]      withdraw the risk acceptance (cut)
  status [--json]                        show the home, the store, the settings and the counters
  verify [--json]                        check the record and the audit log
  version [--json]                       print the build version
  help                                   print this message
";

/// What an invocation runs in: the build, the home variables and the
/// process's input, each injected so a test can stand in for them.
pub struct Context<'a> {
    pub build: &'a Build<'a>,
    /// `PARINN_HOME`, as the process found it.
    pub parinn_home: Option<OsString>,
    /// `HOME`, as the process found it.
    pub home: Option<OsString>,
    /// `PARINN_HOME_ID`, the home id the environment supplies (SEC-03).
    pub home_id: Option<OsString>,
    /// Whether the hook handler panics: a development build's seam for
    /// the `@ENG-04` scenario, never set in a release build.
    pub hook_panics: bool,
    /// Whether the hook handler hangs, as one the harness kills before
    /// it finishes: a development build's seam for the `@NFR-02`
    /// scenario, never set in a release build.
    pub hook_hangs: bool,
    /// Whether the process's standard input and output are both a
    /// terminal, which a verb that writes a principal act needs
    /// (OWN-12): a person's are, an agent's tool call's are not.
    pub terminal: bool,
    /// The working directory, whose repository's configuration is a
    /// settings layer (SEC-11).
    pub working_dir: Option<PathBuf>,
    /// Where managed policy is read from (ADM-04).
    pub managed_policy: PathBuf,
    /// Where the node identity is read from instead of this OS's source
    /// (REC-24): a development build's seam for the tests, never set in
    /// a release build.
    pub node_identity: Option<PathBuf>,
    /// The running OS user, whom managed policy's files must not let
    /// write (ADM-04).
    pub os_user: OsUser,
    /// Starts reading the process's input, one line at a time.
    pub input: &'a dyn Fn() -> Receiver<io::Result<String>>,
    /// The wall clock, in seconds since the Unix epoch, which only
    /// audit entries, ingest's holds, start markers and session
    /// bindings read.
    pub clock: &'a dyn Fn() -> u64,
    /// Whether the harness's attended mark is in the process's
    /// environment (ASM-22): only a hook handler reads it (PRV-04).
    pub attended: bool,
    /// A hook budget in place of each hook's own: a development build's
    /// seam for the tests, never set in a release build.
    pub hook_budget: Option<Duration>,
    /// Reads a hook handler's process ancestry (PRV-08), and the MCP
    /// server's own harness process (REC-13): this process's own, or
    /// what a test stands in for it.
    pub ancestry: fn() -> Option<hook::ancestry::Ancestry>,
    /// How many lines of a transcript the MCP server's live ingest reads
    /// at most: a development build's seam for the tests, never set in a
    /// release build.
    pub live_limit: Option<usize>,
    /// How long a search may run before it is cancelled, in place of
    /// the default 2 s (SEC-04): a development build's seam for the
    /// tests, never set in a release build.
    pub query_deadline: Option<Duration>,
}

impl Context<'_> {
    /// The home, checked before anything reads or writes in it: it and
    /// everything in it owner-only (SEC-02), and its home id the one the
    /// environment supplies (SEC-03).
    fn home(&self) -> Result<Home, administration::AdminError> {
        let mut home = Home::locate(self.parinn_home.as_deref(), self.home.as_deref())?;
        if let Some(path) = &self.node_identity {
            home = home.with_identity(IdentitySource::Files(vec![path.clone()]));
        }
        home.check()?;
        home.check_home_id(self.home_id.as_deref())?;

        Ok(home)
    }

    /// Appends `occurrence` to the checked home's audit log, making the
    /// home first where it does not exist yet.
    fn audit(&self, occurrence: &Occurrence) -> Result<Entry, Box<dyn std::error::Error>> {
        let home = self.home()?;
        home.create()?;

        Ok(Log::new(home.audit()).append(occurrence, (self.clock)())?)
    }
}

/// Where a run's managed policy is, beside the principal's
/// configuration in its home: what the structured log reads the
/// redaction patterns from. A repository's never apply (SEC-08).
#[derive(Debug, Clone)]
pub struct LogSettings {
    pub managed: PathBuf,
    pub os_user: configuration::source::OsUser,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            managed: PathBuf::new(),
            os_user: configuration::source::OsUser::Unchecked,
        }
    }
}

/// The structured log (ENG-05, OPS-05) for the home the environment
/// names: written to its log file once the home exists and passes its
/// checks (SEC-02, SEC-03), and nowhere before; each field redacted by
/// the maintained redaction rule set and the redaction patterns the
/// principal's configuration and managed policy name (SEC-08), compiled
/// at the first event, so a run that logs nothing starts no slower.
/// While the principal's configuration fails, its patterns are unknown,
/// so no line is written; each is counted as dropped.
#[must_use]
pub fn structured_log(
    parinn_home: Option<OsString>,
    home: Option<OsString>,
    home_id: Option<OsString>,
    settings: LogSettings,
    clock: fn() -> u64,
) -> structured_log::StructuredLog {
    let (compile_home, compile_os_home) = (parinn_home.clone(), home.clone());
    // With no home there are no settings layers to read and nowhere to
    // write: the rule set alone, and the log opens nothing.
    let compile: structured_log::Compile =
        Box::new(
            move || match Home::locate(compile_home.as_deref(), compile_os_home.as_deref()) {
                Ok(home) => log_redactor(&home, &settings),
                Err(_) => Redactor::new(&[])
                    .map(|(r, _)| r)
                    .map_err(|e| e.to_string()),
            },
        );
    let open: structured_log::Open = Box::new(move || {
        let Ok(home) = Home::locate(parinn_home.as_deref(), home.as_deref()) else {
            return Ok(None);
        };
        if !home.root().exists() || home.check().is_err() {
            return Ok(None);
        }
        if home.check_home_id(home_id.as_deref()).is_err() {
            return Ok(None);
        }
        let file = structured_log::open_log_file(&home.root().join("logs"))?;
        Ok(Some(Box::new(file)))
    });
    structured_log::StructuredLog::new(open, compile, Box::new(clock))
}

/// The redactor log fields pass through: the rule set and the redaction
/// patterns the principal's configuration and managed policy of `home`
/// name, as written, so a pattern still awaiting acceptance redacts the
/// log too; a repository's never (SEC-08). None while the principal's
/// configuration fails.
fn log_redactor(home: &Home, settings: &LogSettings) -> Result<Redactor, String> {
    let sources = configuration::Sources {
        configuration: home.root().join(configuration::source::CONFIGURATION),
        repository: None,
        managed: settings.managed.clone(),
        running: settings.os_user,
    };
    let loaded = configuration::load(&sources);
    if loaded.principal_layer_failed() {
        return Err("the configuration failed, so its redaction patterns are unknown".to_owned());
    }
    let mut patterns: Vec<String> = Vec::new();
    for layer in [&loaded.layers.configuration, &loaded.layers.managed] {
        let named = configuration::key::key("redaction.extra_patterns").and_then(|k| layer.get(&k));
        if let Some(configuration::Value::Texts(items)) = named {
            patterns.extend(
                items
                    .iter()
                    .filter(|p| !patterns.contains(p))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
    }

    Redactor::new(&patterns)
        .map(|(r, _)| r)
        .map_err(|e| e.to_string())
}

/// Audits the log lines a run could not write (I6).
pub fn audit_dropped_log_lines(ctx: &Context, dropped: u64) {
    for _ in 0..dropped {
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::Dropped,
            counter: Counter::LogLineDropped,
            operation: "log",
            detail: "log line dropped",
        });
    }
}

/// Runs one invocation and returns its exit code. It is `main` without
/// the process: every test drives the command line through here.
pub fn run(
    args: impl IntoIterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        None => {
            let _ = stderr.write_all(USAGE.as_bytes());
            EXIT_USAGE
        }
        Some("configuration") => act::configuration(args, ctx, stdout, stderr),
        Some("counter") => act::counter(args, ctx, stdout, stderr),
        Some("doctor") => doctor(args, ctx, stdout, stderr),
        Some("event") => event::event(args, ctx, stdout, stderr),
        Some("hook") => hook_handlers::hook_command(args.next().as_deref(), ctx, stdout),
        Some("ingest") => ingest(args, ctx, stdout, stderr),
        Some("licenses") => licenses::licenses(args, stdout, stderr),
        Some("mcp") => mcp(args, ctx, stdout, stderr),
        Some("purge") => purge::purge(args, ctx, stdout, stderr),
        Some("quarantine") => quarantine::quarantine(args, ctx, stdout, stderr),
        Some("rebuild") => rebuild::rebuild(args, ctx, stdout, stderr),
        Some("repository") => repository_identity::repository(args, ctx, stdout, stderr),
        Some("risk-acceptance") => act::risk_acceptance(args, ctx, stdout, stderr),
        Some("status") => status(args, ctx, stdout, stderr),
        Some("verify") => verify_command(args, ctx, stdout, stderr),
        Some("version") => version(args, ctx.build, stdout, stderr),
        Some("help" | "-h" | "--help") => written(stdout.write_all(USAGE.as_bytes()), stderr),
        Some(other) => {
            let _ = write!(stderr, "parinn: unknown command {other:?}\n\n{USAGE}");
            EXIT_USAGE
        }
    }
}

/// `parinn version [--json]`: the stamped version as a bare line, or as
/// `{"version": ..., "commit": ...}`, the commit left out when the
/// build stamped none.
fn version(
    args: impl Iterator<Item = String>,
    build: &Build,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("version", args, stderr) else {
        return EXIT_USAGE;
    };
    let line = if as_json {
        version_json(build)
    } else {
        build.version.to_owned()
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// The JSON object `parinn version --json` prints.
fn version_json(build: &Build) -> String {
    let mut out = format!("{{\"version\":{}", json_string(build.version));
    if let Some(commit) = build.commit {
        out.push_str(&format!(",\"commit\":{}", json_string(commit)));
    }
    out.push('}');

    out
}

/// The flags a command takes, with `--json` among them; anything else
/// is a usage error, reported on `stderr`.
pub(crate) fn json_flag(
    command: &str,
    args: impl Iterator<Item = String>,
    stderr: &mut dyn Write,
) -> Option<bool> {
    let mut as_json = false;
    for arg in args {
        if arg != "--json" {
            let _ = writeln!(stderr, "parinn {command}: unknown flag {arg:?}");
            return None;
        }
        as_json = true;
    }

    Some(as_json)
}

/// Logs the failure of `result`, if it failed, as what `what` says: for
/// work that must not stop the command it runs in (I9).
pub(crate) fn logged<T, E: std::fmt::Display>(what: &str, result: Result<T, E>) {
    if let Err(e) = result {
        tracing::warn!(error = e.to_string().as_str(), what, "failed");
    }
}

/// Reports a failure of `command` on `stderr`.
pub(crate) fn failed(command: &str, e: &dyn std::fmt::Display, stderr: &mut dyn Write) -> u8 {
    let _ = writeln!(stderr, "parinn {command}: {e}");
    EXIT_FAILURE
}

/// Reports a store that failed to open: one of a newer schema version is
/// an integrity failure, refused and named, exiting 3 (ADM-05); any other
/// a failure.
fn store_failed(command: &str, e: &store::StoreError, stderr: &mut dyn Write) -> u8 {
    let code = failed(command, e, stderr);
    if matches!(e, store::StoreError::Newer { .. }) {
        EXIT_INTEGRITY
    } else {
        code
    }
}

/// `parinn mcp`: serves the MCP tools over the process's input and
/// `stdout` until the input closes.
fn mcp(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    if let Some(arg) = args.into_iter().next() {
        let _ = writeln!(stderr, "parinn mcp: unknown flag {arg:?}");
        return EXIT_USAGE;
    }
    let home = match ctx.home() {
        Ok(home) => home,
        Err(e) => return failed("mcp", &e, stderr),
    };
    // The MCP server counts each start marker a stopped hook handler
    // left behind, as it starts (NFR-02).
    if home.root().exists() {
        hook_handlers::count_unfinished(&home, (ctx.clock)(), None);
    }
    // It records its harness session's runs, so it starts the node as
    // ingest does (REC-13).
    let served = start_node(ctx, &home, stderr).and_then(|mut store| {
        repository_identity::bind_observed(ctx, &home, &mut store);
        let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
        let server = mcp::Server::new(
            store,
            ctx.build.version,
            node.device().public(),
            node.personal_room(),
        );
        Ok((server, node.device_writer()))
    });
    let (server, device_writer) = match served {
        Ok(served) => served,
        Err(e) => return failed("mcp", &e, stderr),
    };
    // What an MCP server killed before it stopped left is audited by
    // the next one, as it starts (REC-13).
    capture::count_stopped(&home, "mcp", &|o| {
        let _ = ctx.audit(o);
    });
    let shared = std::sync::Arc::new(capture::Shared::default());
    // Neither a recall event nor a line of a transcript may be written
    // without the redaction patterns in force (SEC-08).
    let in_force = audit_settings(ctx, &home, stderr);
    let redactor = recall::redactor(ctx, &in_force, stderr);
    let mut server = server
        .with_runs(Box::new(capture::Finder(shared.clone())))
        .with_recall_writer(recall::recall_writer(device_writer, redactor.clone()));
    let live = redactor
        .map(|r| live_capture(ctx, &home, &shared, &in_force, r))
        .map(capture::Capture::start)
        .map(spawn_capture);
    if let Some(deadline) = ctx.query_deadline {
        server = server.with_query_deadline(deadline);
    }
    // The panic and the cancelled search each audit, and say on stderr
    // when the audit log takes no entry, so both write through it.
    let stderr = RefCell::new(stderr);
    let (mut panic_err, mut cancel_err) = (Shared(&stderr), Shared(&stderr));
    let mut cancelled = on_query_cancelled(&home, ctx.clock, &mut cancel_err);
    let served = mcp::serve(
        &mut |l| {
            let reply = server.handle(l);
            record_found(ctx, &server.take_found());
            cancelled(server.take_cancelled());
            recall::audit_notes(ctx, &server.take_notes());
            reply
        },
        &(ctx.input)(),
        stdout,
        IDLE,
        &mut on_mcp_panic(&home, ctx.clock, &mut panic_err),
    );
    // Its input closed: live ingest stops, audits what it left and closes
    // its segments, within a bound; past it the next reader audits it.
    shared.stop();
    if let Some(done) = live {
        let _ = done.recv_timeout(CAPTURE_STOP);
    }
    match served {
        Ok(()) => EXIT_OK,
        Err(e) => failed("mcp", &e, &mut **stderr.borrow_mut()),
    }
}

/// How long `parinn mcp` waits for live ingest to stop once its input
/// closes.
const CAPTURE_STOP: Duration = Duration::from_secs(5);

/// The MCP server's live ingest in `home` (REC-13): its own harness
/// process's session bindings, its transcripts resolved within the
/// transcript roots, redacted by `redactor`, the patterns of the
/// settings in force.
fn live_capture(
    ctx: &Context,
    home: &Home,
    shared: &std::sync::Arc<capture::Shared>,
    in_force: &settings::InForce,
    redactor: Redactor,
) -> capture::Capture {
    let settings = &in_force.resolved.settings;
    let os_home = ctx.home.as_deref().map(Path::new);
    let roots = settings
        .texts("transcript.roots")
        .iter()
        .fold(AllowedRoots::default(), |allowed, root| {
            allowed.with(&allowed_root::expand_home(root, os_home), Access::Read)
        });
    let base = (ctx.clock)();
    let started = now_instant();

    capture::Capture {
        ceiling: ctx.home.as_deref().map(PathBuf::from),
        home: home.clone(),
        roots,
        redactor,
        inputs: ingest::Inputs {
            payload: payload_settings(settings),
            now_ms: base.saturating_mul(1000),
            interactive: settings.deployment_mode() == "interactive",
            ..ingest::Inputs::default()
        },
        harness: (ctx.ancestry)().and_then(|a| a.harness),
        limit: ctx.live_limit,
        clock: Box::new(move || base + started.elapsed().as_secs()),
        shared: shared.clone(),
    }
}

/// Now, for live ingest's audit clock.
#[expect(
    clippy::disallowed_methods,
    reason = "live ingest's audit entries record when they were made; no derived artifact \
              reads it (I10)"
)]
fn now_instant() -> Instant {
    Instant::now()
}

/// Runs `capture` on a thread of its own; what it sends tells it ended.
fn spawn_capture(capture: capture::Running) -> Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        capture.run();
        let _ = tx.send(());
    });

    rx
}

/// Logs what finding each tool call's run came to (PRV-14, OPS-05),
/// through redaction: its tool-use id, the run or why none was found,
/// and how long it waited; each refusal counted and audited (I6).
fn record_found(ctx: &Context, found: &[mcp::tool_call::Found]) {
    use mcp::tool_call::Cause;
    for f in found {
        let id = f.tool_use_id.as_deref().unwrap_or_default();
        let waited_ms = u64::try_from(f.waited.as_millis()).unwrap_or(u64::MAX);
        let cause = match &f.run {
            Ok(run) => {
                tracing::info!(tool_use_id = id, run = run.as_str(), waited_ms, "tool call");
                continue;
            }
            Err(cause) => *cause,
        };
        tracing::info!(
            tool_use_id = id,
            cause = cause.as_str(),
            waited_ms,
            "tool call refused"
        );
        let counter = match cause {
            Cause::NoToolUseId => Counter::ToolCallWithoutToolUseId,
            Cause::NotFound => Counter::ToolCallRunNotFound,
            Cause::Ambiguous => Counter::ToolCallRunAmbiguous,
        };
        let _ = ctx.audit(&Occurrence::Degraded {
            kind: Kind::Rejected,
            counter,
            operation: "mcp",
            detail: "a tool call whose run could not be found was refused",
        });
    }
}

/// A writer two closures share, each writing in turn.
struct Shared<'a, 'w>(&'a RefCell<&'w mut dyn Write>);

impl Write for Shared<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.borrow_mut().flush()
    }
}

/// What the MCP server does with each search the deadline cancelled:
/// audit and count it, saying on `stderr` when the audit log takes no
/// entry (SEC-04, I6).
fn on_query_cancelled<'a>(
    home: &'a Home,
    clock: &'a dyn Fn() -> u64,
    stderr: &'a mut dyn Write,
) -> impl FnMut(u64) + 'a {
    move |cancelled| {
        for _ in 0..cancelled {
            if let Err(e) = query_cancelled(home, clock()) {
                let _ = writeln!(stderr, "parinn mcp: audit a cancelled search: {e}");
            }
        }
    }
}

/// Audits a search the deadline cancelled.
fn query_cancelled(home: &Home, time: u64) -> Result<Entry, AuditError> {
    let occurrence = Occurrence::Degraded {
        kind: Kind::TimedOut,
        counter: Counter::QueryCancelled,
        operation: "mcp",
        detail: "query cancelled at its deadline",
    };

    Log::new(home.audit()).append(&occurrence, time)
}

/// The development build's seam for the `@ENG-04` scenario: the hook
/// handler panics.
#[expect(
    clippy::panic,
    reason = "the seam's whole purpose is a panic the hook entry point must catch"
)]
fn seam_panic() {
    panic!("the hook handler panics, as PARINN_TEST_FAULT asked");
}

/// The hook input on stdin, its lines joined, once the input closes
/// by `deadline`. Reading stops once it holds more than 1 MiB (SEC-05).
///
/// # Errors
///
/// [`Failure::TooLarge`] past 1 MiB or for a line the reader cut at its
/// limit; [`Failure::Malformed`] when the input cannot be read or does
/// not close in time.
#[expect(
    clippy::disallowed_methods,
    reason = "the deadline on reading the hook input (ENG-03); no derived artifact reads it (I10)"
)]
fn hook_input(ctx: &Context, deadline: Instant) -> Result<String, Failure> {
    let input = (ctx.input)();
    let (mut text, mut lines) = (String::new(), 0usize);
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Failure::Malformed)?;
        match input.recv_timeout(left) {
            Ok(Ok(line)) => {
                if lines > 0 {
                    text.push('\n');
                }
                text.push_str(&line);
                lines += 1;
                if text.len() > hook::HOOK_INPUT_LIMIT {
                    return Err(Failure::TooLarge);
                }
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::FileTooLarge => return Err(Failure::TooLarge),
            Ok(Err(_)) | Err(RecvTimeoutError::Timeout) => return Err(Failure::Malformed),
            Err(RecvTimeoutError::Disconnected) => return Ok(text),
        }
    }
}

/// The lines `reader` holds, each without its line ending. A line
/// longer than `limit` bytes is never held whole (SEC-05): an error of
/// kind [`io::ErrorKind::FileTooLarge`] stands in its place, its rest
/// skipped, and the lines after it go on, as they do after one that is
/// not UTF-8. A read that fails ends them.
pub fn bounded_lines<R: io::BufRead>(
    reader: R,
    limit: usize,
) -> impl Iterator<Item = io::Result<String>> {
    let mut reader = Some(reader);
    std::iter::from_fn(move || {
        let r = reader.as_mut()?;
        let mut buf = Vec::new();
        // The limit, and room for a line ending of two bytes.
        let read = io::BufRead::read_until(
            &mut io::Read::take(r.by_ref(), limit as u64 + 2),
            b'\n',
            &mut buf,
        );
        match read {
            Ok(0) => None,
            Ok(_) if buf.last() != Some(&b'\n') && buf.len() > limit => {
                let skipped = io::BufRead::skip_until(r, b'\n');
                Some(skipped.and(Err(io::ErrorKind::FileTooLarge.into())))
            }
            Ok(_) => Some(line_of(buf, limit)),
            Err(e) => {
                reader = None;
                Some(Err(e))
            }
        }
    })
}

/// One line read with its ending, checked against `limit`.
fn line_of(mut buf: Vec<u8>, limit: usize) -> io::Result<String> {
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
    }
    if buf.len() > limit {
        return Err(io::ErrorKind::FileTooLarge.into());
    }

    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// What the MCP server does with each panic it catches: audit it,
/// saying on `stderr` when the audit log takes no entry.
fn on_mcp_panic<'a>(
    home: &'a Home,
    clock: &'a dyn Fn() -> u64,
    stderr: &'a mut dyn Write,
) -> impl FnMut() + 'a {
    move || {
        if let Err(e) = mcp_panicked(home, clock()) {
            let _ = writeln!(stderr, "parinn mcp: audit a caught panic: {e}");
        }
    }
}

/// Audits a panic the MCP server caught.
fn mcp_panicked(home: &Home, time: u64) -> Result<Entry, AuditError> {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Failed,
        counter: Counter::McpPanicked,
        operation: "mcp",
        detail: "panic recovered",
    };

    Log::new(home.audit()).append(&occurrence, time)
}

/// Turns the result of writing the command's output into its exit
/// code: a write that fails, to a closed stdout say, is a failure.
pub(crate) fn written(result: std::io::Result<()>, stderr: &mut dyn Write) -> u8 {
    match result {
        Ok(()) => EXIT_OK,
        Err(e) => {
            let _ = writeln!(stderr, "parinn: write the output: {e}");
            EXIT_FAILURE
        }
    }
}

/// Reports each failed settings layer but managed policy, exiting 2
/// (ADM-04): the error names the layer, the key and the problem.
pub(crate) fn invalid_layers(command: &str, loaded: &Loaded, stderr: &mut dyn Write) -> u8 {
    for failure in loaded
        .failures
        .iter()
        .filter(|f| f.layer != SettingsLayer::ManagedPolicy)
    {
        let _ = writeln!(stderr, "parinn {command}: {failure}");
    }

    EXIT_USAGE
}

mod act;
mod capture;
mod event;
pub mod forks;
mod hook_handlers;
mod identity_status;
mod index_gaps;
mod ingest_command;
mod ingest_path;
mod ingest_report;
mod licenses;
mod purge;
mod quarantine;
mod rebuild;
mod recall;
mod repository_identity;
mod retention;
mod segments;
mod settings;
mod sources;
mod sourcing;
mod status_command;
mod verify_command;

pub use ingest_command::audit_limited_log_fields;
use ingest_command::*;
use status_command::*;
use verify_command::*;

pub use hook_handlers::budget_override;
pub use settings::managed_policy;

#[cfg(test)]
mod tests;
