use super::*;

const REVIEWED: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";
const MOVED: &str = "490efa734617788d1dc1f831b26d0e23888e90fa";

fn approve() -> ReviewOutcome {
    ReviewOutcome {
        verdict: APPROVE_VERDICT.into(),
        summary: "Looks right.".into(),
        findings: vec![],
    }
}

fn run(name: &str, status: &str, conclusion: &str) -> CheckRun {
    CheckRun {
        name: name.into(),
        status: status.into(),
        conclusion: conclusion.into(),
        ..CheckRun::default()
    }
}

fn at(r: CheckRun, started_at: &str, id: i64) -> CheckRun {
    CheckRun {
        started_at: started_at.into(),
        id,
        ..r
    }
}

fn green() -> Vec<CheckRun> {
    vec![
        run("CI", "completed", "success"),
        run("test", "completed", "success"),
    ]
}

fn input(outcome: ReviewOutcome, check_runs: Vec<CheckRun>) -> Input {
    Input {
        outcome,
        reviewed: REVIEWED.into(),
        head: REVIEWED.into(),
        check_runs,
        checked: vec![],
        dispatched: BTreeSet::new(),
        consulted: BTreeSet::new(),
        ..Input::default()
    }
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

fn finding(path: &str, line: i64, severity: &str, body: &str) -> Finding {
    Finding {
        path: path.into(),
        line,
        severity: severity.into(),
        body: body.into(),
    }
}

#[test]
fn parse_outcome_reads_the_agents_output() {
    let got = parse_outcome(
            r#"{"verdict":"request_changes","summary":"ENG-28 has no drift case.",
            "findings":[{"path":"tooling/drift/src/cases.rs","line":12,"severity":"blocking","body":"add one"}]}"#,
        )
        .unwrap();

    assert_eq!(
        got,
        ReviewOutcome {
            verdict: REQUEST_CHANGES_VERDICT.into(),
            summary: "ENG-28 has no drift case.".into(),
            findings: vec![finding(
                "tooling/drift/src/cases.rs",
                12,
                BLOCKING,
                "add one"
            )],
        }
    );
    assert!(
        parse_outcome(r#"{"verdict":"approve","summary":"s","findings":null}"#)
            .unwrap()
            .findings
            .is_empty()
    );
}

#[test]
fn parse_outcome_refuses_what_the_schema_does_not() {
    let cases = [
        ("not JSON", "approve", "expected value"),
        (
            "an unknown field",
            r#"{"verdict":"approve","summary":"s","findings":[],"approve":true}"#,
            "unknown field",
        ),
        (
            "an unknown finding field",
            r#"{"verdict":"approve","summary":"s","findings":[{"x":1}]}"#,
            "unknown field",
        ),
        (
            "trailing data",
            r#"{"verdict":"approve","summary":"s","findings":[]} {}"#,
            "trailing characters",
        ),
        (
            "an unknown answer",
            r#"{"verdict":"lgtm","summary":"s","findings":[]}"#,
            r#"verdict "lgtm" is neither"#,
        ),
        (
            "no summary",
            r#"{"verdict":"approve","summary":" ","findings":[]}"#,
            "no summary",
        ),
        (
            "an unknown severity",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":1,"severity":"minor","body":"b"}]}"#,
            r#"finding 0: severity "minor""#,
        ),
        (
            "a finding with no path",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"","line":1,"severity":"nit","body":"b"}]}"#,
            "finding 0 has no path or no body",
        ),
        (
            "a finding with no body",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":1,"severity":"nit","body":" "}]}"#,
            "finding 0 has no path or no body",
        ),
        (
            "a negative line",
            r#"{"verdict":"approve","summary":"s","findings":[{"path":"a","line":-1,"severity":"nit","body":"b"}]}"#,
            "finding 0: line -1",
        ),
    ];
    for (name, body, want) in cases {
        let err = parse_outcome(body).unwrap_err();
        assert!(matches!(err, Error::Outcome(_)), "{name}");
        assert!(
            err.to_string()
                .starts_with("review: malformed review outcome: "),
            "{name}: {err}"
        );
        assert!(err.to_string().contains(want), "{name}: {err}");
        assert!(std::error::Error::source(&err).is_none());
    }
}

