//! The steps of REC-01 to REC-04, REC-07 and REC-08 that phase 8 of plan
//! 2609292012 carries: `parinn ingest` reads the transcript roots, splits
//! each transcript by run, resumes each transcript source past its
//! ingest position, starts a new transcript generation when a source
//! shrinks or is rewritten, records a repeated line once, and `parinn
//! verify` names a source gone before it was fully ingested.
//!
//! `@REC-01`, `@REC-03` and `@REC-07` are bound; REC-07's steps for a
//! purged line are in `record_purged`. `@REC-02` stays pending:
//! the rooms a later run reaches through the `rooms` scope and its
//! restore blocks are M2's and M7's. `@REC-04` stays pending: fork
//! observations are phase 9's hook observations, live ingest phase 10's
//! and purge phase 14's. `@REC-08` stays pending: `parinn event expand`
//! is phase 13's.
//!
//! A transcript a step writes holds lines of a type no supported version
//! records, each with its own `uuid`: the record keeps each whole.

use std::fs;
use std::path::{Path, PathBuf};

use ::record::digest::Digest;
use ::record::event::EventKind;
use ::record::id::WriterId;
use cucumber::{given, then, when};
use serde_json::{Value, json};
use store::{IngestedRun, Placed, Source, Store};

use crate::World;
use crate::administration::configure;
use crate::record::{home, last_run, run_core, run_core_at_terminal};
use crate::record_transcript::recorded;
use crate::security_redaction::{append, name_transcript, transcript};

/// What the ingest steps track: where a run's transcript stood before a
/// harness resume, by name.
#[derive(Debug, Default)]
pub struct IngestSteps {
    /// The lines each transcript held before its harness resume.
    resumed_at: Vec<(String, u64)>,
    /// The lines each fork repeats of its parent.
    repeats: Vec<(String, u64)>,
}

fn steps(w: &mut World) -> &mut IngestSteps {
    w.section::<IngestSteps>()
}

/// Line `n` of the transcript `name`: a JSON object with its own `uuid`.
fn line(name: &str, n: u64) -> Value {
    json!({"type": "x", "uuid": format!("{name}-{n}"), "sessionId": name, "n": n})
}

/// Lines `from` to `to` of the transcript `name`, each ended.
pub(crate) fn lines(name: &str, from: u64, to: u64) -> String {
    (from..=to)
        .map(|n| format!("{}\n", line(name, n)))
        .collect()
}

