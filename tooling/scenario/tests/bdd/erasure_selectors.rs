//! The steps of SEC-12 and REC-17 that phase 14 of plan 2609292012
//! carries: the selectors `parinn quarantine` takes, and the second half of
//! the erasure steps, moved from `erasure.rs` to keep each file in its limit.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use ::record::chain;
use ::record::commitment::commit;
use ::record::event::{EventKind, Provenance};
use ::record::id::WriterId;
use ::record::range::Range;
use cucumber::{given, then};
use serde_json::{Value, json};

use crate::World;
use crate::record::{home, last_run, run_core, run_core_at_terminal};
use crate::record_search::call_tool;
use crate::record_transcript::ingest;
use crate::security_redaction::{found_in_home, name_transcript};

use crate::erasure::*;

/// The lines of the run `name`: three web pages the agent fetched, then
/// small lines up to `count`, those up to `early` stamped before 2026 and
/// the rest `stamp`. Small lines keep the cap's pages few.
fn run_lines(name: &str, count: u64, early: u64, stamp: &str) -> Result<Vec<Value>, String> {
    let mut lines = Vec::new();
    for n in 1..=3 {
        let mut pair = crate::recall::pair(n, &format!("{name} page {n}"))?;
        pair[0]["message"]["content"][0]["name"] = json!("WebFetch");
        lines.extend(pair);
    }
    let first = if name == "r-1" { "1" } else { "2" };
    for (i, line) in lines.iter_mut().enumerate() {
        // Lines of two runs never share a `uuid`.
        let uuid = line["uuid"].as_str().unwrap_or_default().to_owned();
        line["uuid"] = json!(format!("{first}{}", &uuid[1..]));
        line["sessionId"] = json!(name);
        line["timestamp"] = json!(if (i as u64) < early {
            "2025-12-15T10:00:00Z"
        } else {
            stamp
        });
    }
    for seq in lines.len() as u64 + 1..=count {
        let at = if seq <= early {
            "2025-12-15T10:00:00Z"
        } else {
            stamp
        };
        lines.push(json!({"type": "queue-operation", "operation": "dequeue",
                          "timestamp": at, "sessionId": name}));
    }

    Ok(lines)
}

/// Makes the run `name`, ingests it and names its transcript.
fn make_and_ingest(w: &mut World, name: &str, lines: &[Value]) -> Result<(), String> {
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path: PathBuf = dir.join(format!("{name}.jsonl"));
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    fs::write(&path, text).map_err(|e| e.to_string())?;
    name_transcript(w, name, path.clone());
    ingest(w, &path)
}

#[given(regex = r"^a store whose events match the quarantine selector (.+)$")]
fn store_matching(w: &mut World, selector: String) -> Result<(), String> {
    if selector.starts_with("--flag") {
        return Err(
            "no event carries a flag before PRV-07 (M5), so no store holds an event this selector matches"
                .to_owned(),
        );
    }
    // Run r-1 has 130 lines, those up to 60 stamped in 2025; run r-2
    // has 10, stamped in 2026.
    make_and_ingest(
        w,
        "r-1",
        &run_lines("r-1", 130, 60, "2026-02-15T10:00:00Z")?,
    )?;
    make_and_ingest(w, "r-2", &run_lines("r-2", 10, 0, "2026-03-01T10:00:00Z")?)?;
    crate::recall::record_runs(w)?;
    let store = open(w)?;
    let runs = store.ingested_runs().map_err(|e| e.to_string())?;
    let first = runs
        .iter()
        .find(|r| r.harness_session == "r-1")
        .ok_or("run r-1 was not recorded")?;
    let seat = store
        .seat(first.writer)
        .map_err(|e| e.to_string())?
        .ok_or("run r-1 has no seat")?;
    let steps = er(w);
    steps.selector = Some(selector);
    steps.placeholders = vec![
        ("s-1".to_owned(), seat.seat.to_string()),
        ("room-1".to_owned(), seat.room.to_string()),
        ("w-1".to_owned(), first.writer.to_string()),
    ];
    // The first run seat a node starts is `A1`.
    drop(store);
    if writer_labelled(w, "A1").ok() != Some(first.writer) {
        // Labels are assigned when an address is first asked for.
        run_core(w, &["event", "expand", "A1:1"], None)?;
    }

    Ok(())
}