#[test]
fn parse_check_runs_reads_one_per_line() {
    let got = parse_check_runs(
        "{\"name\":\"CI\",\"status\":\"completed\",\"conclusion\":\"success\",\"extra\":1}\n\n\
             {\"name\":\"review\",\"status\":\"in_progress\",\"conclusion\":null}\n",
    )
    .unwrap();

    assert_eq!(
        got,
        [
            run("CI", "completed", "success"),
            run("review", "in_progress", "")
        ]
    );
}

#[test]
fn parse_check_runs_refuses_a_line_that_is_no_check_run() {
    let err = parse_check_runs("{\"name\":\"CI\"}\nnot json\n").unwrap_err();

    assert!(
        err.to_string().starts_with("review: check run line 2: "),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn parse_dispatch_lists_every_generalist_and_specialist() {
    let got =
        parse_dispatch(r#"{"review-code":["lang-rust","test-engineer-unit"],"review-spec":[]}"#)
            .unwrap();

    assert_eq!(
        got,
        set(&[
            "lang-rust",
            "review-code",
            "review-spec",
            "test-engineer-unit"
        ])
    );
    for bad in ["[]", r#"{"review-code":"lang-rust"}"#, "nope"] {
        let err = parse_dispatch(bad).unwrap_err();
        assert!(
            err.to_string().starts_with("review: malformed dispatch: "),
            "{err}"
        );
        assert!(std::error::Error::source(&err).is_some(), "{bad}");
    }
}

#[test]
fn parse_consulted_reads_the_agents_the_review_consulted() {
    assert_eq!(
        parse_consulted(r#"["lang-rust","review-code","lang-rust"]"#).unwrap(),
        set(&["lang-rust", "review-code"])
    );
    for (bad, has_cause) in [("{}", true), ("[1]", true), (r#"[""]"#, false)] {
        let err = parse_consulted(bad).unwrap_err();
        assert!(
            err.to_string()
                .starts_with("review: malformed consulted agents: "),
            "{err}"
        );
        assert_eq!(
            std::error::Error::source(&err).is_some(),
            has_cause,
            "{bad}"
        );
    }
}

#[test]
fn decide_refuses_a_review_that_left_a_dispatched_agent_out() {
    let mut i = input(approve(), green());
    i.dispatched = set(&["lang-rust", "review-code", "srs-security"]);
    i.consulted = set(&["review-code", "lang-toml"]);

    let err = decide(&i).unwrap_err();

    assert_eq!(
        err.to_string(),
        "review: dispatched but not consulted: lang-rust, srs-security"
    );
    assert!(std::error::Error::source(&err).is_none());

    i.consulted = set(&["lang-rust", "review-code", "srs-security", "extra"]);
    assert_eq!(decide(&i).unwrap().unwrap().event, APPROVE);
}

#[test]
fn decide_skips_a_head_that_moved_past_the_review() {
    let got = decide(&Input {
        head: MOVED.into(),
        ..input(approve(), green())
    })
    .unwrap();

    assert!(got.is_none());
}

#[test]
fn decide_approves_only_an_approving_outcome_on_a_green_head() {
    let r = decide(&input(approve(), green())).unwrap().unwrap();

    assert_eq!(r.event, APPROVE);
    assert_eq!(r.commit_id, REVIEWED);
    assert!(r.body.contains("Looks right."), "{}", r.body);
    assert!(
        r.body
            .starts_with("**Approved** by the review gate on `206b0a76a81a`."),
        "{}",
        r.body
    );
}

#[test]
fn decide_refuses_a_head_whose_ci_did_not_pass() {
    let cases = [
        ("no CI check", vec![run("lint", "completed", "success")]),
        ("CI failed", vec![run("CI", "completed", "failure")]),
        ("CI running", vec![run("CI", "in_progress", "")]),
    ];
    for (name, runs) in cases {
        let err = decide(&input(approve(), runs)).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("review: CI has not passed on the reviewed head: {REVIEWED}"),
            "{name}"
        );
        assert!(std::error::Error::source(&err).is_none());
    }
}

#[test]
fn decide_requests_changes_otherwise() {
    let mut failed = green();
    failed.push(run("CodeQL", "completed", "failure"));
    let blocking = ReviewOutcome {
        findings: vec![finding("a.rs", 3, BLOCKING, "breaks I4")],
        ..approve()
    };
    let changes = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "No scenario.".into(),
        findings: vec![],
    };
    let cases = [
        (
            "the agent requests changes",
            input(changes, green()),
            "No scenario.",
        ),
        (
            "another check failed",
            input(approve(), failed),
            "- check `CodeQL`: failure",
        ),
        (
            "an approval with a blocking finding",
            input(blocking, green()),
            "`a.rs:3`: breaks I4",
        ),
    ];
    for (name, input, want) in cases {
        let r = decide(&input).unwrap().unwrap();
        assert_eq!(r.event, REQUEST_CHANGES, "{name}");
        assert!(
            r.body.starts_with("**Changes requested**"),
            "{name}: {}",
            r.body
        );
        assert!(r.body.contains(want), "{name}: {}", r.body);
    }
}

#[test]
fn decide_ignores_checks_still_running_and_passing_ones() {
    let mut runs = green();
    runs.extend([
        run("review", "in_progress", ""),
        run("codecov/patch", "completed", "neutral"),
        run("release", "completed", "skipped"),
    ]);

    assert_eq!(
        decide(&input(approve(), runs)).unwrap().unwrap().event,
        APPROVE
    );
}

#[test]
fn decide_judges_each_check_by_its_latest_run() {
    // A second CI run on one commit cancels the first; the first
    // run's cancelled jobs and its red gate job must not count.
    let runs = vec![
        at(run("CI", "completed", "failure"), "2026-09-30T21:38:10Z", 0),
        at(
            run("test", "completed", "cancelled"),
            "2026-09-30T21:38:09Z",
            0,
        ),
        at(run("CI", "completed", "success"), "2026-09-30T21:40:01Z", 0),
        at(
            run("test", "completed", "success"),
            "2026-09-30T21:39:10Z",
            0,
        ),
    ];

    let r = decide(&input(approve(), runs)).unwrap().unwrap();

    assert_eq!(r.event, APPROVE, "{}", r.body);
}

#[test]
fn decide_still_refuses_when_the_latest_ci_run_failed() {
    let runs = vec![
        at(run("CI", "completed", "success"), "2026-09-30T21:38:10Z", 0),
        at(run("CI", "completed", "failure"), "2026-09-30T21:40:01Z", 0),
    ];

    assert!(matches!(decide(&input(approve(), runs)), Err(Error::Ci(_))));
}

#[test]
fn latest_keeps_the_newest_run_of_each_check_in_first_seen_order() {
    let runs = [
        at(run("b", "", "failure"), "2026-09-30T21:00:00Z", 0),
        at(run("a", "", "success"), "2026-09-30T21:00:00Z", 0),
        at(run("b", "", "success"), "2026-09-30T21:05:00Z", 0),
        at(run("b", "", "cancelled"), "2026-09-30T21:01:00Z", 0),
    ];

    assert_eq!(latest(&runs), [&runs[2], &runs[1]]);
}

#[test]
fn latest_breaks_a_same_second_tie_by_the_higher_run_id() {
    // started_at has one-second resolution, so two runs can tie;
    // GitHub numbers check runs in creation order.
    let t = "2026-09-30T21:00:00Z";
    let runs = [
        at(run("CI", "", "success"), t, 7),
        at(run("CI", "", "failure"), t, 9),
        at(run("CI", "", "cancelled"), t, 8),
    ];

    assert_eq!(latest(&runs), [&runs[1]]);
}

#[test]
fn newer_orders_by_start_then_by_id() {
    let early = at(CheckRun::default(), "2026-09-30T21:00:00Z", 9);
    let late = at(CheckRun::default(), "2026-09-30T21:00:01Z", 1);

    assert!(newer(&late, &early), "a later start wins over a higher id");
    assert!(!newer(&early, &late));
    assert!(newer(
        &at(CheckRun::default(), "2026-09-30T21:00:00Z", 10),
        &early
    ));
    assert!(!newer(&early, &early));
}

#[test]
fn render_lists_blocking_findings_before_nits() {
    let outcome = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "Two things.".into(),
        findings: vec![
            finding("b.rs", 0, NIT, "rename"),
            finding("a.rs", 7, BLOCKING, "unwrapped error"),
        ],
    };

    let body = render(&outcome, &[], &[], REQUEST_CHANGES, REVIEWED);

    let (blocking, nit) = (
        body.find("unwrapped error").unwrap(),
        body.find("rename").unwrap(),
    );
    assert!(blocking < nit, "{body}");
    assert!(body.contains("- nit `b.rs`: rename"), "{body}");
    assert!(body.contains(&REVIEWED[..12]), "{body}");
    assert!(render(&outcome, &[], &[], REQUEST_CHANGES, "abc").contains("on `abc`."));
}

#[test]
fn location_names_the_line_only_when_there_is_one() {
    assert_eq!(location(&finding("a.rs", 0, NIT, "b")), "a.rs");
    assert_eq!(location(&finding("a.rs", 4, NIT, "b")), "a.rs:4");
}

#[test]
fn failed_counts_only_completed_failures() {
    assert!(!failed(&run("a", "completed", "success")));
    assert!(failed(&run("b", "completed", "cancelled")));
    assert!(!failed(&run("c", "queued", "")));
}

#[test]
fn passed_ci_wants_the_required_check_completed_green() {
    assert!(passed_ci(&run("CI", "completed", "success")));
    assert!(!passed_ci(&run("ci", "completed", "success")));
}

#[test]
fn is_blocking_is_the_blocking_severity() {
    assert!(is_blocking(&finding("a", 0, BLOCKING, "b")));
    assert!(!is_blocking(&finding("a", 0, NIT, "b")));
}

#[test]
fn defuse_mentions_wraps_bare_at_words_in_code() {
    let cases = [
        (
            "@LANE-16 is retagged from @P2 @I2",
            "`@LANE-16` is retagged from `@P2` `@I2`",
        ),
        ("(I4 and @I10 added)", "(I4 and `@I10` added)"),
        ("tags `@P1 @I2` stay", "tags `@P1 @I2` stay"),
        ("mail a@b.com", "mail a@b.com"),
        ("a lone @ sign", "a lone @ sign"),
        ("ends with @", "ends with @"),
        ("path `x`: @I4.", "path `x`: `@I4`."),
        ("@I4", "`@I4`"),
    ];
    for (input, want) in cases {
        assert_eq!(defuse_mentions(input), want, "{input}");
    }
}

#[test]
fn the_body_mentions_no_one() {
    let outcome = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "@I2 widens.".into(),
        findings: vec![finding(
            "features/lane.feature",
            0,
            BLOCKING,
            "@LANE-16 is retagged @P1 @I4",
        )],
    };

    let body = render(&outcome, &[], &[], REQUEST_CHANGES, REVIEWED);

    assert!(body.contains("`@I2` widens."), "{body}");
    assert!(
        body.contains("`@LANE-16` is retagged `@P1` `@I4`"),
        "{body}"
    );
}

#[test]
fn an_unmatched_backtick_opens_no_code_span() {
    assert_eq!(defuse_mentions("a ` b @user"), "a ` b `@user`");
    assert_eq!(
        defuse_mentions("`x` and ` then @user"),
        "`x` and ` then `@user`"
    );
}

#[test]
fn paths_and_check_names_cannot_break_out_of_their_code_span() {
    let outcome = ReviewOutcome {
        findings: vec![finding("x` @user `y", 0, BLOCKING, "b")],
        ..approve()
    };
    let failed = run("lint` @team `z", "completed", "failure");

    let body = render(&outcome, &[], &[&failed], REQUEST_CHANGES, REVIEWED);

    assert!(body.contains("- blocking `x' @user 'y`: b"), "{body}");
    assert!(body.contains("- check `lint' @team 'z`: failure"), "{body}");
}

#[test]
fn decide_approves_an_outcome_with_only_nits() {
    let nits = ReviewOutcome {
        findings: vec![finding("a.rs", 2, NIT, "rename")],
        ..approve()
    };

    let r = decide(&input(nits, green())).unwrap().unwrap();

    assert_eq!(r.event, APPROVE);
    assert!(r.body.contains("- nit `a.rs:2`: rename"), "{}", r.body);
}

#[test]
fn parse_checked_reads_the_deterministic_findings() {
    let got = parse_checked(
        r#"[{"path":"features/record.feature","line":4,"severity":"blocking","body":"REC-01 went back to @pending"}]"#,
    )
    .unwrap();

    assert_eq!(
        got,
        vec![finding(
            "features/record.feature",
            4,
            BLOCKING,
            "REC-01 went back to @pending"
        )]
    );
    assert!(parse_checked("[]").unwrap().is_empty());
}

#[test]
fn parse_checked_refuses_what_the_schema_does_not() {
    for (name, data, want) in [
        ("not a list", "{}", "malformed deterministic findings"),
        (
            "an unknown severity",
            r#"[{"path":"a","line":0,"severity":"fatal","body":"b"}]"#,
            "severity",
        ),
        (
            "no body",
            r#"[{"path":"a","line":0,"severity":"nit","body":" "}]"#,
            "no path or no body",
        ),
    ] {
        let err = parse_checked(data).unwrap_err().to_string();
        assert!(err.contains(want), "{name}: {err}");
    }
}

#[test]
fn decide_withholds_approval_on_a_blocking_deterministic_finding() {
    let mut i = input(approve(), green());
    i.checked = vec![finding("docs/srs/invariants.md", 0, BLOCKING, "no ADR")];

    let got = decide(&i).unwrap().unwrap();

    assert_eq!(got.event, REQUEST_CHANGES);
    assert!(
        got.body
            .contains("- blocking `docs/srs/invariants.md`: no ADR"),
        "{}",
        got.body
    );
    assert!(
        got.body
            .contains("### Deterministic checks (review-bundle)"),
        "{}",
        got.body
    );
}

#[test]
fn decide_approves_beside_a_deterministic_nit() {
    let mut i = input(approve(), green());
    i.checked = vec![finding("a.rs", 1, NIT, "route deep")];

    let got = decide(&i).unwrap().unwrap();

    assert_eq!(got.event, APPROVE);
    assert!(
        got.body.contains("- nit `a.rs:1`: route deep"),
        "{}",
        got.body
    );
}

#[test]
fn parse_checked_names_the_exact_fault() {
    for (name, data, want) in [
        (
            "an empty path",
            r#"[{"path":"","line":0,"severity":"nit","body":"b"}]"#,
            "review: malformed deterministic findings: finding 0 has no path or no body",
        ),
        (
            "a negative line",
            r#"[{"path":"a","line":-3,"severity":"nit","body":"b"}]"#,
            "review: malformed deterministic findings: finding 0: line -3",
        ),
        (
            "an unknown field",
            r#"[{"path":"a","line":0,"severity":"nit","body":"b","x":1}]"#,
            "review: malformed deterministic findings: unknown field `x`, \
             expected one of `path`, `line`, `severity`, `body` at line 1 column 53",
        ),
    ] {
        let err = parse_checked(data).unwrap_err();
        assert!(matches!(err, Error::Checked(_)), "{name}");
        assert_eq!(err.to_string(), want, "{name}");
        assert!(std::error::Error::source(&err).is_none(), "{name}");
    }
}

/// The body `decide` posts for `outcome` beside `checked` and `runs`.
fn body(outcome: ReviewOutcome, checked: Vec<Finding>, runs: Vec<CheckRun>) -> String {
    let mut i = input(outcome, runs);
    i.checked = checked;
    decide(&i).unwrap().unwrap().body
}

#[test]
fn render_states_the_agents_verdict_and_why_the_gate_differs() {
    let mut failed = green();
    failed.push(run("CodeQL", "completed", "failure"));
    let blocking = || vec![finding("a.rs", 1, BLOCKING, "b")];
    let changes = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        ..approve()
    };
    let cases = [
        (
            "an approval",
            body(approve(), vec![], green()),
            "**Approved** by the review gate on `206b0a76a81a`.\n\n\
             The review agent's verdict: approve.\n\n",
        ),
        (
            "the agent requests changes",
            body(changes.clone(), vec![], failed.clone()),
            "**Changes requested** by the review gate on `206b0a76a81a`.\n\n\
             The review agent's verdict: request changes.\n\n",
        ),
        (
            "a blocking deterministic finding, before a failed check",
            body(
                ReviewOutcome {
                    findings: blocking(),
                    ..approve()
                },
                blocking(),
                failed.clone(),
            ),
            "**Changes requested** by the review gate on `206b0a76a81a`.\n\n\
             The review agent's verdict: approve. The gate requests changes: \
             a deterministic check found a blocking finding.\n\n",
        ),
        (
            "a failed check, before the agent's blocking finding",
            body(
                ReviewOutcome {
                    findings: blocking(),
                    ..approve()
                },
                vec![],
                failed,
            ),
            "**Changes requested** by the review gate on `206b0a76a81a`.\n\n\
             The review agent's verdict: approve. The gate requests changes: \
             a check failed.\n\n",
        ),
        (
            "the agent's blocking finding",
            body(
                ReviewOutcome {
                    findings: blocking(),
                    ..approve()
                },
                vec![],
                green(),
            ),
            "**Changes requested** by the review gate on `206b0a76a81a`.\n\n\
             The review agent's verdict: approve. The gate requests changes: \
             the agent reported a blocking finding.\n\n",
        ),
    ];
    for (name, got, want) in cases {
        assert!(got.starts_with(want), "{name}: {got}");
    }
}