/// Writes lines 1 to `n` into the transcript `name` when it is empty.
fn fill(w: &mut World, name: &str, n: u64) -> Result<PathBuf, String> {
    let path = transcript(w, name)?;
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() == 0 {
        fs::write(&path, lines(name, 1, n)).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

/// Runs `parinn ingest --all`, failing unless it exits 0.
fn ingest_all(w: &mut World) -> Result<(), String> {
    run_core(w, &["ingest", "--all"], None)?;
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "parinn ingest exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

pub(crate) fn open(w: &World) -> Result<Store, String> {
    Store::open(&home(w)?.store()).map_err(|e| e.to_string())
}

/// The path the store keeps the transcript at `path` by: resolved.
fn kept(path: &Path) -> Result<String, String> {
    fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Every generation of the transcript source of `name`, oldest first.
pub(crate) fn generations(w: &mut World, name: &str) -> Result<Vec<Source>, String> {
    let path = kept(&transcript(w, name)?)?;
    open(w)?.generations(&path).map_err(|e| e.to_string())
}

/// Every event read from the transcript `name`, in every generation.
pub(crate) fn placed(w: &mut World, name: &str) -> Result<Vec<Placed>, String> {
    let store = open(w)?;
    let mut out = Vec::new();
    for generation in generations(w, name)? {
        out.extend(store.placed(generation.id).map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// The ingested runs read from the transcript `name`.
fn runs_of(w: &mut World, name: &str) -> Result<Vec<IngestedRun>, String> {
    let path = kept(&transcript(w, name)?)?;
    let runs = open(w)?.ingested_runs().map_err(|e| e.to_string())?;
    Ok(runs.into_iter().filter(|r| r.path == path).collect())
}

/// The writer of the transcript `name`'s own run: the first kept.
fn own_writer(w: &mut World, name: &str) -> Result<WriterId, String> {
    runs_of(w, name)?
        .first()
        .map(|r| r.writer)
        .ok_or_else(|| format!("no run was ingested from {name:?}"))
}

/// Fails unless every event read from `name` at a line past `after` is
/// in a writer `wanted` accepts.
fn events_in(
    w: &mut World,
    name: &str,
    after: u64,
    wanted: impl Fn(WriterId) -> bool,
) -> Result<(), String> {
    let events = placed(w, name)?;
    let past: Vec<&Placed> = events.iter().filter(|p| p.place.line > after).collect();
    if past.is_empty() {
        return Err(format!("no event was read from {name:?} past line {after}"));
    }
    match past.iter().find(|p| !wanted(p.address.writer)) {
        None => Ok(()),
        Some(p) => Err(format!(
            "line {} of {name:?} is in writer {}",
            p.place.line, p.address.writer
        )),
    }
}

#[given(
    regex = r#"^the person's configuration (leaves transcript\.roots unset|sets transcript\.roots to \["([^"]+)"\])$"#
)]
fn configuration_roots(w: &mut World, _which: String, root: String) -> Result<(), String> {
    if root.is_empty() {
        return Ok(());
    }
    configure(w, &format!("[transcript]\nroots = [{root:?}]\n"))
}

#[given(regex = r#"^"([^"]+)" lies under "([^"]+)"$"#)]
fn lies_under(w: &mut World, name: String, root: String) -> Result<(), String> {
    let root = match root.strip_prefix("~/") {
        Some(rest) => w.home.join(rest),
        None => PathBuf::from(&root),
    };
    let dir = root.join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let to = dir.join(format!("{name}.jsonl"));
    let from = transcript(w, &name)?;
    if from != to {
        fs::rename(&from, &to).map_err(|e| e.to_string())?;
    }
    name_transcript(w, &name, to);
    fill(w, &name, 12).map(drop)
}

#[then(regex = r#"^every line of "([^"]+)" is stored as an event of its run$"#)]
fn every_line_stored(w: &mut World, name: String) -> Result<(), String> {
    let text = fs::read_to_string(transcript(w, &name)?).map_err(|e| e.to_string())?;
    let events = placed(w, &name)?;
    let own = own_writer(w, &name)?;
    let numbers: Vec<u64> = events.iter().map(|p| p.place.line).collect();
    let all: Vec<u64> = (1..=text.lines().count() as u64).collect();
    if numbers == all && events.iter().all(|p| p.address.writer == own) {
        return Ok(());
    }
    Err(format!(
        "the run of {name:?} holds lines {numbers:?} of {}",
        all.len()
    ))
}

#[given(regex = r#"^"([^"]+)" has (\d+) lines and has been ingested, then grows by (\d+) lines$"#)]
fn ingested_then_grows(w: &mut World, name: String, n: u64, more: u64) -> Result<(), String> {
    ingested(w, name.clone(), n)?;
    let path = transcript(w, &name)?;
    let mut text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    text.push_str(&lines(&name, n + 1, n + more));
    fs::write(&path, text).map_err(|e| e.to_string())
}

#[given(regex = r#"^"([^"]+)" has (\d+) lines and has been ingested$"#)]
fn ingested(w: &mut World, name: String, n: u64) -> Result<(), String> {
    let path = transcript(w, &name)?;
    fs::write(&path, lines(&name, 1, n)).map_err(|e| e.to_string())?;
    ingest_all(w)
}

#[then(regex = r#"^the record contains exactly (\d+) events, one per line of "([^"]+)"$"#)]
fn exactly_one_per_line(w: &mut World, n: u64, name: String) -> Result<(), String> {
    let store = open(w)?;
    let mut lines_recorded = 0;
    for writer in store.writers().map_err(|e| e.to_string())? {
        lines_recorded += store
            .log(writer)
            .map_err(|e| e.to_string())?
            .iter()
            .filter(|e| {
                matches!(
                    e.link.header.kind,
                    EventKind::TranscriptLine
                        | EventKind::UnparsedLine
                        | EventKind::RepeatReference
                )
            })
            .count() as u64;
    }
    let numbers: Vec<u64> = placed(w, &name)?.iter().map(|p| p.place.line).collect();
    let want: Vec<u64> = (1..=n).collect();
    if lines_recorded == n && numbers == want {
        return Ok(());
    }
    Err(format!(
        "the record holds {lines_recorded} transcript events, lines {numbers:?}"
    ))
}

#[then(
    regex = r#"^the ingest position of "([^"]+)" stores its byte length and the SHA-256 of its consumed prefix$"#
)]
fn position_stored(w: &mut World, name: String) -> Result<(), String> {
    let bytes = fs::read(transcript(w, &name)?).map_err(|e| e.to_string())?;
    let source = generations(w, &name)?
        .pop()
        .ok_or_else(|| format!("{name:?} has no ingest position"))?;
    let position = source.position;
    if position.offset == bytes.len() as u64 && position.prefix == Digest::of(&[&bytes]) {
        return Ok(());
    }
    Err(format!(
        "the ingest position stands at {} of {} bytes",
        position.offset,
        bytes.len()
    ))
}

#[when(
    regex = r#"^"([^"]+)" is (truncated to (\d+) lines|rewritten with a different first line)$"#
)]
fn changed(w: &mut World, name: String, how: String, keep: String) -> Result<(), String> {
    let path = transcript(w, &name)?;
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let count = text.lines().count() as u64;
    let changed = if how.starts_with("truncated") {
        let keep: u64 = keep.parse().map_err(|_| format!("{keep:?} lines"))?;
        lines(&name, 1, keep)
    } else {
        rewritten(&name, count)
    };
    fs::write(&path, changed).map_err(|e| e.to_string())
}

/// The transcript `name` of `count` lines, rewritten: another first
/// line, then its own lines 2 to `count`.
pub(crate) fn rewritten(name: &str, count: u64) -> String {
    format!(
        "{}\n{}",
        json!({"type": "x", "first": "other"}),
        lines(name, 2, count)
    )
}

#[then(
    regex = r#"^"([^"]+)" starts transcript generation (\d+) and all (\d+) events of transcript generation 1 remain in the record$"#
)]
fn new_generation(w: &mut World, name: String, generation: u64, n: usize) -> Result<(), String> {
    let generations = generations(w, &name)?;
    let newest = generations.last().map(|s| s.generation);
    let store = open(w)?;
    let first = generations.first().ok_or("no generation")?;
    let kept = store.placed(first.id).map_err(|e| e.to_string())?;
    let mut present = 0;
    for p in &kept {
        let event = store.event(p.address).map_err(|e| e.to_string())?;
        present += usize::from(event.is_some_and(|e| e.content.is_some()));
    }
    if newest == Some(generation) && kept.len() == n && present == n {
        return Ok(());
    }
    Err(format!(
        "{name:?} is at generation {newest:?}; generation 1 keeps {present} of {} events",
        kept.len()
    ))
}

