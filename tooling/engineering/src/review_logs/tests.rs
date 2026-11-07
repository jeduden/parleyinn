use std::fs;
use std::path::Path;

use review_eval::host::{Command, Host};
use testkit::TempDir;

use super::*;
use crate::tests::checkout;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_review_run_reads_every_log_whatever_its_run_ended_with() {
    let work = TempDir::new().unwrap();

    let problems = review_run_reads_every_log(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn every_mode_has_its_own_name() {
    let names: Vec<&str> = Mode::ALL.iter().map(|m| m.name()).collect();

    assert_eq!(
        names,
        [
            "cut",
            "cut-silent",
            "partial",
            "all-well",
            "malformed",
            "silent"
        ]
    );
}

#[test]
fn nothing_written_is_named_for_every_mode() {
    let problems: Vec<String> = Mode::ALL
        .iter()
        .flat_map(|mode| mode_problems(*mode, &Wrote::read(Path::new("/nonexistent")), 0))
        .collect();

    assert_eq!(
        problems,
        [
            "cut: a specialist cut off after reading its whole diff is not consulted",
            "cut: the finding the specialist stated before it was cut off is not its report",
            "cut: a review whose cut specialist read its whole diff is not decided as any other",
            "cut: a specialist cut off after it was charged is run again",
            "cut-silent: a specialist cut off before it stated anything is not consulted",
            "cut-silent: a specialist cut off before it stated anything is not named partial",
            "cut-silent: a specialist cut off before it stated anything is not named in a \
             blocking finding",
            "cut-silent: the report of a specialist cut off before it stated anything reads as \
             a clean pass",
            "cut-silent: the gate does not post the request for changes a specialist cut off \
             before it stated anything makes",
            "cut-silent: a specialist cut off after it was charged is run again",
            "partial: a specialist that read part of its diff is not consulted",
            "partial: the lines a specialist did not read are not its gap",
            "partial: the lines a specialist did not read are not named in a blocking finding",
            "partial: the gate does not post the request for changes a partial specialist makes",
            "all-well: the finding beside a line saying all is well is not kept",
            "all-well: a line saying all is well is not listed in review/ignored.json",
            "all-well: a line set aside as no finding blocks the review",
            "malformed: the finding beside a line in no form is not kept",
            "malformed: a line in no form is not its own blocking finding quoting it",
            "malformed: the gate does not post the request for changes a line in no form makes",
            "silent: a specialist that did not run and cost nothing is not run once more",
            "silent: a specialist that did not run is not named in a blocking finding",
            "silent: the gate posts a review a dispatched specialist did not run in",
        ]
    );
}

#[test]
fn a_specialist_named_not_reviewed_or_consulted_when_it_did_not_run_is_named() {
    let wrote = Wrote {
        consulted: format!("[\"{SPECIALIST}\"]"),
        gaps: vec![review_run::gaps::Gap::new(
            SPECIALIST,
            None,
            "partial: x".to_owned(),
        )],
        gate: Ok(review_gate::APPROVE.to_owned()),
        ..Wrote::default()
    };

    let cut = mode_problems(Mode::Cut, &wrote, 1);
    let silent = mode_problems(Mode::Silent, &wrote, 2);

    assert!(
        cut.contains(
            &"cut: a specialist that read its whole diff is named not reviewed".to_owned()
        ),
        "{cut:?}"
    );
    assert_eq!(
        silent,
        [
            "silent: a specialist whose log holds no model call is consulted",
            "silent: a specialist that did not run is not named in a blocking finding",
            "silent: the gate posts a review a dispatched specialist did not run in",
        ]
    );
}

#[test]
fn a_checkout_without_the_definitions_or_a_review_run_that_fails_is_named_by_mode() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let problems = review_run_reads_every_log(&c, work.path());

    assert_eq!(problems.len(), Mode::ALL.len(), "{problems:?}");
    assert!(
        problems[0].starts_with("read .claude/agents/"),
        "{problems:?}"
    );

    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("cut/review/outcome.json")).unwrap();
    let problems = review_run_reads_every_log(&repo(), work.path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("cut: review-run exited 1: review-run: write "),
        "{problems:?}"
    );
}

#[test]
fn a_diff_that_cannot_be_written_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join(format!("review/diffs/{SPECIALIST}.diff"))).unwrap();

    let err = prepare(&repo(), work.path()).unwrap_err();

    assert!(err.starts_with("write "), "{err}");
}

#[test]
fn the_stand_in_fails_a_run_that_logs_nowhere_or_cannot_log() {
    let work = TempDir::new().unwrap();
    let host = LogHost::new(Mode::Cut);
    let args = ["--agent", GENERALIST];
    let nowhere = Command::new("claude", &args, work.path());

    assert_eq!(
        host.run(&nowhere),
        Err("the run logs to no file".to_owned())
    );
    let unwritable = nowhere.logging_to(&work.path().join("no/such/dir/log.jsonl"));
    assert!(host.run(&unwritable).unwrap_err().starts_with("write "));
    assert_eq!(host.runs(), 0);
}

#[test]
fn a_silent_specialist_logs_a_result_alone_and_a_partial_one_reads_its_first_lines() {
    let work = TempDir::new().unwrap();
    let prompt = format!("Read only your own diff, `review/diffs/{SPECIALIST}.diff`, alone");
    let command = Command::new("claude", &["-p", &prompt], work.path());

    let silent = review_eval::log::parse(&specialist_log(&command, Mode::Silent));
    let partial = review_eval::log::parse(&specialist_log(&command, Mode::Partial));

    assert!(silent.texts.is_empty() && silent.reads.is_empty() && silent.cost == 0.0);
    assert_eq!(partial.reads.len(), 1);
    assert_eq!(
        (partial.reads[0].offset, partial.reads[0].limit),
        (Some(1), Some(PART))
    );
    assert!(
        partial.reads[0]
            .path
            .ends_with("review/diffs/lang-rust.diff")
    );
}