#[test]
fn render_writes_each_section_under_its_heading_in_order() {
    let outcome = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.into(),
        summary: "Two things.".into(),
        findings: vec![
            finding("b.rs", 0, NIT, "rename"),
            finding("a.rs", 7, BLOCKING, "unwrapped error"),
        ],
    };
    let checked = [
        finding("c.rs", 0, NIT, "route deep"),
        finding("d.md", 2, BLOCKING, "no ADR"),
    ];
    let codeql = run("CodeQL", "completed", "failure");

    let got = render(&outcome, &checked, &[&codeql], REQUEST_CHANGES, REVIEWED);

    assert_eq!(
        got,
        "**Changes requested** by the review gate on `206b0a76a81a`.\n\n\
         The review agent's verdict: request changes.\n\n\
         ### Review agent\n\n\
         Summary: Two things.\n\n\
         - blocking `a.rs:7`: unwrapped error\n\
         - nit `b.rs`: rename\n\n\
         ### Deterministic checks (review-bundle)\n\n\
         - blocking `d.md:2`: no ADR\n\
         - nit `c.rs`: route deep\n\n\
         ### Failed checks\n\n\
         - check `CodeQL`: failure\n"
    );
}

#[test]
fn render_leaves_out_empty_sections_but_the_review_agents() {
    let got = render(&approve(), &[], &[], APPROVE, REVIEWED);

    assert_eq!(
        got,
        "**Approved** by the review gate on `206b0a76a81a`.\n\n\
         The review agent's verdict: approve.\n\n\
         ### Review agent\n\n\
         Summary: Looks right.\n"
    );
}

