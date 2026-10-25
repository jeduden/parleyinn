//! ENG-28: every agent the review bundle can dispatch is defined under
//! `.claude/agents`, so the orchestrator can consult it: each SRS
//! section's specialist, each domain-model file's, the language and
//! test-layer specialists, and the generalists. Specialists run at low
//! effort and generalists at medium, as their front matter says.

use std::fs;

use review_bundle::dispatch::{
    FIXED_SPECIALISTS, GENERALISTS, MODEL, model_specialist, section_specialist,
};

use crate::Checkout;

/// Where the agent definitions live, relative to the checkout.
pub const AGENTS: &str = ".claude/agents";

/// The effort a specialist runs at.
const SPECIALIST_EFFORT: &str = "low";
/// The effort a generalist runs at.
const GENERALIST_EFFORT: &str = "medium";

/// Problems with the agents the review bundle can dispatch: one that is
/// not defined, names itself otherwise, runs at the wrong effort, or,
/// for a section or domain-model specialist, does not name its file.
#[must_use]
pub fn every_dispatched_agent_is_defined(checkout: &Checkout) -> Vec<String> {
    let mut want: Vec<(String, &str, Option<String>)> = Vec::new();
    want.extend(
        GENERALISTS
            .iter()
            .map(|n| ((*n).to_owned(), GENERALIST_EFFORT, None)),
    );
    want.extend(
        FIXED_SPECIALISTS
            .iter()
            .map(|n| ((*n).to_owned(), SPECIALIST_EFFORT, None)),
    );
    for path in markdown_files(checkout, "docs/srs") {
        if let Some(name) = section_specialist(&path) {
            want.push((name, SPECIALIST_EFFORT, Some(path)));
        }
    }
    for path in markdown_files(checkout, MODEL.trim_end_matches('/')) {
        let file = path
            .strip_prefix(MODEL)
            .and_then(|f| f.strip_suffix(".md"))
            .unwrap_or_default();
        if let Some(name) = model_specialist(file) {
            let names = (file != "index").then_some(path);
            want.push((name, SPECIALIST_EFFORT, names));
        }
    }
    want.sort();
    want.dedup();

    want.iter()
        .filter_map(|(name, effort, names)| check(checkout, name, effort, names.as_deref()))
        .collect()
}

/// The problem with agent `name`, if any.
fn check(checkout: &Checkout, name: &str, effort: &str, names: Option<&str>) -> Option<String> {
    let path = format!("{AGENTS}/{name}.md");
    let Ok(text) = checkout.read(&path) else {
        return Some(format!("no agent {name} at {path}"));
    };
    let own = front_matter(&text, "name").unwrap_or_default();
    if own != name {
        return Some(format!("{name} names itself {own}"));
    }
    let has = front_matter(&text, "effort").unwrap_or_default();
    if has != effort {
        return Some(format!("{name} runs at effort {has}, want {effort}"));
    }
    names
        .filter(|file| !text.contains(file))
        .map(|file| format!("{name} does not name {file}"))
}

/// The value of `key` in the front matter of `text`, the block between
/// its opening and closing `---`.
fn front_matter(text: &str, key: &str) -> Option<String> {
    let block = text.strip_prefix("---\n")?;
    let (block, _) = block.split_once("\n---")?;
    block.lines().find_map(|l| {
        l.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
            .map(|v| v.trim().to_owned())
    })
}

/// The Markdown files directly under `dir` of the checkout, as paths
/// relative to its root, in name order.
fn markdown_files(checkout: &Checkout, dir: &str) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(checkout.root.join(dir))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".md"))
        .map(|n| format!("{dir}/{n}"))
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests;
