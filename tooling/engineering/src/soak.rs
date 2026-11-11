//! ENG-09: the nightly workflow's soak job runs the store's soak test
//! with 50 writers at 1,000,000 events, which is the evidence for that
//! claim, and the pull request workflow's Miri job runs the tests of
//! the one crate that may hold unsafe code.

use crate::review_workflow::{self, Job};
use crate::static_analysis::UNSAFE_SHIM;

/// The soak test's source.
pub const SOAK: &str = "crates/store/tests/soak.rs";

/// The workflow that runs the soak every night.
pub const NIGHTLY: &str = ".github/workflows/nightly.yml";

/// The pull request workflow, which runs Miri.
pub const CI: &str = ".github/workflows/ci.yml";

/// The events the nightly soak must run.
pub const EVENTS: &str = "1000000";

/// The writers the soak runs, as its source declares them.
const WRITERS: &str = "const WRITERS: u8 = 50;";

/// The job `name` of the workflow `text`, comments left out.
fn job(text: &str, name: &str) -> Option<Job> {
    review_workflow::parse(text)
        .jobs
        .into_iter()
        .find(|j| j.name == name)
}

/// Whether some line of `job` holds every one of `parts`, each as whole
/// words: `--test soak` is not held by a line that runs `--test
/// soak_fast`.
fn has_line(job: &Job, parts: &[&str]) -> bool {
    job.text.lines().any(|l| {
        let line = format!(" {} ", l.trim());
        parts.iter().all(|p| line.contains(&format!(" {p} ")))
    })
}

/// What is wrong with the nightly workflow's soak job, given the
/// workflow's text and the soak test's source.
#[must_use]
pub fn nightly_problems(nightly: &str, soak: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !soak.contains(WRITERS) {
        problems.push(format!(
            "{SOAK} does not declare {WRITERS:?}: the soak must run 50 writers"
        ));
    }
    let Some(soak_job) = job(nightly, "soak") else {
        problems.push(format!(
            "{NIGHTLY} has no job \"soak\": add it, running \"cargo test --locked --release -p store --test soak\""
        ));
        return problems;
    };
    if !has_line(&soak_job, &["cargo test", "-p store", "--test soak"]) {
        problems.push(format!(
            "the soak job of {NIGHTLY} does not run \"cargo test -p store --test soak\": the soak test is the evidence"
        ));
    }
    if !has_line(
        &soak_job,
        &["PARINN_SOAK_EVENTS:", &format!("\"{EVENTS}\"")],
    ) {
        problems.push(format!(
            "the soak job of {NIGHTLY} does not set PARINN_SOAK_EVENTS to \"{EVENTS}\": set it, or the {EVENTS}-event claim has no evidence"
        ));
    }

    problems
}

/// What is wrong with the pull request workflow's Miri job, given the
/// workflow's text: it must run the tests of the one crate that may
/// hold unsafe code.
#[must_use]
pub fn miri_problems(ci: &str) -> Vec<String> {
    let krate = UNSAFE_SHIM.trim_start_matches("crates/");
    let Some(miri) = job(ci, "miri") else {
        return vec![format!(
            "{CI} has no job \"miri\": add it, running \"cargo miri test -p {krate}\""
        )];
    };
    if has_line(&miri, &["miri test", &format!("-p {krate}")]) {
        return Vec::new();
    }

    vec![format!(
        "the miri job of {CI} does not run \"cargo miri test -p {krate}\": the {krate} crate holds the unsafe code Miri must run"
    )]
}

#[cfg(test)]
mod tests;