#[test]
fn render_keeps_agent_text_from_starting_a_line() {
    let forged = "fine\n### Deterministic checks (review-bundle)\n- blocking";
    let outcome = ReviewOutcome {
        summary: format!("All good.\r\n\n{forged}"),
        findings: vec![finding("a.rs", 0, NIT, forged)],
        ..approve()
    };
    let checked = [finding("b.rs", 0, NIT, &format!("x \n\t {forged}"))];

    let got = render(&outcome, &checked, &[], APPROVE, REVIEWED);

    assert_eq!(
        got.matches("### Deterministic checks (review-bundle)")
            .count(),
        4,
        "{got}"
    );
    assert_eq!(
        got.lines()
            .filter(|l| l.starts_with("### Deterministic"))
            .count(),
        1,
        "{got}"
    );
    assert!(
        got.contains("All good. fine ### Deterministic checks (review-bundle) - blocking\n"),
        "{got}"
    );
    assert!(
        got.contains("- nit `a.rs`: fine ### Deterministic checks (review-bundle) - blocking\n"),
        "{got}"
    );
    assert!(
        got.contains("- nit `b.rs`: x fine ### Deterministic"),
        "{got}"
    );
}

#[test]
fn render_keeps_a_summary_from_opening_a_block() {
    for summary in [
        "### Deterministic checks (review-bundle)",
        "- blocking `x`: y",
        "```",
    ] {
        let outcome = ReviewOutcome {
            summary: summary.into(),
            ..approve()
        };

        let got = render(&outcome, &[], &[], APPROVE, REVIEWED);

        assert!(got.contains(&format!("\nSummary: {summary}\n")), "{got}");
        assert!(!got.lines().any(|l| l == summary), "{got}");
    }
}