/// The writer holding `label`, when one is assigned.
fn writer_labelled(w: &World, label: &str) -> Result<WriterId, String> {
    writer_of(w, label)
}

#[derive(Debug, Clone)]
struct Event {
    writer: WriterId,
    seq: u64,
    provenance: Provenance,
    stamp: Option<String>,
}

/// Every event of the two runs, with what a selector reads of it.
fn events(w: &World) -> Result<(Vec<Event>, WriterId), String> {
    let store = open(w)?;
    let mut out = Vec::new();
    let mut first = None;
    for run in store.ingested_runs().map_err(|e| e.to_string())? {
        if run.harness_session == "r-1" {
            first = Some(run.writer);
        }
        for stored in store.log(run.writer).map_err(|e| e.to_string())? {
            out.push(Event {
                writer: run.writer,
                seq: stored.link.header.address.seq.get(),
                provenance: stored.link.header.provenance.clone(),
                stamp: stored.transcript_timestamp,
            });
        }
    }

    Ok((out, first.ok_or("run r-1 was not recorded")?))
}

/// Whether the quarantine `selector` holds `event`, as the scenario
/// reads its words, `first` being the writer of run r-1.
fn selected(selector: &str, event: &Event, first: WriterId) -> Result<bool, String> {
    let mut parts = selector.split_whitespace();
    let (flag, value) = (
        parts.next().unwrap_or_default(),
        parts.next().unwrap_or_default(),
    );
    Ok(match flag {
        "--range" => {
            let range = Range::parse(value).map_err(|e| e.to_string())?;
            event.writer == first && range.from.get() <= event.seq && event.seq <= range.to.get()
        }
        "--run" | "--author" | "--writer" => event.writer == first,
        // Both runs lie in the personal room.
        "--room" => true,
        "--provenance" => event.provenance.name() == value,
        "--since" => event.stamp.as_deref().is_some_and(|s| s >= value),
        other => return Err(format!("no step knows the selector {other:?}")),
    })
}

/// The arguments a selector is added with: the scenario's words, the
/// ids they stand for put in.
fn added_args(w: &mut World, selector: &str) -> Vec<String> {
    words(selector)
        .into_iter()
        .map(|a| {
            er(w)
                .placeholders
                .iter()
                .find(|(shown, _)| *shown == a)
                .map_or(a, |(_, real)| real.clone())
        })
        .collect()
}

/// The last principal act of `name` the device seat recorded.
fn last_act(w: &World, name: &str) -> Result<(Provenance, Value), String> {
    let store = open(w)?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    store
        .log(node.device_writer())
        .map_err(|e| e.to_string())?
        .into_iter()
        .rev()
        .filter(|s| s.link.header.kind == EventKind::PrincipalAct)
        .find_map(|s| {
            let content = s.content?;
            (content["act"] == name).then_some((s.link.header.provenance, content))
        })
        .ok_or_else(|| format!("the device seat recorded no {name} act"))
}

#[then(regex = r#"^an "operator" event records the quarantine with the reason "([^"]+)"$"#)]
fn operator_event_records(w: &mut World, reason: String) -> Result<(), String> {
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn quarantine add exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let (provenance, act) = last_act(w, "quarantine_add")?;
    if provenance != Provenance::Operator {
        return Err(format!("the quarantine is a {} event", provenance.name()));
    }
    if act["reason"] != reason {
        return Err(format!(
            "the quarantine states the reason {}",
            act["reason"]
        ));
    }
    let selector = er(w).selector.clone().unwrap_or_default();
    let kind = selector
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches("--");
    let recorded = act["selector"]["by"].as_str().unwrap_or_default();
    let want = match kind {
        "author" | "since" | "run" | "room" | "writer" | "range" | "provenance" | "flag" => kind,
        other => return Err(format!("no step knows the selector {other:?}")),
    };
    if recorded != want {
        return Err(format!(
            "the quarantine records the selector {recorded:?}, wanting {want:?}"
        ));
    }
    er(w).added = added_args(w, &selector);

    Ok(())
}

