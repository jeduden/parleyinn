//! The steps of SEC-08 that phase 4 of plan 2609292012 carries: secrets
//! and harness account identifiers are redacted before anything is
//! written. The agent's run leaves its transcript under the scenario's
//! default transcript root; each step that adds to it appends a line in
//! Claude Code's shape. The scenario stays pending: its `parinn ingest
//! --all` is phase 8's, and `parinn purge --run` is ADM-07's run scope,
//! in M3.

use std::fs;
use std::path::{Path, PathBuf};

use cucumber::{given, then};
use record::digest::Digest;
use serde_json::json;

use crate::World;
use crate::administration::configure;
use crate::record::{home, last_run, run_core};

/// An AWS access key, as AWS documents an example one.
const AWS_ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
/// What the person's redaction pattern matches.
const ACME: &str = "ACME-12345678";
/// The harness account's email address and organization id.
const EMAIL: &str = "person@example.com";
const ORGANIZATION: &str = "4f7d2c1e-9a3b-4c8d-8e2f-1a2b3c4d5e6f";

/// A field of the agent's run no rule matches.
const CLEAN_FIELD: &str = "00000000-0000-4000-8000-000000000001";

/// What the redaction steps track: each transcript by name.
#[derive(Debug, Default)]
pub struct RedactionSteps {
    transcripts: Vec<(String, PathBuf)>,
}

fn red(w: &mut World) -> &mut RedactionSteps {
    w.section::<RedactionSteps>()
}

/// The transcript the run `name` leaves.
pub(crate) fn transcript(w: &mut World, name: &str) -> Result<PathBuf, String> {
    red(w)
        .transcripts
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, p)| p.clone())
        .ok_or_else(|| format!("no agent run left a transcript {name:?}"))
}

/// Names the transcript at `path` `name`, in place of any it named.
pub(crate) fn name_transcript(w: &mut World, name: &str, path: PathBuf) {
    let transcripts = &mut red(w).transcripts;
    transcripts.retain(|(n, _)| n != name);
    transcripts.push((name.to_owned(), path));
}

/// Appends `line` to the transcript of the run `name`.
pub(crate) fn append(w: &mut World, name: &str, line: &serde_json::Value) -> Result<(), String> {
    let path = transcript(w, name)?;
    let mut text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    text.push_str(&line.to_string());
    text.push('\n');
    fs::write(&path, text).map_err(|e| e.to_string())
}

/// Every file the home holds, with its bytes.
fn home_files(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            home_files(&path, out)?;
        } else {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            out.push((path, bytes));
        }
    }
    Ok(())
}

/// The first of `needles` some file in the home holds, with the file.
fn found_in_home(w: &World, needles: &[Vec<u8>]) -> Result<Option<String>, String> {
    let mut files = Vec::new();
    home_files(home(w)?.root(), &mut files)?;
    for (path, bytes) in &files {
        for needle in needles {
            if bytes.windows(needle.len()).any(|w| w == needle.as_slice()) {
                return Ok(Some(format!(
                    "{} holds {:?}",
                    path.display(),
                    String::from_utf8_lossy(needle)
                )));
            }
        }
    }
    Ok(None)
}

/// Fails naming the file that holds any of `needles`.
fn none_in_home(w: &World, needles: &[&str]) -> Result<(), String> {
    let needles: Vec<Vec<u8>> = needles.iter().map(|n| n.as_bytes().to_vec()).collect();
    match found_in_home(w, &needles)? {
        Some(found) => Err(found),
        None => Ok(()),
    }
}

#[given(regex = r#"^a redaction pattern the person defined "([^"]+)"$"#)]
fn person_pattern(w: &mut World, pattern: String) -> Result<(), String> {
    configure(w, &format!("[redaction]\nextra_patterns = [{pattern:?}]\n"))
}

#[given(regex = r#"^an agent run with a Claude Code transcript "([^"]+)"$"#)]
fn agent_run_with_transcript(w: &mut World, name: String) -> Result<(), String> {
    let dir = w.home.join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.jsonl"));
    fs::write(&path, "").map_err(|e| e.to_string())?;
    red(w).transcripts.push((name, path));
    Ok(())
}

#[given(
    regex = r#"^"([^"]+)" contains a Bash tool result whose output carries an AWS access key in both "message.content" and "toolUseResult"$"#
)]
fn bash_result_with_key(w: &mut World, name: String) -> Result<(), String> {
    let output = format!("export AWS_ACCESS_KEY_ID={AWS_ACCESS_KEY}\nexport TOKEN={ACME}");
    let line = json!({
        "type": "user",
        "message": {"role": "user", "content": [
            {"type": "tool_result", "tool_use_id": "toolu_1", "content": output},
        ]},
        "toolUseResult": {"stdout": output, "stderr": "", "interrupted": false},
        "uuid": CLEAN_FIELD,
    });
    append(w, &name, &line)
}

