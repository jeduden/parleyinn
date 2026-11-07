use super::*;

#[test]
fn a_gap_takes_its_kind_from_its_name_and_labels_its_part() {
    let whole = Gap::new("review-code", None, silent("it was not run"));
    let part = Gap::new("lang-rust", Some(2), silent("x"));

    assert_eq!(whole.kind, "generalist");
    assert_eq!(whole.label(), "review-code");
    assert_eq!(part.kind, "specialist");
    assert_eq!(part.label(), "lang-rust part 2");
}

#[test]
fn a_run_cut_off_at_its_turns_is_incomplete_and_any_other_silent() {
    assert_eq!(
        silent("its run ended with error_max_turns"),
        "incomplete: its run ended with error_max_turns"
    );
    assert_eq!(silent("claude: not found"), "silent: claude: not found");
}

#[test]
fn a_run_a_cost_cap_stopped_or_the_breaker_kept_from_starting_says_so() {
    assert_eq!(
        silent("its run ended with error_max_budget_usd"),
        "over budget: its run ended with error_max_budget_usd"
    );
    for why in [
        "over budget: the review's runs cost 40.0000 USD, its cap is 40.00 USD, so it was not started",
        "aborted: 3 of 5 specialists failed (mass failure), so it was not started",
    ] {
        assert_eq!(silent(why), why);
    }
}

#[test]
fn a_gap_list_reads_back_as_written() {
    let gaps = vec![
        Gap::new("lang-rust", Some(2), silent("x")),
        Gap::new("review-code", None, "unread: y".to_owned()),
    ];
    let text = serde_json::to_string(&gaps).unwrap();

    assert_eq!(parse(&text).unwrap(), gaps);
    assert!(!text.contains("\"part\":null"), "{text}");
    assert_eq!(parse("[]").unwrap(), []);
}

#[test]
fn a_gap_list_that_names_no_dispatched_agent_or_part_is_refused() {
    for (text, want) in [
        ("{}", "malformed gaps: "),
        (
            r#"[{"name": "x", "kind": "specialist", "reason": "r", "extra": 1}]"#,
            "malformed gaps: unknown field",
        ),
        (
            r#"[{"name": "../x", "kind": "specialist", "reason": "r"}]"#,
            "is no dispatched agent",
        ),
        (
            r#"[{"name": "persona-agent", "kind": "consultant", "reason": "r"}]"#,
            "is no dispatched agent",
        ),
        (
            r#"[{"name": "lang-rust", "kind": "generalist", "reason": "r"}]"#,
            "lang-rust is a specialist, not a generalist",
        ),
        (
            r#"[{"name": "lang-rust", "kind": "specialist", "part": 0, "reason": "r"}]"#,
            "lang-rust has no part Some(0)",
        ),
        (
            r#"[{"name": "review-code", "kind": "generalist", "part": 1, "reason": "r"}]"#,
            "review-code has no part Some(1)",
        ),
    ] {
        let err = parse(text).unwrap_err();
        assert!(err.contains(want), "{text}: {err}");
    }
}

#[test]
fn the_page_names_each_gap_with_why_and_never_reads_as_clean() {
    let gaps = vec![Gap::new("lang-rust", Some(2), silent("x"))];

    let page = markdown(&gaps);

    assert_eq!(
        page,
        "# Not reviewed\n\nThe review did not review these dispatched agents, or parts, \
         whole: each did not run, read only part of its diff, stated a line that does not \
         read, or gave an answer the review could not take. None of them is clean.\n\n\
         - `lang-rust part 2` (specialist): silent: x\n"
    );
    assert_eq!(
        markdown(&[]),
        "# Not reviewed\n\nEvery dispatched agent has an accepted report or answer.\n"
    );
}

#[test]
fn a_partial_or_unreadable_specialist_says_which_lines() {
    let line = crate::reading::ReportLine {
        agent: "lang-rust".to_owned(),
        part: None,
        line: 2,
        text: "a.rs:1 — x".to_owned(),
        why: "has a report whose line 2 is neither".to_owned(),
    };

    assert_eq!(partial("lines 1-4 unread"), "partial: lines 1-4 unread");
    assert_eq!(
        unreadable(&line),
        "unreadable: has a report whose line 2 is neither: `a.rs:1 — x`"
    );
}

#[test]
fn a_quoted_line_is_whole_up_to_its_cap_and_cut_with_an_ellipsis_past_it() {
    let at_cap = "é".repeat(MOST_QUOTED);
    assert_eq!(quoted(&at_cap), at_cap);
    assert_eq!(quoted(&format!("{at_cap}x")), format!("{at_cap}…"));
    assert_eq!(quoted("a.rs:1 — x"), "a.rs:1 — x");
}
