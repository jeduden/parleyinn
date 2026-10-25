//! The steps of SRS §5.1a, administration (ADM). `@ADM-04`'s rows are
//! bound here: a settings layer that fails validation fails `parinn
//! status`, naming the key and the problem; a widening setting stands
//! until a widening principal act accepts it; a repository
//! configuration's widening key is ignored and counted; managed policy
//! the person's OS user can write is refused and counted. The core
//! records every event of an agent's run, except while the principal's
//! configuration fails: then it refuses the ingest, counted, since the
//! redaction patterns it names are unknown (SEC-08). Whether flags stay
//! on is PRV-07's to show.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use audit::log::Log;
use cucumber::{given, then, when};
use serde_json::Value;

use crate::World;
use crate::record::{MANAGED_POLICY, home, last_run, run_core};

/// The transcript an agent's run leaves: one recorded harness session.
const TRANSCRIPT: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl";

/// What the administration steps track: the lines the agent's run left,
/// the events `parinn ingest` recorded of them, and its refusal when it
/// refused.
#[derive(Debug, Default)]
pub struct AdministrationSteps {
    lines: usize,
    recorded: Option<u64>,
    refused: Option<String>,
}

fn adm(w: &mut World) -> &mut AdministrationSteps {
    w.section::<AdministrationSteps>()
}

/// Writes the principal's configuration in the scenario's home,
/// owner-only as the home wants it.
pub fn configure(w: &World, text: &str) -> Result<(), String> {
    let home = crate::record::home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let path = w.parinn_home.join("config.toml");
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())
}

/// An agent's run: the core ingests its transcript, as `parinn ingest`
/// does, and the step keeps how many lines it left and how many events
/// were recorded; it fails when the ingest does.
pub fn agent_runs(w: &mut World) -> Result<(), String> {
    agent_runs_or_refused(w)?;
    match adm(w).refused.take() {
        Some(refused) => Err(refused),
        None => Ok(()),
    }
}

/// An agent's run, as [`agent_runs`], keeping a refused ingest for a
/// later step to judge.
fn agent_runs_or_refused(w: &mut World) -> Result<(), String> {
    let path = testkit::repo_root().join(TRANSCRIPT);
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let placed = testkit::place_transcript(&w.home, &path).map_err(|e| e.to_string())?;
    let shown = placed.display().to_string();
    run_core(w, &["ingest", "--path", &shown, "--json"], None)?;
    let out = last_run(w)?;
    let lines = text.lines().count();
    if !out.status.success() {
        let refused = format!(
            "parinn ingest exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
        let state = adm(w);
        (state.lines, state.recorded, state.refused) = (lines, None, Some(refused));
        return Ok(());
    }
    let report: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let recorded = report["events"].as_u64();
    let state = adm(w);
    (state.lines, state.recorded, state.refused) = (lines, recorded, None);
    Ok(())
}

#[given(regex = r#"^the person's config\.toml containing "([^"]*)"$"#)]
fn config_containing(w: &mut World, setting: String) -> Result<(), String> {
    configure(w, &format!("{setting}\n"))
}

/// A repository holding the scenario's home: `.parinn.toml` beside a
/// `.git` at its top, the working directory every command starts in.
#[given(regex = r#"^a repository "\.parinn\.toml" containing "([^"]*)"$"#)]
fn repository_containing(w: &mut World, setting: String) -> Result<(), String> {
    let git = w.home.join(".git");
    fs::create_dir_all(&git).map_err(|e| format!("{}: {e}", git.display()))?;
    let path = w.home.join(".parinn.toml");
    fs::write(&path, format!("{setting}\n")).map_err(|e| format!("{}: {e}", path.display()))
}

/// Managed policy the scenario's own OS user owns, so can write.
#[given(regex = r#"^a managed policy file the person's OS user can write, containing "([^"]*)"$"#)]
fn writable_managed_policy(w: &mut World, setting: String) -> Result<(), String> {
    let path = w.home.join(MANAGED_POLICY);
    fs::write(&path, format!("{setting}\n")).map_err(|e| format!("{}: {e}", path.display()))
}

#[when(regex = r#"^an agent runs and the person starts "parinn ([^"]+)"$"#)]
fn agent_runs_and_person_starts(w: &mut World, args: String) -> Result<(), String> {
    agent_runs_or_refused(w)?;
    let args: Vec<&str> = args.split_whitespace().collect();
    run_core(w, &args, None)
}

#[then(
    regex = r#"^"parinn [^"]+" exits (\d+), and the error names the key "([^"]+)" and the problem "([^"]+)"$"#
)]
fn exits_naming(w: &mut World, code: i32, key: String, problem: String) -> Result<(), String> {
    let out = last_run(w)?;
    let err = String::from_utf8_lossy(&out.stderr);
    let named = format!("key \"{key}\": {problem}");
    if out.status.code() == Some(code) && err.contains(&named) {
        return Ok(());
    }
    Err(format!(
        "the command exited {}, wanting {code} with {named:?}: {}",
        out.status,
        err.trim()
    ))
}

