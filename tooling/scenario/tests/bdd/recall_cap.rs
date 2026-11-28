//! The steps of RCL-01, RCL-03, RCL-04 and RCL-07 on the cap and its
//! continuation cursor. Moved from `recall.rs` to keep each file in its limit.

use ::record::id::Address;
use cucumber::then;
use envelope::tokens;
use serde_json::{Value, json};
use store::Store;

use crate::World;
use crate::live::{self, Harness};
use crate::record::home;
use crate::record_search::last_reply;

use crate::recall::*;

/// How many pages a cursor walk keeps before its server is replaced.
const HALFWAY: usize = 20;

/// Fetches every page of the `event_expand` the last step started, by
/// its cursor, keeping each answer. The agent calls one MCP server for
/// the whole walk, as a harness keeps one through its session: a page
/// costs a request, not a server start. After [`HALFWAY`] pages, that server
/// stops and a new one takes the next cursor, so a cursor that worked
/// only on the server that issued it fails the walk. The servers serve
/// the scope `room`, as the first call's did. Each is stopped, its
/// trace checked (ENG-12), even when the walk fails.
fn follow_cursor(w: &mut World) -> Result<(), String> {
    let page = envelope_of(w)?;
    recall(w).pages = vec![page.clone()];
    recall(w).page_texts = vec![last_reply(w)?.text];
    let Some(cursor) = page["next_cursor"].as_str().map(str::to_owned) else {
        return Ok(());
    };
    let mut harness = live::start(w, true, &[])?;
    let walked = walk(w, &mut harness, cursor, HALFWAY);
    let stopped = harness.stop();
    let Some(cursor) = walked? else {
        return stopped;
    };
    stopped?;
    let mut harness = live::start(w, true, &[])?;
    let walked = walk(w, &mut harness, cursor, usize::MAX);
    let stopped = harness.stop();
    walked?;
    stopped
}

/// Follows `cursor` on `harness`'s MCP server, keeping each page, until
/// the pages kept reach `until` or the walk ends; the cursor left to
/// follow, none at the end.
fn walk(
    w: &mut World,
    harness: &mut Harness,
    mut cursor: String,
    until: usize,
) -> Result<Option<String>, String> {
    loop {
        let arguments = json!({"cursor": cursor, "scope": "room"});
        let text = page_text(harness, arguments)?;
        let page: Value = serde_json::from_str(&text)
            .map_err(|e| format!("event_expand answered no envelope: {e}: {text}"))?;
        recall(w).pages.push(page.clone());
        recall(w).page_texts.push(text);
        if recall(w).pages.len() > 400 {
            return Err("the cursor never ends".to_owned());
        }
        match page["next_cursor"].as_str() {
            None => return Ok(None),
            Some(next) if recall(w).pages.len() >= until => return Ok(Some(next.to_owned())),
            Some(next) => cursor = next.to_owned(),
        }
    }
}

/// The text `event_expand` answers `arguments` with on `harness`'s MCP
/// server; a refusal fails.
fn page_text(harness: &mut Harness, arguments: Value) -> Result<String, String> {
    let params = json!({"name": "event_expand", "arguments": arguments});
    let reply = harness.request("tools/call", params)?;
    let result = &reply["result"];
    let text = result["content"][0]["text"].as_str().unwrap_or_default();
    if result["isError"] == true {
        return Err(format!("event_expand refused the cursor: {text}"));
    }
    Ok(text.to_owned())
}

#[then(
    regex = r"^the items carry the exact post-redaction content with payload references resolved$"
)]
fn exact_content(w: &mut World) -> Result<(), String> {
    let envelope = envelope_of(w)?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let mut resolved = 0;
    let items = envelope["items"].as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return Err("the envelope holds no item".to_owned());
    }
    for item in &items {
        let cut = item["continuation"]
            .as_array()
            .is_some_and(|f| f.iter().any(|f| f == "cut"));
        if item["flags"].as_array().is_some_and(|f| !f.is_empty()) {
            return Err(format!(
                "{} carries flags: {}",
                item["address"], item["flags"]
            ));
        }
        let address = Address::parse(item["address"].as_str().unwrap_or_default())
            .map_err(|e| e.to_string())?
            .0;
        let stored = store.event(address).map_err(|e| e.to_string())?;
        let kept = stored
            .as_ref()
            .and_then(|s| s.content.as_ref())
            .and_then(|c| c["text"].as_str())
            .ok_or_else(|| format!("{address} holds no text"))?;
        let shown = item["content"].as_str().unwrap_or_default();
        if stored.as_ref().is_some_and(|s| s.payload.is_some()) {
            resolved += 1;
        }
        let exact = if cut {
            kept.starts_with(shown)
        } else {
            kept == shown
        };
        if !exact {
            return Err(format!(
                "{address} shows other content than the record holds"
            ));
        }
    }
    if resolved == 0 {
        let seen: Vec<String> = items
            .iter()
            .map(|i| {
                format!(
                    "{}:{}",
                    i["address"]
                        .as_str()
                        .unwrap_or_default()
                        .rsplit('/')
                        .next()
                        .unwrap_or_default(),
                    i["content"].as_str().map_or(0, str::len)
                )
            })
            .collect();
        return Err(format!("no item of the range was a payload: {seen:?}"));
    }
    Ok(())
}