#[then(
    regex = r"^the matched events are absent from every later recall, landmark, injection and trusted-only export on this node$"
)]
fn matched_absent(w: &mut World) -> Result<(), String> {
    marker_is_not_a_tombstone()?;
    let selector = er(w).selector.clone().ok_or("no step chose a selector")?;
    let (events, first) = events(w)?;
    let writers: BTreeSet<WriterId> = events.iter().map(|e| e.writer).collect();
    for writer in writers {
        let seen = recall_all(w, writer)?;
        for event in events.iter().filter(|e| e.writer == writer) {
            let held = selected(&selector, event, first)?;
            let shown = seen.items.contains(&event.seq);
            if held && (shown || !seen.quarantined(event.seq)) {
                return Err(format!(
                    "{}:{} matches {selector:?} yet is {}",
                    event.writer,
                    event.seq,
                    if shown {
                        "recalled"
                    } else {
                        "not marked quarantined"
                    }
                ));
            }
            if !held && !shown {
                return Err(format!(
                    "{}:{} matches no quarantine yet is held back",
                    event.writer, event.seq
                ));
            }
        }
    }
    // M1 records no landmark, restore block or export: the landmarks a
    // quarantined event could be summarized into do not exist.
    let landmarks = call_tool(w, "landmark_list", json!({}))?;
    if !landmarks.items().is_empty() {
        return Err("landmark_list returned an item, though M1 records no landmark".to_owned());
    }

    Ok(())
}

#[then(
    regex = r"^a quarantine that would remove a pin, a pin version or a landmark from a restore block takes effect only as a confirmed widening principal act$"
)]
fn no_pin_to_remove(w: &mut World) -> Result<(), String> {
    // M1 records no pin and no landmark, so no quarantine here removes
    // one from a restore block: the act is a cut act, and its widening
    // twin (a pin or landmark in a restore block) has nothing to remove.
    let pins = call_tool(w, "pin_list", json!({}))?;
    let landmarks = call_tool(w, "landmark_list", json!({}))?;
    if !pins.items().is_empty() || !landmarks.items().is_empty() {
        return Err("a pin or a landmark is recorded, which a quarantine could remove".to_owned());
    }
    let (_, act) = last_act(w, "quarantine_add")?;
    if act["class"] != "cut" {
        return Err(format!("the quarantine is a {} act", act["class"]));
    }

    Ok(())
}