#[then(
    regex = r#"^"parinn [^"]+" exits 0, shows deployment mode "([^"]+)" and "([^"]+)" awaiting acceptance: it stands as it stood until a widening principal act records that file's digest$"#
)]
fn shows_awaiting(w: &mut World, mode: String, key: String) -> Result<(), String> {
    let shown = succeeded(w)?;
    let lines = [
        format!("deployment mode: {mode}"),
        format!("awaiting acceptance (parinn configuration accept): {key}"),
    ];
    shows_lines(&shown, &lines)
}

#[then(
    regex = r#"^"parinn [^"]+" exits 0, shows deployment mode "([^"]+)" and counts "([^"]+)": repository configuration only tightens$"#
)]
fn shows_ignored(w: &mut World, mode: String, counter: String) -> Result<(), String> {
    let shown = succeeded(w)?;
    shows_lines(&shown, &[format!("deployment mode: {mode}")])?;
    counted(&shown, &counter)
}

#[then(
    regex = r#"^"parinn [^"]+" exits 0, shows managed policy refused as writable by the person's OS user and counts "([^"]+)"$"#
)]
fn shows_managed_refused(w: &mut World, counter: String) -> Result<(), String> {
    let shown = succeeded(w)?;
    let refused = shown.lines().any(|l| {
        l.starts_with("managed policy refused: ")
            && l.contains("is writable by the running OS user, so it is no managed policy")
    });
    if !refused {
        return Err(format!("no refused managed policy in:\n{shown}"));
    }
    counted(&shown, &counter)
}

/// The last command's stdout, once it exited 0.
fn succeeded(w: &mut World) -> Result<String, String> {
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "the command exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Checks `shown` holds each of `lines` as a whole line.
fn shows_lines(shown: &str, lines: &[String]) -> Result<(), String> {
    match lines
        .iter()
        .find(|l| !shown.lines().any(|s| s == l.as_str()))
    {
        None => Ok(()),
        Some(missing) => Err(format!("no line {missing:?} in:\n{shown}")),
    }
}

/// Checks `parinn status`'s counters line counts `counter` at least once.
fn counted(shown: &str, counter: &str) -> Result<(), String> {
    let line = shown
        .lines()
        .find_map(|l| l.strip_prefix("counters: "))
        .unwrap_or_default();
    let total = line
        .split(", ")
        .find_map(|c| c.strip_prefix(&format!("{counter} ")))
        .and_then(|rest| rest.split(' ').next())
        .and_then(|n| n.parse::<u64>().ok());
    match total {
        Some(n) if n > 0 => Ok(()),
        _ => Err(format!("{counter} is not counted in:\n{shown}")),
    }
}

#[then(
    regex = r#"^the core refuses the run's ingest, audits and counts it in "([^"]+)", and records none of it$"#
)]
fn refuses_ingest(w: &mut World, counter: String) -> Result<(), String> {
    let state = adm(w);
    if state.refused.is_none() || state.recorded.is_some() {
        return Err(format!(
            "the ingest was not refused: recorded {:?}",
            state.recorded
        ));
    }
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    if !entries
        .iter()
        .any(|e| e.counter.as_deref() == Some(counter.as_str()))
    {
        return Err(format!("no audit entry counts {counter:?}"));
    }
    if home(w)?.store().exists() {
        return Err("the refused ingest left a store".to_owned());
    }
    Ok(())
}

#[then(regex = r"^the core records every event of the run$")]
fn records_every_event(w: &mut World) -> Result<(), String> {
    if let Some(refused) = adm(w).refused.clone() {
        return Err(refused);
    }
    let state = adm(w);
    if state.recorded == Some(state.lines as u64) && state.lines > 0 {
        return Ok(());
    }
    Err(format!(
        "the run left {} lines and the core recorded {:?} events",
        state.lines, state.recorded
    ))
}
