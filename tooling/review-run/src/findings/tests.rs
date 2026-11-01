use super::*;

fn finding(path: &str, severity: &str, body: &str) -> (Finding, String) {
    (
        Finding {
            path: path.to_owned(),
            line: 3,
            severity: severity.to_owned(),
            body: body.to_owned(),
        },
        "review-code".to_owned(),
    )
}

/// Every cited file hashes as `hash`.
fn same(hash: &'static str) -> impl Fn(&str) -> Option<String> {
    move |_| Some(hash.to_owned())
}

#[test]
fn a_first_review_keeps_every_finding_new_once() {
    let found = [
        finding("a.rs", BLOCKING, "x — I2"),
        finding("a.rs", BLOCKING, "X. — I2"),
        finding("b.rs", "nit", "y"),
        (
            finding("review/", BLOCKING, "stalled: lang-rust failed 3 times").0,
            REVIEW_RUN.to_owned(),
        ),
        finding("c.rs", BLOCKING, "stalled: a generalist's own finding"),
    ];

    let t = track(&found, None, "h1", &same("f"));

    assert_eq!(t.kept.len(), 3);
    assert_eq!(t.kept[2].fid.path, "c.rs");
    assert!(!t.previous);
    assert!(t.resolved.is_empty());
    let a = &t.kept[0];
    assert_eq!(
        (a.seen_count, a.blocking_streak, a.first_seen_run.as_str()),
        (1, 1, "h1")
    );
    assert_eq!(a.status(), "new");
    assert_eq!(a.agent, "review-code");
    assert_eq!(a.file.as_deref(), Some("f"));
    assert_eq!(t.kept[1].blocking_streak, 0);
    assert_eq!(summary(&t), "");
    assert!(markdown(&t).contains("No previous review was given"));
}

#[test]
fn a_later_review_marks_each_finding_new_still_open_or_resolved() {
    let before = track(
        &[
            finding("a.rs", BLOCKING, "x — I2"),
            finding("c.rs", "nit", "z — I9"),
        ],
        None,
        "h1",
        &same("f"),
    )
    .kept;

    let t = track(
        &[
            finding("a.rs", BLOCKING, "x — I2"),
            finding("b.rs", "nit", "y"),
        ],
        Some(&before),
        "h2",
        &same("f"),
    );

    assert_eq!(t.kept[0].status(), "still open (seen 2 times)");
    assert_eq!(t.kept[0].first_seen_run, "h1");
    assert_eq!(t.kept[1].status(), "new");
    assert_eq!(t.resolved.len(), 1);
    assert_eq!(
        summary(&t),
        " Since the previous review: 1 new, 1 still open and 1 resolved (c.rs:3 (I9))."
    );
    let notes = markdown(&t);
    for want in [
        "## New\n\n- `b.rs:3` (nit, review-code): y — new\n",
        "## Still open\n\n- `a.rs:3 (I2)` (blocking, review-code): x — I2 — still open (seen 2 times)\n",
        "## Resolved\n\n- `c.rs:3 (I9)` (nit, review-code): z — I9 — resolved\n",
    ] {
        assert!(notes.contains(want), "{want}\n{notes}");
    }
    let none = track(&[], Some(&[]), "h3", &same("f"));
    assert_eq!(
        summary(&none),
        " Since the previous review: 0 new, 0 still open and 0 resolved."
    );
    assert!(markdown(&none).contains("## New\n\nNone.\n"));
}

