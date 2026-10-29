use super::*;

fn row(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn graded(
    outcome: &str,
    finding: &str,
    forbidden: &str,
    answer: &str,
    report: &str,
    answered: &str,
    run: &str,
) -> BTreeMap<String, String> {
    row(&[
        ("outcome", outcome),
        ("finding", finding),
        ("forbidden", forbidden),
        ("answer", answer),
        ("report", report),
        ("answered", answered),
        ("run", run),
    ])
}

#[test]
fn the_table_s_component_is_a_strict_lined_specialist_and_its_case_a_defect() {
    let component = table_component();
    assert_eq!(component.name, COMPONENT);
    assert_eq!(component.kind, Kind::Specialist);
    assert!(component.strict_lines);
    let case = table_case();
    assert_eq!(case.kind, "defect");
    assert_eq!(case.expect.finding[0].path, WANTED);
    assert_eq!(case.expect.none[0].path, FORBIDDEN);
    assert_eq!((case.repetitions, case.passes), (3, 2));
}

#[test]
fn the_grading_table_of_the_scenario_holds() {
    let rows = vec![
        graded(
            "as expected",
            "found",
            "none",
            "last line and path",
            "written",
            "yes",
            "passes",
        ),
        graded(
            "other",
            "found",
            "none",
            "last line and path",
            "written",
            "yes",
            "fails on outcome",
        ),
        graded(
            "as expected",
            "missing",
            "none",
            "last line and path",
            "written",
            "yes",
            "fails on finding",
        ),
        graded(
            "as expected",
            "found",
            "found",
            "last line and path",
            "written",
            "yes",
            "fails on forbidden",
        ),
        graded(
            "as expected",
            "found",
            "none",
            "more",
            "written",
            "yes",
            "passes, slips on answer",
        ),
        graded(
            "as expected",
            "found",
            "none",
            "last line and path",
            "missing",
            "yes",
            "fails on report",
        ),
        graded(
            "as expected",
            "found",
            "none",
            "last line and path",
            "written",
            "no",
            "fails on no answer",
        ),
    ];

    assert_eq!(run_graded(&rows), Vec::<String>::new());
}

#[test]
fn a_wrong_grading_row_is_named() {
    let rows = vec![
        graded(
            "other",
            "found",
            "none",
            "last line and path",
            "written",
            "yes",
            "passes",
        ),
        graded(
            "as expected",
            "found",
            "none",
            "more",
            "written",
            "yes",
            "fails on answer",
        ),
    ];

    let got = run_graded(&rows);

    assert_eq!(got.len(), 2);
    assert!(got[0].starts_with("row 1 "), "{}", got[0]);
    assert!(
        got[0].ends_with(": the grader says fails on outcome"),
        "{}",
        got[0]
    );
    assert!(
        got[1].ends_with(": the grader says passes, slips on answer"),
        "{}",
        got[1]
    );
}

#[test]
fn every_failure_is_named_by_its_first_words() {
    assert_eq!(failed_on("did not answer: deadline"), "no answer");
    assert_eq!(failed_on("wrote no report"), "report");
    assert_eq!(failed_on("wrote an empty report"), "report");
    assert_eq!(failed_on("report does not end with its outcome"), "report");
    assert_eq!(failed_on("outcome sound, want blocking"), "outcome");
    assert_eq!(failed_on("no finding at a.rs"), "finding");
    assert_eq!(failed_on("a finding at b.rs: x"), "forbidden");
    assert_eq!(
        failed_on("answer holds more than the last line and the path: x"),
        "answer"
    );
    assert_eq!(failed_on("wrote review/reports/other.md"), "something else");
}

#[test]
fn the_decision_table_of_the_scenario_holds() {
    let rows = vec![
        row(&[
            ("runs", "pass, pass"),
            ("case", "passes"),
            ("runs made", "2"),
        ]),
        row(&[
            ("runs", "pass, fail, pass"),
            ("case", "passes"),
            ("runs made", "3"),
        ]),
        row(&[
            ("runs", "fail, fail"),
            ("case", "fails"),
            ("runs made", "2"),
        ]),
        row(&[
            ("runs", "fail, pass, fail"),
            ("case", "fails"),
            ("runs made", "3"),
        ]),
        row(&[
            ("runs", "none, pass, pass"),
            ("case", "passes"),
            ("runs made", "3"),
        ]),
    ];

    assert_eq!(case_decided(&rows), Vec::<String>::new());
}

#[test]
fn a_wrong_decision_row_is_named_and_an_undecided_list_fails() {
    let rows = vec![row(&[
        ("runs", "pass, fail"),
        ("case", "passes"),
        ("runs made", "2"),
    ])];

    let got = case_decided(&rows);

    assert_eq!(got.len(), 1);
    assert!(
        got[0].ends_with(": the case fails after 2 runs"),
        "{}",
        got[0]
    );
    assert_eq!(decide("pass"), ("fails", 1));
    assert_eq!(decide("pass, pass, pass"), ("passes", 2));
}

#[test]
fn the_checkout_s_cases_are_checked_through_the_crate() {
    let checkout = Checkout::new(&testkit::repo_root());

    assert_eq!(components_have_their_cases(&checkout), Vec::<String>::new());
    assert_eq!(cases_apply_and_dispatch(&checkout), Vec::<String>::new());
}
