//! What a specialist's run is read as from its log: one whose diff is
//! not there, and one cut off before it stated anything, are no whole
//! review.

use super::*;

#[test]
fn a_specialist_whose_diff_is_not_found_is_partial_naming_it() {
    let dir = workdir();
    fs::remove_file(dir.path().join("review/diffs/lang-rust.diff")).unwrap();

    let run = one_run(dir.path(), |c| fake::answer(c, &[], "outcome: none — x."));

    assert!(run.answered());
    assert!(!run.whole());
    assert_eq!(
        run.partial,
        [(
            None,
            "its diff review/diffs/lang-rust.diff was not found, so no read of it can be shown"
                .to_owned()
        )]
    );
}

#[test]
fn a_specialist_cut_off_before_it_stated_anything_is_no_whole_review() {
    let dir = workdir();

    let run = one_run(dir.path(), |c| {
        let reads = fake::reads(c);
        let log = fake::reading_log(&reads, &["I read the diff."], "error_max_turns", "", 0.03);
        fake::write_log(c, &log);
        Ok(Output::default())
    });

    assert!(run.answered(), "{:?}", run.failure);
    assert!(run.partial.is_empty(), "{:?}", run.partial);
    assert_eq!(
        run.stated_nothing,
        [(None, "its run ended with error_max_turns".to_owned())]
    );
    assert!(!run.whole());
    let report = fs::read_to_string(dir.path().join(report_file("lang-rust"))).unwrap();
    assert_eq!(
        report,
        format!("{}\n", review_gate::report::CUT_BEFORE_STATING)
    );
}

#[test]
fn a_whole_specialist_whose_diff_is_more_lines_than_one_read_takes_is_told_the_ranges() {
    let dir = workdir();
    let lines = review_eval::coverage::DEFAULT_LIMIT + 1;
    let diff = dir.path().join("review/diffs/lang-rust.diff");
    fs::write(&diff, "+a\n".repeat(lines)).unwrap();
    let host = FakeHost::new(fake::either);

    let runs = review_with(
        &host,
        dir.path(),
        &plan(r#"{"review-code": ["lang-rust"]}"#),
    );

    let calls = host.calls.lock().unwrap();
    let prompt = prompt_of(calls.iter().find(|c| agent_of(c) == "lang-rust").unwrap());
    assert!(
        prompt.ends_with(
            "One `Read` does not return `review/diffs/lang-rust.diff` whole: read its 2001 lines \
             in 2 calls, each with `offset` and `limit`, lines 1-2000 and 2001-2001, and leave no \
             line unread."
        ),
        "{prompt}"
    );
    assert!(runs[0].whole(), "{:?}", runs[0].partial);
}

#[test]
fn a_whole_specialist_whose_diff_one_read_returns_is_told_no_ranges() {
    let dir = workdir();
    fs::write(dir.path().join("review/diffs/lang-rust.diff"), "+a\n").unwrap();
    let host = FakeHost::new(fake::either);

    review_with(
        &host,
        dir.path(),
        &plan(r#"{"review-code": ["lang-rust"]}"#),
    );

    let calls = host.calls.lock().unwrap();
    let prompt = prompt_of(calls.iter().find(|c| agent_of(c) == "lang-rust").unwrap());
    assert!(!prompt.contains("does not return"), "{prompt}");
}