#[then(regex = r#"^no audit entry records "([^"]+)"$"#)]
fn no_audit_entry_records(w: &mut World, what: String) -> Result<(), String> {
    let entries = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    match entries.iter().find(|e| e.detail == what || e.kind == what) {
        Some(_) => Err(format!("an audit entry records {what:?}")),
        None => Ok(()),
    }
}

#[given(
    regex = r#"^"([^"]+)" has (\d+) lines of which (\d+) were ingested before both transcript files were deleted$"#
)]
fn cut_short(w: &mut World, name: String, _lines: u64, landed: u64) -> Result<(), String> {
    let dir = transcript(w, "main-run")?
        .parent()
        .map(Path::to_path_buf)
        .ok_or("no transcript directory")?;
    let path = dir.join(format!("{name}.jsonl"));
    // The harness was still writing the next line when ingest read it.
    let mut text = lines(&name, 1, landed);
    text.push_str("{\"type\": \"x\", \"uuid\": \"cut");
    fs::write(&path, text).map_err(|e| e.to_string())?;
    name_transcript(w, &name, path.clone());
    ingest_all(w)?;
    let main = transcript(w, "main-run")?;
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    fs::remove_file(&main).map_err(|e| e.to_string())
}

#[then(
    regex = r#"^the output names "([^"]+)", and only it, as a transcript source that disappeared before it was fully ingested$"#
)]
fn names_lost(w: &mut World, name: String) -> Result<(), String> {
    let out = last_run(w)?;
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let lost: Vec<&str> = report["lost_sources"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let named = |p: &&str| Path::new(p).file_stem().is_some_and(|s| s == name.as_str());
    if lost.len() == 1 && lost.iter().all(named) {
        return Ok(());
    }
    Err(format!("parinn verify names {lost:?} as lost"))
}

/// The transcript directory of the run `name`.
fn directory(w: &mut World, name: &str) -> Result<PathBuf, String> {
    transcript(w, name)?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("{name:?} lies in no directory"))
}

