use super::*;

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|r| r.iter().map(|c| (*c).to_owned()).collect())
        .collect()
}

const HEADER: &[&str] = &["verdict", "finding", "head", "CI", "other check", "review"];

#[test]
fn gate_decides_compares_every_row() {
    let right = table_rows(&rows(&[
        HEADER,
        &["approve", "none", "current", "passed", "passed", "APPROVE"],
    ]))
    .unwrap();
    assert!(gate_decides(&right).is_empty());

    let wrong = table_rows(&rows(&[
        HEADER,
        &[
            "approve",
            "none",
            "current",
            "passed",
            "passed",
            "REQUEST_CHANGES",
        ],
    ]))
    .unwrap();
    let problems = gate_decides(&wrong);
    assert!(problems[0].starts_with("row 1 "), "{problems:?}");
}

#[test]
fn table_rows_keys_cells_by_header() {
    let got = table_rows(&rows(&[&["a", "b"], &["1", "2"]])).unwrap();

    assert_eq!(
        got,
        [BTreeMap::from([
            ("a".to_owned(), "1".to_owned()),
            ("b".to_owned(), "2".to_owned())
        ])]
    );
    assert_eq!(
        table_rows(&rows(&[HEADER])).unwrap_err(),
        "the table has no rows under its header"
    );
    assert_eq!(
        table_rows(&[]).unwrap_err(),
        "the table has no rows under its header"
    );
}

#[test]
fn gate_outcome_names_what_the_gate_does() {
    let row = |verdict: &str, finding: &str, head: &str, ci: &str, other: &str| {
        [
            ("verdict", verdict),
            ("finding", finding),
            ("head", head),
            ("CI", ci),
            ("other check", other),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(
        gate_outcome(&row("approve", "none", "current", "passed", "passed")),
        "APPROVE"
    );
    assert_eq!(
        gate_outcome(&row("approve", "nit", "current", "passed", "passed")),
        "APPROVE"
    );
    assert_eq!(
        gate_outcome(&row("approve", "blocking", "current", "passed", "passed")),
        "REQUEST_CHANGES"
    );
    assert_eq!(
        gate_outcome(&row("approve", "none", "current", "passed", "failed")),
        "REQUEST_CHANGES"
    );
    assert_eq!(
        gate_outcome(&row("approve", "none", "moved", "passed", "passed")),
        "nothing"
    );
    assert_eq!(
        gate_outcome(&row("approve", "none", "current", "failed", "passed")),
        "an error"
    );
    assert_eq!(
        gate_outcome(&row("malformed", "none", "current", "passed", "passed")),
        "an error"
    );
    assert_eq!(
        gate_outcome(&row("missing", "none", "current", "passed", "passed")),
        "an error"
    );
}

#[test]
fn gate_outcome_hands_the_gate_a_deterministic_finding() {
    let row = |check: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("check finding", check),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(gate_outcome(&row("none")), "APPROVE");
    assert_eq!(gate_outcome(&row("nit")), "APPROVE");
    assert_eq!(gate_outcome(&row("missing")), "an error");
    assert_eq!(gate_outcome(&row("blocking")), "REQUEST_CHANGES");
    assert_eq!(gate_outcome(&row("malformed")), "an error");
}

#[test]
fn gate_outcome_hands_the_gate_the_agents_consulted() {
    let row = |consulted: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("consulted", consulted),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(gate_outcome(&row("all")), "APPROVE");
    assert_eq!(gate_outcome(&row("not all")), "an error");
    assert_eq!(gate_outcome(&row("malformed")), "an error");
    assert_eq!(gate_outcome(&row("al")), UNKNOWN_CELL);
}

#[test]
fn gate_outcome_hands_the_gate_the_dispatch() {
    let row = |dispatch: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("dispatch", dispatch),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    assert_eq!(gate_outcome(&row("given")), "APPROVE");
    assert_eq!(gate_outcome(&row("missing")), "an error");
    assert_eq!(gate_outcome(&row("malformed")), "an error");
    assert_eq!(gate_outcome(&row("givne")), UNKNOWN_CELL);
}

#[test]
fn gate_outcome_hands_the_gate_the_specialist_s_report() {
    let row = |cell: &str, value: &str| {
        [
            ("verdict", "approve"),
            ("finding", "none"),
            ("head", "current"),
            ("CI", "passed"),
            ("other check", "passed"),
            (cell, value),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<BTreeMap<_, _>>()
    };

    for (value, want) in [
        ("reported", "APPROVE"),
        ("requested", "APPROVE"),
        ("no report", "an error"),
        ("empty report", "an error"),
        ("incomplete", "an error"),
        ("no outcome answered", "an error"),
        ("other outcome answered", "an error"),
        ("malformed finding", "an error"),
        ("revised nothing", "APPROVE"),
        ("withdrew nothing", "APPROVE"),
        ("none beside a finding", "an error"),
        ("findings with none", "an error"),
        ("unknown id", "APPROVE"),
        ("reportd", UNKNOWN_CELL),
    ] {
        assert_eq!(gate_outcome(&row("specialist", value)), want, "{value}");
    }
    for (value, want) in [
        ("none", "APPROVE"),
        ("failed", "APPROVE"),
        ("faild", UNKNOWN_CELL),
    ] {
        assert_eq!(gate_outcome(&row("own check", value)), want, "{value}");
    }
}

#[test]
fn a_stray_revision_is_the_only_finding_of_its_report() {
    let row = [("specialist".to_owned(), "revised nothing".to_owned())].into();
    let (report, _) = specialist(&row).unwrap();
    let report = report.unwrap();

    // Dropped, the revision would leave an outcome of findings beside
    // none, which the gate refuses: only standing does it approve.
    assert!(report.contains("b.rs:2 — request: add y — why"), "{report}");
    assert!(!report.contains("a.rs:1"), "{report}");
}

#[test]
fn own_check_is_a_failed_review_gate_run_from_the_reviewer_app() {
    assert_eq!(own_check("none"), Some(String::new()));
    assert_eq!(own_check("x"), None);
    let line = own_check("failed").unwrap();
    let run: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(run["name"], review_gate::OWN_CHECK);
    assert_eq!(run["conclusion"], "failure");
    assert_eq!(run["app"], review_gate::REVIEWER_APP_ID);
}
