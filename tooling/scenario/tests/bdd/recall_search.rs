//! The steps of RCL-02: `event_search` honours its filters, returns at
//! most `k` hits ranked by BM25 and damps a run that matches over and
//! over.
//!
//! The calls run in `parinn mcp` with the scope `room`, as the other
//! recall steps do, but for the `rooms` scope, which needs a run: that
//! row serves the transcript "mixed-provenance" live, as RCL-05's
//! stand-in harness does, and calls as the run. The `kind` filter names
//! a provenance class family (`tool_result` matches `tool_result:Bash`).
//! No event a search can find is trusted yet: until OQ-40 provides a
//! signing key every run leaves R3 and R8 open, so no `user` event is
//! trusted (PRV-02, PRV-08). The `trust "trusted"` row returns no hit;
//! its step proves the filter, not the fixture, left nothing.

use std::collections::BTreeMap;

use ::record::id::Address;
use cucumber::then;
use serde_json::{Value, json};
use store::Store;

use crate::World;
use crate::recall::{fixture, keep_recorded, last_asked, record_runs};
use crate::record::home;
use crate::record_search::{Reply, call_tool, last_reply, set_reply};
use crate::security_redaction::{agent_run_with_transcript, transcripts};

/// The run `rooms` is asked of.
const RUN: &str = "mixed-provenance";

/// The run that matches once or twice.
const SHORT: &str = "short-run";

/// A call with the scope `rooms`: a bound run makes it, the run the
/// scenario's transcript holds.
pub fn call_rooms(w: &mut World, arguments: Value) -> Result<(), String> {
    keep_recorded(w, RUN);
    let lines = fixture(RUN)?;
    crate::recall_scope::bind_run(w, RUN, &lines)?;
    crate::recall::asked(w, "event_search", &arguments);
    crate::recall_scope::call_bound(w, "event_search", arguments).map(drop)
}

/// Whether the provenance class `provenance` is in the family `kind`.
fn in_family(provenance: &str, kind: &str) -> bool {
    provenance == kind
        || provenance
            .strip_prefix(kind)
            .is_some_and(|rest| rest.starts_with(':'))
}

/// The strings the filter `name` of `arguments` lists.
fn listed(arguments: &Value, name: &str) -> Vec<String> {
    arguments[name]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

/// Fails unless the event of `item` lies in the time range `arguments`
/// names, as the record's transcript time says.
fn in_range(store: &Store, item: &Value, arguments: &Value) -> Result<(), String> {
    let (since, until) = (arguments["since"].as_str(), arguments["until"].as_str());
    if since.is_none() && until.is_none() {
        return Ok(());
    }
    let address = Address::parse(item["address"].as_str().unwrap_or_default())
        .map_err(|e| e.to_string())?
        .0;
    let stamp = store
        .event(address)
        .map_err(|e| e.to_string())?
        .and_then(|e| e.transcript_timestamp)
        .ok_or_else(|| format!("the hit at {address} has no transcript time"))?;
    if since.is_some_and(|s| stamp.as_str() < s) {
        return Err(format!("{address} at {stamp} is before {since:?}"));
    }
    if until.is_some_and(|u| {
        stamp
            .chars()
            .take(u.chars().count())
            .collect::<String>()
            .as_str()
            > u
    }) {
        return Err(format!("{address} at {stamp} is after {until:?}"));
    }
    Ok(())
}

/// Fails unless `item` matches each filter of `arguments`.
fn matches(store: &Store, item: &Value, arguments: &Value) -> Result<(), String> {
    if let Some(run) = arguments["run"].as_str()
        && item["writer"] != run
    {
        return Err(format!(
            "a hit is from writer {}, not {run}",
            item["writer"]
        ));
    }
    let provenance = item["provenance"].as_str().unwrap_or_default();
    let classes = listed(arguments, "provenance");
    if !classes.is_empty() && !classes.iter().any(|c| c == provenance) {
        return Err(format!("{provenance} is none of {classes:?}"));
    }
    let kinds = listed(arguments, "kind");
    if !kinds.is_empty() && !kinds.iter().any(|k| in_family(provenance, k)) {
        return Err(format!("{provenance} is in none of the families {kinds:?}"));
    }
    if let Some(trust) = arguments["trust"].as_str()
        && item["trust"] != trust
    {
        return Err(format!("a hit is {}, not {trust}", item["trust"]));
    }
    in_range(store, item, arguments)
}

/// Replaces the fixture name used by the feature with the writer id the
/// `run` filter accepts. A run is named by its writer until runs derive
/// from seats.
pub fn resolve_run(w: &mut World, arguments: &mut Value) -> Result<(), String> {
    let Some(name) = arguments["run"].as_str().map(str::to_owned) else {
        return Ok(());
    };
    let path = transcripts(w)
        .into_iter()
        .find(|(fixture, _)| fixture == &name)
        .map(|(_, path)| path)
        .ok_or_else(|| format!("no transcript fixture is named {name:?}"))?;
    let path = path.display().to_string();
    let writer = Store::open(&home(w)?.store())
        .map_err(|e| e.to_string())?
        .ingested_runs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|run| run.path == path)
        .map(|run| run.writer.to_string())
        .ok_or_else(|| format!("no run was ingested from {path}"))?;
    arguments["run"] = json!(writer);
    Ok(())
}