#[given(regex = r#"^a subagent transcript "([^"]+)" of "([^"]+)" naming the agent "([^"]+)"$"#)]
fn subagent_transcript(
    w: &mut World,
    name: String,
    parent: String,
    agent: String,
) -> Result<(), String> {
    fill(w, &parent, 6)?;
    let dir = directory(w, &parent)?.join(&parent).join("subagents");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("agent-{agent}.jsonl"));
    let text: String = (1..=4)
        .map(|n| {
            let mut l = line(&name, n);
            l["agentId"] = json!(agent);
            l["sessionId"] = json!(parent);
            format!("{l}\n")
        })
        .collect();
    fs::write(&path, text).map_err(|e| e.to_string())?;
    name_transcript(w, &name, path);
    Ok(())
}

#[given(regex = r#"^"([^"]+)" continues after a harness resume that kept its harness session$"#)]
fn resumed(w: &mut World, name: String) -> Result<(), String> {
    let path = fill(w, &name, 6)?;
    ingest_all(w)?;
    let mut text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let before = text.lines().count() as u64;
    text.push_str(&lines(&name, 100, 104));
    fs::write(&path, text).map_err(|e| e.to_string())?;
    steps(w).resumed_at.push((name, before));
    Ok(())
}

/// Writes the transcript `name` beside the run `beside`'s, holding
/// `text`.
fn beside(w: &mut World, beside: &str, name: &str, text: &str) -> Result<(), String> {
    let path = directory(w, beside)?.join(format!("{name}.jsonl"));
    fs::write(&path, text).map_err(|e| e.to_string())?;
    name_transcript(w, name, path);
    Ok(())
}

#[given(
    regex = r#"^a harness resume of "([^"]+)" that started a new harness session, with the transcript "([^"]+)"$"#
)]
fn resumed_anew(w: &mut World, parent: String, name: String) -> Result<(), String> {
    fill(w, &parent, 6)?;
    beside(w, &parent, &name, &lines(&name, 1, 3))
}

#[given(regex = r#"^a harness fork of "([^"]+)", with the transcript "([^"]+)"$"#)]
fn forked(w: &mut World, parent: String, name: String) -> Result<(), String> {
    let path = fill(w, &parent, 6)?;
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut fork = String::new();
    let repeats = 3;
    for original in text.lines().take(repeats) {
        let mut repeated: Value = serde_json::from_str(original).map_err(|e| e.to_string())?;
        repeated["sessionId"] = json!(name);
        fork.push_str(&format!("{repeated}\n"));
    }
    fork.push_str(&lines(&name, 50, 52));
    beside(w, &parent, &name, &fork)?;
    steps(w).repeats.push((name, repeats as u64));
    Ok(())
}

#[given(
    regex = r#"^a compaction of "([^"]+)" whose summary the harness wrote with an agent of its own$"#
)]
fn compacted(w: &mut World, name: String) -> Result<(), String> {
    fill(w, &name, 6)?;
    let mut summary = recorded("a compaction summary", |l| l["isCompactSummary"] == true)?;
    summary["sessionId"] = json!(name);
    append(w, &name, &summary)
}

