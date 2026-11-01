//! The ENG-28 steps: the review workflow runs the review's agents by
//! code, keeps them away from the reviewer app's key, and the review gate
//! decides as the requirement says.

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
    regex = r#"^the job that runs the review's agents holds no write permission and not the "([^"]+)" key$"#
)]
fn agents_unprivileged(w: &mut World, key: String) -> Result<(), String> {
    let e = eng(w);
    e.key = key;
    // The run logs hold every tool result an agent read: they never
    // leave the job.
    let mut problems = e.flow.agents_unprivileged(&e.key);
    problems.extend(e.flow.run_logs_stay(review_workflow::RUN_LOGS));
    outcome(problems)
}

#[then(
    regex = r#"^that job runs every agent the review bundle dispatches with "([^"]+)", and no agent routes the review$"#
)]
fn agents_run_by(w: &mut World, cmd: String) -> Result<(), String> {
    outcome(eng(w).flow.agents_run_by(&cmd))
}

#[then(
    regex = r"^review-run runs each specialist the review bundle dispatches as its own agent, assembles its report from its run log, runs every generalist only once every specialist has ended, and writes the review outcome from what they answered$"
)]
fn review_runs_by_code(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_runs::review_runs_by_code(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run runs a specialist whose diff is past its split limit in parts, each on whole files of its diff, merges their reports into its one report, and fails the review naming a part that did not answer$"
)]
fn review_runs_in_parts(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_parts::review_runs_in_parts(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run refuses a specialist's report that cites an id no requirement or invariant has, runs that specialist once more told why, and names the report refused again and a generalist's finding line it cannot read, each in a blocking finding, so the review requests changes and the gate posts that request rather than failing$"
)]
fn review_run_names_refusals(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(
        engineering::review_refusals::review_run_names_what_it_refuses(
            eng(w).checkout()?,
            work.path(),
        ),
    )
}

#[then(
    regex = r"^review-run writes each dispatched agent left without an accepted report beside the review outcome and names it not reviewed, and a review resumed from that list runs again that agent, each agent whose head, inputs, definition or diff changed and each generalist whose specialist runs again, keeps nothing across a changed head, definition or diff, and approves as a full run would$"
)]
fn review_run_resumes(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_resume::review_resumes(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(regex = r"^its agents can write nothing: review-run allows them Read, Glob and Grep alone$")]
fn agents_write_nothing(w: &mut World) -> Result<(), String> {
    outcome(
        eng(w)
            .flow
            .agents_write_nothing(&review_workflow::RUN_TOOLS),
    )
}

#[then(
    regex = r#"^no step that runs an agent holds the "([^"]+)" token: one step in each agent job stages it where no agent reads it, review-run refuses its agents every read of the staged credential's directory by name, of the process, system and device files and outside their trees, and every file an agent job uploads waits for a scrub of that token$"#
)]
fn token_kept_from_agents(w: &mut World, token: String) -> Result<(), String> {
    let mut problems = review_workflow::every_workflow_keeps_the_token(eng(w).checkout()?, &token)?;
    problems.extend(review_workflow::Workflow::reads_refused(
        review_eval::runner::DISALLOWED,
        review_eval::runner::SETTINGS,
    ));
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    problems.extend(engineering::review_leak::review_run_refuses_the_credential(
        eng(w).checkout()?,
        work.path(),
    ));
    outcome(problems)
}

#[then(
    regex = r"^every text the review writes, prints or uploads is scrubbed of that token: review-run, its agents' output holding the token, leaves it in no file under review/ and in no line it prints$"
)]
fn review_run_scrubs(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_leak::review_run_scrubs_what_it_writes(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^a review in which an agent's output held that token requests changes, naming the leak in a blocking finding, and fails, so nothing is posted$"
)]
fn review_run_fails_closed(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_leak::review_run_fails_closed_on_a_leak(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(regex = r#"^only one job holds the "([^"]+)" key, and it runs no agent$"#)]
fn key_held_apart(w: &mut World, key: String) -> Result<(), String> {
    let e = eng(w);
    e.key = key;
    e.flow.key_held_apart(&e.key)
}

#[then(regex = r"^that job runs only after the agents' job succeeded$")]
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
