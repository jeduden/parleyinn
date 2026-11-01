use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_review_run_reuses_answers_by_key_and_stores_only_accepted_ones() {
    let work = TempDir::new().unwrap();

    let problems = review_answers(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn the_checkout_s_review_run_stops_running_an_agent_that_stalled() {
    let work = TempDir::new().unwrap();

    let problems = review_stalls(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_checkout_without_the_definitions_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    let answers = review_answers(&c, work.path());
    let stalls = review_stalls(&c, work.path());

    // Each run fails, and with no run id there is no key to look for.
    assert_eq!(answers.len(), 6, "{answers:?}");
    assert!(
        answers[..4]
            .iter()
            .all(|p| p.starts_with("read .claude/agents/")),
        "{answers:?}"
    );
    assert!(
        answers[4..]
            .iter()
            .all(|p| p.ends_with("No such file or directory (os error 2)"))
    );
    assert_eq!(stalls.len(), 1, "{stalls:?}");
    assert!(stalls[0].starts_with("read .claude/agents/"), "{stalls:?}");
}

#[test]
fn a_review_run_that_fails_is_named_with_its_last_line() {
    let work = TempDir::new().unwrap();
    let store = work.path().join("answers");

    let err = stored(
        &repo(),
        work.path(),
        StoreHost::default(),
        &store,
        &|d| fs::remove_file(d.join("review/dispatch.json")).map_err(|e| e.to_string()),
        &[],
    )
    .unwrap_err();

    assert!(
        err.starts_with("review-run exited 1: review-run: read "),
        "{err}"
    );
}

#[test]
fn a_run_id_without_a_key_or_at_all_is_named() {
    let work = TempDir::new().unwrap();

    let err = key_in(work.path(), GAP).unwrap_err();
    assert!(err.starts_with("read "), "{err}");

    let id = work.path().join(review_run::run_id::RUN_ID);
    fs::create_dir_all(id.parent().unwrap()).unwrap();
    fs::write(
        &id,
        r#"{"head": null, "inputs": "x", "split": 1, "agents": {}}"#,
    )
    .unwrap();
    let err = key_in(work.path(), GAP).unwrap_err();
    assert_eq!(err, format!("review-run recorded no answer key for {GAP}"));
}

#[test]
fn a_refused_report_stored_or_a_poisoned_one_trusted_is_named() {
    let work = TempDir::new().unwrap();
    // A first review leaves no store to tamper with.
    let problems = poisoned(&repo(), work.path(), &work.path().join("answers"));
    assert!(problems[0].starts_with("read "), "{problems:?}");

    let first = work.path().join("first");
    let store = work.path().join("answers");
    stored(
        &repo(),
        &first,
        StoreHost::default(),
        &store,
        &|_| Ok(()),
        &[],
    )
    .unwrap();
    let key = key_in(&first, GAP).unwrap();
    fs::remove_file(store.join(&key).join(review_run::store::REPORT)).unwrap();
    let problems = poisoned(&repo(), work.path(), &store);
    assert!(problems[0].starts_with("tamper with "), "{problems:?}");
}

#[test]
fn a_refusing_host_states_a_line_in_no_form_for_its_agent_alone() {
    let work = TempDir::new().unwrap();
    let store = work.path().join("answers");
    let host = StoreHost {
        refuse: Some(GAP),
        ..StoreHost::default()
    };

    let started = stored(&repo(), work.path(), host, &store, &|_| Ok(()), &[]).unwrap();

    assert!(started.contains(GAP));
    let report = fs::read_to_string(work.path().join(format!("review/reports/{GAP}.md"))).unwrap();
    assert!(report.contains(NO_FORM), "{report}");
    let key = key_in(work.path(), GAP).unwrap();
    assert!(!store.join(key).exists());
}

#[test]
fn a_refusing_host_whose_log_is_gone_is_named() {
    let work = TempDir::new().unwrap();
    let host = StoreHost {
        refuse: Some(GAP),
        ..StoreHost::default()
    };
    let log = work.path().join("missing").join("x.jsonl");
    let command =
        Command::new("claude", &["-p", "x", "--agent", GAP], work.path()).logging_to(&log);

    let err = host.run(&command).unwrap_err();

    assert!(err.starts_with("write "), "{err}");
}

#[test]
fn a_refused_report_found_in_the_store_is_named() {
    let work = TempDir::new().unwrap();
    let store = work.path().join("answers");
    let key = "a".repeat(64);
    let id = work.path().join(review_run::run_id::RUN_ID);
    fs::create_dir_all(id.parent().unwrap()).unwrap();
    let text = format!(
        r#"{{"head": null, "inputs": "x", "split": 1, "agents": {{"{GAP}": {{"key": "{key}"}}}}}}"#
    );
    fs::write(&id, text).unwrap();
    assert!(refused_stored(work.path(), &store).is_empty());

    fs::create_dir_all(store.join(&key)).unwrap();

    assert_eq!(
        refused_stored(work.path(), &store),
        [format!("{GAP}'s refused report was stored as {key}")]
    );
    assert!(refused_stored(&work.path().join("none"), &store)[0].starts_with("read "));
}

#[test]
fn a_poisoned_run_that_cannot_be_prepared_is_named() {
    let work = TempDir::new().unwrap();
    let store = work.path().join("answers");
    stored(
        &repo(),
        &work.path().join("first"),
        StoreHost::default(),
        &store,
        &|_| Ok(()),
        &[],
    )
    .unwrap();
    fs::write(work.path().join("poisoned"), "a file").unwrap();

    let problems = poisoned(&repo(), work.path(), &store);

    assert!(problems[0].starts_with("make "), "{problems:?}");
}

#[test]
fn a_resume_after_a_stall_that_reruns_the_agent_or_does_not_name_it_is_named() {
    let started: BTreeSet<String> = [GAP, ITS_GENERALIST].map(String::from).into();

    let problems = stall_problems(&started, "{}");

    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[1].starts_with("the review after a stall does not say"));
}

#[test]
fn the_checkout_s_review_run_names_a_blocking_finding_kept_three_reviews_as_needing_a_ruling() {
    let work = TempDir::new().unwrap();

    let problems = review_rulings(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn the_checkout_s_review_run_drops_a_stored_answer_by_its_key_and_no_other() {
    let work = TempDir::new().unwrap();

    let problems = review_drops(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_drop_that_leaves_the_answer_or_fails_is_named() {
    let work = TempDir::new().unwrap();
    let key = "a".repeat(64);
    assert!(drop_left(0, work.path(), &key).is_empty());

    fs::create_dir_all(work.path().join(&key)).unwrap();
    for code in [0, 1] {
        let problems = drop_left(code, work.path(), &key);
        assert_eq!(
            problems,
            [format!("answers drop left {GAP}'s answer stored as {key}")]
        );
    }
    assert_eq!(drop_left(1, &work.path().join("none"), &key).len(), 1);
}

#[test]
fn a_drop_check_that_cannot_run_or_review_after_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();
    let problems = review_drops(&c, work.path());
    assert!(
        problems[0].starts_with("read .claude/agents/"),
        "{problems:?}"
    );

    let work = TempDir::new().unwrap();
    fs::write(work.path().join("drop-after"), "a file").unwrap();
    let problems = review_drops(&repo(), work.path());
    assert!(problems[0].starts_with("make "), "{problems:?}");
    assert_eq!(problems.len(), 1);
}

#[test]
fn a_ruling_missing_from_the_third_review_or_early_in_the_second_is_named() {
    let ruling = format!("{}: x", review_run::findings::RULING);
    let none: Vec<String> = Vec::new();

    assert!(ruling_problems(&[none.clone(), none.clone(), vec![ruling.clone()]]).is_empty());
    let problems = ruling_problems(&[none.clone(), vec![ruling], none]);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("second"));
    assert!(problems[1].contains("third"));
}

#[test]
fn a_ruling_check_that_cannot_prepare_run_or_read_the_outcome_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();
    let problems = review_rulings(&c, work.path());
    assert!(
        problems[0].starts_with("read .claude/agents/"),
        "{problems:?}"
    );

    let dir = work.path().join("run");
    fs::create_dir_all(dir.join("pr-head")).unwrap();
    fs::create_dir_all(dir.join("pr-head/src/a.rs")).unwrap();
    let err = ruling_run(&repo(), &dir, None).unwrap_err();
    assert!(err.starts_with("write "), "{err}");

    let blocked = work.path().join("blocked");
    fs::create_dir_all(&blocked).unwrap();
    fs::write(blocked.join("pr-head"), "a file").unwrap();
    let err = ruling_run(&repo(), &blocked, None).unwrap_err();
    assert!(err.starts_with("make "), "{err}");

    let err = outcome_bodies(&work.path().join("none")).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
    let empty = work.path().join("empty");
    fs::create_dir_all(empty.join("review")).unwrap();
    fs::write(empty.join(review_run::OUTCOME), "{").unwrap();
    assert!(outcome_bodies(&empty).unwrap_err().starts_with("parse "));
    fs::write(empty.join(review_run::OUTCOME), "{}").unwrap();
    assert!(outcome_bodies(&empty).unwrap().is_empty());
}

#[test]
fn a_blocking_host_rewrites_each_generalist_s_log_and_names_a_log_that_is_gone() {
    let work = TempDir::new().unwrap();
    let host = StoreHost {
        blocking: true,
        ..StoreHost::default()
    };
    let log = work.path().join("g.jsonl");
    let command = Command::new(
        "claude",
        &["-p", "x", "--agent", "review-code"],
        work.path(),
    )
    .logging_to(&log);

    host.run(&command).unwrap();

    let text = fs::read_to_string(&log).unwrap();
    assert!(text.contains(" — blocking — "), "{text}");
    let log = work.path().join("missing").join("g.jsonl");
    let command = Command::new(
        "claude",
        &["-p", "x", "--agent", "review-code"],
        work.path(),
    )
    .logging_to(&log);
    assert!(host.run(&command).is_err());
}