#[then(
    regex = r#"^the run ingested from "([^"]+)" is tied to the run of "([^"]+)" by a parent link, not a delegation link$"#
)]
fn parent_link(w: &mut World, name: String, parent: String) -> Result<(), String> {
    let sub = runs_of(w, &name)?;
    let parent_run = runs_of(w, &parent)?;
    let tied = sub.first().zip(parent_run.first()).is_some_and(|(s, p)| {
        s.agent_id.is_some() && p.agent_id.is_none() && s.harness_session == p.harness_session
    });
    if tied {
        return Ok(());
    }
    Err(format!(
        "{name:?}'s runs {sub:?} are not tied to {parent:?}'s {parent_run:?}"
    ))
}

#[then(regex = r#"^the run ingested from "([^"]+)" records the harness's agent id "([^"]+)"$"#)]
fn agent_id(w: &mut World, name: String, agent: String) -> Result<(), String> {
    let runs = runs_of(w, &name)?;
    if runs.len() == 1 && runs[0].agent_id.as_deref() == Some(agent.as_str()) {
        return Ok(());
    }
    Err(format!("{name:?}'s runs are {runs:?}"))
}

#[then(
    regex = r#"^the events "([^"]+)" gained after the harness resume that kept its harness session belong to the run of "([^"]+)"$"#
)]
fn resumed_events(w: &mut World, name: String, parent: String) -> Result<(), String> {
    let after = steps(w)
        .resumed_at
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, at)| *at)
        .ok_or("no step resumed it")?;
    let own = own_writer(w, &parent)?;
    events_in(w, &name, after, |writer| writer == own)
}

#[then(regex = r#"^the events of "([^"]+)" belong to a new run, not to the run of "([^"]+)"$"#)]
fn new_run(w: &mut World, name: String, parent: String) -> Result<(), String> {
    let parent_writer = own_writer(w, &parent)?;
    let own = own_writer(w, &name)?;
    events_in(w, &name, 0, |writer| {
        writer == own && writer != parent_writer
    })
}

#[then(
    regex = r#"^the events "([^"]+)" adds after the lines it repeats belong to a new run, not to the run of "([^"]+)"$"#
)]
fn forked_events(w: &mut World, name: String, parent: String) -> Result<(), String> {
    let after = steps(w)
        .repeats
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, k)| *k)
        .ok_or("no step forked it")?;
    let parent_writer = own_writer(w, &parent)?;
    let own = own_writer(w, &name)?;
    events_in(w, &name, after, |writer| {
        writer == own && writer != parent_writer
    })
}

#[then(
    regex = r#"^the run count of the recorded Claude Code 2\.1\.296 harness session "subagent-compaction" is its main run plus its one subagent, none for the harness's own call that wrote each compaction summary$"#
)]
fn compaction_runs(w: &mut World) -> Result<(), String> {
    let recorded =
        testkit::repo_root().join("testdata/claude-code/2.1.296/transcripts/projects/-w-sub2");
    let session = "ff036475-48bc-4843-ae69-a8fe3aba87ea";
    let main = recorded.join(format!("{session}.jsonl"));
    let placed = testkit::place_transcript(&w.home, &main).map_err(|e| e.to_string())?;
    let dir = placed.parent().ok_or("no project directory")?.to_path_buf();
    let subagents = recorded.join(session).join("subagents");
    let into = dir.join(session).join("subagents");
    fs::create_dir_all(&into).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(&subagents).map_err(|e| e.to_string())? {
        let from = entry.map_err(|e| e.to_string())?.path();
        if from.extension().is_some_and(|e| e == "jsonl") {
            let name = from.file_name().ok_or("no name")?;
            fs::copy(&from, into.join(name)).map_err(|e| e.to_string())?;
        }
    }
    ingest_all(w)?;
    let dir = kept(&dir)?;
    let runs: Vec<IngestedRun> = open(w)?
        .ingested_runs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|r| r.path.starts_with(&dir))
        .collect();
    let subagent_runs = runs.iter().filter(|r| r.agent_id.is_some()).count();
    if runs.len() == 2 && subagent_runs == 1 {
        return Ok(());
    }
    Err(format!(
        "the recorded harness session gave the runs {runs:?}"
    ))
}

