//! The steps of REC-11 and SEC-04 that phase 12 of plan 2609292012
//! carries: event and payload text is full-text indexed up to the
//! indexing cap and searched with ranked results, and the code that
//! builds a search query treats caller text as literal terms within
//! bounds. The agent's `event_search` call is a request to `parinn mcp`
//! on its standard input, as the harness sends it; the scope is `room`,
//! as `event_get` takes until the session binding names a run (phase 9)
//! and RCL-05 builds the recall scopes (phase 13).
//!
//! `@RCL-02` stays pending, so its steps are unbound: its calls run in
//! a run the live ingest records (REC-13, phase 10), its `trust` filter
//! needs trust levels (PRV-02, phase 11), its `rooms` scope is RCL-05's,
//! and its transcript "mixed-provenance" has no steps. Its M1 part is
//! proven by named tests in the `mcp` and `store` crates.
//!
//! The 1M events of SEC-04's store are written to `store.db` by one SQL
//! statement beside three real appends, so the scenario need not append
//! a million events through the chain; a search reads no chain.

use std::path::Path;

use ::record::commitment::CommitmentKey;
use ::record::event::{EventKind, Provenance};
use ::record::id::{Address, Seq};
use administration::node::Node;
use audit::log::Log;
use cucumber::{given, then, when};
use key::random::OsEntropy;
use serde_json::{Value, json};
use store::{NewEvent, Store};

use crate::World;
use crate::provenance::{bash_result, typed};
use crate::record::{first_start, home, last_run, run_core, set_env};
use crate::security_redaction::append;

/// How many events SEC-04's store holds.
const EVENTS: u64 = 1_000_000;

/// The text of the canary event the literal-terms row looks for.
const CANARY: &str = "foo OR bar NEAR(baz)";

/// One answer of the MCP server to `event_search`.
#[derive(Debug, Clone)]
pub struct Reply {
    pub is_error: bool,
    /// The envelope's text, or the refusal.
    pub text: String,
}

impl Reply {
    /// The envelope's items; none for a refusal.
    fn items(&self) -> Vec<Value> {
        serde_json::from_str::<Value>(&self.text)
            .ok()
            .and_then(|e| e["items"].as_array().cloned())
            .unwrap_or_default()
    }
}

/// What the search steps track: the last answer.
#[derive(Debug, Default)]
pub struct SearchSteps {
    reply: Option<Reply>,
}

fn steps(w: &mut World) -> &mut SearchSteps {
    w.section::<SearchSteps>()
}

/// The answer of the last call.
fn reply(w: &mut World) -> Result<Reply, String> {
    steps(w)
        .reply
        .clone()
        .ok_or_else(|| "no step called event_search".to_owned())
}

/// Asks the MCP server `parinn mcp` for `event_search` with `arguments`,
/// the scope `room` unless they name one, and keeps its answer.
pub fn call_search(w: &mut World, mut arguments: Value) -> Result<Reply, String> {
    if arguments.get("scope").is_none() {
        arguments["scope"] = json!("room");
    }
    let requests = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "event_search", "arguments": arguments,
        }}),
    ];
    let input: String = requests.iter().map(|r| format!("{r}\n")).collect();
    run_core(w, &["mcp"], Some(input.as_bytes()))?;
    let out = last_run(w)?;
    let replies: Vec<Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let result = replies
        .iter()
        .find(|r| r["id"] == 2)
        .map(|r| &r["result"])
        .ok_or_else(|| format!("event_search answered nothing: {replies:?}"))?;
    let answer = Reply {
        is_error: result["isError"] == true,
        text: result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    };
    steps(w).reply = Some(answer.clone());

    Ok(answer)
}

/// The words a query description stands for: the scenario names a
/// query by its shape, and the step writes one of that shape.
fn query_of(w: &mut World, described: &str) -> String {
    match described {
        "17 terms" => (1..=17)
            .map(|n| format!("term{n}"))
            .collect::<Vec<_>>()
            .join(" "),
        "one term of 65 characters" => "a".repeat(65),
        "a query running past 2 s" => {
            // A development build's seam shortens the deadline: no
            // search over a bounded window runs for 2 s, so the step
            // asks the core to cancel at once.
            set_env(w, "PARINN_TEST_QUERY_DEADLINE_MS", "0");
            "filler".to_owned()
        }
        literal => literal.to_owned(),
    }
}

/// The arguments the step text names: `key value` pairs, a value any
/// JSON value, separated by commas.
fn arguments_of(w: &mut World, text: &str) -> Result<Value, String> {
    let mut arguments = json!({});
    let mut rest = text.trim();
    while !rest.is_empty() {
        let (key, after) = rest
            .split_once(' ')
            .ok_or_else(|| format!("{text:?} names no value for {rest:?}"))?;
        let mut values = serde_json::Deserializer::from_str(after).into_iter::<Value>();
        let value = values
            .next()
            .ok_or_else(|| format!("{text:?} names no value for {key}"))?
            .map_err(|e| format!("{text:?}: {e}"))?;
        let value = match (key, value.as_str()) {
            ("query", Some(described)) => json!(query_of(w, described)),
            _ => value,
        };
        arguments[key] = value;
        rest = after[values.byte_offset()..]
            .trim_start_matches(',')
            .trim_start();
    }

    Ok(arguments)
}

