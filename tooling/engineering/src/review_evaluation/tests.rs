use std::fs;

use testkit::TempDir;

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
fn a_failed_case_is_one_blocking_finding_and_the_scorecard_line_reads_back() {
    assert_eq!(failed_case_is_a_blocking_finding(), Vec::<String>::new());
    assert_eq!(scorecard_line_records_the_suite(), Vec::<String>::new());
    let suite = table_suite();
    assert_eq!(suite.cases_passed(), (1, 2));
    assert_eq!(suite.runs_passed(), (2, 4));
    assert_eq!(suite.slipped_runs(), 1);
}

#[test]
fn a_wrong_finding_or_line_is_named() {
    assert_eq!(
        finding_problems(&[]),
        ["0 findings for one failed case and one passed, want one"]
    );
    let wrong = GateFinding {
        path: "docs/x.md".to_owned(),
        line: 0,
        severity: "nit".to_owned(),
        body: "something else".to_owned(),
    };
    let problems = finding_problems(&[wrong]);
    assert_eq!(problems.len(), 5, "{problems:?}");
    assert_eq!(
        problems[0],
        "the finding is at docs/x.md, want the component's file"
    );
    assert_eq!(problems[1], "the finding is nit, want blocking");
    assert!(problems[2].starts_with("the finding does not name \"lang-rust/unwrap\""));

    assert_eq!(
        line_problems(Err("bad".to_owned())),
        ["the scorecard does not read back: bad"]
    );
    assert_eq!(
        line_problems(Ok(Vec::new())),
        ["the scorecard holds no line"]
    );
    let mut line = published::lines(&Scorecard {
        suites: vec![table_suite()],
    })
    .remove(0);
    line.cases = 3;
    line.cost = 9.0;
    let problems = line_problems(Ok(vec![line]));
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].starts_with("the line reads"), "{}", problems[0]);
    assert!(
        problems[1].starts_with("the line's cost, turns and seconds read"),
        "{}",
        problems[1]
    );
}

#[test]
fn the_scheduled_run_s_workflow_is_held_to_its_failure_step() {
    let flow = "      run: |\n          cargo run -p review-eval -- run\n          code=$?\n          cat review-eval/report.md >> \"$GITHUB_STEP_SUMMARY\"\n          echo \"code=$code\" >> \"$GITHUB_OUTPUT\"\n      - name: Fail\n        if: steps.run.outputs.code != '0'\n        run: |\n          echo \"::error::failed\"\n          exit 1\n";

    assert_eq!(
        scheduled_run_fails_on_failed_cases(flow),
        Vec::<String>::new()
    );
    assert_eq!(
        scheduled_run_fails_on_failed_cases(&flow.replace("exit 1", "exit 0")),
        ["no step fails the scheduled run on a failed case"]
    );
    assert_eq!(
        scheduled_run_fails_on_failed_cases("run: cargo run -p review-eval -- run\n"),
        [
            "the scheduled run does not record the run's exit code",
            "the scheduled run's summary does not hold the report naming each failed case",
            "no step fails the scheduled run on a failed case",
        ]
    );
}

#[test]
fn the_checkout_s_cases_are_checked_through_the_crate() {
    let checkout = Checkout::new(&testkit::repo_root());

    assert_eq!(components_have_their_cases(&checkout), Vec::<String>::new());
    assert_eq!(cases_apply_and_dispatch(&checkout), Vec::<String>::new());
}

/// An agent's text with its own line and an include of each section,
/// `body` expanded inside.
fn agent(own: &str, sections: &[&str], body: &str) -> String {
    let mut text = format!("{own}\n");
    for s in sections {
        text.push_str(&format!(
            "<?include\nfile: ../../{s}\n?>\n{body}\n<?/include?>\n"
        ));
    }
    text
}

#[test]
fn edit_section_changes_only_the_named_section_s_body() {
    let text = agent("own", &["docs/agents/s.md", "docs/agents/t.md"], "b");

    let (edited, sections) = edit_section(&text, "docs/agents/t.md").unwrap();

    assert_eq!(
        sections,
        ["../../docs/agents/s.md", "../../docs/agents/t.md"]
    );
    assert_eq!(
        edited,
        text.replace("t.md\n?>\n", &format!("t.md\n?>\n{SHARED_EDIT}\n"))
    );
    assert_eq!(edit_section(&text, "docs/agents/u.md"), None);
    assert_eq!(edit_section("own\n", "docs/agents/s.md"), None);
}