#[then(regex = r#"^after a harness clear of "([^"]+)", its later events belong to a new run$"#)]
fn cleared(w: &mut World, parent: String) -> Result<(), String> {
    beside(w, &parent, "cleared-run", &lines("cleared-run", 1, 3))?;
    ingest_all(w)?;
    let parent_writer = own_writer(w, &parent)?;
    let own = own_writer(w, "cleared-run")?;
    events_in(w, "cleared-run", 0, |writer| {
        writer == own && writer != parent_writer
    })
}

#[then(
    regex = r#"^that new run belongs to the agent of "([^"]+)" only when the harness gives an agent id stable across harness sessions, else it is its own agent$"#
)]
fn own_agent(w: &mut World, parent: String) -> Result<(), String> {
    // Claude Code gives a main agent no id stable across harness
    // sessions, so the cleared run is its own agent: a run of its own,
    // naming no agent id it could share.
    let cleared = runs_of(w, "cleared-run")?;
    let parent_runs = runs_of(w, &parent)?;
    let own = cleared.len() == 1
        && cleared[0].agent_id.is_none()
        && parent_runs.iter().all(|p| p.writer != cleared[0].writer);
    if own {
        return Ok(());
    }
    Err(format!("the cleared run is {cleared:?}"))
}

#[when(
    regex = r#"^the hook "SessionStart" runs with the session_id of "([^"]+)" and the transcript_path of "([^"]+)"$"#
)]
fn hook_crossed(w: &mut World, session: String, name: String) -> Result<(), String> {
    fill(w, &session, 6)?;
    let path = transcript(w, &name)?.display().to_string();
    let input = json!({
        "session_id": session,
        "transcript_path": path,
        "cwd": w.home.display().to_string(),
        "hook_event_name": "SessionStart",
        "source": "startup",
    });
    run_core(
        w,
        &["hook", "SessionStart"],
        Some(input.to_string().as_bytes()),
    )
}

#[then(
    regex = r#"^every event read from "([^"]+)" is attributed to the subagent run of "([^"]+)"$"#
)]
fn attributed_to_subagent(w: &mut World, name: String, _same: String) -> Result<(), String> {
    let runs = runs_of(w, &name)?;
    let sub = runs
        .iter()
        .find(|r| r.agent_id.is_some())
        .map(|r| r.writer)
        .ok_or_else(|| format!("no subagent run was read from {name:?}"))?;
    events_in(w, &name, 0, |writer| writer == sub)
}

#[then(regex = r#"^no event read from "([^"]+)" is attributed to "([^"]+)"$"#)]
fn not_attributed(w: &mut World, name: String, parent: String) -> Result<(), String> {
    let parent_writer = own_writer(w, &parent)?;
    events_in(w, &name, 0, |writer| writer != parent_writer)
}

#[given(
    regex = r#"^a transcript "([^"]+)" in the transcript directory of "([^"]+)" that repeats the uuid of a line of "([^"]+)" with other text$"#
)]
fn planted(w: &mut World, name: String, parent: String, _same: String) -> Result<(), String> {
    fill(w, &parent, 6)?;
    let mut copy = line(&parent, 1);
    copy["n"] = json!("other text");
    beside(w, &parent, &name, &format!("{copy}\n"))
}

#[then(
    regex = r#"^that line of "([^"]+)" is recorded as its own event, the counter "repeat_conflicts" increases by 1 and an audit entry records the conflict$"#
)]
fn conflict(w: &mut World, name: String) -> Result<(), String> {
    let events = placed(w, &name)?;
    let own = events.len() == 1 && events[0].kind != EventKind::RepeatReference;
    let audit = home(w)?.audit();
    let entries = audit::log::Log::new(audit)
        .entries()
        .map_err(|e| e.to_string())?;
    let conflicts = entries
        .iter()
        .filter(|e| e.counter.as_deref() == Some("repeat_conflicts"))
        .count();
    if own && conflicts == 1 {
        return Ok(());
    }
    Err(format!(
        "{name:?} holds {events:?}; {conflicts} audit entries count a conflict"
    ))
}

