use super::*;

fn gap(name: &str, part: Option<usize>, reason: &str) -> Gap {
    Gap::new(name, part, reason.to_owned())
}

/// The stalls of `runs` runs in a row, each ending in `gaps`.
fn runs(n: usize, gaps: &[Gap]) -> Vec<Stall> {
    (0..n).fold(Vec::new(), |earlier, _| tally(gaps, &earlier))
}

#[test]
fn a_gap_that_counts_is_one_of_its_way() {
    for (reason, class) in [
        ("silent: x", Some("silent:")),
        ("incomplete: x", Some("incomplete:")),
        ("over budget: x", Some("over budget:")),
        ("refused: x", Some("refused:")),
        ("unread: x", Some("unread:")),
        // A specialist that skims its diff, or states a line that does
        // not read, every run is no less stuck.
        ("partial: x", Some("partial:")),
        ("unreadable: x", Some("unreadable:")),
        ("aborted: x", None),
        ("stalled: x", None),
    ] {
        assert_eq!(class_of(reason), class, "{reason}");
    }
}

#[test]
fn the_same_gap_in_a_row_counts_up_and_another_starts_again() {
    let silent = [gap("lang-rust", None, "silent: a")];

    let three = runs(3, &silent);

    assert_eq!(three.len(), 1);
    assert_eq!(three[0].times, 3);
    assert_eq!(three[0].reasons, ["silent: a"; 3]);
    let refused = tally(&[gap("lang-rust", None, "refused: b")], &three);
    assert_eq!(refused[0].times, 1);
    let part = tally(&[gap("lang-rust", Some(2), "silent: a")], &three);
    assert_eq!(part[0].times, 1);
    let aborted = tally(&[gap("review-code", None, "aborted: c")], &three);
    assert!(aborted.is_empty());
    // A full run starts every count again.
    assert_eq!(tally(&silent, &[])[0].times, 1);
}

#[test]
fn a_gap_stalls_once_it_recurred_in_more_resumes_than_allowed() {
    let silent = [gap("lang-rust", None, "silent: a")];
    for (n, stuck_now) in [(1, false), (2, false), (3, true)] {
        let stalls = runs(n, &silent);
        assert_eq!(stalls[0].stuck(MAX_RESUMES), stuck_now, "{n}");
        assert_eq!(stuck(&stalls, MAX_RESUMES).len(), usize::from(stuck_now));
    }
    let stalls = runs(2, &silent);
    assert_eq!(stuck(&stalls, 1).len(), 1);
}

#[test]
fn a_stalled_agent_s_why_names_it_its_count_and_every_reason() {
    let stalls = runs(
        3,
        &[
            gap("lang-rust", Some(1), "silent: a"),
            gap("lang-rust", Some(2), "incomplete: b"),
        ],
    );

    let stuck = stuck(&stalls, MAX_RESUMES);

    assert_eq!(
        stuck["lang-rust"],
        "stalled: lang-rust failed 3 times the same way; needs a person: silent: a; silent: a; \
         silent: a; incomplete: b; incomplete: b; incomplete: b"
    );
}

#[test]
fn an_agent_not_run_again_keeps_its_stalls() {
    let stalls = runs(3, &[gap("lang-rust", None, "silent: a")]);
    let why = stuck(&stalls, MAX_RESUMES)["lang-rust"].clone();

    let carried = tally(&[gap("lang-rust", None, &why)], &stalls);

    assert_eq!(carried, stalls);
    assert_eq!(tally(&[gap("lang-rust", None, &why)], &[]), []);
}

#[test]
fn a_stuck_agent_is_a_blocking_finding_of_its_own() {
    let mut outcome = ReviewOutcome {
        verdict: "approve".to_owned(),
        summary: String::new(),
        findings: Vec::new(),
    };
    let stalls = runs(3, &[gap("lang-rust", None, "silent: a")]);

    assert_eq!(
        amend(&mut outcome, &runs(2, &stalls_gaps()), MAX_RESUMES),
        0
    );
    assert_eq!(outcome.verdict, "approve");
    assert_eq!(amend(&mut outcome, &stalls, MAX_RESUMES), 1);

    assert_eq!(outcome.verdict, REQUEST_CHANGES_VERDICT);
    assert_eq!(outcome.findings[0].severity, BLOCKING);
    assert_eq!(outcome.findings[0].path, REVIEW_PATH);
    assert!(
        outcome.findings[0]
            .body
            .starts_with("stalled: lang-rust failed 3 times")
    );
}

fn stalls_gaps() -> Vec<Gap> {
    vec![gap("lang-rust", None, "silent: a")]
}

#[test]
fn a_stalls_file_reads_back_and_another_shape_is_refused() {
    let stalls = runs(2, &stalls_gaps());
    let text = serde_json::to_string(&stalls).unwrap();

    assert_eq!(parse(&text).unwrap(), stalls);
    assert!(parse("{").unwrap_err().starts_with("malformed stalls: "));
}