#[then(
    regex = r#"^releasing the quarantine, a widening principal act, is recorded as an "operator" event the same way$"#
)]
fn release_is_recorded(w: &mut World) -> Result<(), String> {
    // Before the release the held events answer quarantine markers: a
    // recall that read none would pass the check after it vacuously.
    let (before, _) = events(w)?;
    let mut marked = 0;
    for writer in before.iter().map(|e| e.writer).collect::<BTreeSet<_>>() {
        let seen = recall_all(w, writer)?;
        marked += seen.markers.len();
    }
    if marked == 0 {
        return Err("no quarantine marker stands before the release".to_owned());
    }
    let mut args = vec!["quarantine".to_owned(), "release".to_owned()];
    args.extend(er(w).added.clone());
    let shown: Vec<&str> = args.iter().map(String::as_str).collect();
    run_core_at_terminal(w, &shown)?;
    expect_success(w, "parinn quarantine release")?;
    let (provenance, act) = last_act(w, "quarantine_release")?;
    if provenance != Provenance::Operator || act["class"] != "widening" {
        return Err(format!(
            "the release is a {} {} act",
            act["class"],
            provenance.name()
        ));
    }
    // The release gave the events back to recall.
    let selector = er(w).selector.clone().ok_or("no step chose a selector")?;
    let (events, _) = events(w)?;
    let writers: BTreeSet<WriterId> = events.iter().map(|e| e.writer).collect();
    for writer in writers {
        let seen = recall_all(w, writer)?;
        if let Some(mark) = seen.markers.first() {
            return Err(format!(
                "{selector:?} is released, yet {mark:?} stays held back"
            ));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------- REC-17

#[given(
    regex = r"^a home whose record contains (\d+) events, of which seq (\d+)-(\d+) have since been purged$"
)]
fn forty_events(w: &mut World, count: u64, from: u64, to: u64) -> Result<(), String> {
    let lines: Vec<Value> = (1..=count)
        .map(|n| {
            let text = format!("confidential note {n}: zebra-secret-{n}");
            json!({"type": "x", "uuid": format!("forty-{n}"), "sessionId": "forty", "text": text})
        })
        .collect();
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("forty.jsonl");
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    fs::write(&path, text).map_err(|e| e.to_string())?;
    name_transcript(w, "forty", path.clone());
    ingest(w, &path)?;
    crate::recall::record_runs(w)?;
    let store = open(w)?;
    let run = store.ingested_runs().map_err(|e| e.to_string())?.remove(0);
    let log = store.log(run.writer).map_err(|e| e.to_string())?;
    if log.len() as u64 != count {
        return Err(format!(
            "the run holds {} events, wanting {count}",
            log.len()
        ));
    }
    let originals: Vec<Original> = log
        .iter()
        .filter(|s| (from..=to).contains(&s.link.header.address.seq.get()))
        .map(|s| {
            Ok(Original {
                seq: s.link.header.address.seq.get(),
                content: s.content.clone().ok_or("an event holds no content")?,
                key: s
                    .commitment_key
                    .as_ref()
                    .ok_or("an event holds no key")?
                    .to_hex(),
            })
        })
        .collect::<Result<_, String>>()?;
    drop(store);
    accept_risks(w)?;
    let range = format!("{}:{from}-{to}", run.writer);
    run_core_at_terminal(w, &["purge", "--range", &range])?;
    expect_success(w, "parinn purge")?;
    let steps = er(w);
    steps.purged = originals;
    steps.writer = Some(run.writer);

    Ok(())
}

#[then(
    regex = r"^every event carries a commitment to its canonical content under a random per-event key of at least 256 bits, stored with the content and absent from its chained header$"
)]
fn commitments_and_keys(w: &mut World) -> Result<(), String> {
    let writer = er(w).writer.ok_or("no step purged a range")?;
    let log = open(w)?.log(writer).map_err(|e| e.to_string())?;
    let mut keys = BTreeSet::new();
    for stored in &log {
        let header = &stored.link.header;
        let (Some(content), Some(key)) = (&stored.content, &stored.commitment_key) else {
            continue;
        };
        if key.as_bytes().len() * 8 < 256 {
            return Err(format!("{} has a key of under 256 bits", header.address));
        }
        let opens = commit(key, content).map_err(|e| e.to_string())? == header.commitment;
        if !opens {
            return Err(format!("{} does not open its commitment", header.address));
        }
        if header.canonical().contains(&key.to_hex()) {
            return Err(format!("{} carries its key in its header", header.address));
        }
        if !keys.insert(key.to_hex()) {
            return Err(format!(
                "{} shares its key with another event",
                header.address
            ));
        }
    }
    let purged = er(w).purged.len();
    if keys.len() + purged != log.len() {
        return Err(format!(
            "{} events hold a key and {purged} were purged, of {}",
            keys.len(),
            log.len()
        ));
    }

    Ok(())
}