#[then(
    regex = r#"^the envelope is at most 8,000 estimated model tokens with "truncated" true and a "next_cursor"$"#
)]
fn at_most_the_cap(w: &mut World) -> Result<(), String> {
    let answer = last_reply(w)?;
    let envelope = envelope_of(w)?;
    let spent = tokens::estimate(&answer.text);
    if spent > 8_000 {
        return Err(format!("the envelope is {spent} estimated model tokens"));
    }
    if envelope["truncated"] != true || !envelope["next_cursor"].is_string() {
        return Err("the envelope is neither truncated nor continued by a cursor".to_owned());
    }
    Ok(())
}

#[then(
    regex = r#"^calling "event_expand" with that cursor returns the following events without gap or overlap$"#
)]
fn cursor_continues(w: &mut World) -> Result<(), String> {
    follow_cursor(w)?;
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let pages = recall(w).pages.clone();
    for (n, text) in recall(w).page_texts.iter().enumerate() {
        let spent = tokens::estimate(text);
        if spent > 8_000 {
            return Err(format!("page {n} is {spent} estimated model tokens"));
        }
    }
    // Each event in order, a cut one made whole again from its pieces.
    let mut events: Vec<(String, String)> = Vec::new();
    for item in pages
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
    {
        let address = item["address"].as_str().unwrap_or_default().to_owned();
        let content = item["content"].as_str().unwrap_or_default();
        let continued = item["continuation"]
            .as_array()
            .is_some_and(|f| f.iter().any(|f| f == "continued"));
        match events.last_mut() {
            Some((last, text)) if continued && *last == address => text.push_str(content),
            _ => events.push((address, content.to_owned())),
        }
    }
    let first = Address::parse(&events[0].0).map_err(|e| e.to_string())?.0;
    let mut want = 0;
    for (n, (address, text)) in events.iter().enumerate() {
        let parsed = Address::parse(address).map_err(|e| e.to_string())?.0;
        want = first.seq.get() + n as u64;
        if parsed.writer != first.writer || parsed.seq.get() != want {
            return Err(format!(
                "event {n} is {address}, wanting seq {want}: a gap or an overlap"
            ));
        }
        let kept = store.event(parsed).map_err(|e| e.to_string())?;
        let kept = kept
            .as_ref()
            .and_then(|s| s.content.as_ref())
            .and_then(|c| c["text"].as_str());
        if kept != Some(text.as_str()) {
            return Err(format!(
                "{address} came back other than the record holds it"
            ));
        }
    }
    if events.len() < 400 || want != first.seq.get() + events.len() as u64 - 1 {
        return Err(format!(
            "the pages cover {} events, wanting 400",
            events.len()
        ));
    }
    Ok(())
}

// RCL-07.

#[then(
    regex = r#"^the record gains one recall event with provenance "assistant" carrying the query$"#
)]
fn gains_one_recall_event(w: &mut World) -> Result<(), String> {
    let before = recall(w).before;
    let events = recall_events(w)?;
    if events.len() != before + 1 {
        return Err(format!(
            "the record held {before} recall events and now holds {}",
            events.len()
        ));
    }
    let (address, content) = events.last().ok_or("no recall event")?.clone();
    let store = Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let stored = store
        .event(address)
        .map_err(|e| e.to_string())?
        .ok_or("event gone")?;
    if stored.link.header.provenance.name() != "assistant" {
        return Err(format!(
            "its provenance is {}",
            stored.link.header.provenance.name()
        ));
    }
    if content["tool"] != "event_search" || content["arguments"]["query"] != "deploy" {
        return Err(format!("the recall event carries {content}"));
    }
    Ok(())
}

#[then(regex = r"^that event lists the addresses of the returned events$")]
fn lists_addresses(w: &mut World) -> Result<(), String> {
    let (_, content) = recall_events(w)?.last().ok_or("no recall event")?.clone();
    let envelope = envelope_of(w)?;
    let returned: Vec<&str> = envelope["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i["address"].as_str())
        .collect();
    let listed: Vec<&str> = content["returned"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if returned.is_empty() || returned != listed {
        return Err(format!(
            "the call returned {returned:?}, the event lists {listed:?}"
        ));
    }
    Ok(())
}

#[then(regex = r#"^the stat "recalls" increases by 1$"#)]
fn recalls_stat_increases(w: &mut World) -> Result<(), String> {
    let before = recall(w)
        .recalls_before
        .ok_or("no step read the recalls stat before the call")?;
    let (held, left) = recall(w).recall_events_around_read;
    // Reading the stat leaves a recall event of its own, which the
    // later read counts beside the search. The record, not the stat,
    // says how many of each there were; the second read comes after
    // both are counted, so it is the stat's own answer to compare.
    let searched = recall_events(w)?.len().saturating_sub(left);
    let after = recalls_stat(w)?;
    let want = before + (left - held) as u64 + searched as u64;
    if searched == 1 && after == want {
        return Ok(());
    }
    Err(format!(
        "the search left {searched} recall events; stat_list gave {before} recalls before it \
         and {after} after, wanting {want}"
    ))
}