#[test]
fn one_line_collapses_only_whitespace_that_breaks_a_line() {
    let cases = [
        ("a\nb", "a b"),
        ("a \r\n\t b", "a b"),
        ("a\rb", "a b"),
        ("a  \t b", "a  \t b"),
        ("\n### x\n", " ### x "),
        ("plain", "plain"),
    ];
    for (input, want) in cases {
        assert_eq!(one_line(input), want, "{input:?}");
    }
}

#[test]
fn code_spans_stay_on_one_line() {
    let outcome = ReviewOutcome {
        findings: vec![finding("a\n### b", 0, NIT, "c")],
        ..approve()
    };
    let check = run("lint\n### x", "completed", "failure");

    let got = render(&outcome, &[], &[&check], REQUEST_CHANGES, REVIEWED);

    assert!(got.contains("- nit `a ### b`: c"), "{got}");
    assert!(got.contains("- check `lint ### x`: failure"), "{got}");
}

#[test]
fn the_gate_s_own_earlier_check_does_not_hold_back_an_approval() {
    let mut runs = green();
    runs.push(CheckRun {
        app: REVIEWER_APP_ID,
        ..run(OWN_CHECK, "completed", "failure")
    });

    let r = decide(&input(approve(), runs)).unwrap().unwrap();

    assert_eq!(r.event, APPROVE);
}

