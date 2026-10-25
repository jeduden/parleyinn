//! Function complexity, held at build time: the workspace lints deny
//! clippy's complexity lints that are off by default, and the clippy
//! configuration every workspace crate builds with holds each lint's
//! threshold at or below its ceiling. CLAUDE.md asks for small
//! functions; this keeps that from resting on review alone. CI gates
//! on clippy with warnings denied (ENG-16), so a function over a limit
//! fails the build. A site that has to exceed one says so with an
//! `expect` and a reason, under review.

use crate::Checkout;
use crate::product_crates::{clippy_config_of, strip_comment};
use crate::static_analysis::{UNSAFE_SHIM, inherits_workspace_lints, members, table, value};

/// The complexity lints the workspace lints deny. clippy leaves each
/// off by default; `too_many_arguments` and `type_complexity`, on by
/// default, need no entry.
pub const LINTS: [&str; 4] = [
    "cognitive_complexity",
    "too_many_lines",
    "excessive_nesting",
    "fn_params_excessive_bools",
];

/// The ceiling each complexity lint is held to, by the clippy.toml key
/// that sets its threshold: a function's cognitive complexity, its
/// lines of code, a block's nesting depth and a function's `bool`
/// parameters. A clippy.toml may hold a lint tighter, never looser.
pub const LIMITS: [(&str, u64); 4] = [
    ("cognitive-complexity-threshold", 15),
    ("too-many-lines-threshold", 80),
    ("excessive-nesting-threshold", 5),
    ("max-fn-params-bools", 3),
];

/// The thresholds clippy reads 0 as switching their lint off, so a
/// clippy.toml may not set them that low.
const OFF_AT_ZERO: [&str; 1] = ["excessive-nesting-threshold"];

/// The levels that make a lint fail the build.
const DENYING: [&str; 2] = ["deny", "forbid"];

/// Requires `[workspace.lints.clippy]` to deny or forbid every lint of
/// [`LINTS`], and names each it does not.
#[must_use]
pub fn workspace_denies(root_manifest: &str) -> Vec<String> {
    undenied(&table(root_manifest, "workspace.lints.clippy"))
        .map(|lint| format!("the workspace lints do not deny clippy::{lint}"))
        .collect()
}

/// The lints of [`LINTS`] a lints table's `lines` neither deny nor
/// forbid.
fn undenied(lines: &[String]) -> impl Iterator<Item = &'static str> {
    LINTS.into_iter().filter(|lint| {
        let level = value(lines, lint).map(|v| v.replace(' ', ""));
        !level.is_some_and(|v| {
            DENYING
                .iter()
                .any(|d| v == format!("\"{d}\"") || v.contains(&format!("level=\"{d}\"")))
        })
    })
}

/// The integer a clippy.toml sets its top-level `key` to: a line before
/// the first `[table]` header, comments left out. None when the key is
/// missing there or holds no integer.
#[must_use]
pub fn threshold(clippy_toml: &str, key: &str) -> Option<u64> {
    clippy_toml
        .lines()
        .map(strip_comment)
        .take_while(|line| !line.trim_start().starts_with('['))
        .find_map(|line| {
            let (k, v) = line.split_once('=')?;
            (k.trim() == key).then(|| v.trim().parse().ok())?
        })
}

/// Requires `clippy_toml`, read from `file`, to set every threshold of
/// [`LIMITS`] at or below its ceiling, and none of [`OFF_AT_ZERO`] to 0,
/// and names each it does not.
#[must_use]
pub fn holds_limits(file: &str, clippy_toml: &str) -> Vec<String> {
    LIMITS
        .iter()
        .filter_map(|(key, max)| match threshold(clippy_toml, key) {
            None => Some(format!("{file} sets no {key}")),
            Some(set) if set > *max => Some(format!("{file} sets {key} to {set}, above {max}")),
            Some(0) if OFF_AT_ZERO.contains(key) => Some(format!(
                "{file} sets {key} to 0, which switches the lint off"
            )),
            Some(_) => None,
        })
        .collect()
}

/// Requires the workspace lints to deny every complexity lint, every
/// member crate to inherit them, and the clippy configuration each
/// member builds with to hold every limit.
#[must_use]
pub fn crates_hold_limits(c: &Checkout) -> Vec<String> {
    let root = match c.read("Cargo.toml") {
        Ok(body) => body,
        Err(e) => return vec![e],
    };
    let mut problems = workspace_denies(&root);
    let members = match members(c, &root) {
        Ok(members) => members,
        Err(e) => {
            problems.push(e);
            return problems;
        }
    };
    for member in members {
        problems.extend(member_holds_limits(c, &member));
    }

    problems
}

/// Requires the crate in `member` to inherit the workspace lints, or as
/// the one unsafe shim to deny the complexity lints in its own, and its
/// clippy configuration to hold every limit.
fn member_holds_limits(c: &Checkout, member: &str) -> Vec<String> {
    match c.read(&format!("{member}/Cargo.toml")) {
        Err(e) => return vec![e],
        Ok(manifest) if member == UNSAFE_SHIM && !inherits_workspace_lints(&manifest) => {
            let loose: Vec<String> = undenied(&table(&manifest, "lints.clippy"))
                .map(|lint| {
                    format!("crate {member}, the one unsafe shim, does not deny clippy::{lint}")
                })
                .collect();
            if !loose.is_empty() {
                return loose;
            }
        }
        Ok(manifest) if !inherits_workspace_lints(&manifest) => {
            return vec![format!(
                "crate {member} does not inherit the workspace lints, so the complexity lints miss it"
            )];
        }
        Ok(_) => {}
    }
    let file = clippy_config_of(c, member);
    match c.read(&file) {
        Err(e) => vec![e],
        Ok(body) => holds_limits(&file, &body),
    }
}

#[cfg(test)]
mod tests;
