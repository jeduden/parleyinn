//! ENG-07: coverage-guided fuzzing covers every parser the requirement
//! names. Each input has a cargo-fuzz target declared in the fuzz
//! workspace, a seed corpus committed under `fuzz/corpus/<target>`, and
//! the nightly workflow's fuzz job runs it from that corpus on a
//! schedule.

use crate::review_workflow;

/// The fuzz workspace's manifest.
pub const MANIFEST: &str = "fuzz/Cargo.toml";

/// Where each target's seed corpus is committed.
pub const CORPUS: &str = "fuzz/corpus";

/// The workflow that fuzzes every night.
pub const NIGHTLY: &str = ".github/workflows/nightly.yml";

/// The nightly job that fuzzes.
const JOB: &str = "fuzz";

/// The loop the nightly job fuzzes every declared target with.
const EVERY_TARGET: &str = "fuzz list";

/// Each input ENG-07 names, in the scenario's words, and its target.
pub const INPUTS: [(&str, &str); 7] = [
    ("transcript parser", "transcript_parser"),
    ("hook input decoder", "hook_input_decoder"),
    ("event_search query parser", "query_parser"),
    ("envelope encoder", "envelope_encoder"),
    ("field sanitizer", "field_sanitizer"),
    ("redactor", "redactor"),
    ("configuration parser", "configuration_parser"),
];

/// The target that fuzzes `input`.
///
/// # Errors
///
/// Fails when ENG-07 names no such input.
pub fn target_for(input: &str) -> Result<&'static str, String> {
    INPUTS
        .iter()
        .find(|(name, _)| *name == input)
        .map(|(_, target)| *target)
        .ok_or_else(|| format!("ENG-07 names no input {input:?}"))
}

/// The targets `manifest` declares, each with its source path relative
/// to the fuzz workspace, in the manifest's order.
#[must_use]
pub fn targets(manifest: &str) -> Vec<(String, String)> {
    manifest
        .split("[[bin]]")
        .skip(1)
        .filter_map(|block| Some((value(block, "name")?, value(block, "path")?)))
        .collect()
}

/// The quoted value of `key` in a manifest block.
fn value(block: &str, key: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let (k, v) = line.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_owned())
    })
}

/// What keeps `target` from being a cargo-fuzz target: undeclared in
/// `manifest`, or a source `read` cannot read or that has no
/// `fuzz_target!` entry point. `read` takes a path relative to the
/// checkout.
pub fn target_problems(
    manifest: &str,
    target: &str,
    read: impl Fn(&str) -> Result<String, String>,
) -> Vec<String> {
    let Some((_, path)) = targets(manifest).into_iter().find(|(n, _)| n == target) else {
        return vec![format!("{MANIFEST} declares no target {target}")];
    };
    let file = format!("fuzz/{path}");
    match read(&file) {
        Ok(source) if source.contains("fuzz_target!") => Vec::new(),
        Ok(_) => vec![format!("{file} has no fuzz_target! entry point")],
        Err(e) => vec![e],
    }
}

/// What keeps `seeds`, the files of `target`'s corpus, from seeding it.
#[must_use]
pub fn corpus_problems(target: &str, seeds: &[String]) -> Vec<String> {
    if seeds.is_empty() {
        vec![format!("{CORPUS}/{target} holds no seed")]
    } else {
        Vec::new()
    }
}

/// What keeps the nightly workflow `text` from fuzzing `target` from
/// its committed corpus on a schedule.
#[must_use]
pub fn nightly_problems(text: &str, target: &str) -> Vec<String> {
    let workflow = review_workflow::parse(text);
    if !workflow.triggers.iter().any(|t| t == "schedule") {
        return vec!["the nightly workflow runs on no schedule".to_owned()];
    }
    let Some(job) = workflow.jobs.iter().find(|j| j.name == JOB) else {
        return vec!["the nightly workflow has no fuzz job".to_owned()];
    };
    let every = job.text.contains(EVERY_TARGET);
    if !job.text.contains("fuzz run") || !(every || job.text.contains(target)) {
        return vec![format!("the nightly fuzz job does not fuzz {target}")];
    }
    let looped = format!("{CORPUS}/$target");
    if job.text.contains(&looped) || job.text.contains(&format!("{CORPUS}/{target}")) {
        Vec::new()
    } else {
        vec![format!(
            "the nightly fuzz job does not fuzz {target} from {CORPUS}/{target}"
        )]
    }
}

#[cfg(test)]
mod tests;