#[test]
fn a_check_of_the_gate_s_name_from_another_app_still_counts() {
    let mut runs = green();
    runs.push(CheckRun {
        app: 15_368,
        ..run(OWN_CHECK, "completed", "failure")
    });

    let r = decide(&input(approve(), runs)).unwrap().unwrap();

    assert_eq!(r.event, REQUEST_CHANGES);
}

#[test]
fn a_check_run_reads_its_app() {
    let runs = parse_check_runs(
        r#"{"name":"review-gate","status":"completed","conclusion":"failure","app":5140085}
{"name":"CI","status":"completed","conclusion":"success","app":null}"#,
    )
    .unwrap();

    assert_eq!(runs[0].app, REVIEWER_APP_ID);
    assert_eq!(runs[1].app, 0);
}

fn reported(answer: &str, report: Option<&str>) -> Input {
    let mut i = input(approve(), green());
    i.specialists = ["lang-rust".to_owned()].into();
    if let Some(r) = report {
        i.reports = [("lang-rust".to_owned(), r.to_owned())].into();
    }
    i.answers = [("lang-rust".to_owned(), answer.to_owned())].into();
    i
}

#[test]
fn a_specialist_s_report_must_end_with_the_outcome_it_answered() {
    let report = "a.rs:1 — nit — x\n\noutcome: nits — one sentence.";
    for answer in [
        "outcome: nits — one sentence.\nreport: review/reports/lang-rust.md",
        "agentId: z\n  outcome: nits — one sentence.  \n",
    ] {
        let r = decide(&reported(answer, Some(&format!("{report}\n\n"))));
        assert_eq!(r.unwrap().unwrap().event, APPROVE, "{answer}");
    }
    let reworded = "a.rs:1 — nit — x, said otherwise\noutcome: nits — one sentence.";
    let r = decide(&reported("outcome: nits — one sentence.", Some(reworded)));
    assert_eq!(r.unwrap().unwrap().event, APPROVE);

    for (answer, file, want) in [
        (report, None, "review: lang-rust wrote no report"),
        (
            report,
            Some("  \n"),
            "review: lang-rust wrote an empty report",
        ),
        (
            "a.rs:1 — nit — x",
            Some("a.rs:1 — nit — x"),
            "review: lang-rust has a report that does not end with its outcome",
        ),
        (
            "outcome: maybe — x",
            Some("outcome: maybe — x"),
            "review: lang-rust has a report that does not end with its outcome",
        ),
        (
            "outcome: blocking — one sentence.",
            Some(report),
            "review: lang-rust has a report whose outcome differs from its answer in the run log",
        ),
        (
            "the outcome: nits — one sentence. was it",
            Some(report),
            "review: lang-rust has a report whose outcome differs from its answer in the run log",
        ),
    ] {
        let err = decide(&reported(answer, file)).unwrap_err();
        assert_eq!(err.to_string(), want);
        assert!(std::error::Error::source(&err).is_none());
    }
    let mut no_answer = reported(report, Some(report));
    no_answer.answers.clear();
    assert!(matches!(
        decide(&no_answer),
        Err(Error::Report {
            problem: "has a report whose outcome differs from its answer in the run log",
            ..
        })
    ));
}

