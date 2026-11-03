//! The steps of SEC-16 that phase 9 of plan 2609292012 carries: a hook
//! input whose field fails the schema is rejected fail-open, injecting
//! nothing, audited and counted. `@SEC-16` is bound.
//!
//! It also carries SEC-18's: a path a hook input names, found by the run's
//! MCP server, is read only within the transcript roots or the run's
//! worktree, and one the person names on `parinn ingest --path` at a
//! terminal is a widening principal act (REC-22). The hook rows start a
//! stand-in harness ([`crate::live`]) whose MCP server takes the session
//! binding; the person rows run the core at a terminal.

use std::fs;
use std::path::{Path, PathBuf};

use administration::act::acts;
use administration::node::Node;
use audit::log::Log;
use cucumber::{given, then, when};
use serde_json::json;

use crate::World;
use crate::administration::configure;
use crate::live::{Harness, lines_read, source_key, store, wait_until};
use crate::provenance::typed;
use crate::record::{home, last_run, run_core, run_core_at_terminal};
use crate::record_hooks::{SESSION, answered_empty, session_start};
use crate::record_live::taken_bindings;
use crate::record_transcript::{recalled_item, remember_counters, set_item};
use crate::security_redaction::found_in_home;

#[when(regex = r#"^the hook "([A-Za-z]+)" runs with a hook input whose "([a-z_]+)" is a number$"#)]
fn hook_input_with_a_number(w: &mut World, hook: String, field: String) -> Result<(), String> {
    remember_counters(w)?;
    let mut input = session_start(w, SESSION, "startup")?;
    input[field.as_str()] = json!(42);
    input["hook_event_name"] = json!(hook);
    run_core(w, &["hook", &hook], Some(input.to_string().as_bytes()))
}

#[then(regex = r"^the hook handler exits 0 with empty output and no injection$")]
fn exits_0_empty(w: &mut World) -> Result<(), String> {
    answered_empty(w)
}

/// An AWS access key, which the redaction rules match.
const KEY: &str = "AKIAIOSFODNN7EXAMPLE";
/// What the private key and the system file hold, to find them in the record.
const PRIVATE_KEY: &str = "BEGIN OPENSSH PRIVATE KEY";

/// What the path steps track: the run's `cwd`, and the path a step named
/// with how it was named.
#[derive(Debug, Default)]
pub struct PathSteps {
    cwd: Option<PathBuf>,
    path: Option<PathBuf>,
    by_person: bool,
    by_configuration: bool,
    configuration_transcript: Option<PathBuf>,
    /// The bytes of the file the path resolves to, before it was named.
    before: Option<Vec<u8>>,
    /// Every file of the repository, with its bytes, before it was named.
    repository: Vec<(PathBuf, Vec<u8>)>,
}

fn paths(w: &mut World) -> &mut PathSteps {
    w.section::<PathSteps>()
}

/// `text` with `~/` the scenario's home and `/tmp/` its own `tmp`.
fn expand(w: &World, text: &str) -> PathBuf {
    if let Some(rest) = text.strip_prefix("~/") {
        return w.home.join(rest);
    }
    match text.strip_prefix("/tmp/") {
        Some(rest) => w.home.join("tmp").join(rest),
        None => PathBuf::from(text),
    }
}

#[given(regex = r#"^the transcript roots are "([^"]+)"$"#)]
fn roots_are(w: &mut World, root: String) -> Result<(), String> {
    configure(w, &format!("[transcript]\nroots = [{root:?}]\n"))
}

