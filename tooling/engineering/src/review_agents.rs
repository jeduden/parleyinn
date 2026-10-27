//! ENG-28: every agent the review bundle can dispatch is defined under
//! `.claude/agents`, so the orchestrator can consult it: each SRS
//! section's specialist, each domain-model file's, the language and
//! test-layer specialists, and the generalists. Specialists run at low
//! effort and generalists at medium, as their front matter says; each
//! generalist's deep instance runs on the strongest model at high effort
//! with a larger turn budget.

use std::collections::BTreeMap;
use std::fs;

use review_bundle::dispatch::{
    DEEP_GENERALISTS, FIXED_SPECIALISTS, GENERALISTS, MODEL, generalist_of, model_specialist,
    section_specialist,
};

use crate::Checkout;

/// Where the agent definitions live, relative to the checkout.
pub const AGENTS: &str = ".claude/agents";

/// The tools a specialist has: it reads, and writes its own report
/// under `review/reports`, the one path the review workflow allows.
const SPECIALIST_TOOLS: &str = "Read, Grep, Glob, Write";
/// The tools a generalist has: it reads its specialists' reports and
/// writes nothing.
const GENERALIST_TOOLS: &str = "Read, Grep, Glob";
/// The effort a specialist runs at.
const SPECIALIST_EFFORT: &str = "low";
/// The effort a generalist runs at.
const GENERALIST_EFFORT: &str = "medium";
/// The effort, model and turn budget of a generalist's deep instance:
/// the depth a deep review had when one agent ran it alone.
const DEEP_EFFORT: &str = "high";
const DEEP_MODEL: &str = "opus";
const DEEP_TURNS: &str = "40";

/// Problems with the agents the review bundle can dispatch: one that is
/// not defined, names itself otherwise, runs at the wrong effort, names
/// another generalist than the dispatch routes it to, or,
/// for a section or domain-model specialist, does not name its file.
/// A directory of sections or model files that cannot be read is named,
/// so the check never passes without having looked.
#[must_use]
pub fn every_dispatched_agent_is_defined(checkout: &Checkout) -> Vec<String> {
    let mut problems = Vec::new();
    let mut want: Vec<(String, &str, Option<String>)> = Vec::new();
    want.extend(
        GENERALISTS
            .iter()
            .map(|n| ((*n).to_owned(), GENERALIST_EFFORT, None)),
    );
    want.extend(
        DEEP_GENERALISTS
            .iter()
            .map(|n| ((*n).to_owned(), DEEP_EFFORT, None)),
    );
    want.extend(
        FIXED_SPECIALISTS
            .iter()
            .map(|(n, _)| ((*n).to_owned(), SPECIALIST_EFFORT, None)),
    );
    match markdown_files(checkout, "docs/srs") {
        Ok(paths) => want.extend(paths.into_iter().filter_map(|path| {
            section_specialist(&path).map(|name| (name, SPECIALIST_EFFORT, Some(path)))
        })),
        Err(e) => problems.push(e),
    }
    match markdown_files(checkout, MODEL.trim_end_matches('/')) {
        Ok(paths) => want.extend(paths.into_iter().filter_map(|path| {
            let file = path
                .strip_prefix(MODEL)
                .and_then(|f| f.strip_suffix(".md"))
                .unwrap_or_default();
            let name = model_specialist(file)?;
            let names = (file != "index").then(|| path.clone());
            Some((name, SPECIALIST_EFFORT, names))
        })),
        Err(e) => problems.push(e),
    }
    want.sort();
    want.dedup();

    problems.extend(
        want.iter()
            .filter_map(|(name, effort, names)| check(checkout, name, effort, names.as_deref())),
    );
    problems
}

/// The problem with agent `name`, if any.
fn check(checkout: &Checkout, name: &str, effort: &str, names: Option<&str>) -> Option<String> {
    let path = format!("{AGENTS}/{name}.md");
    let Ok(text) = checkout.read(&path) else {
        return Some(format!("no agent {name} at {path}"));
    };
    let fields = match adr::front_matter(&text) {
        Ok((fields, _)) => fields,
        Err(e) => return Some(format!("{name}'s front matter does not read: {e}")),
    };
    let field = |key: &str| fields.get(key).map_or("", String::as_str);
    let own = field("name");
    if own != name {
        return Some(format!("{name} names itself {own}"));
    }
    let tools = if generalist_of(name).is_some() {
        SPECIALIST_TOOLS
    } else {
        GENERALIST_TOOLS
    };
    let has_tools = field("tools");
    if has_tools != tools {
        return Some(format!("{name} has tools {has_tools}, want {tools}"));
    }
    let has = field("effort");
    if has != effort {
        return Some(format!("{name} runs at effort {has}, want {effort}"));
    }
    if DEEP_GENERALISTS.contains(&name)
        && let Some(problem) = deep_settings(name, &fields)
    {
        return Some(problem);
    }
    if let Some(want) = generalist_of(name) {
        match field("generalist") {
            "" => return Some(format!("{name} names no generalist, want {want}")),
            named if named != want => {
                return Some(format!("{name} names generalist {named}, want {want}"));
            }
            _ => {}
        }
    }
    names
        .filter(|file| !text.contains(file))
        .map(|file| format!("{name} does not name {file}"))
}

/// The problem with deep instance `name`'s model or turn budget, as
/// its front matter `fields` read, if any.
fn deep_settings(name: &str, fields: &BTreeMap<String, String>) -> Option<String> {
    let field = |key: &str| fields.get(key).map_or("", String::as_str);
    let model = field("model");
    if model != DEEP_MODEL {
        return Some(format!("{name} runs on model {model}, want {DEEP_MODEL}"));
    }
    let turns = field("maxTurns");
    (turns != DEEP_TURNS).then(|| format!("{name} runs for maxTurns {turns}, want {DEEP_TURNS}"))
}

/// The Markdown files directly under `dir` of the checkout, as paths
/// relative to its root, in name order.
///
/// # Errors
///
/// Fails, naming `dir`, when the directory or one of its entries cannot
/// be read, and naming the entry when its name is not UTF-8.
fn markdown_files(checkout: &Checkout, dir: &str) -> Result<Vec<String>, String> {
    // One read of the directory and all its entries, so an error in
    // either is the same named problem.
    let entries = fs::read_dir(checkout.root.join(dir))
        .and_then(|entries| entries.collect::<Result<Vec<_>, _>>())
        .map_err(|e| format!("read {dir}: {e}"))?;
    let mut out = Vec::new();
    for name in entries.iter().map(fs::DirEntry::file_name) {
        let name = name
            .into_string()
            .map_err(|n| format!("{dir}/{} is not UTF-8", n.to_string_lossy()))?;
        if name.ends_with(".md") {
            out.push(format!("{dir}/{name}"));
        }
    }
    out.sort();

    Ok(out)
}

#[cfg(test)]
mod tests;
