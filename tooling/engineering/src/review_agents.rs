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
        FIXED_SPECIALISTS
            .iter()
            .map(|n| ((*n).to_owned(), SPECIALIST_EFFORT, None)),
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
    let has = field("effort");
    if has != effort {
        return Some(format!("{name} runs at effort {has}, want {effort}"));
    }
    names
        .filter(|file| !text.contains(file))
        .map(|file| format!("{name} does not name {file}"))
}

/// The Markdown files directly under `dir` of the checkout, as paths
/// relative to its root, in name order.
///
/// # Errors
///
/// Fails, naming `dir`, when the directory or one of its entries cannot
/// be read.
fn markdown_files(checkout: &Checkout, dir: &str) -> Result<Vec<String>, String> {
    // One read of the directory and all its entries, so an error in
    // either is the same named problem.
    let entries = fs::read_dir(checkout.root.join(dir))
        .and_then(|entries| entries.collect::<Result<Vec<_>, _>>())
        .map_err(|e| format!("read {dir}: {e}"))?;
    let mut out: Vec<String> = entries
        .iter()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".md"))
        .map(|n| format!("{dir}/{n}"))
        .collect();
    out.sort();

    Ok(out)
}

#[cfg(test)]
mod tests;
