//! The steps of ENG-05: the core logs through `tracing`, one JSON line
//! per event in the home's log file, every field redacted first. The
//! log field is the transcript path `parinn ingest` logs, and the path
//! carries an AWS access key in its file name.

use std::fs;
use std::path::PathBuf;

use cucumber::{given, then, when};
use serde_json::Value;

use crate::World;
use crate::record::{home, run_core};

/// An AWS access key, as AWS documents an example one.
const AWS_ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";

/// What the log steps track.
#[derive(Debug, Default)]
pub struct LogSteps {
    /// The transcript whose path is the log field.
    transcript: Option<PathBuf>,
}

fn logs(w: &mut World) -> &mut LogSteps {
    w.section::<LogSteps>()
}

/// The home's log file's lines.
fn log_lines(w: &World) -> Result<Vec<String>, String> {
    let path = home(w)?.root().join("logs").join("parinn.jsonl");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text.lines().map(str::to_owned).collect())
}

#[given(regex = r"^a log field containing an AWS access key$")]
fn log_field_with_key(w: &mut World) -> Result<(), String> {
    let dir = w.home.join(".claude").join("projects").join("p");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{AWS_ACCESS_KEY}.jsonl"));
    fs::write(&path, "{\"type\":\"user\"}\n").map_err(|e| e.to_string())?;
    logs(w).transcript = Some(path);
    Ok(())
}

#[when(regex = r"^ParleyInn writes the log line$")]
fn parinn_writes_log_line(w: &mut World) -> Result<(), String> {
    let path = logs(w).transcript.clone().ok_or("no log field was given")?;
    let path = path.to_str().ok_or("the transcript path is not UTF-8")?;
    run_core(w, &["ingest", "--path", path], None)
}

#[then(regex = r"^the log line is JSON emitted through tracing$")]
fn log_line_is_json(w: &mut World) -> Result<(), String> {
    let lines = log_lines(w)?;
    let line = lines
        .iter()
        .find(|l| l.contains("\"ingested\""))
        .ok_or(format!("no log line says ingested among {lines:?}"))?;
    let value: Value = serde_json::from_str(line).map_err(|e| format!("{line}: {e}"))?;
    let traced = value["target"] == "parinn"
        && value["level"] == "info"
        && value["fields"]["message"] == "ingested"
        && value["fields"]["transcript"].is_string();
    if traced {
        return Ok(());
    }
    Err(format!(
        "the log line is no tracing event of the core: {line}"
    ))
}

#[then(regex = r#"^the key appears only as a "\[REDACTED:" marker$"#)]
fn key_only_as_marker(w: &mut World) -> Result<(), String> {
    let text = log_lines(w)?.join("\n");
    if text.contains(AWS_ACCESS_KEY) {
        return Err(format!("the log carries the key: {text}"));
    }
    if text.contains("[REDACTED:aws_access_key]") {
        return Ok(());
    }
    Err(format!("the log carries no marker: {text}"))
}