// REC-22: events `parinn ingest` appends are `ingested` and untrusted,
// and a transcript from outside the roots is ingested only when the
// person names it on `parinn ingest --path` at a terminal. The scenario
// stays pending: no surface shows a freshness mark yet (VIEW-04), and
// its first `Then` step says so.

/// The line `parinn ingest` found past the ingest marker of "cut-short":
/// the server read this many lines of it.
const CUT_AT: u64 = 3;

#[given(
    regex = r#"^a transcript "([^"]+)" under the configured transcript roots, written before ParleyInn's hook handlers were installed, containing a user turn and lifecycle metadata$"#
)]
fn pre_install(w: &mut World, name: String) -> Result<(), String> {
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.jsonl"));
    let lines = [
        crate::provenance::typed()?,
        recorded("a mode line", |l| l["type"] == "mode")?,
    ];
    let body: String = lines.iter().map(|l| format!("{l}\n")).collect();
    fs::write(&path, body).map_err(|e| e.to_string())?;
    name_transcript(w, &name, path);
    Ok(())
}

#[given(regex = r#"^a transcript "([^"]+)" from outside the configured transcript roots$"#)]
fn outside_roots(w: &mut World, name: String) -> Result<(), String> {
    let dir = w.home.join("elsewhere");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.jsonl"));
    let body: String = [crate::provenance::typed()?, crate::provenance::typed()?]
        .iter()
        .map(|l| format!("{l}\n"))
        .collect();
    fs::write(&path, body).map_err(|e| e.to_string())?;
    name_transcript(w, &name, path);
    Ok(())
}

#[given(regex = r#"^a watched transcript "([^"]+)" whose hook handler left an ingest marker$"#)]
fn watched_cut_short(w: &mut World, name: String) -> Result<(), String> {
    crate::record_live::make_run(w, &name, 7, None)?;
    let path = crate::record_live::named(w, &name)?;
    name_transcript(w, &name, path.clone());
    let limit = CUT_AT.to_string();
    crate::record_live::serve(w, &name, "startup", &[("PARINN_TEST_INGEST_LIMIT", &limit)])?;
    crate::live::wait_until("the MCP server to read its lines", || {
        Ok(crate::live::lines_read(w, &path)? >= CUT_AT)
    })?;
    let harness = crate::record_live::live(w)
        .harness
        .take()
        .ok_or("no MCP server is running")?;
    harness.stop()
}

#[when(
    regex = r#"^the person runs "parinn ingest --path" on "([^"]+)", on "([^"]+)" and on "([^"]+)"$"#
)]
fn ingest_three(w: &mut World, a: String, b: String, c: String) -> Result<(), String> {
    // A widening act on a node with runs needs the standing risk
    // acceptance (OWN-22).
    run_core_at_terminal(w, &["risk-acceptance", "record"])?;
    for name in [a, b, c] {
        let shown = transcript(w, &name)?.display().to_string();
        run_core_at_terminal(w, &["ingest", "--path", &shown])?;
        let out = last_run(w)?;
        if !out.status.success() {
            return Err(format!(
                "parinn ingest of {name:?} exited {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stdout).trim()
            ));
        }
    }
    Ok(())
}

/// Every event of the transcript `name`, recalled through `event_get`,
/// as (line, item).
fn recalled_events(w: &mut World, name: &str) -> Result<Vec<(u64, Value)>, String> {
    let mut out = Vec::new();
    for p in placed(w, name)? {
        crate::record_transcript::set_item(w, p.address.writer.to_string(), p.address.seq.get());
        out.push((p.place.line, crate::record_transcript::recalled_item(w)?));
    }
    if out.is_empty() {
        return Err(format!("no event was read from {name:?}"));
    }
    Ok(out)
}

