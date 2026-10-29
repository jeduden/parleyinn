//! The ENG-32 steps: every reviewer component has its evaluation cases,
//! the cases apply and dispatch, the review workflow evaluates what a
//! pull request changes before the gate decides, and the harness grades
//! a run and decides a case as the requirement says.

use cucumber::gherkin::Step;
use cucumber::{then, when};
use engineering::{outcome, review_evaluation, review_workflow};

use crate::World;
use crate::engineering::eng;

/// The review workflow, which the evaluate step reads.
const REVIEW_WORKFLOW: &str = ".github/workflows/review.yml";

#[when(regex = r#"^the reviewer evaluation cases are read from "([^"]+)"$"#)]
fn read_cases(w: &mut World, dir: String) {
    eng(w).cases_dir = dir;
}

#[then(
    regex = r"^every reviewer component, the agents the review bundle can dispatch, the orchestrator, each deep instance and every other agent under \.claude/agents among them, has a case that plants a defect, a case that is clean and a case that addresses the reviewer$"
)]
fn every_component_has_its_cases(w: &mut World) -> Result<(), String> {
    let e = eng(w);
    if e.cases_dir != review_eval::CASES {
        return Err(format!(
            "the cases are read from {}, but the check reads {}",
            e.cases_dir,
            review_eval::CASES
        ));
    }
    outcome(review_evaluation::components_have_their_cases(
        e.checkout()?,
    ))
}

#[then(
    regex = r"^every case applies to the checkout, and the review bundle dispatches its component for it where it dispatches that component at all$"
)]
fn every_case_applies_and_dispatches(w: &mut World) -> Result<(), String> {
    outcome(review_evaluation::cases_apply_and_dispatch(
        eng(w).checkout()?,
    ))
}

#[then(
    regex = r#"^the review workflow's evaluate job runs "([^"]+)" on the components the pull request changes, after the agent's job and before the job that holds the "([^"]+)" key$"#
)]
fn evaluate_job_runs(w: &mut World, cmd: String, key: String) -> Result<(), String> {
    let body = eng(w).checkout()?.read(REVIEW_WORKFLOW)?;
    outcome(review_workflow::parse(&body).evaluate_job_runs(&cmd, &key)?)
}

#[then(regex = r"^a run of a case is graded:$")]
fn a_run_is_graded(_w: &mut World, step: &Step) -> Result<(), String> {
    let table = step.table.as_ref().ok_or("the step has no table")?;
    outcome(review_evaluation::run_graded(&review_workflow::table_rows(
        &table.rows,
    )?))
}

#[then(
    regex = r"^a case that fails is a blocking finding at its component's file, naming the case, the model and the first failure$"
)]
fn a_failed_case_is_a_blocking_finding(_w: &mut World) -> Result<(), String> {
    outcome(review_evaluation::failed_case_is_a_blocking_finding())
}

#[then(
    regex = r"^a scorecard line records a component's cases, runs, slips, cost, turns and seconds on its model$"
)]
fn a_scorecard_line_records_the_suite(_w: &mut World) -> Result<(), String> {
    outcome(review_evaluation::scorecard_line_records_the_suite())
}

#[then(
    regex = r#"^the scheduled run's workflow "([^"]+)" fails on a failed case, with the report that names it in its summary$"#
)]
fn the_scheduled_run_fails(w: &mut World, path: String) -> Result<(), String> {
    let body = eng(w).checkout()?.read(&path)?;
    outcome(review_evaluation::scheduled_run_fails_on_failed_cases(
        &body,
    ))
}

#[then(regex = r"^a case passes with two of three runs, stopping once decided:$")]
fn a_case_is_decided(_w: &mut World, step: &Step) -> Result<(), String> {
    let table = step.table.as_ref().ok_or("the step has no table")?;
    outcome(review_evaluation::case_decided(
        &review_workflow::table_rows(&table.rows)?,
    ))
}
