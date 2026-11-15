use super::*;

/// The comments `scripts/review-collect-comments.sh` collects, with one
/// of the stakeholder's whose body is `body`.
fn comments(dir: &Path, body: &str) -> String {
    let file = dir.join("stakeholder-comments.json");
    let json = serde_json::json!([{"author": "owner", "at": "2026-11-15T10:00:00Z", "body": body}]);
    fs::write(&file, json.to_string()).unwrap();

    text(&file)
}

#[test]
fn plan_sorts_the_changed_cases_by_the_comments_of_consent() {
    let base = checkout();
    let head = checkout();
    let cases = head.path().join(CASES).join("lang-rust");
    for kind in ["defect", "clean"] {
        let file = cases.join(format!("{kind}.toml"));
        let changed = fs::read_to_string(&file).unwrap().replace("the ", "a ");
        fs::write(file, changed).unwrap();
    }
    let host = FakeHost::new(base.path());
    let out_file = head.path().join("plan.json");
    let consent = comments(
        head.path(),
        "I consent to tooling/review-eval/cases/lang-rust/defect.toml",
    );
    let plan = |consent: &str| {
        run_cli(
            &host,
            &[
                "plan",
                "--base",
                &text(base.path()),
                "--head",
                &text(head.path()),
                "--consent",
                consent,
                "--out",
                &text(&out_file),
            ],
        )
    };

    let (code, out, err) = plan(&consent);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(
        out,
        "lang-rust: cases changed\n  \
         the pull request's version judges, a stakeholder comment names it: defect.toml\n  \
         main's version judges, no stakeholder comment names it: clean.toml\n\
         estimate: 3 cases, about 7 runs, about 1 min, about $0.28, 12 at a time\n"
    );
    let planned: Vec<Planned> =
        serde_json::from_str(&fs::read_to_string(&out_file).unwrap()).unwrap();
    assert_eq!(planned[0].consented, ["defect.toml"]);
    assert_eq!(planned[0].unconsented, ["clean.toml"]);

    // Comments that do not read fail the plan, naming the file.
    let (code, _, err) = plan(&text(&head.path().join("none.json")));
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-eval: read "), "{err}");
    assert!(err.contains("none.json"), "{err}");
    fs::write(head.path().join("stakeholder-comments.json"), "{}").unwrap();
    let (code, _, err) = plan(&consent);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("not an array of comments"), "{err}");
}

#[test]
fn run_judges_a_consented_case_as_the_pull_request_has_it() {
    let dir = checkout();
    sound_cases(dir.path());
    // Main's three cases of lang-rust, under other names than the pull
    // request's: the pull request removed each and added its own.
    let base = TempDir::new().unwrap();
    for kind in check::REQUIRED_KINDS {
        let from = dir
            .path()
            .join(CASES)
            .join(format!("lang-rust/{kind}.toml"));
        let to = base.path().join(format!("lang-rust/main-{kind}.toml"));
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(from, to).unwrap();
    }
    let mut host = FakeHost::new(dir.path());
    host.claude = obedient_claude();
    let out = TempDir::new().unwrap();
    let out_dir = out.path().join("out");
    // The stakeholder consented to the removal of main's defect case
    // and to the one the pull request adds in its place, and to no more.
    let plan = out.path().join("plan.json");
    fs::write(
        &plan,
        "[{\"component\": \"lang-rust\", \"reason\": \"cases changed\", \
          \"consented\": [\"defect.toml\", \"main-defect.toml\"], \
          \"unconsented\": [\"clean.toml\"]}]",
    )
    .unwrap();

    let (code, out, err) = run_cli(
        &host,
        &[
            "run",
            "--checkout",
            &text(dir.path()),
            "--plan",
            &text(&plan),
            "--base-cases",
            &text(base.path()),
            "--out",
            &text(&out_dir),
            "--repetitions",
            "1",
            "--passes",
            "1",
        ],
    );

    assert_eq!(code, EXIT_OK, "{out}\n{err}");
    let scorecard: Scorecard =
        serde_json::from_str(&fs::read_to_string(out_dir.join(SCORECARD_JSON)).unwrap()).unwrap();
    let names: Vec<&str> = scorecard.suites[0]
        .cases
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["defect", "main-clean", "main-injection"]);
    // The run says which cases the consent moved (I6).
    assert!(
        err.contains(
            "by the stakeholder's consent the pull request's version judges 2 cases: \
             lang-rust/defect, lang-rust/main-defect\n"
        ),
        "{err}"
    );
}

#[test]
fn a_consented_case_file_is_known_by_its_suite_and_name() {
    assert_eq!(
        case_of("review-code-deep", "keeps.toml"),
        ("review-code".to_owned(), "keeps".to_owned())
    );
    assert_eq!(
        case_of("lang-rust", "notes"),
        ("lang-rust".to_owned(), "notes".to_owned())
    );
}