#[then(
    regex = r#"^every event of "([^"]+)" carries the "ingested" origin, freshness mark and trust mark with its transcript source and ingest position, shown on every surface, and none is shown as witnessed$"#
)]
fn carries_origin(w: &mut World, name: String) -> Result<(), String> {
    let source = kept(&transcript(w, &name)?)?;
    let store = open(w)?;
    let recorded_source = store
        .source(&source)
        .map_err(|e| e.to_string())?
        .ok_or("no transcript source is recorded")?;
    if recorded_source.position.lines == 0 {
        return Err("the ingest position is not recorded".to_owned());
    }
    for (line, item) in recalled_events(w, &name)? {
        if item["origin"] != "ingested" || item["trust"] != "untrusted" {
            return Err(format!("line {line} is recalled as {item}"));
        }
        if item.get("freshness").is_none() {
            return Err(format!(
                "line {line} shows no freshness mark: no surface shows one before VIEW-04"
            ));
        }
    }
    Ok(())
}

#[then(
    regex = r#"^the "user" event and the "harness_meta" event from "([^"]+)" have trust "untrusted"$"#
)]
fn user_and_meta_untrusted(w: &mut World, name: String) -> Result<(), String> {
    // The user turn is the transcript's "user" line; no hook handler saw
    // its prompt, so it is not a "user" provenance (PRV-08).
    let line_type = |item: &Value| {
        let line: Value = serde_json::from_str(item["content"].as_str().unwrap_or_default())
            .unwrap_or(Value::Null);
        line["type"].as_str().map(str::to_owned)
    };
    let events = recalled_events(w, &name)?;
    for (what, is) in [
        (
            "user",
            &(|i: &Value| line_type(i).as_deref() == Some("user")) as &dyn Fn(&Value) -> bool,
        ),
        ("harness_meta", &|i: &Value| {
            i["provenance"] == "harness_meta"
        }),
    ] {
        let mut found = false;
        for (line, item) in events.iter().filter(|(_, i)| is(i)) {
            found = true;
            if item["trust"] != "untrusted" {
                return Err(format!("line {line} is recalled as {item}"));
            }
        }
        if !found {
            return Err(format!("{name:?} holds no {what} event"));
        }
    }
    Ok(())
}

#[then(
    regex = r#"^this node records "([^"]+)" as an ingested run in the principal's personal room, shown as "ingested", and every event of it has trust "untrusted"$"#
)]
fn personal_room_run(w: &mut World, name: String) -> Result<(), String> {
    let runs = runs_of(w, &name)?;
    let run = runs.first().ok_or("no run was ingested from the path")?;
    let store = open(w)?;
    let node = administration::node::Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let seat = store
        .seat(run.writer)
        .map_err(|e| e.to_string())?
        .ok_or("the run has no seat")?;
    if seat.room != node.personal_room() {
        return Err("the run is not in the personal room".to_owned());
    }
    for (line, item) in recalled_events(w, &name)? {
        if item["origin"] != "ingested" || item["trust"] != "untrusted" {
            return Err(format!("line {line} is recalled as {item}"));
        }
    }
    Ok(())
}

#[then(
    regex = r#"^ingesting "([^"]+)" is recorded as the person's widening principal act at a terminal \(OWN-11, OWN-12\), and the same path named in a hook input or a transcript line is rejected as outside the allowed roots \(SEC-18\)$"#
)]
fn recorded_as_act(w: &mut World, name: String) -> Result<(), String> {
    let path = transcript(w, &name)?;
    crate::security_hooks::names_act(w, &fs::canonicalize(&path).map_err(|e| e.to_string())?)?;
    crate::security_hooks::hook_names(w, &path.display().to_string(), &w.home.clone())?;
    crate::security_hooks::wait_refused(w)
}

#[then(
    regex = r#"^every event "parinn ingest" read past the ingest marker of "([^"]+)" carries the "ingested" origin and has trust "untrusted"$"#
)]
fn past_the_marker(w: &mut World, name: String) -> Result<(), String> {
    let mut past = 0;
    for (line, item) in recalled_events(w, &name)? {
        if line > CUT_AT {
            past += 1;
            if item["origin"] != "ingested" || item["trust"] != "untrusted" {
                return Err(format!("line {line} is recalled as {item}"));
            }
        } else if item["origin"] != "witnessed" {
            return Err(format!("line {line} was read by the server: {item}"));
        }
    }
    if past == 0 {
        return Err("parinn ingest read nothing past the marker".to_owned());
    }
    Ok(())
}
