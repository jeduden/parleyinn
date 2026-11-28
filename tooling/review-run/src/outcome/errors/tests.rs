use review_eval::component::Kind;
use review_eval::log::RunLog;
use review_gate::Finding;

use super::name;
use crate::outcome::REVIEW_PATH;
use crate::run::AgentRun;

const SILENT: &str = "review-code answered no finding, no dropped one and no `none —` line, so its \
                      weighing cannot be read";

fn run(name: &str, error: &str) -> AgentRun {
    let log = RunLog {
        error: error.to_owned(),
        ..RunLog::default()
    };
    AgentRun::of(name.to_owned(), Kind::Generalist, None, log, 1)
}

fn finding(path: &str, body: &str) -> Finding {
    Finding {
        path: path.to_owned(),
        line: 0,
        severity: "blocking".to_owned(),
        body: body.to_owned(),
    }
}

fn unread(name: &str) -> Vec<(String, String)> {
    vec![(name.to_owned(), "it answered nothing".to_owned())]
}

#[test]
fn the_error_a_run_ended_on_is_added_on_one_line_to_the_finding_on_its_silence() {
    let runs = [run("review-code", "API Error: flagged\n\nDetails: `[x]`\n")];
    let mut findings = [
        finding("src/a.rs", SILENT),
        finding(REVIEW_PATH, "review-spec answered no finding"),
        finding(REVIEW_PATH, SILENT),
    ];

    name(&runs, &unread("review-code"), &mut findings);

    // A finding at another path, or on another agent, is left alone.
    assert_eq!(findings[0].body, SILENT);
    assert_eq!(findings[1].body, "review-spec answered no finding");
    assert_eq!(
        findings[2].body,
        "review-code answered no finding, no dropped one and no `none —` line (its run ended on \
         an error in place of an answer: API Error: flagged Details: `[x]`), so its weighing \
         cannot be read"
    );
}

#[test]
fn a_run_that_ended_on_no_error_keeps_its_finding() {
    let runs = [run("review-code", " \n")];
    let mut findings = [finding(REVIEW_PATH, SILENT)];

    name(&runs, &unread("review-code"), &mut findings);

    assert_eq!(findings[0].body, SILENT);
}

#[test]
fn an_agent_with_no_run_or_no_finding_changes_nothing() {
    let runs = [run("review-code", "API Error: flagged")];
    let mut findings = [finding(REVIEW_PATH, "review-spec weighs nothing")];

    name(&runs, &unread("review-spec"), &mut findings);
    name(&runs, &unread("review-code"), &mut findings);

    assert_eq!(findings[0].body, "review-spec weighs nothing");
}

#[test]
fn a_run_that_ended_on_an_error_but_was_read_keeps_its_finding() {
    // Only an unread generalist's silence is named, whatever the runs.
    let runs = [run("review-code", "API Error: flagged")];
    let mut findings = [finding(REVIEW_PATH, SILENT)];

    name(&runs, &unread("review-spec"), &mut findings);

    assert_eq!(findings[0].body, SILENT);
}