#[given(
    regex = r#"^a run whose hook "cwd" is "([^"]+)", inside a repository whose top level is "([^"]+)"$"#
)]
fn run_in_repository(w: &mut World, cwd: String, top: String) -> Result<(), String> {
    let (cwd, top) = (expand(w, &cwd), expand(w, &top));
    let git = top.join(".git");
    for dir in [cwd.clone(), git.clone(), top.join("docs")] {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").map_err(|e| e.to_string())?;
    let ssh = w.home.join(".ssh");
    fs::create_dir_all(&ssh).map_err(|e| e.to_string())?;
    fs::write(ssh.join("id_ed25519"), format!("-----{PRIVATE_KEY}-----\n"))
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(w.home.join(".claude").join("projects").join("p"))
        .map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    crate::security_redaction::home_files(&top, &mut files)?;
    let steps = paths(w);
    steps.cwd = Some(cwd);
    steps.repository = files;
    Ok(())
}

/// A transcript of one line holding a secret, at `path`.
fn write_transcript(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("no parent")?).map_err(|e| e.to_string())?;
    let line = json!({"type": "atis-latch", "atis": KEY, "sessionId": "s"});
    fs::write(path, format!("{line}\n")).map_err(|e| e.to_string())
}

/// Makes what the table's `shown` path stands for, and the path.
fn fixture(w: &mut World, shown: &str) -> Result<PathBuf, String> {
    let (text, note) = match shown.split_once(" (") {
        Some((text, note)) => (text, note.trim_end_matches(')')),
        None => (shown, ""),
    };
    let path = expand(w, text);
    let parent = path.parent().ok_or("no parent")?;
    if let Some(target) = note.strip_prefix("a symlink to ") {
        let target = expand(w, target);
        if target != Path::new("/") && !target.exists() {
            write_transcript(&target)?;
        }
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        std::os::unix::fs::symlink(&target, &path).map_err(|e| e.to_string())?;
    } else if note == "a directory" {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    } else if note.starts_with("a regular file") {
        write_transcript(&path)?;
    }
    Ok(path)
}

#[when(
    regex = r#"^(the hook "SessionStart" input|a transcript line the run's MCP server ingests|the accepted configuration setting "transcript.roots"|the person, on "parinn ingest --path" at a terminal) names the path "(.+)"$"#
)]
fn names_the_path(w: &mut World, by: String, shown: String) -> Result<(), String> {
    let path = fixture(w, &shown)?;
    let by_person = by.starts_with("the person");
    let by_transcript = by.starts_with("a transcript line");
    let by_configuration = by.starts_with("the accepted configuration");
    let target = fs::canonicalize(&path).ok();
    let before = target.as_ref().and_then(|t| fs::read(t).ok());
    let steps = paths(w);
    steps.path = Some(path.clone());
    steps.by_person = by_person;
    steps.by_configuration = by_configuration;
    steps.before = before;
    let shown = path.display().to_string();
    if by_person {
        return run_core_at_terminal(w, &["ingest", "--path", &shown]);
    }
    let cwd = paths(w).cwd.clone().ok_or("no step named the run's cwd")?;
    if by_configuration {
        let target = fs::canonicalize(&path).map_err(|e| e.to_string())?;
        let transcript = target.join("configuration-source.jsonl");
        write_transcript(&transcript)?;
        paths(w).configuration_transcript = Some(transcript);
        configure(w, &format!("[transcript]\nroots = [{shown:?}]\n"))?;
        run_core_at_terminal(w, &["configuration", "accept"])?;
        return run_core(w, &["ingest", "--all"], None);
    }
    if by_transcript {
        return transcript_names(w, &shown, &cwd);
    }
    hook_names(w, &shown, &cwd)
}

/// Starts a run from a transcript inside the configured root whose line
/// names `path` in its harness `cwd` field. The line, rather than the
/// `SessionStart` hook input, is the path's source.
fn transcript_names(w: &mut World, path: &str, cwd: &Path) -> Result<(), String> {
    let transcript = w
        .home
        .join(".claude")
        .join("projects")
        .join("path-from-line.jsonl");
    let mut line = typed()?;
    line["cwd"] = json!(path);
    line["sessionId"] = json!(SESSION);
    fs::write(&transcript, format!("{line}\n")).map_err(|e| e.to_string())?;
    hook_names(w, &transcript.display().to_string(), cwd)?;
    wait_until("the MCP server to ingest the transcript line", || {
        Ok(lines_read(w, &transcript)? >= 1)
    })
}

