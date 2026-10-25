//! The contract tests of SRS §2.3's assumptions register (ENG-17): the
//! hook inputs and transcripts the M0 spikes recorded from Claude Code
//! live under `testdata/claude-code/<version>/`, and the bound
//! `@ASM-*` scenarios in the `scenario` crate's `bdd` target read them
//! through this crate and call its checks. Every check returns its
//! problems, one line each, so a failing step names all of them at
//! once. It is repository tooling and never links into a shipped
//! executable.

pub mod hook_input;
pub mod measured;
pub mod plugin;
pub mod recording;
pub mod replay;
pub mod residual;
pub mod shapes;
pub mod snapshot;
pub mod transcript;
pub mod transcript_tree;

use serde_json::Value;

/// The value at `dotted`, a `.`-separated path of object keys.
#[must_use]
pub fn at<'a>(value: &'a Value, dotted: &str) -> Option<&'a Value> {
    dotted.split('.').try_fold(value, |v, key| v.get(key))
}

/// The string at `dotted`, or `""` when it is missing or not a string.
#[must_use]
pub fn text<'a>(value: &'a Value, dotted: &str) -> &'a str {
    at(value, dotted)
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