#[when(regex = r#"^the agent calls the MCP tool "event_search" with (.+)$"#)]
fn calls_event_search(w: &mut World, args: String) -> Result<(), String> {
    let arguments = arguments_of(w, &args)?;
    call_search(w, arguments).map(drop)
}

/// A typed user line saying `text`, its own `uuid` `n`.
fn user_line(text: &str, n: u64) -> Result<Value, String> {
    let mut line = typed()?;
    line["message"]["content"] = json!(text);
    line["uuid"] = json!(format!("00000000-0000-4000-8000-0000000012{n:02}"));
    Ok(line)
}

#[given(
    regex = r#"^"([^"]+)" contains a 2 MiB tool result with "([^"]+)" in its first KiB and "([^"]+)" beyond its first 1 MiB$"#
)]
fn big_tool_result(w: &mut World, name: String, early: String, late: String) -> Result<(), String> {
    let mut output = format!("{early} ");
    output.push_str(&"x ".repeat(768 * 1024));
    output.push_str(&late);
    output.push(' ');
    output.push_str(&"y ".repeat(256 * 1024));
    for line in bash_result(Some(&output))? {
        append(w, &name, &line)?;
    }
    Ok(())
}

#[given(
    regex = r#"^"([^"]+)" contains one "user" event naming "([^"]+)" once and another naming it five times$"#
)]
fn user_events_naming(w: &mut World, name: String, word: String) -> Result<(), String> {
    let once = format!("please remember the {word} when you plan this work");
    let five = [word.as_str(); 5].join(" ");
    append(w, &name, &user_line(&once, 1)?)?;
    append(w, &name, &user_line(&five, 2)?)
}

#[then(regex = r"^the tool result event is a hit$")]
fn tool_result_is_a_hit(w: &mut World) -> Result<(), String> {
    let answer = reply(w)?;
    if answer.is_error {
        return Err(format!("event_search refused: {}", answer.text));
    }
    let provenances: Vec<String> = answer
        .items()
        .iter()
        .map(|i| i["provenance"].as_str().unwrap_or_default().to_owned())
        .collect();
    if provenances.iter().any(|p| p.starts_with("tool_result:")) {
        return Ok(());
    }
    Err(format!("no hit is a tool result: {provenances:?}"))
}

#[then(
    regex = r#"^a search for "([^"]+)" returns no hit, because indexing stops at the default 1 MiB cap$"#
)]
fn no_hit_past_the_cap(w: &mut World, word: String) -> Result<(), String> {
    let held = crate::record_transcript::transcript_events(w)?
        .iter()
        .any(|(_, e)| text_of(e).contains(&word));
    if !held {
        return Err(format!("the record holds no event whose text names {word}"));
    }
    let answer = call_search(w, json!({"query": word}))?;
    if !answer.is_error && answer.items().is_empty() {
        return Ok(());
    }
    Err(format!("a search for {word} answered: {}", answer.text))
}

/// The text an event's content holds.
fn text_of(stored: &store::Stored) -> &str {
    stored
        .content
        .as_ref()
        .and_then(|c| c["text"].as_str())
        .unwrap_or_default()
}

#[then(
    regex = r#"^the hits are ranked by score, the "user" event naming "([^"]+)" five times above the one naming it once$"#
)]
fn ranked_by_score(w: &mut World, word: String) -> Result<(), String> {
    let answer = reply(w)?;
    if answer.is_error {
        return Err(format!("event_search refused: {}", answer.text));
    }
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut named = Vec::new();
    let mut last = f64::INFINITY;
    for item in answer.items() {
        let score = item["score"].as_f64().ok_or("a hit carries no score")?;
        if score > last {
            return Err(format!(
                "the hits are not ranked by score: {score} after {last}"
            ));
        }
        last = score;
        let address = Address::parse(item["address"].as_str().unwrap_or_default())
            .map_err(|e| e.to_string())?
            .0;
        let event = store.event(address).map_err(|e| e.to_string())?;
        let count = event
            .as_ref()
            .map_or(0, |e| text_of(e).matches(&word).count());
        named.push(count);
    }
    if named == [5, 1] {
        return Ok(());
    }
    Err(format!("the hits name {word} {named:?} times, not [5, 1]"))
}

/// A new event of the canary kind: a transcript line saying `text`.
fn line_event(text: &str) -> Result<NewEvent, String> {
    Ok(NewEvent {
        kind: EventKind::TranscriptLine,
        provenance: Provenance::HarnessText,
        content: json!({"line": 1, "text": text}),
        commitment_key: CommitmentKey::from_bytes(
            key::random::draw(&mut OsEntropy).map_err(|e| e.to_string())?,
        ),
        transcript_timestamp: None,
    })
}