#[test]
fn an_outcome_line_names_sound_nits_or_blocking_in_any_case_and_markup() {
    for (text, want) in [
        ("x\noutcome: sound — y\n\n", Some("outcome: sound — y")),
        ("Outcome: **blocking**. y", Some("Outcome: **blocking**. y")),
        ("outcome: `nits`", Some("outcome: `nits`")),
        ("outcome: sound — y\nmore", None),
        ("outcome: soundly", None),
        ("outcome:", None),
        ("", None),
    ] {
        assert_eq!(outcome_line(text), want, "{text:?}");
    }
}

#[test]
fn parse_specialists_reads_the_dispatch_s_lists_and_refuses_a_path() {
    assert_eq!(
        parse_specialists(r#"{"review-code":["lang-rust","test-engineer-unit"],"review-spec":[]}"#)
            .unwrap(),
        ["lang-rust".to_owned(), "test-engineer-unit".to_owned()].into()
    );
    for bad in [
        r#"{"review-code":["../x"]}"#,
        r#"{"review-code":[""]}"#,
        r#"{"review-code":["A"]}"#,
    ] {
        let err = parse_specialists(bad).unwrap_err();
        assert!(
            err.to_string()
                .starts_with("review: malformed dispatch: agent name"),
            "{err}"
        );
        assert!(std::error::Error::source(&err).is_none());
    }
    let err = parse_specialists("nope").unwrap_err();
    assert!(err.to_string().starts_with("review: malformed dispatch: "));
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_report_check_reads_a_report_and_an_answer_alone() {
    let report = "a.rs:1 — nit — x\n\noutcome: nits — one sentence.\n";
    let answer = "outcome: nits — one sentence.\nreview/reports/lang-rust.md";

    assert_eq!(report_check(Some(report), Some(answer)), None);
    assert_eq!(report_check(None, Some(answer)), Some("wrote no report"));
    assert_eq!(
        report_check(Some(" \n"), Some(answer)),
        Some("wrote an empty report")
    );
    assert_eq!(
        report_check(Some("a.rs:1 — nit — x"), Some(answer)),
        Some("has a report that does not end with its outcome")
    );
    assert_eq!(
        report_check(Some(report), None),
        Some("has a report whose outcome differs from its answer in the run log")
    );
}

#[test]
fn an_outcome_written_out_reads_back_as_itself() {
    let outcome = ReviewOutcome {
        verdict: REQUEST_CHANGES_VERDICT.to_owned(),
        summary: "s".to_owned(),
        findings: vec![finding("a.rs", 3, BLOCKING, "b")],
    };

    let json = serde_json::to_string(&outcome).unwrap();

    assert_eq!(parse_outcome(&json).unwrap(), outcome);
}

#[test]
fn parse_answers_reads_a_map_of_name_to_text() {
    assert_eq!(
        parse_answers(r#"{"lang-rust":"r"}"#).unwrap(),
        [("lang-rust".to_owned(), "r".to_owned())].into()
    );
    let err = parse_answers("[]").unwrap_err();
    assert!(
        err.to_string()
            .starts_with("review: malformed agents' answers: ")
    );
    assert!(std::error::Error::source(&err).is_some());
}
