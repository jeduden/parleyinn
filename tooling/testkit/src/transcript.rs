//! Transcript lines for tests, built from lines Claude Code 2.1.296
//! recorded under `testdata/claude-code` (REC-26, ENG-17): a builder
//! starts from a recorded line and changes only the fields a test
//! names, so a built line keeps the recorded shape. The unit tests
//! replay each builder's output through `harness-contract`, the core's
//! own parser and classifier, and fail on a line it cannot parse or
//! finds drifted from the recorded shapes.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::repo_root;

/// The recorded Claude Code version the lines come from.
pub const VERSION: &str = "2.1.296";

/// Every line the recordings of [`VERSION`] hold, by transcript.
///
/// The recordings are checked-in test data no test writes, so the run
/// parses them once, on first use, and every builder reads that one
/// parse: a fixture of a hundred lines no longer parses megabytes of
/// JSON per line. A failed parse is kept too, and fails every builder
/// that asks.
///
/// # Errors
///
/// Fails when a recording cannot be listed, read or parsed.
pub fn recorded_transcripts() -> Result<&'static [Vec<Value>], String> {
    static PARSED: OnceLock<Result<Vec<Vec<Value>>, String>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            let version = repo_root().join("testdata/claude-code").join(VERSION);
            transcript_files(&version)?
                .iter()
                .map(|path| read_lines(path))
                .collect()
        })
        .as_deref()
        .map_err(Clone::clone)
}

/// The transcripts `version` recorded: every `.jsonl` file under a
/// directory named `transcripts`, in path order.
fn transcript_files(version: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(files(version)?
        .into_iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "transcripts"))
        .collect())
}

/// The `.jsonl` files under `dir`, recursively, in path order.
fn files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let failed = |e: std::io::Error| format!("list {}: {e}", dir.display());
    let mut paths = fs::read_dir(dir)
        .map_err(failed)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(failed)?;
    paths.sort();
    let mut found = Vec::new();
    for path in paths {
        if path.is_dir() {
            found.extend(files(&path)?);
        } else if path.to_string_lossy().ends_with(".jsonl") {
            found.push(path);
        }
    }
    Ok(found)
}

/// The JSON Lines file at `path`, one value per non-empty line.
fn read_lines(path: &Path) -> Result<Vec<Value>, String> {
    let body = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    body.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(n, line)| {
            serde_json::from_str(line)
                .map_err(|e| format!("{}: line {}: {e}", path.display(), n + 1))
        })
        .collect()
}

/// The first recorded line `pick` accepts.
///
/// # Errors
///
/// Fails when the recordings cannot be read, or no line is accepted;
/// `what` names the line sought.
pub fn recorded(what: &str, pick: impl Fn(&Value) -> bool) -> Result<Value, String> {
    recorded_transcripts()?
        .iter()
        .flatten()
        .find(|line| pick(line))
        .cloned()
        .ok_or_else(|| format!("no recorded line is {what}"))
}

/// A recorded assistant line calling `tool`, and the line holding its
/// result, from one transcript.
///
/// # Errors
///
/// Fails when the recordings cannot be read, or none holds a call of
/// `tool` with its result.
pub fn recorded_call(tool: &str) -> Result<(Value, Value), String> {
    for lines in recorded_transcripts()? {
        for call in lines {
            let Some(id) = tool_use_id(call, tool) else {
                continue;
            };
            let result = lines.iter().find(|l| {
                l["message"]["content"][0]["tool_use_id"].as_str() == Some(id.as_str())
                    && l["message"]["content"].as_array().map(Vec::len) == Some(1)
            });
            if let Some(result) = result {
                return Ok((call.clone(), result.clone()));
            }
        }
    }
    Err(format!(
        "no recording holds a call of {tool} with its result"
    ))
}

/// The id of the call of `tool` the assistant line `line` makes.
fn tool_use_id(line: &Value, tool: &str) -> Option<String> {
    let block = &line["message"]["content"][0];
    (line["type"] == "assistant"
        && block["type"] == "tool_use"
        && block["name"].as_str().is_some_and(|n| n.starts_with(tool)))
    .then(|| block["id"].as_str().map(str::to_owned))
    .flatten()
}

