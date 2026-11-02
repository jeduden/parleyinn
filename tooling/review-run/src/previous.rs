//! The `review-run previous` command: which earlier review of a pull
//! request its findings are carried from (ENG-28). `previous select`
//! reads the artifacts GitHub lists under a pull request's name and the
//! runs that uploaded them, and names the newest one a run of main's
//! review workflow uploaded; an artifact of that name from any other
//! run is skipped and counted, since any workflow a pull request
//! defines may upload one. `previous check` refuses a findings file
//! that is too large or does not read, so a broken history is named
//! and never taken for an empty one (I6). The tool decides and the
//! caller's script only fetches.

use std::io::Write;

use crate::cli::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

/// How to call it.
pub const USAGE: &str = "usage: review-run previous select --pull N --artifacts FILE --runs FILE\n\
       review-run previous check FILE";

/// The workflow whose runs may upload the findings a review carries.
pub const WORKFLOW_PATH: &str = ".github/workflows/review.yml";
/// The events on which main defines that workflow's run: a run on any
/// other event, `pull_request` among them, may use the pull request's
/// own definition.
pub const EVENTS: [&str; 2] = ["workflow_run", "issue_comment"];
/// The most bytes a findings file may hold.
pub const MAX_BYTES: usize = 1 << 20;

/// What the lookup found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The artifact to carry findings from, when one qualified.
    pub artifact: Option<u64>,
    /// The artifacts of the name, unexpired, refused for their origin.
    pub skipped: u32,
}

impl Selection {
    /// The lines the job's outputs take.
    #[must_use]
    pub fn lines(&self) -> String {
        String::new()
    }
}

/// The artifact name a pull request's findings are kept under.
///
/// # Errors
///
/// Fails when `pull` is not a pull request number.
pub fn name(pull: &str) -> Result<String, String> {
    Err(format!("unimplemented: {pull}"))
}

/// Picks the artifact of the pull request `pull` to carry findings
/// from: `artifacts` is GitHub's listing, `runs` the runs it names as
/// an array of `{id, path, event}`.
///
/// # Errors
///
/// Fails when either input does not read, or `pull` is no number.
pub fn select(pull: &str, artifacts: &str, runs: &str) -> Result<Selection, String> {
    Err(format!("unimplemented: {pull} {artifacts} {runs}"))
}

/// Checks a findings file's text.
///
/// # Errors
///
/// Fails when it is past [`MAX_BYTES`] or does not read as findings.
pub fn check(text: &str) -> Result<(), String> {
    Err(format!("unimplemented: {}", text.len()))
}

/// The arguments after `previous` when `args`, past the job summary's
/// flag the executable adds first, name the `previous` command.
#[must_use]
pub fn subcommand(args: &[String]) -> Option<Vec<String>> {
    let _ = args;
    None
}

/// Runs `previous` with `args`, the arguments after it.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let _ = (args, stdout);
    let _ = writeln!(stderr, "review-run previous: unimplemented\n{USAGE}");
    let _ = (EXIT_OK, EXIT_FAILURE, EXIT_USAGE);
    EXIT_USAGE
}

#[cfg(test)]
mod tests;
