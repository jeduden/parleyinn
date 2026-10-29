//! The ENG-28 steps: the review workflow keeps the reviewing agent away
//! from the reviewer app's key, and the review gate decides as the
//! requirement says.

use cucumber::gherkin::Step;
use cucumber::{then, when};
use engineering::{outcome, review_workflow};

use crate::World;
use crate::engineering::eng;

#[when(regex = r#"^the review workflow is read from "([^"]+)"$"#)]
fn read_review_workflow(w: &mut World, file: String) -> Result<(), String> {
    let body = eng(w).checkout()?.read(&file)?;
    eng(w).flow = review_workflow::parse(&body);
    Ok(())
}

#[then(
    regex = r#"^it runs only when the "([^"]+)" workflow completes or a pull request gets a comment, as the default branch defines it$"#
)]
fn runs_after(w: &mut World, name: String) -> Result<(), String> {
    eng(w).flow.runs_after(&name)
}

#[then(
    regex = r#"^the job that runs the reviewing agent holds no write permission and not the "([^"]+)" key$"#
)]
fn agents_unprivileged(w: &mut World, key: String) -> Result<(), String> {
    let e = eng(w);
    e.key = key;
    outcome(e.flow.agents_unprivileged(&e.key))
}

#[then(regex = r#"^its agents may write only their reports, under "([^"]+)"$"#)]
fn agents_write_only(w: &mut World, dir: String) -> Result<(), String> {
    outcome(eng(w).flow.agents_write_only(&dir))
}

#[then(regex = r#"^only one job holds the "([^"]+)" key, and it runs no agent$"#)]
fn key_held_apart(w: &mut World, key: String) -> Result<(), String> {
    let e = eng(w);
    e.key = key;
    e.flow.key_held_apart(&e.key)
}

#[then(regex = r"^that job runs only after the agent's job succeeded$")]
fn key_job_after_agents(w: &mut World) -> Result<(), String> {
    let e = eng(w);
    outcome(e.flow.key_job_after_agents(e.key()?)?)
}

#[then(regex = r#"^that job decides the review with "([^"]+)"$"#)]
fn key_job_runs(w: &mut World, cmd: String) -> Result<(), String> {
    let e = eng(w);
    e.flow.key_job_runs(e.key()?, &cmd)
}

#[then(regex = r#"^that job posts its decision as the "([^"]+)" check$"#)]
fn key_job_posts_check(w: &mut World, check: String) -> Result<(), String> {
    let e = eng(w);
    outcome(e.flow.key_job_posts_check(e.key()?, &check)?)
}

#[then(
    regex = r"^every language the repository's source and configuration are written in has its specialist$"
)]
fn every_language_has_its_specialist(w: &mut World) -> Result<(), String> {
    outcome(engineering::review_languages::every_language_has_its_specialist(eng(w).checkout()?))
}

#[then(
    regex = r"^every agent the review bundle can dispatch is defined, specialists at low effort with 20 turns, generalists at medium and their deep instances on opus at high effort with 40 turns$"
)]
fn review_agents_defined(w: &mut World) -> Result<(), String> {
    outcome(engineering::review_agents::every_dispatched_agent_is_defined(eng(w).checkout()?))
}

#[then(regex = r"^the review gate decides:$")]
fn review_gate_decides(_w: &mut World, step: &Step) -> Result<(), String> {
    let table = step.table.as_ref().ok_or("the step has no table")?;
    outcome(review_workflow::gate_decides(&review_workflow::table_rows(
        &table.rows,
    )?))
}
