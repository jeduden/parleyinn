use super::*;

fn ids(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

const ANSWER: &str = "outcome: findings — one sentence.";

#[test]
fn a_linked_finding_reads_its_place_consequence_and_ids() {
    let got = line("crates/x/src/a.rs:12 — if not addressed: a hook panics — ENG-04, I9");

    assert_eq!(
        got,
        Line::Linked(Linked {
            path: "crates/x/src/a.rs".to_owned(),
            line: 12,
            consequence: "a hook panics".to_owned(),
            ids: vec!["ENG-04".to_owned(), "I9".to_owned()],
        })
    );
}

#[test]
fn a_linked_finding_tolerates_markup_dashes_and_a_dash_in_its_consequence() {
    let got =
        line("- `a/b.md:3-5` -- If not addressed: the pin is lost -- or summarised -- `PIN-02`.");

    let Line::Linked(l) = got else {
        panic!("not linked: {got:?}");
    };
    assert_eq!((l.path.as_str(), l.line), ("a/b.md", 3));
    assert_eq!(l.consequence, "the pin is lost — or summarised");
    assert_eq!(l.ids, ["PIN-02"]);
}

#[test]
fn a_request_reads_its_change_requirement_and_motivation() {
    let got = line(
        "docs/srs/10-engineering-quality.md:59 — request: add a cap on report length — reports grow — a lot",
    );

    assert_eq!(
        got,
        Line::Request(Request {
            path: "docs/srs/10-engineering-quality.md".to_owned(),
            line: 59,
            change: "add".to_owned(),
            requirement: "a cap on report length".to_owned(),
            motivation: "reports grow — a lot".to_owned(),
        })
    );
    for change in ["Change", "remove"] {
        let Line::Request(r) = line(&format!("a.rs:1 — Request: {change} ENG-28 — why")) else {
            panic!("{change}");
        };
        assert_eq!(r.change, change.to_lowercase());
    }
}

#[test]
fn a_finding_line_in_neither_form_is_malformed() {
    for text in [
        "a.rs:1 — blocking — an unwrap — ENG-16",
        "a.rs:1 — nit — x",
        "a.rs:1 — if not addressed: a hook panics",
        "a.rs:1 — if not addressed: — ENG-04",
        "a.rs:1 — if not addressed: a hook panics — ENG-04 and I9",
        "a.rs:1 — if not addressed: a hook panics — eng-04",
        "a.rs:1 — if not addressed: a hook panics — -04",
        "a.rs:1 — if not addressed: a hook panics — ENG-",
        "a.rs:1 — if not addressed: a hook panics — I",
        "a.rs:1 — if not addressed: a hook panics — ",
        "a.rs:1 — request: add a cap",
        "a.rs:1 — request: add a cap — ",
        "a.rs:1 — request: add — why",
        "a.rs:1 — request: rename ENG-28 — why",
        "a.rs:1 — requested: add ENG-28 — why",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn a_line_with_no_place_is_prose() {
    for text in [
        "",
        "# lang-rust",
        "I read the diff.",
        "outcome: findings — one sentence.",
        "I read line: 3 — and it is fine",
        "I checked a.rs:3 — and it is fine",
        "a.rs:12",
        "incomplete: its run ended with error_max_turns",
    ] {
        assert_eq!(line(text), Line::Prose, "{text:?}");
    }
}

#[test]
fn a_finding_with_its_place_in_markup_reads_as_one() {
    for text in [
        "**a.rs:1** — if not addressed: a hook panics — ENG-04",
        "- **`a.rs:1`** — if not addressed: a hook panics — ENG-04",
        "*a.rs:1* — if not addressed: a hook panics — ENG-04",
        "__a.rs:1__ — if not addressed: a hook panics — ENG-04",
        "**a.rs**:1 — if not addressed: a hook panics — ENG-04",
        "[a.rs:1](a.rs#L1) — if not addressed: a hook panics — ENG-04",
        "**[a.rs:1](a.rs#L1)** — if not addressed: a hook panics — ENG-04",
        "finding: a.rs:1 — if not addressed: a hook panics — ENG-04",
        "Finding, from the row: `a.rs:1` — if not addressed: a hook panics — ENG-04",
    ] {
        let Line::Linked(l) = line(text) else {
            panic!("not linked: {text:?}");
        };
        assert_eq!((l.path.as_str(), l.line), ("a.rs", 1), "{text:?}");
    }
    for text in [
        "**a.rs:2** — request: add a cap — reports grow",
        "In a.rs:2 — request: add a cap — reports grow",
    ] {
        let Line::Request(r) = line(text) else {
            panic!("not a request: {text:?}");
        };
        assert_eq!((r.path.as_str(), r.line), ("a.rs", 2), "{text:?}");
    }
}

#[test]
fn a_finding_shaped_line_with_no_place_it_can_read_is_malformed_not_prose() {
    for text in [
        "a.rs — if not addressed: x — ENG-04",
        "a.rs:x — if not addressed: x — ENG-04",
        "In ci.yml:x — request: change the shards — they repeat the build",
        "see the diff — If not addressed: x — ENG-04",
        "a.rs:1x — request: add a cap — why",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn a_finding_line_missing_the_dash_after_its_place_is_malformed_not_prose() {
    for text in [
        "a.rs:1 if not addressed: x — I2",
        "a.rs:1— if not addressed: x — ENG-04",
        "`a.rs:1` request: add a cap — why",
        "— if not addressed: x — ENG-04",
        "–request: add a cap — why",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

/// Finding lines with no spaced dash after their place, and none or
/// one later: a space or a colon where the dash belongs.
const UNDASHED: [&str; 8] = [
    "a.rs:1 if not addressed: x",
    "a.rs:1 if not addressed: x, I2",
    "a.rs:1: if not addressed: x — I2",
    "a.rs:1: if not addressed: x",
    "`a.rs:1`: if not addressed: x — I2",
    "a.rs:1 request: add a cap",
    "a.rs:1: request: add a cap — why",
    "- **a.rs:1**, request: add a cap",
];

#[test]
fn a_finding_line_with_a_space_or_colon_for_the_dash_after_its_place_is_malformed() {
    for text in UNDASHED {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn a_finding_line_with_no_dash_after_its_place_is_kept_and_refused_naming_the_dash() {
    for (at, text) in UNDASHED.iter().enumerate() {
        let got = assemble(&texts(&[text]), ANSWER, None);

        assert_eq!(got, format!("{}\n{ANSWER}\n", bare(text)), "{text:?}");
        assert_eq!(
            report_check(Some(&got), Some(ANSWER), None),
            Some(Problem::Undashed { line: 1 }),
            "{at}: {text:?}"
        );
    }
}

#[test]
fn a_dash_with_no_place_before_its_form_stays_malformed_not_undashed() {
    let report = format!("— if not addressed: x — ENG-04\n{ANSWER}");

    assert_eq!(
        report_check(Some(&report), Some(ANSWER), None),
        Some(Problem::Malformed { line: 1 })
    );
}

#[test]
fn a_place_followed_by_prose_without_a_dash_stays_prose() {
    for text in [
        "a.rs:1 is fine",
        "a.rs:1— fine",
        "— and so on",
        "a.rs:1: it holds",
        "see a.rs:1: if not addressed: is the form",
    ] {
        assert_eq!(line(text), Line::Prose, "{text:?}");
    }
}

#[test]
fn a_finding_line_ending_with_all_is_well_is_kept_as_a_visible_note_not_refused() {
    let logged = texts(&[
        "a.rs:1 — if not addressed: the range matches — no finding.",
        "- b.rs:2 — the row holds — Fine",
        "c.rs:3 — if not addressed: nothing — **None**",
        "d.rs:4 — sound!",
        "e.rs:5 — if not addressed: x — No findings",
    ]);
    let answer = "outcome: none — all holds.";

    let got = assemble(&logged, answer, None);

    assert_eq!(
        got,
        "note: all is well, not a finding: a.rs:1 — if not addressed: the range matches — no finding.\n\
         note: all is well, not a finding: b.rs:2 — the row holds — Fine\n\
         note: all is well, not a finding: c.rs:3 — if not addressed: nothing — **None**\n\
         note: all is well, not a finding: d.rs:4 — sound!\n\
         note: all is well, not a finding: e.rs:5 — if not addressed: x — No findings\n\
         outcome: none — all holds.\n"
    );
    assert_eq!(report_check(Some(&got), Some(answer), None), None);
}

#[test]
fn a_report_check_passes_a_left_over_all_is_well_line_by() {
    let report = "a.rs:1 — if not addressed: the range matches — no finding.\n\
                  outcome: none — all holds.";

    assert_eq!(
        report_check(Some(report), Some("outcome: none — all holds."), None),
        None
    );
}

#[test]
fn an_all_is_well_word_inside_a_finding_or_after_a_request_is_no_note() {
    for (text, want) in [
        (
            "a.rs:1 — request: add a cap — none",
            Line::Request(Request {
                path: "a.rs".to_owned(),
                line: 1,
                change: "add".to_owned(),
                requirement: "a cap".to_owned(),
                motivation: "none".to_owned(),
            }),
        ),
        ("a.rs:1 — if not addressed: x — not fine", Line::Malformed),
        ("a b.rs:1 — fine", Line::Prose),
    ] {
        let got = assemble(&texts(&[text]), ANSWER, None);

        assert!(!got.starts_with(ALL_WELL), "{text:?}: {got}");
        assert_eq!(line(text), want, "{text:?}");
    }
}

#[test]
fn a_report_check_names_a_finding_line_with_a_remark_after_its_ids() {
    for text in [
        "- `a.rs:1 — if not addressed: x — ENG-11` (provisional id; confirming against the SRS)",
        "a.rs:1 — if not addressed: x — I2 (provisional)",
        "a.rs:1 — if not addressed: x — ENG-04, I9 and more",
    ] {
        let report = format!("{text}\n{ANSWER}");

        assert_eq!(
            report_check(Some(&report), Some(ANSWER), None),
            Some(Problem::AfterIds { line: 1 }),
            "{text:?}"
        );
    }
}

#[test]
fn a_report_check_names_a_line_at_the_template_s_place() {
    let report = "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
                  `path:line` — if not addressed: x — ENG-28\n\
                  outcome: findings — one sentence.";

    assert_eq!(
        report_check(Some(report), Some(ANSWER), None),
        Some(Problem::TemplatePlace { line: 2 })
    );
}

#[test]
fn a_report_check_names_a_line_at_a_place_that_cites_no_id() {
    for text in [
        "a.rs:1 — if not addressed: the range matches, so no reader is misled — as checked.",
        "a.rs:1 — if not addressed: the range matches",
        "a.rs:1 — the range matches the section. No finding there.",
    ] {
        let report = format!("{text}\noutcome: none — one sentence.");

        assert_eq!(
            report_check(Some(&report), Some("outcome: none — one sentence."), None),
            Some(Problem::Unlinked { line: 1 }),
            "{text:?}"
        );
    }
}

#[test]
fn a_line_at_a_place_with_a_request_or_more_dashes_stays_malformed() {
    for text in [
        "a.rs:1 — request: add",
        "a.rs:1 — nit — path:line",
        "revised: a.rs:1 — if not addressed: x",
        "a.rs:x — if not addressed: x",
    ] {
        let report = format!("{text}\noutcome: findings — one sentence.");

        assert_eq!(
            report_check(Some(&report), Some(ANSWER), None),
            Some(Problem::Malformed { line: 1 }),
            "{text:?}"
        );
    }
}

#[test]
fn a_malformed_line_naming_a_real_place_is_no_template() {
    let report = "a.rs:1 — nit — path:line\noutcome: findings — one sentence.";

    assert_eq!(
        report_check(Some(report), Some(ANSWER), None),
        Some(Problem::Malformed { line: 1 })
    );
}

#[test]
fn a_revision_reads_its_place_and_finding() {
    let finding = "a.rs:3 — if not addressed: a pin is lost — I3";

    assert_eq!(
        line(&format!("- **Revised:** {finding}")),
        Line::Revised(Revision {
            path: "a.rs".to_owned(),
            line: 3,
            finding: finding.to_owned(),
        })
    );
    let request = "b.rs:9 — request: add a rule — none says so";
    assert_eq!(
        line(&format!("revised: {request}")),
        Line::Revised(Revision {
            path: "b.rs".to_owned(),
            line: 9,
            finding: request.to_owned(),
        })
    );
}

#[test]
fn a_revision_in_no_form_is_malformed() {
    for text in [
        "revised:",
        "revised: a.rs:3 — nit — x",
        "revised: a.rs:3 — if not addressed: x",
        "revised: I changed my mind",
        "revised: a.rs → a.rs:3 — if not addressed: x — I3",
        "revised: a b.rs:1 → a.rs:3 — if not addressed: x — I3",
        "revised: revised: a.rs:3 — if not addressed: x — I3",
        "revised: withdrawn: a.rs:3 — x",
        // A revision takes one full finding line: a moved finding is
        // withdrawn, then stated, so an arrow form is in no form.
        "revised: `a.rs:2` → a.rs:3 — if not addressed: x — I3",
        "revised: a.rs:2 -> b.rs:9 — request: add a rule — none says so",
        "revised: a.rs:3 — if not addressed: x → y — I3",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn a_withdrawal_reads_its_place_and_reason() {
    assert_eq!(
        line("Withdrawn: `a.rs:3-5` — the line is in a test — I was wrong"),
        Line::Withdrawn(Withdrawal {
            path: "a.rs".to_owned(),
            line: 3,
            reason: "the line is in a test — I was wrong".to_owned(),
        })
    );
    for text in [
        "withdrawn:",
        "withdrawn: a.rs:3",
        "withdrawn: a.rs:3 — ",
        "withdrawn: a.rs — no line",
        "withdrawn: a b.rs:3 — a space",
    ] {
        assert_eq!(line(text), Line::Malformed, "{text:?}");
    }
}

#[test]
fn the_invariants_are_i1_to_i10() {
    let all = invariants();

    assert_eq!(all.len(), 10);
    assert!(all.contains("I1") && all.contains("I10"));
    assert!(!all.contains("I11"));
}

#[test]
fn an_outcome_line_names_findings_or_none_in_any_case_and_markup() {
    for (text, want) in [
        (
            "x\noutcome: findings — y\n\n",
            Some(("outcome: findings — y", FINDINGS)),
        ),
        ("Outcome: **none**. y", Some(("Outcome: **none**. y", NONE))),
        ("outcome: `none`", Some(("outcome: `none`", NONE))),
        ("outcome: none — y\nmore", None),
        ("outcome: nothing", None),
        ("outcome: blocking — y", None),
        ("outcome: sound — y", None),
        ("outcome:", None),
        ("outcomes", None),
        ("", None),
    ] {
        assert_eq!(outcome_line(text), want, "{text:?}");
    }
}

#[test]
fn an_answer_s_outcome_is_its_last_outcome_line() {
    assert_eq!(
        answer_outcome("outcome: findings — a\n\nThanks."),
        Some(("outcome: findings — a", FINDINGS))
    );
    assert_eq!(
        answer_outcome("outcome: findings — a\noutcome: none — b"),
        Some(("outcome: none — b", NONE))
    );
    assert_eq!(answer_outcome("done\nreview/reports/x.md"), None);
    assert_eq!(answer_outcome(""), None);
}

#[test]
fn a_report_check_takes_well_formed_findings_and_requests() {
    let report = "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
                  I read the hook.\n\n\
                  a.rs:2 — request: add a deadline rule — none says so\n\
                  outcome: findings — one sentence.\n";

    assert_eq!(report_check(Some(report), Some(ANSWER), None), None);
    assert_eq!(
        report_check(Some(report), Some(ANSWER), Some(&ids(&["ENG-04"]))),
        None
    );
    let none = "# lang-rust\nI read the diff.\noutcome: none — nothing to say.";
    assert_eq!(
        report_check(
            Some(none),
            Some("outcome: none — nothing to say."),
            Some(&ids(&[]))
        ),
        None
    );
}

#[test]
fn a_report_check_refuses_what_the_gate_refuses() {
    let finding = "a.rs:1 — if not addressed: a hook panics — ENG-04\n";
    let ending = "outcome: findings — one sentence.";
    let report = format!("{finding}{ending}");
    for (report, answer, want) in [
        (None, Some(ANSWER), Problem::Missing),
        (Some(" \n".to_owned()), Some(ANSWER), Problem::Empty),
        (Some(finding.to_owned()), Some(ANSWER), Problem::NoOutcome),
        (
            Some(format!("x\na.rs:1 — nit — x\n{ending}")),
            Some(ANSWER),
            Problem::Malformed { line: 2 },
        ),
        (
            Some(format!("{finding}outcome: none — one sentence.")),
            Some("outcome: none — one sentence."),
            Problem::NoneWithFindings,
        ),
        (
            Some(format!("I read it.\n{ending}")),
            Some(ANSWER),
            Problem::FindingsWithNone,
        ),
        (Some(report.clone()), None, Problem::NoAnswerOutcome),
        (
            Some(report.clone()),
            Some("I am done.\nreview/reports/lang-rust.md"),
            Problem::NoAnswerOutcome,
        ),
        (
            Some(report.clone()),
            Some("outcome: none — one sentence."),
            Problem::AnswerDiffers,
        ),
    ] {
        assert_eq!(
            report_check(report.as_deref(), answer, None),
            Some(want.clone()),
            "{report:?}"
        );
    }
}

#[test]
fn a_report_check_reads_the_answer_s_last_outcome_line_and_tolerates_a_trailing_line() {
    let report =
        "a.rs:1 — if not addressed: a hook panics — ENG-04\noutcome: findings — one sentence.";
    for answer in [
        "outcome: findings — one sentence.",
        "**Outcome: findings** - another sentence;\n",
        "I read the diff.\noutcome: findings — fine\n\nThat is all.",
        "outcome: none — early\noutcome: findings — final",
    ] {
        assert_eq!(
            report_check(Some(report), Some(answer), None),
            None,
            "{answer:?}"
        );
    }
}

#[test]
fn a_report_check_with_the_known_ids_refuses_an_id_no_requirement_has() {
    let report = "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
                  b.rs:9 — if not addressed: a pin is lost — I3, PIN-99\n\
                  outcome: findings — one sentence.";
    let known: BTreeSet<String> = ids(&["ENG-04"]).into_iter().chain(invariants()).collect();

    assert_eq!(
        report_check(Some(report), Some(ANSWER), Some(&known)),
        Some(Problem::UnknownId {
            line: 2,
            id: "PIN-99".to_owned()
        })
    );
    assert_eq!(report_check(Some(report), Some(ANSWER), None), None);
}

#[test]
fn a_findings_check_refuses_a_report_without_an_outcome() {
    assert_eq!(
        findings_check("a.rs:1 — if not addressed: x — I2", None),
        Some(Problem::NoOutcome)
    );
}

#[test]
fn every_problem_reads_as_a_sentence_about_the_report() {
    for (problem, want) in [
        (Problem::Missing, "has no report"),
        (Problem::Empty, "has an empty report"),
        (
            Problem::NoOutcome,
            "has a report that does not end with its outcome",
        ),
        (
            Problem::Malformed { line: 3 },
            "has a report whose line 3 is neither a linked finding nor a requirement request",
        ),
        (
            Problem::Unlinked { line: 2 },
            "has a report whose line 2 cites no requirement or invariant id: a line saying all \
             is well is not a finding, so leave it out",
        ),
        (
            Problem::Undashed { line: 6 },
            "has a report whose line 6 names a place with no spaced dash after it: write \
             `path:line — if not addressed: …` or `path:line — request: …`, the dash between \
             spaces",
        ),
        (
            Problem::AfterIds { line: 7 },
            "has a report whose line 7 writes more after its ids: a finding line ends with its \
             ids, so state a finding only once it stands, and correct it later with revised:",
        ),
        (
            Problem::TemplatePlace { line: 5 },
            "has a report whose line 5 names the place path:line, the form's template, not a file and line",
        ),
        (
            Problem::UnknownId {
                line: 4,
                id: "ENG-99".to_owned(),
            },
            "has a report whose line 4 cites ENG-99, which no requirement or invariant has",
        ),
        (
            Problem::NoneWithFindings,
            "has a report whose outcome is none but which holds findings",
        ),
        (
            Problem::FindingsWithNone,
            "has a report whose outcome is findings but which holds none",
        ),
        (
            Problem::NoAnswerOutcome,
            "answered no outcome line in the run log",
        ),
        (
            Problem::AnswerDiffers,
            "answered another outcome in the run log than its report ends with",
        ),
    ] {
        assert_eq!(problem.to_string(), want);
    }
}

const LOGGED: [&str; 3] = [
    "I read the diff.\na.rs:1 — if not addressed: a hook panics — ENG-04\n",
    "- `b.rs:2` — request: add a cap — reports grow\na.rs:1 — if not addressed: a hook panics — ENG-04",
    "a.rs:7 — if not addressed: a pin is lost — I3\n`a.rs:1` -- If not addressed: a hook panics -- `ENG-04`.\nTime to answer.",
];

fn texts(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|l| (*l).to_owned()).collect()
}

#[test]
fn an_assembled_report_replays_the_logged_findings_once_each_and_ends_with_the_answer_s_outcome() {
    let got = assemble(&texts(&LOGGED), "outcome: findings — three.\n", None);

    assert_eq!(
        got,
        "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
         b.rs:2 — request: add a cap — reports grow\n\
         a.rs:7 — if not addressed: a pin is lost — I3\n\
         outcome: findings — three.\n"
    );
    assert_eq!(
        report_check(Some(&got), Some("outcome: findings — three."), None),
        None
    );
}

#[test]
fn an_assembled_report_keeps_a_finding_whose_place_is_in_bold_and_refuses_one_it_cannot_read() {
    let got = assemble(
        &texts(&[
            "**a.rs:1** — if not addressed: a hook panics — ENG-04",
            "a.rs:x — if not addressed: a pin is lost — I3",
        ]),
        "outcome: findings — two.",
        None,
    );

    assert_eq!(
        got,
        "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
         a.rs:x — if not addressed: a pin is lost — I3\n\
         outcome: findings — two.\n"
    );
    assert_eq!(
        report_check(Some(&got), Some("outcome: findings — two."), None),
        Some(Problem::Malformed { line: 2 })
    );
}

#[test]
fn a_revision_replaces_every_earlier_finding_at_its_place_where_the_first_stood() {
    let mut logged = texts(&LOGGED);
    logged.push("a.rs:1 — request: change ENG-04 — it reads two ways".to_owned());
    logged.push("revised: a.rs:1 — if not addressed: a hook hangs — ENG-03".to_owned());

    let got = assemble(&logged, "outcome: findings — x", None);

    assert_eq!(
        got,
        "a.rs:1 — if not addressed: a hook hangs — ENG-03\n\
         b.rs:2 — request: add a cap — reports grow\n\
         a.rs:7 — if not addressed: a pin is lost — I3\n\
         outcome: findings — x\n"
    );
}

#[test]
fn a_withdrawal_removes_every_earlier_finding_at_its_place() {
    let mut logged = texts(&LOGGED);
    logged.push("withdrawn: a.rs:1 — the hook is a test's".to_owned());

    let got = assemble(&logged, "outcome: findings — x", None);

    assert_eq!(
        got,
        "b.rs:2 — request: add a cap — reports grow\n\
         a.rs:7 — if not addressed: a pin is lost — I3\n\
         outcome: findings — x\n"
    );
}

#[test]
fn a_withdrawal_that_finds_nothing_leaves_a_note_and_a_revision_states_its_finding() {
    let logged = texts(&[
        "a.rs:1 — if not addressed: a hook panics — ENG-04",
        "withdrawn: a.rs:2 — I checked it and it holds",
        "revised: c.rs:1 — if not addressed: x — I2",
        "revised: d.rs:4 — request: add y — why",
    ]);

    let got = assemble(&logged, "outcome: findings — x", None);

    assert_eq!(
        got,
        "a.rs:1 — if not addressed: a hook panics — ENG-04\n\
         note: withdrew nothing at a.rs:2 — I checked it and it holds\n\
         c.rs:1 — if not addressed: x — I2\n\
         d.rs:4 — request: add y — why\n\
         outcome: findings — x\n"
    );
    assert_eq!(
        report_check(Some(&got), Some("outcome: findings — x"), None),
        None
    );
}

#[test]
fn a_note_of_a_withdrawal_reads_as_prose_whatever_its_reason_says() {
    for text in [
        "note: withdrew nothing at a.rs:2 — I checked it",
        "note: withdrew nothing at a.rs:2 — if not addressed: x — I2",
        "note: withdrew nothing at a.rs:2 — request: add x — why",
    ] {
        assert_eq!(line(text), Line::Prose, "{text}");
    }
}

#[test]
fn a_withdrawal_that_finds_nothing_beside_no_finding_keeps_an_outcome_of_none() {
    let logged = texts(&["withdrawn: a.rs:2 — it holds"]);

    let got = assemble(&logged, "outcome: none — fine", None);

    assert_eq!(
        got,
        "note: withdrew nothing at a.rs:2 — it holds\noutcome: none — fine\n"
    );
    assert_eq!(
        report_check(Some(&got), Some("outcome: none — fine"), None),
        None
    );
}

#[test]
fn a_report_check_reads_a_leftover_revision_as_its_finding_and_a_withdrawal_as_nothing() {
    let report = "revised: b.rs:2 — request: add x — why\n\
                  withdrawn: a.rs:3 — it was fine\n\
                  outcome: findings — one sentence.";
    assert_eq!(report_check(Some(report), Some(ANSWER), None), None);
    let unknown =
        "revised: b.rs:2 — if not addressed: x — ENG-99\noutcome: findings — one sentence.";
    assert_eq!(
        report_check(Some(unknown), Some(ANSWER), Some(&ids(&["ENG-04"]))),
        Some(Problem::UnknownId {
            line: 1,
            id: "ENG-99".to_owned()
        })
    );
    let none = "withdrawn: a.rs:3 — it was fine\noutcome: none — one sentence.";
    assert_eq!(
        report_check(Some(none), Some("outcome: none — one sentence."), None),
        None
    );
}

#[test]
fn a_malformed_line_stays_in_the_assembled_report_and_a_withdrawal_can_remove_it() {
    let logged = texts(&["a.rs:1 — nit — x", "a.rs:1 — nit — x"]);
    let got = assemble(&logged, "outcome: findings — x", None);
    assert_eq!(got, "a.rs:1 — nit — x\noutcome: findings — x\n");
    assert_eq!(
        report_check(Some(&got), Some("outcome: findings — x"), None),
        Some(Problem::Malformed { line: 1 })
    );

    let logged = texts(&["a.rs:1 — nit — x", "withdrawn: a.rs:1 — said wrong"]);
    assert_eq!(
        assemble(&logged, "outcome: none — x", None),
        "outcome: none — x\n"
    );
}

#[test]
fn an_incomplete_run_s_report_is_headed_and_ends_with_no_outcome() {
    let got = assemble(
        &texts(&LOGGED[..1]),
        "",
        Some("its run ended with error_max_turns after 20 turns"),
    );

    assert_eq!(
        got,
        "incomplete: its run ended with error_max_turns after 20 turns\n\
         a.rs:1 — if not addressed: a hook panics — ENG-04\n"
    );
    assert_eq!(
        report_check(Some(&got), None, None),
        Some(Problem::NoOutcome)
    );
    assert_eq!(assemble(&[], "", None), "");
    assert_eq!(
        assemble(&[], "", Some("it was not run")),
        "incomplete: it was not run\n"
    );
}

#[test]
fn render_gives_a_finding_its_canonical_shape_and_another_line_nothing() {
    assert_eq!(
        render(&line("- `a.rs:1` -- If not addressed: x -- `I2`, ENG-04.")),
        Some("a.rs:1 — if not addressed: x — I2, ENG-04".to_owned())
    );
    assert_eq!(
        render(&line("a.rs:1 — Request: Add a rule — why — not")),
        Some("a.rs:1 — request: add a rule — why — not".to_owned())
    );
    for text in ["prose", "a.rs:1 — nit — x", "withdrawn: a.rs:1 — x"] {
        assert_eq!(render(&line(text)), None, "{text:?}");
    }
}

#[test]
fn a_place_reads_a_path_and_a_line_or_a_range() {
    assert_eq!(place("`a/b.rs:12`"), Some(("a/b.rs".to_owned(), 12)));
    assert_eq!(place("a/b.rs:3–4"), Some(("a/b.rs".to_owned(), 3)));
    assert_eq!(place("`a/b.rs`:7"), Some(("a/b.rs".to_owned(), 7)));
    assert_eq!(place("a/b.rs:12**"), Some(("a/b.rs".to_owned(), 12)));
    assert_eq!(place("**a/b.rs**:12"), Some(("a/b.rs".to_owned(), 12)));
    assert_eq!(place("a/b.rs:12`**"), Some(("a/b.rs".to_owned(), 12)));
    assert_eq!(
        place("[a/b.rs:12](a/b.rs#L12)"),
        Some(("a/b.rs".to_owned(), 12))
    );
    assert_eq!(place("[a/b.rs:12-14]"), Some(("a/b.rs".to_owned(), 12)));
    // A link that is not the whole place is read as written.
    assert_eq!(
        place("[a/b.rs](x):12"),
        Some(("[a/b.rs](x)".to_owned(), 12))
    );
    assert_eq!(place("a/b.rs:-4"), None);
    assert_eq!(place(":4"), None);
    assert_eq!(place("a/b.rs"), None);
}

#[test]
fn bare_drops_list_markers_numbers_emphasis_and_quotes() {
    assert_eq!(bare("- a"), "a");
    assert_eq!(bare("* a"), "a");
    assert_eq!(bare("12. **a"), "a");
    assert_eq!(bare("> `a"), "a");
    assert_eq!(bare("12a"), "12a");
    assert_eq!(bare(". a"), ". a");
}

#[test]
fn split_dashes_cuts_at_each_spaced_dash() {
    assert_eq!(
        split_dashes("a — b – c -- d - e"),
        ["a", "b", "c", "d", "e"]
    );
    assert_eq!(split_dashes("a-b"), ["a-b"]);
}