#[test]
fn a_blocking_finding_kept_three_reviews_in_a_row_on_an_unchanged_file_needs_a_ruling() {
    let found = [finding("a.rs", BLOCKING, "x — I2")];
    let mut previous: Option<Vec<Tracked>> = None;
    let mut streaks = Vec::new();
    for run in ["h1", "h2", "h3"] {
        let t = track(&found, previous.as_deref(), run, &same("f"));
        streaks.push(t.kept[0].blocking_streak);
        assert_eq!(rulings(&t).is_empty(), run != "h3", "{run}");
        previous = Some(t.kept);
    }
    assert_eq!(streaks, [1, 2, RULING_REVIEWS]);

    let t = track(&found, previous.as_deref(), "h4", &same("f"));
    let ruled = rulings(&t);
    assert_eq!(ruled.len(), 1);
    assert_eq!(ruled[0].severity, BLOCKING);
    assert_eq!(ruled[0].path, "a.rs");
    assert!(
        ruled[0].body.starts_with(
            "stalled: needs a stakeholder ruling: review-code kept this finding blocking in 4 \
             reviews in a row with a.rs unchanged"
        ),
        "{}",
        ruled[0].body
    );
    // A ruling is itself never tracked.
    let again = track(
        &[(ruled[0].clone(), "review-run".to_owned())],
        None,
        "h5",
        &same("f"),
    );
    assert!(again.kept.is_empty());
}

#[test]
fn a_changed_file_a_nit_or_an_unreadable_file_starts_the_streak_again() {
    let blocking = [finding("a.rs", BLOCKING, "x — I2")];
    let first = track(&blocking, None, "h1", &same("f")).kept;
    let second = track(&blocking, Some(&first), "h2", &same("f")).kept;
    assert_eq!(second[0].blocking_streak, 2);

    let changed = track(&blocking, Some(&second), "h3", &same("g"));
    assert_eq!(changed.kept[0].blocking_streak, 1);
    let unread = track(&blocking, Some(&second), "h3", &|_| None);
    assert_eq!(unread.kept[0].blocking_streak, 1);
    let nit = track(
        &[finding("a.rs", "nit", "x — I2")],
        Some(&second),
        "h3",
        &same("f"),
    );
    assert_eq!(nit.kept[0].blocking_streak, 0);
    assert!(rulings(&nit).is_empty());
}

#[test]
fn a_findings_file_reads_back_and_another_shape_is_refused() {
    let kept = track(
        &[finding("a.rs", BLOCKING, "x — I2")],
        None,
        "h1",
        &|_| None,
    )
    .kept;
    let text = serde_json::to_string(&kept).unwrap();
    assert!(!text.contains("\"file\""), "{text}");

    assert_eq!(parse(&text).unwrap(), kept);
    let err = parse("{").unwrap_err();
    assert!(err.starts_with("malformed findings: "), "{err}");
}

#[test]
fn a_finding_is_the_generalist_s_that_kept_it_or_the_review_s() {
    let plan = Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: crate::plan::parse_dispatch(
            r#"{"review-code": ["lang-rust"], "review-spec": ["lang-markdown"]}"#,
        )
        .unwrap(),
    };
    let answers: BTreeMap<String, String> = [
        (
            "review-spec".to_owned(),
            "a.md:2 — nit — y — lang-markdown".to_owned(),
        ),
        ("lang-rust".to_owned(), "a.rs:3 — nit — x — I2".to_owned()),
    ]
    .into();
    let found = [
        Finding {
            path: "a.md".to_owned(),
            line: 2,
            severity: "nit".to_owned(),
            body: "y — lang-markdown (kept from a requirement request, so a nit)".to_owned(),
        },
        Finding {
            path: "a.rs".to_owned(),
            line: 3,
            severity: "nit".to_owned(),
            body: "x — I2".to_owned(),
        },
    ];

    let agents: Vec<String> = attributed(&found, &answers, &plan)
        .into_iter()
        .map(|(_, a)| a)
        .collect();

    assert_eq!(agents, ["review-spec", REVIEW_RUN]);
}

#[test]
fn a_cited_file_hashes_inside_the_pull_request_s_tree_alone() {
    let dir = testkit::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("pr-head/src")).unwrap();
    std::fs::write(dir.path().join("pr-head/src/a.rs"), "abc").unwrap();
    std::fs::write(dir.path().join("secret"), "s").unwrap();

    assert_eq!(cited(dir.path(), "src/a.rs"), Some(sha256(b"abc")));
    for path in [
        "src/b.rs",
        "../secret",
        "/etc/passwd",
        "src/../../secret",
        "review/",
    ] {
        assert_eq!(cited(dir.path(), path), None, "{path}");
    }
}
