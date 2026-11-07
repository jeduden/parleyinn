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
    eng(w).flow = review_workflow::load(eng(w).checkout()?, &file)?;
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
    regex = r"^review-run runs each specialist the review bundle dispatches as its own agent, told to read only its own diff, its part's when the diff runs in parts, and the review context the review bundle wrote for it, which names the review class, the touched ids, its dispatch row and the deterministic findings on its files, never the whole bundle, assembles its report from its run log, runs each generalist once its own specialists have ended and the mass failure is decided, and writes the review outcome from what they answered$"
)]
fn review_runs_by_code(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_runs::review_runs_by_code(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run passes each agent the cost cap of its tier, starts a run only once its tier's cap fits in the pull request's cap beside what the earlier reviews spent, what the runs that ended were charged and the caps of the runs still running, none at all when the earlier reviews reached it, charges a run paid for that left no result its tier's cap and one whose result reported no cost nothing, names each agent it did not start over budget, so the review requests changes, and keeps what the pull request's reviews spent$"
)]
fn review_run_caps_cost(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    let mut problems = engineering::review_controls::caps_problems(eng(w).checkout()?, work.path());
    problems.extend(engineering::review_spend::cap_problems(
        eng(w).checkout()?,
        &work.path().join("spend"),
    ));
    outcome(problems)
}

#[then(
    regex = r"^review-run starts no generalist once more than a third of the dispatched specialists, those a cost cap stopped or kept from starting among them, ended without an accepted report, and opens the review outcome with a blocking finding naming the mass failure$"
)]
fn review_run_breaks(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_controls::breaker_problems(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run waits with backoff before it runs again an agent the API turned away as overloaded or rate-limited, runs it a third time at most, leaves it a named gap when no deadline is left to wait, and never runs again one a cost cap stopped, whose credential was refused or that was charged$"
)]
fn review_run_backs_off(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_controls::retry_problems(
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
    regex = r"^review-run reads a generalist's weighing from its whole run log, so a finding it stated before a final answer that only names its scope is kept, and itself decides partial a weighing that weighs no finding at a place its specialist reported, naming that place in a blocking finding and the generalist as not reviewed, whatever the generalist says of its scope$"
)]
fn review_run_reads_whole_weighings(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(
        engineering::review_weighings::review_run_reads_whole_weighings(
            eng(w).checkout()?,
            work.path(),
        ),
    )
}

#[then(
    regex = r"^review-run refuses a specialist's report that cites an id no requirement or invariant has, does not run that specialist again, it was paid for, and names its refused report and a generalist's finding line it cannot read, each in a blocking finding, so the review requests changes and the gate posts that request rather than failing$"
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
    regex = r"^review-run writes each dispatched agent left without an accepted report beside the review outcome and names it not reviewed, and a review resumed from that list runs again that agent, each agent whose answer key changed and each generalist whose specialist runs again, keeps every agent whose key is unchanged on a new commit, keeps nothing across a changed file every key reads, and approves as a full run would$"
)]
fn review_run_resumes(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_resume::review_resumes(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run given an answer store takes every accepted answer stored under an unchanged key, runs on a new commit only the specialist whose diff changed and its generalist, never stores a refused report, and runs again an agent whose stored report fails the report check$"
)]
fn review_run_answer_store(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_answers::review_answers(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run removes the answer stored under a key on request and no other, so a review given the store runs that agent again and nothing else$"
)]
fn review_run_drops_an_answer(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_answers::review_drops(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run names a blocking finding kept in three reviews in a row with its cited file unchanged in a blocking finding of its own, as needing a stakeholder ruling$"
)]
fn review_run_names_rulings(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_answers::review_rulings(
        eng(w).checkout()?,
        work.path(),
    ))
}

#[then(
    regex = r"^review-run carries the findings of the previous review of a pull request from the newest artifact under the pull request's own name that a run of main's review workflow uploaded, refuses and counts an artifact of that name from any other run, names a findings file past 1 MiB or one that does not read, and the review job fetches them before the Claude Code OAuth token is staged, passes them as --previous, scrubs the findings it kept and keeps them for 90 days for the next review, and says in the job summary which earlier review it carried or that it found none$"
)]
fn review_carries_findings(w: &mut World) -> Result<(), String> {
    outcome(engineering::review_previous::review_carries_findings(
        eng(w).checkout()?,
    )?)
}

#[then(
    regex = r"^review-run sums what each earlier review of a pull request spent from every artifact under the pull request's own name, of any run attempt, that a run of main's review workflow uploaded, an earlier attempt of its own run's included, refuses and counts an artifact of that name from any other run, names a spend file past 1 KiB or one that does not read, and the review job fetches them before the Claude Code OAuth token is staged, fails on a run behind one it cannot look up, passes the sum as --spent-before, scrubs what this review spent and keeps it for 90 days for the next review, under its run's attempt and replacing none, however the review ended, and says in the job summary how many earlier reviews it summed and what they spent or that it found none$"
)]
fn review_carries_spend(w: &mut World) -> Result<(), String> {
    outcome(engineering::review_spend::carry_problems(
        eng(w).checkout()?,
    )?)
}

#[then(
    regex = r"^review-run runs no more an agent whose gap recurred the same way in two resumes in a row, and names it stalled in a blocking finding so a person acts$"
)]
fn review_run_stalls(w: &mut World) -> Result<(), String> {
    let work = testkit::TempDir::new().map_err(|e| format!("make a work directory: {e}"))?;
    outcome(engineering::review_answers::review_stalls(
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
    regex = r"^every agent the review bundle can dispatch is defined, specialists at low effort, generalists at medium and their deep instances each naming its own effort, model and turn budget$"
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