fn planned(component: &str, reason: &str) -> Planned {
    Planned {
        component: component.to_owned(),
        reason: reason.to_owned(),
    }
}

#[test]
fn plan_problems_wants_one_component_per_set_covering_the_rest() {
    let sets = BTreeMap::from([
        (vec!["s".to_owned()], vec!["a".to_owned(), "b".to_owned()]),
        (vec!["s".to_owned(), "t".to_owned()], vec!["c".to_owned()]),
    ]);
    let good = [
        planned("a", "shared sections changed; also covers b"),
        planned("c", "shared sections changed"),
    ];
    assert_eq!(plan_problems(&sets, &good), Vec::<String>::new());

    let every = [
        planned("a", "shared sections changed"),
        planned("b", "shared sections changed"),
        planned("c", "instructions changed"),
        planned("x", "shared sections changed"),
    ];
    assert_eq!(
        plan_problems(&sets, &every),
        [
            "component c planned for instructions changed, want the shared section's change alone",
            "component x planned, but it includes no changed section",
            "2 components planned for the sections [s], want one",
        ]
    );
    let short = [
        planned("a", "shared sections changed"),
        planned("c", "shared sections changed"),
    ];
    assert_eq!(
        plan_problems(&sets, &short),
        ["component a covers [a], want [a b]"]
    );
    assert_eq!(
        plan_problems(&sets, &[]),
        [
            "0 components planned for the sections [s], want one",
            "0 components planned for the sections [s t], want one",
        ]
    );
}

#[test]
fn a_shared_section_change_to_a_tree_plans_one_component_per_set() {
    let checkout = TempDir::new().unwrap();
    let write = |rel: &str, body: &str| {
        let path = checkout.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    write("docs/srs/06-security.md", "# 6\n");
    write("docs/domain-model/index.md", "# hub\n");
    write("tooling/review-eval/cases/lang-a/x.toml", "case\n");
    // No component: only a .md file under .claude/agents is one.
    write(".claude/agents/notes.txt", "file: ../../docs/agents/s.md\n");
    for (name, sections) in [
        ("lang-a", &["docs/agents/s.md"][..]),
        ("lang-b", &["docs/agents/s.md"][..]),
        ("lang-c", &["docs/agents/s.md", "docs/agents/t.md"][..]),
        ("lang-d", &["docs/agents/t.md"][..]),
    ] {
        write(
            &format!(".claude/agents/{name}.md"),
            &agent(name, sections, "body"),
        );
    }
    let checkout = Checkout::new(checkout.path());

    let head = TempDir::new().unwrap();
    assert_eq!(
        shared_section_plans_one_per_set(&checkout, "docs/agents/s.md", head.path()),
        Ok(Vec::new())
    );
    let head = TempDir::new().unwrap();
    assert_eq!(
        shared_section_plans_one_per_set(&checkout, "docs/agents/u.md", head.path()),
        Err("no reviewer component includes docs/agents/u.md".to_owned())
    );
    let err = shared_section_plans_one_per_set(
        &Checkout::new(&checkout.root.join("none")),
        "docs/agents/s.md",
        head.path(),
    )
    .unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn copy_dir_copies_a_tree_and_fails_on_a_missing_one() {
    let from = TempDir::new().unwrap();
    fs::create_dir_all(from.path().join("a/b")).unwrap();
    fs::write(from.path().join("a/b/c.md"), "c").unwrap();
    let to = TempDir::new().unwrap();

    copy_dir(from.path(), &to.path().join("x")).unwrap();

    assert_eq!(
        fs::read_to_string(to.path().join("x/a/b/c.md")).unwrap(),
        "c"
    );
    let err = copy_dir(&from.path().join("none"), to.path()).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn the_checkout_s_shared_report_section_plans_one_component_per_set() {
    let checkout = Checkout::new(&testkit::repo_root());
    let head = TempDir::new().unwrap();

    assert_eq!(
        shared_section_plans_one_per_set(
            &checkout,
            "docs/agents/specialist-report.md",
            head.path()
        ),
        Ok(Vec::new())
    );
}
