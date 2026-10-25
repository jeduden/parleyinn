//! The residual-identifier check (ENG-17): no recorded fixture may
//! carry what identifies the account or the machine it was recorded
//! on. The recording kit replaces each with a placeholder
//! (`record/redact.sh`, `record/strip.sh`); this check finds any it
//! missed:
//!
//! - an email address other than the placeholders;
//! - an organization id other than the nil one;
//! - a home path other than `/home/u`;
//! - a claude.ai connector named other than `Connector <letter>`;
//! - a skill a skill listing names that is not one of the listed ones.

use std::path::Path;

use serde_json::Value;

use crate::{recording, text};

/// The email addresses a fixture may carry: the placeholder the kit
/// writes for the account's, and the harness's own attribution.
pub const EMAILS: [&str; 2] = ["user@example.com", "noreply@anthropic.com"];

/// The organization id the kit writes for the account's.
pub const NIL_ORGANIZATION: &str = "00000000-0000-0000-0000-000000000000";

/// The home a fixture's paths may name: the kit's placeholder.
pub const HOME: &str = "u";

/// The skills a fixture's skill listing may name: the harness's own,
/// and the fixtures' own (`greet`).
pub const SKILLS: [&str; 19] = [
    "artifact-capabilities",
    "artifact-design",
    "artifact-diagramming",
    "claude-api",
    "claude-in-chrome",
    "code-review",
    "dataviz",
    "fewer-permission-prompts",
    "greet",
    "init",
    "keybindings-help",
    "loop",
    "plugin-authoring",
    "run",
    "schedule",
    "security-review",
    "simplify",
    "update-config",
    "workflow-authoring",
];

/// Every residual identifier in `body`, one problem each.
#[must_use]
pub fn residual(body: &str) -> Vec<String> {
    let mut problems = Vec::new();
    problems.extend(emails(body));
    problems.extend(organizations(body));
    problems.extend(homes(body));
    problems.extend(connectors(body));
    problems.extend(body.lines().flat_map(skills));
    problems
}

/// Every residual identifier in each file under `root`, recursively,
/// each problem led by its file's path. A file that is not UTF-8 is
/// read lossily.
///
/// # Errors
///
/// Fails when a directory cannot be listed or a file read.
pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let mut problems = Vec::new();
    for path in recording::files(root, "")? {
        let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let body = String::from_utf8_lossy(&bytes);
        problems.extend(
            residual(&body)
                .into_iter()
                .map(|p| format!("{}: {p}", path.display())),
        );
    }
    Ok(problems)
}

fn local_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "._%+-".contains(c)
}

fn domain_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || ".-".contains(c)
}

/// Each email address in `body` that is not a placeholder.
fn emails(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (at, _) in body.match_indices('@') {
        let local_len = body[..at]
            .chars()
            .rev()
            .take_while(|c| local_char(*c))
            .map(char::len_utf8)
            .sum::<usize>();
        let domain: String = body[at + 1..]
            .chars()
            .take_while(|c| domain_char(*c))
            .collect();
        let domain = domain.trim_end_matches(['.', '-']);
        let tld = domain.rsplit_once('.').map(|(_, t)| t).unwrap_or_default();
        if local_len == 0 || tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        let address = format!("{}@{domain}", &body[at - local_len..at]);
        if !EMAILS.contains(&address.as_str()) {
            found.push(format!("an email address {address}"));
        }
    }
    found
}

/// Each organization id in `body` other than the nil one.
fn organizations(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (at, key) in body.match_indices("organizationUuid") {
        let value: String = body[at + key.len()..]
            .chars()
            .skip_while(|c| "\\\": ".contains(*c))
            .take_while(|c| c.is_ascii_hexdigit() || *c == '-')
            .collect();
        if value.len() == NIL_ORGANIZATION.len() && value != NIL_ORGANIZATION {
            found.push(format!("an organization id {value}"));
        }
    }
    found
}

/// Each home path in `body` other than the placeholder's.
fn homes(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    for prefix in ["/home/", "/Users/"] {
        for (at, _) in body.match_indices(prefix) {
            let name: String = body[at + prefix.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || "._-".contains(*c))
                .collect();
            if !name.is_empty() && name != HOME && !name.starts_with('.') {
                found.push(format!("a home path {prefix}{name}"));
            }
        }
    }
    found
}

/// Each claude.ai connector in `body` named other than `Connector
/// <letter>`: as an MCP tool's server (`claude_ai_<name>`), or as an
/// MCP server's name where the harness lists it (after `"`, `## ` or
/// an escaped line break).
fn connectors(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (at, key) in body.match_indices("claude_ai_") {
        let name: String = body[at + key.len()..]
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        if name != "Connector" {
            found.push(format!("a claude.ai connector claude_ai_{name}"));
        }
    }
    for lead in ["\"claude.ai ", "## claude.ai ", "\\nclaude.ai "] {
        for (at, _) in body.match_indices(lead) {
            let name: String = body[at + lead.len()..]
                .chars()
                .take_while(|c| !"\"\\\n".contains(*c))
                .collect();
            if !is_placeholder(&name) {
                found.push(format!("a claude.ai connector {name:?}"));
            }
        }
    }
    found
}

/// Whether `name` is a connector's placeholder: `Connector` and one
/// capital letter.
fn is_placeholder(name: &str) -> bool {
    name.strip_prefix("Connector ")
        .is_some_and(|letter| letter.len() == 1 && letter.chars().all(|c| c.is_ascii_uppercase()))
}

/// Each skill the skill listing on `line`, a JSON line, names that is
/// not a listed one: in its names and in its content's entries.
fn skills(line: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };
    if text(&value, "attachment.type") != "skill_listing" {
        return Vec::new();
    }
    let named = value
        .pointer("/attachment/names")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str);
    let listed = text(&value, "attachment.content")
        .lines()
        .filter_map(|l| l.strip_prefix("- ")?.split_once(':').map(|(name, _)| name));
    let mut names: Vec<&str> = named.chain(listed).collect();
    names.sort_unstable();
    names.dedup();
    names
        .into_iter()
        .filter(|n| !SKILLS.contains(n))
        .map(|n| format!("a skill {n}"))
        .collect()
}

#[cfg(test)]
mod tests;