#[given(regex = r"^a store with 1M events$")]
fn store_with_a_million_events(w: &mut World) -> Result<(), String> {
    let home = first_start(w)?;
    let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
    let seat = node
        .certify_ingest_seat(&home, &mut store, &mut OsEntropy)
        .map_err(|e| e.to_string())?;
    let events = [CANARY, "foo only", "bar baz together"]
        .iter()
        .map(|t| line_event(t))
        .collect::<Result<Vec<_>, _>>()?;
    let first = store
        .append(seat.writer, &events)
        .map_err(|e| e.to_string())?;
    let last = first.last().ok_or("the appends made no event")?.seq.get();
    let at = Address::new(
        seat.writer,
        Seq::new(last).ok_or("the last seq is not one")?,
    );
    let row_of_last = store::search_index::row_id_of(at).ok_or("the last seq is not indexed")?;
    drop(store);

    bulk_insert(&home.store(), &seat.writer.to_string(), last, row_of_last)
}

/// Writes [`EVENTS`] events after seq `last` of `writer`, and their rows
/// in the search index, in one transaction. `row_of_last` is the index
/// row of seq `last`, from which the others follow.
fn bulk_insert(path: &Path, writer: &str, last: u64, row_of_last: i64) -> Result<(), String> {
    let mut conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA synchronous = OFF")
        .map_err(|e| e.to_string())?;
    let index = path.with_file_name(store::search_index::FILE);
    let index = index.to_str().ok_or("the index path is not text")?;
    conn.execute("ATTACH DATABASE ?1 AS idx", [index])
        .map_err(|e| e.to_string())?;
    let slot_part = row_of_last - ((last as i64) << store::search_index::SLOT_BITS);
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let numbers = "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < ?1)";
    tx.execute(
        &format!(
            "{numbers} INSERT INTO events (writer, seq, kind, provenance, prev_hash, commitment, hash,
                                            content, commitment_key)
             SELECT ?2, ?3 + i, 'transcript_line', 'harness_text', printf('%064x', i),
                    printf('%064x', i), printf('%064x', i),
                    json_object('line', i, 'text', 'filler line ' || (i % 5000)),
                    printf('%064x', i)
             FROM n"
        ),
        rusqlite::params![EVENTS as i64, writer, last as i64],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        &format!(
            "{numbers} INSERT INTO idx.events_fts (rowid, text)
             SELECT ((?2 + i) << ?3) + ?4, 'filler line ' || (i % 5000) FROM n"
        ),
        rusqlite::params![
            EVENTS as i64,
            last as i64,
            i64::from(store::search_index::SLOT_BITS),
            slot_part
        ],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE idx.search_writers SET indexed_to = ?1 WHERE writer = ?2",
        rusqlite::params![last as i64 + EVENTS as i64, writer],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

/// How many entries of the audit log count `counter`.
fn audited(w: &World, counter: &str) -> Result<usize, String> {
    let audit = home(w)?.audit();
    if !audit.exists() {
        return Ok(0);
    }
    Ok(Log::new(audit)
        .entries()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .count())
}

#[then(regex = r#"^the query is "(.+)"$"#)]
fn the_query_is(w: &mut World, expected: String) -> Result<(), String> {
    let answer = reply(w)?;
    if let Some(refused) = expected.strip_prefix("rejected: ") {
        return refused_with(&answer, refused);
    }
    match expected.as_str() {
        "matched as the literal terms, no FTS5 operator run" => literal_terms(w, &answer),
        "cancelled at the 2 s deadline and audited" => cancelled(w, &answer),
        other => Err(format!("no step knows what {other:?} means")),
    }
}

/// The answer refuses the query with `refused`, which repeats nothing
/// the caller typed.
fn refused_with(answer: &Reply, refused: &str) -> Result<(), String> {
    if answer.is_error && answer.text == refused {
        return Ok(());
    }
    Err(format!(
        "the answer is {} {:?}, not a refusal {refused:?}",
        if answer.is_error {
            "the refusal"
        } else {
            "no refusal:"
        },
        answer.text
    ))
}

/// The query matched the canary alone: had an operator run, "foo" or
/// "bar" alone would have matched the decoys too.
fn literal_terms(w: &mut World, answer: &Reply) -> Result<(), String> {
    if answer.is_error {
        return Err(format!("the query was refused: {}", answer.text));
    }
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut found = Vec::new();
    for item in answer.items() {
        let address = Address::parse(item["address"].as_str().unwrap_or_default())
            .map_err(|e| e.to_string())?
            .0;
        let event = store.event(address).map_err(|e| e.to_string())?;
        found.push(event.as_ref().map(text_of).unwrap_or_default().to_owned());
    }
    if found == [CANARY] {
        return Ok(());
    }
    Err(format!(
        "the query matched {found:?}, not the canary {CANARY:?} alone"
    ))
}

/// The search was cancelled at its deadline, and the audit log counts it.
fn cancelled(w: &mut World, answer: &Reply) -> Result<(), String> {
    if !answer.is_error || !answer.text.contains("cancelled") {
        return Err(format!("the search was not cancelled: {}", answer.text));
    }
    match audited(w, "query_cancelled")? {
        1.. => Ok(()),
        0 => Err("the audit log counts no cancelled search".to_owned()),
    }
}
