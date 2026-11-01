//! The ENG-33 steps: every workflow and composite action pins what it
//! uses from outside the repository to a full commit sha and expands
//! no untrusted value in a `run:` script, and CI runs zizmor over them.

use cucumber::{then, when};
use engineering::{outcome, workflows};

use crate::World;
use crate::engineering::eng;

/// The workflows and composite actions the scenario read, each with
/// its text.
#[derive(Debug, Default)]
struct Read {
    files: Vec<(String, String)>,
}

#[when(
    regex = r#"^every workflow under "\.github/workflows" and composite action under "\.github/actions" is read$"#
)]
fn the_workflows_are_read(w: &mut World) -> Result<(), String> {
    let checkout = eng(w).checkout()?.clone();
    let files = workflows::files(&checkout)?
        .into_iter()
        .map(|f| checkout.read(&f).map(|body| (f, body)))
        .collect::<Result<Vec<_>, _>>()?;
    if files.is_empty() {
        return Err(format!("{} holds no workflow", workflows::WORKFLOWS));
    }
    w.section::<Read>().files = files;
    Ok(())
}

/// Every problem `check` finds in the files read.
fn each_file(w: &mut World, check: fn(&str, &str) -> Vec<String>) -> Vec<String> {
    w.section::<Read>()
        .files
        .iter()
        .flat_map(|(file, body)| check(file, body))
        .collect()
}

#[then(
    regex = r"^every action and reusable workflow they use from outside the repository is pinned to a full commit sha, with its version in a comment, and every docker:// image to its sha256 digest$"
)]
fn every_use_is_pinned(w: &mut World) -> Result<(), String> {
    outcome(each_file(w, workflows::unpinned))
}

#[then(
    regex = r"^no run script expands a github\.event field, github\.head_ref or an inputs value with an expression$"
)]
fn no_script_expands_untrusted_values(w: &mut World) -> Result<(), String> {
    outcome(each_file(w, workflows::untrusted_expansions))
}

#[then(regex = r#"^the CI workflow runs "zizmor \.github"$"#)]
fn ci_runs_zizmor(w: &mut World) -> Result<(), String> {
    let ci = eng(w).checkout()?.read(workflows::CI)?;

    outcome(workflows::runs_zizmor(&ci))
}