/// The items of `answer`.
fn items_of(answer: &Reply) -> Result<Vec<Value>, String> {
    if answer.is_error {
        return Err(format!("event_search refused: {}", answer.text));
    }
    Ok(answer.items())
}

/// Fails unless `items` come in falling order of score.
fn ranked(items: &[Value]) -> Result<(), String> {
    let mut last = f64::INFINITY;
    for item in items {
        let score = item["score"].as_f64().ok_or("a hit carries no score")?;
        if score > last {
            return Err(format!("{score} ranks after {last}"));
        }
        last = score;
    }
    Ok(())
}

/// The hits of the same query with the filter `name` left out, up to
/// the most a search returns, kept apart from the answer.
fn without(w: &mut World, arguments: &Value, name: &str) -> Result<Vec<Value>, String> {
    let kept = last_reply(w)?;
    let mut open = arguments.clone();
    if let Some(fields) = open.as_object_mut() {
        fields.remove(name);
        fields.insert("k".to_owned(), json!(50));
    }
    let answer = call_tool(w, "event_search", open)?;
    set_reply(w, kept);
    items_of(&answer)
}

/// Fails unless the row's answer shows its filters at work. A search
/// for trusted hits returns none, nothing a search finds being trusted
/// yet, while the same search without the filter finds untrusted hits
/// only. Every other row returns a hit, and a provenance class or
/// family filter refuses a hit the same search finds without it, so an
/// empty answer or a filter that keeps everything proves no row.
fn filters_at_work(
    w: &mut World,
    store: &Store,
    items: &[Value],
    arguments: &Value,
) -> Result<(), String> {
    if arguments["trust"] == "trusted" {
        let open = without(w, arguments, "trust")?;
        if open.is_empty() || open.iter().any(|i| i["trust"] != "untrusted") {
            return Err("the search without the trust filter finds no untrusted hit only".into());
        }
        return Ok(());
    }
    if items.is_empty() {
        return Err("the search returned no hit, so the row proves no filter".into());
    }
    for name in ["provenance", "kind"] {
        if arguments.get(name).is_none() {
            continue;
        }
        let open = without(w, arguments, name)?;
        if open.iter().all(|i| matches(store, i, arguments).is_ok()) {
            return Err(format!(
                "the search without the {name} filter finds no hit the filter refuses"
            ));
        }
    }
    Ok(())
}

#[then(
    regex = r"^the envelope contains at most (\d+) items, each matching the filter, ranked by BM25 score$"
)]
fn at_most_k_matching(w: &mut World, most: usize) -> Result<(), String> {
    let (_, arguments) = last_asked(w).ok_or("no step called event_search")?;
    let items = items_of(&last_reply(w)?)?;
    if items.len() > most {
        return Err(format!("{} items, more than {most}", items.len()));
    }
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    for item in &items {
        matches(&store, item, &arguments)?;
    }
    ranked(&items)?;
    filters_at_work(w, &store, &items, &arguments)
}

#[then(
    regex = r"^a relevant hit from a short run still ranks above the repeated hits of a long run$"
)]
fn short_run_above_repeats(w: &mut World) -> Result<(), String> {
    let kept = last_reply(w)?;
    agent_run_with_transcript(w, SHORT.to_owned())?;
    record_runs(w)?;
    let answer = call_tool(w, "event_search", json!({"query": "deploy", "k": 50}))?;
    set_reply(w, kept);
    let items = items_of(&answer)?;
    let mut ranks: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (rank, item) in items.iter().enumerate() {
        let writer = item["writer"].as_str().unwrap_or_default().to_owned();
        ranks.entry(writer).or_default().push(rank);
    }
    let long = ranks
        .values()
        .max_by_key(|r| r.len())
        .ok_or("the search found no hit")?;
    let short = ranks
        .values()
        .filter(|r| *r != long)
        .filter_map(|r| r.first())
        .min()
        .ok_or("no short run has a hit")?;
    let repeats = long.get(1..).unwrap_or_default();
    if long.len() < 3 || repeats.iter().any(|r| r < short) {
        return Err(format!(
            "the short run's hit ranks {short}, the long run's hits {long:?}"
        ));
    }
    Ok(())
}