/// Runs the `SessionStart` hook handler of a new harness, its input
/// naming the transcript `path` and the worktree `cwd`, and waits until
/// the harness's MCP server took the session binding.
pub(crate) fn hook_names(w: &mut World, path: &str, cwd: &Path) -> Result<(), String> {
    let mut input = session_start(w, SESSION, "startup")?;
    input["transcript_path"] = json!(path);
    input["cwd"] = json!(cwd);
    let mut harness = Harness::start(w, true, &[])?;
    harness.hook("SessionStart", &input)?;
    crate::record_live::live(w).harness = Some(harness);
    wait_until("the MCP server to take its session binding", || {
        Ok(taken_bindings(w, SESSION)? > 0)
    })
}

/// The audit entries of `operation` whose detail is `detail`.
fn refusals(w: &World, operation: Option<&str>, detail: &str) -> Result<usize, String> {
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;

    Ok(entries
        .iter()
        .filter(|e| e.detail == detail && operation.is_none_or(|o| e.operation == o))
        .count())
}

/// Stops the harness a hook row started, its MCP server stopped with it.
fn stop_server(w: &mut World) -> Result<(), String> {
    match crate::record_live::live(w).harness.take() {
        Some(harness) => harness.stop(),
        None => Ok(()),
    }
}

/// Fails when the record holds anything of what the path names: a source
/// of it, or the private key or the system file's text.
fn nothing_read(w: &mut World) -> Result<(), String> {
    let path = paths(w).path.clone().ok_or("no step named a path")?;
    if home(w)?.store().exists() {
        let store = store(w)?;
        if store
            .source(&source_key(&path))
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Err("the record holds a source of the path".to_owned());
        }
        for writer in store.writers().map_err(|e| e.to_string())? {
            for event in store.log(writer).map_err(|e| e.to_string())? {
                let text = event.content.map(|c| c.to_string()).unwrap_or_default();
                if text.contains(PRIVATE_KEY) || text.contains("root:x:") || text.contains(KEY) {
                    return Err(format!("writer {writer} recorded what the path names"));
                }
            }
        }
    }
    Ok(())
}

/// Waits until the MCP server of a hook row refused the path, then stops
/// it.
pub(crate) fn wait_refused(w: &mut World) -> Result<(), String> {
    wait_until("the MCP server to refuse the path", || {
        Ok(refusals(w, Some("mcp"), "path outside allowed roots")? > 0)
    })?;
    stop_server(w)
}

#[then(
    regex = r#"^the path is rejected, nothing is read from it, and an audit entry records "path outside allowed roots"$"#
)]
fn rejected(w: &mut World) -> Result<(), String> {
    if !paths(w).by_person {
        wait_refused(w)?;
    } else if refusals(w, None, "path outside allowed roots")? == 0 {
        return Err("no audit entry records the path outside allowed roots".to_owned());
    }
    nothing_read(w)
}

#[then(regex = r"^the file is read, and nothing in the repository is written$")]
fn read_not_written(w: &mut World) -> Result<(), String> {
    let path = paths(w).path.clone().ok_or("no step named a path")?;
    wait_until("the MCP server to read the file", || {
        Ok(lines_read(w, &path)? >= 1)
    })?;
    stop_server(w)?;
    if refusals(w, Some("mcp"), "path outside allowed roots")? > 0 {
        return Err("the MCP server refused a path of the run's repository".to_owned());
    }
    let top = paths(w).cwd.clone().ok_or("no cwd")?;
    let top = top.parent().ok_or("no top level")?.to_path_buf();
    let mut now = Vec::new();
    crate::security_redaction::home_files(&top, &mut now)?;
    if now == paths(w).repository {
        return Ok(());
    }
    Err("a file of the repository changed, or appeared".to_owned())
}

