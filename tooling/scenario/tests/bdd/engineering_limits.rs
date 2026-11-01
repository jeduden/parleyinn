//! The ENG-34 steps: the review bundle flags, with no agent, a changed
//! file past its kind's line limit and a changed file no review
//! specialist reads, each a nit in the finding form, and its limits are
//! those the ENG-34 row states.

use cucumber::{then, when};
use engineering::file_sizes;
use engineering::outcome;
use engineering::review_limits::{self, Built, OVERSIZE, UNREAD};

use crate::World;
use crate::engineering::eng;

/// What the review bundle wrote for the change the scenario built, and
/// what the size check found in its fixture tree.
#[derive(Debug, Default)]
struct Read {
    built: Option<Built>,
    sizes: Vec<String>,
}

#[then(regex = r"^the review bundle's line limits are those the ENG-34 row states$")]
fn limits_match_the_row(w: &mut World) -> Result<(), String> {
    outcome(review_limits::limits_match_the_row(eng(w).checkout()?))
}

/// Runs the review bundle on a change that adds `files`.
fn build(w: &mut World, files: &[(&str, String)]) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    w.section::<Read>().built = Some(review_limits::build(work.path(), files)?);
    Ok(())
}

#[when(
    regex = r"^the review bundle is built for a change that adds a Rust file past its limit and a file no review specialist reads$"
)]
fn built_past_the_limit(w: &mut World) -> Result<(), String> {
    build(w, &review_limits::past_the_limit())
}

#[when(
    regex = r"^the review bundle is built for a change within its limits that review specialists read$"
)]
fn built_within_the_limits(w: &mut World) -> Result<(), String> {
    build(w, &review_limits::within_the_limits())
}

/// What the review bundle wrote, or why there is none.
fn built(w: &mut World) -> Result<Built, String> {
    w.section::<Read>()
        .built
        .clone()
        .ok_or_else(|| "no review bundle was built".to_owned())
}

#[then(
    regex = r"^its deterministic findings hold a nit at the Rust file in the finding form, citing ENG-34$"
)]
fn nit_at_the_rust_file(w: &mut World) -> Result<(), String> {
    outcome(review_limits::nit_at(
        &built(w)?,
        OVERSIZE,
        review_limits::ROW,
    ))
}

#[then(
    regex = r"^its deterministic findings hold a nit at the unread file in the finding form, citing ENG-28$"
)]
fn nit_at_the_unread_file(w: &mut World) -> Result<(), String> {
    outcome(review_limits::nit_at(
        &built(w)?,
        UNREAD,
        review_limits::UNREAD_ID,
    ))
}

#[then(regex = r"^the review bundle lists both nits$")]
fn the_bundle_lists_both(w: &mut World) -> Result<(), String> {
    outcome(review_limits::listed(&built(w)?))
}

#[then(regex = r"^its deterministic findings hold no nit citing ENG-34 or ENG-28$")]
fn no_nit(w: &mut World) -> Result<(), String> {
    outcome(review_limits::no_nit(&built(w)?))
}

#[then(
    regex = r"^no file of the checkout is past its kind's line limit unless the recorded sizes list it at no fewer lines than it holds$"
)]
fn no_file_past_its_limit(w: &mut World) -> Result<(), String> {
    outcome(file_sizes::check_parts(eng(w).checkout()?).0)
}

#[then(regex = r"^the recorded sizes list no file that fits its limit and no file that is gone$")]
fn the_record_lists_no_stale_file(w: &mut World) -> Result<(), String> {
    outcome(file_sizes::check_parts(eng(w).checkout()?).1)
}

#[when(
    regex = r"^the size check reads a tree with a listed file grown, a file past its limit that is not listed, a listed file that fits and a listed file that is gone$"
)]
fn size_check_reads_a_drifted_tree(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    w.section::<Read>().sizes = file_sizes::fixture_problems(work.path())?;
    Ok(())
}

#[then(
    regex = r"^it names the grown file, the unlisted file, the file that fits and the file that is gone$"
)]
fn it_names_each_drift(w: &mut World) -> Result<(), String> {
    outcome(file_sizes::names_each_drift(&w.section::<Read>().sizes))
}

#[then(regex = r"^it names no file held at its recorded size or within its limit$")]
fn it_names_no_held_file(w: &mut World) -> Result<(), String> {
    outcome(file_sizes::names_no_held_file(&w.section::<Read>().sizes))
}