#[then(
    regex = r"^the hash chain, every tombstone and every exported structure refer to event content, structural fields included, only through the commitment, but for the structural fields, provenance and recorder a tombstone keeps of a purged act, tombstone or event the key set holds$"
)]
fn only_through_the_commitment(w: &mut World) -> Result<(), String> {
    let writer = er(w).writer.ok_or("no step purged a range")?;
    let store = open(w)?;
    let log = store.log(writer).map_err(|e| e.to_string())?;
    let links: Vec<_> = log.iter().map(|s| s.link.clone()).collect();
    chain::check_writer(writer, &links).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let tombstones: Vec<Value> = store
        .log(node.device_writer())
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|s| s.link.header.kind == EventKind::Tombstone)
        .filter_map(|s| s.content)
        .collect();
    if tombstones.is_empty() {
        return Err("the purge recorded no tombstone".to_owned());
    }
    let allowed = [
        "writer",
        "first",
        "last",
        "count",
        "reason",
        "commitments",
        "policy",
    ];
    let words: Vec<String> = er(w)
        .purged
        .iter()
        .flat_map(|o| [o.content.to_string(), format!("zebra-secret-{}", o.seq)])
        .collect();
    for tombstone in &tombstones {
        let object = tombstone.as_object().ok_or("a tombstone is no object")?;
        if let Some(extra) = object.keys().find(|k| !allowed.contains(&k.as_str())) {
            return Err(format!("a tombstone holds the field {extra:?}"));
        }
        let text = tombstone.to_string();
        if let Some(word) = words.iter().find(|word| text.contains(word.as_str())) {
            return Err(format!("a tombstone holds {word:?}"));
        }
    }
    // M1 exports nothing: no structure leaves the home to name content.
    Ok(())
}

#[then(regex = r"^the commitment keys of seq (\d+)-(\d+) were erased with their content$")]
fn keys_erased(w: &mut World, from: u64, to: u64) -> Result<(), String> {
    let writer = er(w).writer.ok_or("no step purged a range")?;
    let log = open(w)?.log(writer).map_err(|e| e.to_string())?;
    for stored in log
        .iter()
        .filter(|s| (from..=to).contains(&s.link.header.address.seq.get()))
    {
        if stored.commitment_key.is_some() || stored.content.is_some() || stored.payload.is_some() {
            return Err(format!(
                "{} keeps its key or content",
                stored.link.header.address
            ));
        }
    }
    let needles: Vec<Vec<u8>> = er(w)
        .purged
        .iter()
        .map(|o| o.key.as_bytes().to_vec())
        .collect();
    match found_in_home(w, &needles)? {
        Some(found) => Err(format!("a purged event's key remains: {found}")),
        None => Ok(()),
    }
}

#[then(regex = r"^no retained or exported value confirms a guess at the purged content$")]
fn no_guess_confirmed(w: &mut World) -> Result<(), String> {
    use ::record::digest::Digest;
    let purged = er(w).purged.clone();
    if purged.is_empty() {
        return Err("no step purged a range".to_owned());
    }
    let mut needles: Vec<Vec<u8>> = Vec::new();
    for original in &purged {
        let canonical =
            ::record::canonical::encode(&original.content).map_err(|e| e.to_string())?;
        let text = original.content.as_str().map_or_else(
            || {
                original.content["text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            },
            str::to_owned,
        );
        // The content itself, and every unkeyed hash of a guess at it: a
        // retained value equal to one would confirm the guess.
        needles.push(text.as_bytes().to_vec());
        needles.push(canonical.as_bytes().to_vec());
        for guess in [
            canonical.as_bytes(),
            text.as_bytes(),
            format!("{text}\n").as_bytes(),
        ] {
            needles.push(Digest::of(&[guess]).to_hex().into_bytes());
        }
    }
    if let Some(found) = found_in_home(w, &needles)? {
        return Err(format!("a retained value confirms a guess: {found}"));
    }
    crate::erasure_prefix::closed(w)
}