#[given(
    regex = r#"^"([^"]+)" contains a "session_context" attachment carrying the account's email address and organization id$"#
)]
fn session_context_with_account(w: &mut World, name: String) -> Result<(), String> {
    let context = json!({
        "type": "attachment",
        "attachment": {"type": "session_context", "context": {
            "userEmail": format!("The user's email address is {EMAIL}. Use it only to identify the user."),
        }},
    });
    let org = json!({
        "type": "attachment",
        "attachment": {"type": "credential_org", "organizationUuid": ORGANIZATION},
    });
    append(w, &name, &context)?;
    append(w, &name, &org)
}

#[then(
    regex = r#"^no stored text field, payload, or hook input contains an AWS access key or "([^"]+)"$"#
)]
fn no_secret_stored(w: &mut World, secret: String) -> Result<(), String> {
    none_in_home(w, &[AWS_ACCESS_KEY, &secret])
}

#[then(regex = r"^no stored text field contains the account's email address or organization id$")]
fn no_account_stored(w: &mut World) -> Result<(), String> {
    none_in_home(w, &[EMAIL, ORGANIZATION])
}

#[then(
    regex = r#"^each secret is replaced by "\[REDACTED:<rule>\]", with no hash or other value derived from the secret$"#
)]
fn replaced_by_markers(w: &mut World) -> Result<(), String> {
    let markers = [
        "[REDACTED:aws_access_key]",
        "[REDACTED:extra_pattern_1]",
        "[REDACTED:account_email]",
        "[REDACTED:organization_id]",
    ];
    let wanted: Vec<Vec<u8>> = markers.iter().map(|m| m.as_bytes().to_vec()).collect();
    for (marker, needle) in markers.iter().zip(&wanted) {
        if found_in_home(w, std::slice::from_ref(needle))?.is_none() {
            return Err(format!("no stored text carries {marker}"));
        }
    }
    no_value_confirms_a_guess(w)
}

#[then(
    regex = r"^no stored event, segment or export carries a value from which the secret could be confirmed$"
)]
fn no_value_confirms_a_guess(w: &mut World) -> Result<(), String> {
    let mut derived = Vec::new();
    for secret in [AWS_ACCESS_KEY, ACME, EMAIL, ORGANIZATION] {
        let digest = Digest::of(&[secret.as_bytes()]);
        derived.push(digest.as_bytes().to_vec());
        derived.push(digest.to_hex().into_bytes());
    }
    match found_in_home(w, &derived)? {
        Some(found) => Err(format!("a hash of a secret is stored: {found}")),
        None => Ok(()),
    }
}

#[then(
    regex = r"^any correlation of the repeated AWS access key lives only in a node-local index keyed under this node's own at-rest key$"
)]
fn no_correlation_index(w: &mut World) -> Result<(), String> {
    // ParleyInn keeps no correlation index, so none can confirm a guess:
    // the store holds the record's tables and nothing beside them.
    let store = home(w)?.store();
    let db =
        rusqlite::Connection::open_with_flags(&store, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| e.to_string())?;
    let mut query = db
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .map_err(|e| e.to_string())?;
    let tables: Vec<String> = query
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    if tables == ["event_heads", "events", "meta", "seats"] {
        return Ok(());
    }
    Err(format!(
        "the store holds tables beside the record's: {tables:?}"
    ))
}

#[then(
    regex = r#"^with a repository "\.parinn\.toml" adding the redaction pattern "([^"]+)", no stored text is redacted by that pattern, and an audit entry and a counter record the repository setting as ignored$"#
)]
fn repository_pattern_ignored(w: &mut World, pattern: String) -> Result<(), String> {
    let git = w.home.join(".git");
    fs::create_dir_all(&git).map_err(|e| e.to_string())?;
    let setting = format!("[redaction]\nextra_patterns = [{pattern:?}]\n");
    fs::write(w.home.join(".parinn.toml"), setting).map_err(|e| e.to_string())?;
    let name = red(w).transcripts.first().map(|(n, _)| n.clone());
    let path = transcript(w, &name.ok_or("no agent run left a transcript")?)?;
    let shown = path.display().to_string();
    run_core(w, &["ingest", "--path", &shown], None)?;
    if !last_run(w)?.status.success() {
        return Err("parinn ingest failed with the repository pattern".to_owned());
    }
    none_in_home(w, &["[REDACTED:extra_pattern_2]"])?;
    // Text no rule matches survives whole: the repository's pattern
    // redacted nothing under any name.
    let clean = [CLEAN_FIELD.as_bytes().to_vec()];
    if found_in_home(w, &clean)?.is_none() {
        return Err(format!("{CLEAN_FIELD} did not survive whole"));
    }
    let entries = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let ignored = entries.iter().any(|e| {
        e.counter.as_deref() == Some("repository_configuration_key_ignored")
            && e.detail.ends_with("redaction.extra_patterns")
    });
    if ignored {
        return Ok(());
    }
    Err("no audit entry records the repository's redaction pattern as ignored".to_owned())
}