#[then(
    regex = r"^the configured transcript root is resolved to its target before its JSONL files are read$"
)]
fn configured_root_is_resolved(w: &mut World) -> Result<(), String> {
    if !paths(w).by_configuration {
        return Err("the path did not come from configuration".to_owned());
    }
    let output = last_run(w)?;
    if !output.status.success() {
        return Err(format!(
            "parinn ingest --all exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let transcript = paths(w)
        .configuration_transcript
        .clone()
        .ok_or("no configured transcript was written")?;
    let source = store(w)?
        .source(&source_key(&transcript))
        .map_err(|e| e.to_string())?;
    if source.is_some() {
        return Ok(());
    }
    Err(format!(
        "no source was recorded for the configured transcript {}",
        transcript.display()
    ))
}

/// Fails unless a widening principal act names the resolved `target`.
pub(crate) fn names_act(w: &World, target: &Path) -> Result<(), String> {
    let store = store(w)?;
    let node = Node::find(&home(w)?, &store).map_err(|e| e.to_string())?;
    let digest = ::record::digest::Digest::of(&[target.to_string_lossy().as_bytes()]).to_hex();
    let named = acts(&store, &node)
        .map_err(|e| e.to_string())?
        .into_iter()
        .any(|a| {
            a.name == "ingest_path"
                && a.class.as_str() == "widening"
                && a.content["path_digest"] == digest
        });
    if named {
        return Ok(());
    }
    Err("no widening principal act names the resolved path".to_owned())
}

/// Fails unless the last command succeeded, the principal act that names
/// the resolved path is recorded as widening, and the path's lines are
/// redacted and untrusted with origin `ingested`.
fn read_by_act(w: &mut World, target: &Path) -> Result<(), String> {
    let out = last_run(w)?;
    if !out.status.success() {
        return Err(format!(
            "parinn ingest exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stdout).trim()
        ));
    }
    names_act(w, target)?;
    let store = store(w)?;
    let run = store
        .ingested_runs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|r| Path::new(&r.path) == target)
        .ok_or("the resolved path was not ingested")?;
    set_item(w, run.writer.to_string(), 1);
    let item = recalled_item(w)?;
    if item["origin"] != "ingested" || item["trust"] != "untrusted" {
        return Err(format!("the first line is recalled as {item}"));
    }
    if item["content"].as_str().is_some_and(|c| c.contains(KEY)) {
        return Err("a line was recalled unredacted".to_owned());
    }
    match found_in_home(w, &[KEY.as_bytes().to_vec()])? {
        Some(found) => Err(found),
        None => Ok(()),
    }
}

#[then(
    regex = r#"^the path is resolved and read as the person's widening principal act, its lines are redacted and untrusted with origin "ingested", and nothing is written to the file$"#
)]
fn read_as_act(w: &mut World) -> Result<(), String> {
    let path = paths(w).path.clone().ok_or("no step named a path")?;
    let target = fs::canonicalize(&path).map_err(|e| e.to_string())?;
    read_by_act(w, &target)?;
    if fs::read(&target).ok() == paths(w).before {
        return Ok(());
    }
    Err("the file was written".to_owned())
}

#[then(
    regex = r#"^the path is resolved to its target and read as the person's widening principal act, its lines are redacted and untrusted with origin "ingested"$"#
)]
fn read_target_as_act(w: &mut World) -> Result<(), String> {
    let path = paths(w).path.clone().ok_or("no step named a path")?;
    let target = fs::canonicalize(&path).map_err(|e| e.to_string())?;
    if target == path {
        return Err("the path is no link".to_owned());
    }
    read_by_act(w, &target)
}

#[then(
    regex = r"^the path is refused as not a regular file, nothing is read from it, and an audit entry records it$"
)]
fn refused_not_a_file(w: &mut World) -> Result<(), String> {
    if last_run(w)?.status.success() {
        return Err("parinn ingest accepted a directory".to_owned());
    }
    if refusals(
        w,
        Some("ingest"),
        "refused: named path is not a regular file",
    )? != 1
    {
        return Err("no audit entry records the refusal as not a regular file".to_owned());
    }
    nothing_read(w)
}
