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
        "# Not reviewed\n\nThese dispatched agents have no accepted report or answer, so the \
         review read nothing from them. None of them is clean.\n\n\
         - `lang-rust part 2` (specialist): silent: x\n"
    );
    assert_eq!(
        markdown(&[]),
        "# Not reviewed\n\nEvery dispatched agent has an accepted report or answer.\n"
    );
}
