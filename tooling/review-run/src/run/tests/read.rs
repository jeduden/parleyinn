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
