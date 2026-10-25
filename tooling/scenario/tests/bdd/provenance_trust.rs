//! The steps of PRV-05 that phase 8 of plan 2609292012 carries: no
//! settings layer can trust a provenance class beyond PRV-02. Managed
//! policy, the person's configuration and the repository's
//! `.parinn.toml` each set a class's trust; every ingested event stays
//! untrusted, and each layer's key is audited as a rejected trust
//! override.
//!
//! `@PRV-05` stays pending: its `post` and `summary` rows need a post
//! and a summary, which no transcript line is and which later phases
//! write. Its transcript rows pass.

use std::fs;
use std::path::Path;

use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::World;
use crate::provenance::{attachment, bash_result, record_lines, synthetic_call};
use crate::record::{home, set_env};
use crate::record_transcript::{recorded, recorded_call};

/// The trust policy key setting `class` trusted.
fn trusting(class: &str) -> String {
    format!("\n[trust_policy]\n{class:?} = \"trusted\"\n")
}

/// Appends `text` to the file at `path`, making it when it is not there.
fn append_to(path: &Path, text: &str) -> Result<(), String> {
    let mut body = fs::read_to_string(path).unwrap_or_default();
    body.push_str(text);
    fs::write(path, body).map_err(|e| format!("{}: {e}", path.display()))
}

/// Writes managed policy holding `text` into a directory of its own that
/// another OS user owns, as managed policy must be (ADM-04), and points
/// the core at it, given away as root or through `sudo -n`.
fn managed_policy(w: &mut World, text: &str) -> Result<(), String> {
    let dir = w.home.join("managed");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("managed_policy.toml");
    fs::write(&path, text).map_err(|e| e.to_string())?;
    // nobody's UID, which owns neither the policy nor its directory.
    let other = 65534;
    for p in [&path, &dir] {
        crate::security::give(p, other)?;
    }
    set_env(w, "PARINN_TEST_MANAGED_POLICY", &path.display().to_string());
    Ok(())
}

#[given(
    regex = r#"^managed policy, the person's configuration and the repository's "\.parinn\.toml" each set the trust of "([^"]+)" to "trusted"$"#
)]
fn every_layer_trusts(w: &mut World, class: String) -> Result<(), String> {
    managed_policy(w, &trusting(&class))?;
    let home = home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let config = w.parinn_home.join("config.toml");
    append_to(&config, &trusting(&class))?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    fs::create_dir_all(w.home.join(".git")).map_err(|e| e.to_string())?;
    append_to(&w.home.join(".parinn.toml"), &trusting(&class))
}

/// The lines whose last is an event of `class`.
fn lines_of(class: &str) -> Result<Vec<Value>, String> {
    Ok(match class {
        "assistant" => vec![recorded("an assistant text line", |l| {
            l["type"] == "assistant" && l["message"]["content"][0]["type"] == "text"
        })?],
        "tool_call" => vec![recorded_call("Bash")?.0],
        "tool_result:Bash" => bash_result(None)?,
        "web" => synthetic_call("WebFetch")?,
        "file" => synthetic_call("Read")?,
        "subagent_result" => synthetic_call("Agent")?,
        "mcp:github" => {
            let mut lines = synthetic_call("Bash")?;
            lines[0]["message"]["content"][0]["name"] = json!("mcp__github__search");
            lines
        }
        "harness_text" => vec![attachment("date")?],
        "unparsed" => vec![json!({"type": "future_kind"})],
        other => return Err(format!("no transcript line is an event of {other:?}")),
    })
}

#[when(regex = r#"^an event with provenance "([^"]+)" is ingested$"#)]
fn event_ingested(w: &mut World, class: String) -> Result<(), String> {
    let lines = lines_of(&class)?;
    record_lines(w, &lines)
}

#[then(
    regex = r#"^an audit entry for each settings layer records "(rejected trust override for [^"]+)"$"#
)]
fn each_layer_rejected(w: &mut World, what: String) -> Result<(), String> {
    let entries = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    let mut missing = Vec::new();
    for layer in [
        "managed policy",
        "configuration",
        "repository configuration",
    ] {
        let detail = format!("{layer}: {what}");
        if !entries.iter().any(|e| e.detail == detail) {
            missing.push(detail);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!("no audit entry records {missing:?}"))
}