/// The first recorded line of `kind` whose `field` is `value`.
///
/// # Errors
///
/// Fails when no recorded line is one.
pub fn recorded_where(kind: &str, field: &str, value: &Value) -> Result<Value, String> {
    recorded(&format!("a {kind} line with {field} {value}"), |l| {
        l["type"] == kind && pointer(l, field) == Some(value)
    })
}

/// The value at `field`, a `/`-separated path.
fn pointer<'a>(line: &'a Value, field: &str) -> Option<&'a Value> {
    line.pointer(&format!("/{field}"))
}

/// A recorded attachment line of attachment type `kind`.
///
/// # Errors
///
/// Fails when no recorded line is one.
pub fn attachment(kind: &str) -> Result<Value, String> {
    recorded_where("attachment", "attachment/type", &json!(kind))
}

/// A recorded user line typed at the harness's input.
///
/// # Errors
///
/// Fails when no recorded line is one.
pub fn typed() -> Result<Value, String> {
    recorded_where("user", "promptSource", &json!("typed"))
}

/// A recorded line typed at the harness's input, carrying `text` as
/// prompt `prompt_id` marked `source`, in `session`.
///
/// # Errors
///
/// Fails when no recorded line is typed.
pub fn typed_line(
    text: &str,
    prompt_id: &str,
    source: &str,
    session: &str,
) -> Result<Value, String> {
    let mut line = recorded("a typed line", |l| l["promptSource"] == "typed")?;
    line["message"]["content"] = json!(text);
    line["promptId"] = json!(prompt_id);
    line["promptSource"] = json!(source);
    line["sessionId"] = json!(session);
    Ok(line)
}

/// A typed user line saying `text`, its own `uuid` `n`.
///
/// # Errors
///
/// Fails when no recorded line is typed.
pub fn user_line(text: &str, n: u64) -> Result<Value, String> {
    let mut line = typed()?;
    line["message"]["content"] = json!(text);
    line["uuid"] = json!(format!("00000000-0000-4000-8000-0000000012{n:02}"));
    Ok(line)
}

/// A `uuid` for fixture line `n`.
#[must_use]
pub fn uuid(n: u64) -> String {
    format!("00000000-0000-4000-8000-{n:012x}")
}

/// A Bash call and its result, the result's output replaced by
/// `output` when given.
///
/// # Errors
///
/// Fails when no recording holds a Bash call with its result.
pub fn bash_result(output: Option<&str>) -> Result<Vec<Value>, String> {
    let (call, mut result) = recorded_call("Bash")?;
    if let Some(output) = output {
        result["message"]["content"][0]["content"] = json!(output);
    }
    Ok(vec![call, result])
}

/// A Bash call and its result, numbered `n`, the result's output
/// `output`, each line with a `uuid` of its own.
///
/// # Errors
///
/// Fails when no recording holds a Bash call with its result.
pub fn pair(n: u64, output: &str) -> Result<Vec<Value>, String> {
    let (mut call, mut result) = recorded_call("Bash")?;
    let id = format!("toolu_fixture{n}");
    call["message"]["content"][0]["id"] = json!(id);
    call["uuid"] = json!(uuid(2 * n));
    result["message"]["content"][0]["tool_use_id"] = json!(id);
    result["message"]["content"][0]["content"] = json!(output);
    result["uuid"] = json!(uuid(2 * n + 1));
    set_stdout(&mut result, output);

    Ok(vec![call, result])
}

/// Sets the stdout of `result`'s `toolUseResult` to `output`, when the
/// line records one as an object.
fn set_stdout(result: &mut Value, output: &str) {
    if result["toolUseResult"].is_object() {
        result["toolUseResult"]["stdout"] = json!(output);
    }
}

/// A transcript line in Claude Code 2.1.296's recorded shape, which
/// ingest records with nothing to audit, numbered `n`.
#[must_use]
pub fn clean_line(n: u64) -> String {
    format!(
        "{}\n",
        json!({"type": "atis-latch", "atis": format!("line {n}"), "sessionId": "s"})
    )
}

/// Appends `count` clean lines to the transcript at `path`, numbered on
/// from what it holds.
///
/// # Errors
///
/// Fails when the transcript cannot be read or written.
pub fn append_lines(path: &Path, count: u64) -> Result<(), String> {
    let held = fs::read_to_string(path).unwrap_or_default().lines().count() as u64;
    let text: String = (held + 1..=held + count).map(clean_line).collect();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
