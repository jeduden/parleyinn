use review_eval::log::RunLog;

use super::*;
use crate::plan::parse_dispatch;

fn plan(dispatch: &str) -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(dispatch).unwrap(),
    }
}

const DISPATCH: &str = r#"{"review-code": ["lang-rust", "test-engineer-unit"]}"#;

fn ran(name: &str, kind: Kind, answer: &str) -> AgentRun {
    let log = RunLog {
        answered: true,
        answer: answer.to_owned(),
        ..RunLog::default()
    };
    AgentRun::of(name.to_owned(), kind, None, log, 1)
}

fn silent(name: &str, kind: Kind, why: &str) -> AgentRun {
    AgentRun {
        failure: Some(why.to_owned()),
        ..ran(name, kind, "")
    }
}

fn outcome_line(name: &str) -> String {
    format!("outcome: findings — {name} saw a finding.")
}

/// The ids the reports may cite: the invariants and two requirements.
fn known() -> BTreeSet<String> {
    let mut ids = review_gate::report::invariants();
    ids.extend(["ENG-02".to_owned(), "ENG-16".to_owned()]);
    ids
}

/// A specialist that answered its outcome line, with the report the run
/// assembled from its log.
fn specialist(name: &str) -> (AgentRun, (String, String)) {
    let line = outcome_line(name);
    (
        ran(name, Kind::Specialist, &line),
        (
            name.to_owned(),
            format!("src/a.rs:3 — if not addressed: x breaks — ENG-02\n\n{line}\n"),
        ),
    )
}

/// Every specialist of [`DISPATCH`] answered and reported, and the
/// generalist answered `answer`.
fn all_answered(answer: &str) -> (Vec<AgentRun>, BTreeMap<String, String>) {
    let (rust, rust_report) = specialist("lang-rust");
    let (unit, unit_report) = specialist("test-engineer-unit");
    (
        vec![rust, unit, ran("review-code", Kind::Generalist, answer)],
        [rust_report, unit_report].into(),
    )
}

#[test]
fn a_review_whose_agents_all_answered_with_only_nits_approves() {
    let (runs, reports) = all_answered(
        "src/a.rs:3 — nit — a long line — lang-rust\n\
         dropped — src/b.rs:4 — blocking — no bug there — test-engineer-unit\n\
         The code is sound.",
    );

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve");
    assert_eq!(
        d.outcome.findings,
        vec![Finding {
            path: "src/a.rs".to_owned(),
            line: 3,
            severity: "nit".to_owned(),
            body: "a long line — lang-rust".to_owned(),
        }]
    );
    assert_eq!(
        d.outcome.summary,
        "3 of 3 dispatched agents ran and their logs were read: 2 of 2 specialists and 1 of 1 \
         generalist. The generalists kept 0 blocking and 1 nit findings. \
         Consulted: lang-rust, review-code, test-engineer-unit. \
         review-code: The code is sound."
    );
    assert_eq!(
        d.consulted,
        ["lang-rust", "review-code", "test-engineer-unit"]
            .map(String::from)
            .into()
    );
    assert_eq!(d.answers.len(), 3);
    assert!(d.answers["lang-rust"].starts_with("outcome: findings"));
}

#[test]
fn a_kept_blocking_finding_requests_changes() {
    let (runs, reports) = all_answered("src/a.rs:3 — blocking — an unwrap — lang-rust\nDone.");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(d.outcome.findings[0].severity, "blocking");
    assert!(d.outcome.summary.contains("kept 1 blocking and 0 nit"));
}

#[test]
fn an_agent_that_did_not_answer_is_counted_named_and_blocks() {
    let (mut runs, reports) = all_answered("Sound.");
    runs[1] = silent(
        "test-engineer-unit",
        Kind::Specialist,
        "its run ended with deadline",
    );
    runs.pop();

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(
        d.outcome.findings,
        vec![
            blocking(
                "review/",
                "review-code did not run (it was not run), so the review is incomplete".to_owned()
            ),
            blocking(
                "review/",
                "test-engineer-unit did not run (its run ended with deadline), so the review \
                 is incomplete"
                    .to_owned()
            ),
        ]
    );
    assert!(
        d.outcome.summary.starts_with(
            "1 of 3 dispatched agents ran and their logs were read: 1 of 2 specialists and 0 of \
             1 generalist."
        ),
        "{}",
        d.outcome.summary
    );
    assert!(
        d.outcome
            .summary
            .contains(" Did not run: review-code, test-engineer-unit."),
        "{}",
        d.outcome.summary
    );
    assert_eq!(d.consulted, ["lang-rust".to_owned()].into());
    assert!(!d.answers.contains_key("test-engineer-unit"));
}

#[test]
fn a_report_the_gate_would_refuse_blocks_and_is_named() {
    for (report, problem) in [
        (None, "has no report"),
        (Some(""), "has an empty report"),
        (
            Some("a.rs:1 — nit — x"),
            "has a report that does not end with its outcome",
        ),
        (
            Some(
                "incomplete: its run ended with error_max_turns\na.rs:1 — if not addressed: x breaks — ENG-02",
            ),
            "has a report that does not end with its outcome",
        ),
        (
            Some(
                "note: withdrew nothing at a.rs:2 — nothing there\n\
                 a.rs:3 — if not addressed: x breaks — ENG-02\noutcome: none — lang-rust saw none.",
            ),
            "has a report whose outcome is none but which holds findings",
        ),
        (
            Some("a.rs:1 — nit — x\noutcome: findings — lang-rust saw a finding."),
            "has a report whose line 1 is neither a linked finding nor a requirement request",
        ),
        (
            Some(
                "I read it.\na.rs:1 — if not addressed: x breaks — ENG-99\n\
                 outcome: findings — lang-rust saw a finding.",
            ),
            "has a report whose line 2 cites ENG-99, which no requirement or invariant has",
        ),
    ] {
        let (runs, mut reports) = all_answered(
            "dropped — src/a.rs:3 — fine — lang-rust\nnone — nothing to add — review-code\nSound.",
        );
        match report {
            None => reports.remove("lang-rust"),
            Some(text) => reports.insert("lang-rust".to_owned(), text.to_owned()),
        };

        // The finding quotes the refused line; the summary names the problem.
        let quoted = match problem {
            p if p.contains("line 1 is") => format!("{p}: `a.rs:1 — nit — x`"),
            p if p.contains("line 2 cites") => {
                format!("{p}: `a.rs:1 — if not addressed: x breaks — ENG-99`")
            }
            p => p.to_owned(),
        };

        let d = decide(&plan(DISPATCH), &runs, &reports, &known());

        assert_eq!(d.outcome.verdict, "request_changes", "{problem}");
        assert_eq!(
            d.outcome.findings,
            vec![blocking(
                "review/reports/lang-rust.md",
                format!("lang-rust {quoted}")
            )]
        );
        assert!(
            d.outcome.summary.contains(&format!(
                " Reports the gate refuses: lang-rust ({problem})."
            )),
            "{}",
            d.outcome.summary
        );
        assert!(d.consulted.contains("lang-rust"));
    }
}

#[test]
fn an_answer_whose_outcome_line_carries_markup_or_a_stray_line_is_accepted() {
    for answer in [
        "outcome: findings - lang-rust saw a finding\nreview/reports/lang-rust.md",
        "**Outcome: findings** — lang-rust  saw a finding.\n\nThat is all.",
        "I am done.\n`outcome: findings – lang-rust saw a finding`",
    ] {
        let (mut runs, reports) = all_answered(
            "dropped — src/a.rs:3 — fine — lang-rust\nnone — nothing to add — review-code\nSound.",
        );
        runs[0] = ran("lang-rust", Kind::Specialist, answer);

        let d = decide(&plan(DISPATCH), &runs, &reports, &known());

        assert_eq!(d.outcome.verdict, "approve", "{answer}");
        assert!(d.outcome.findings.is_empty(), "{answer}");
    }
}

#[test]
fn an_empty_dispatch_is_decided_but_not_approved() {
    let d = decide(&plan("{}"), &[], &BTreeMap::new(), &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(d.outcome.summary, "The review bundle dispatched no agent.");
    assert_eq!(d.outcome.findings.len(), 1);
    assert_eq!(d.outcome.findings[0].path, "review/");
    assert!(d.consulted.is_empty());
}

#[test]
fn generalists_findings_are_kept_once_each_with_every_closing() {
    let p =
        plan(r#"{"review-code": ["lang-rust"], "review-security": ["persona-security-officer"]}"#);
    let (rust, rust_report) = specialist("lang-rust");
    let (officer, officer_report) = specialist("persona-security-officer");
    let same = "src/a.rs:3 — nit — a long line — lang-rust";
    let runs = vec![
        rust,
        officer,
        ran(
            "review-code",
            Kind::Generalist,
            &format!("{same}\nCode holds."),
        ),
        ran(
            "review-security",
            Kind::Generalist,
            &format!("{same}\n{same}"),
        ),
    ];

    let d = decide(&p, &runs, &[rust_report, officer_report].into(), &known());

    assert_eq!(d.outcome.verdict, "approve");
    assert_eq!(d.outcome.findings.len(), 1);
    assert!(
        d.outcome
            .summary
            .contains("2 of 2 specialists and 2 of 2 generalists")
    );
    assert!(d.outcome.summary.ends_with(" review-code: Code holds."));
}

#[test]
fn kept_reads_severities_safely() {
    let found = kept(
        "# Findings\n\
         - `src/a.rs:3` — **blocking** — breaks I2 — lang-rust\n\
         src/b.rs:0 — nit — tidy\n\
         src/c.rs:5 — major — odd — mine\n\
         src/d.rs:6 — dropped — not a bug\n\
         **Dropped:** src/e.rs:7 — blocking — gone\n\
         src/f.rs:8 — nit\n\
         a closing sentence.",
    );

    let read: Vec<(&str, i64, &str, &str)> = found
        .iter()
        .map(|f| {
            (
                f.path.as_str(),
                f.line,
                f.severity.as_str(),
                f.body.as_str(),
            )
        })
        .collect();
    assert_eq!(
        read,
        vec![
            ("src/a.rs", 3, "blocking", "breaks I2 — lang-rust"),
            ("src/b.rs", 0, "nit", "tidy"),
            ("src/c.rs", 5, "blocking", "odd — mine (reported as major)"),
            ("src/f.rs", 8, "nit", "src/f.rs:8 — nit"),
        ]
    );
}

#[test]
fn a_blocking_finding_stays_kept_beside_another_source_s_line_at_its_place() {
    let texts = vec![
        "a.rs:3 — blocking — leaks a pin — lang-rust\n\
         a.rs:3 — nit — long — lang-markdown\n\
         dropped — a.rs:3 — misread — lang-markdown"
            .to_owned(),
    ];

    let found = kept(&report::weighing(&texts, "Scope: all."));

    assert_eq!(
        found
            .iter()
            .map(|f| f.severity.as_str())
            .collect::<Vec<_>>(),
        vec!["blocking"]
    );
}

#[test]
fn a_numbered_dropped_line_is_no_closing_sentence() {
    assert_eq!(
        closing("The scope holds.\n1. dropped — b.rs:2 — y — me\n").as_deref(),
        Some("The scope holds.")
    );
}

#[test]
fn a_finding_on_a_path_that_starts_with_dropped_is_still_kept() {
    for line in [
        "dropped_events.rs:12 — blocking — leaks a pin — I3",
        "- `dropped.rs`:3 — blocking — leaks a pin — I3",
        "Droppedfoo/bar.rs:3 — nit — tidy — lang-rust",
    ] {
        assert!(!report::dropped(line), "{line}");
        assert_eq!(kept(line).len(), 1, "{line}");
    }
}

#[test]
fn the_closing_sentence_is_the_last_line_of_prose() {
    assert_eq!(
        closing("a.rs:1 — nit — x\nThe scope holds.\n\ndropped — b.rs:2 — y\n# End\n").as_deref(),
        Some("The scope holds.")
    );
    assert_eq!(closing("a.rs:1 — nit — x\n"), None);
    assert_eq!(closing(""), None);
    assert_eq!(
        closing("The scope holds.\nnone — nothing to add — review-code\n").as_deref(),
        Some("The scope holds.")
    );
}

#[test]
fn a_review_no_agent_answered_names_none_consulted() {
    let runs = vec![
        silent("lang-rust", Kind::Specialist, "it was killed"),
        silent("test-engineer-unit", Kind::Specialist, "it was killed"),
        silent("review-code", Kind::Generalist, "it was killed"),
    ];

    let d = decide(&plan(DISPATCH), &runs, &BTreeMap::new(), &known());

    assert!(d.consulted.is_empty());
    assert!(
        !d.outcome.summary.contains("Consulted:"),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn plural_names_one_and_many() {
    assert_eq!(plural(Kind::Specialist, 1), "specialist");
    assert_eq!(plural(Kind::Generalist, 0), "generalists");
}

#[test]
fn a_generalist_that_says_neither_a_finding_nor_none_requests_changes() {
    let (runs, reports) = all_answered("The code is sound.");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(
        d.outcome.findings,
        vec![blocking(
            "review/",
            "review-code answered no finding, no dropped one and no `none —` line, so its \
             weighing cannot be read"
                .to_owned()
        )]
    );
    assert!(
        d.outcome
            .summary
            .contains(" Answers the review cannot read: review-code."),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn a_generalist_that_says_none_is_a_clean_pass() {
    let (runs, reports) = all_answered(
        "dropped — src/a.rs:3 — fine — lang-rust\nnone — nothing to add — review-code\nThe code is sound.",
    );

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve");
    assert!(d.outcome.findings.is_empty());
    assert!(
        d.outcome
            .summary
            .ends_with(" review-code: The code is sound.")
    );
}

#[test]
fn a_specialist_s_finding_counts_only_as_its_generalist_weighs_it() {
    let (runs, reports) =
        all_answered("dropped — src/a.rs:3 — no bug there — lang-rust\nThe code is sound.");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve");
    assert!(d.outcome.findings.is_empty());
}

#[test]
fn a_generalist_s_blocking_finding_with_its_place_in_bold_is_kept_and_counted() {
    let (runs, reports) = all_answered("**src/a.rs:3** — blocking — an unwrap — lang-rust\nDone.");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(
        (
            d.outcome.findings[0].path.as_str(),
            d.outcome.findings[0].line
        ),
        ("src/a.rs", 3)
    );
    assert!(d.outcome.summary.contains("kept 1 blocking and 0 nit"));
}

#[test]
fn a_generalist_s_finding_line_the_review_cannot_read_blocks_and_is_named() {
    let (runs, reports) = all_answered(
        "src/b.rs:2 — nit — tidy — lang-rust\n\
         src/a.rs:x — blocking — an unwrap — lang-rust\nDone.",
    );

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert!(
        d.outcome.findings.contains(&blocking(
            "review/",
            "review-code answered a finding line the review cannot read, so its weighing \
             cannot be read: src/a.rs:x — blocking — an unwrap — lang-rust"
                .to_owned()
        )),
        "{:?}",
        d.outcome.findings
    );
    assert!(
        d.outcome
            .summary
            .contains(" Answers the review cannot read: review-code."),
        "{}",
        d.outcome.summary
    );

    let (runs, reports) = all_answered("src/a.rs:x — blocking — an unwrap — lang-rust\nDone.");
    let d = decide(&plan(DISPATCH), &runs, &reports, &known());
    assert_eq!(d.outcome.findings.len(), 2, "{:?}", d.outcome.findings);
    assert!(
        d.outcome
            .summary
            .contains(" Answers the review cannot read: review-code."),
        "{}",
        d.outcome.summary
    );
}

/// [`DISPATCH`]'s run with lang-rust's report asking for a requirement
/// at `src/a.rs:3`, beside `linked` when given, and the generalist
/// answering `answer`.
fn requested(linked: Option<&str>, answer: &str) -> Decision {
    let (runs, mut reports) = all_answered(answer);
    let line = outcome_line("lang-rust");
    let linked = linked.map(|l| format!("{l}\n")).unwrap_or_default();
    reports.insert(
        "lang-rust".to_owned(),
        format!("src/a.rs:3 — request: add a rule on line length — none says so\n{linked}{line}\n"),
    );

    decide(&plan(DISPATCH), &runs, &reports, &known())
}

#[test]
fn a_blocking_finding_kept_from_a_requirement_request_stays_a_nit() {
    let d = requested(
        None,
        "src/a.rs:3 — blocking — no rule on line length — `lang-rust`.\nDone.",
    );

    assert_eq!(d.outcome.verdict, "approve");
    assert_eq!(
        d.outcome.findings,
        vec![Finding {
            path: "src/a.rs".to_owned(),
            line: 3,
            severity: NIT.to_owned(),
            body:
                "no rule on line length — `lang-rust`. (kept from a requirement request, so a nit)"
                    .to_owned(),
        }]
    );
    assert!(d.outcome.summary.contains("kept 0 blocking and 1 nit"));
}

#[test]
fn a_blocking_finding_with_another_ground_stays_blocking() {
    for (linked, answer) in [
        // The generalist's own finding at the requested line.
        (None, "src/a.rs:3 — blocking — an unwrap — review-code"),
        // The specialist links a finding at the same line.
        (
            Some("src/a.rs:3 — if not addressed: a hook panics — ENG-16"),
            "src/a.rs:3 — blocking — a hook panics — lang-rust",
        ),
        // Another line than the request's.
        (None, "src/a.rs:4 — blocking — no rule — lang-rust"),
        // Another file than the request's.
        (None, "src/b.rs:3 — blocking — no rule — lang-rust"),
        // A source that is not this generalist's specialist.
        (None, "src/a.rs:3 — blocking — no rule — lang-shell"),
    ] {
        let d = requested(linked, answer);

        assert_eq!(d.outcome.verdict, "request_changes", "{answer}");
        assert_eq!(d.outcome.findings[0].severity, BLOCKING, "{answer}");
    }
}

#[test]
fn a_source_whose_report_is_missing_reads_as_no_request() {
    let (runs, mut reports) =
        all_answered("src/a.rs:3 — blocking — no rule — test-engineer-unit\nDone.");
    reports.remove("test-engineer-unit");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert!(
        d.outcome
            .findings
            .iter()
            .any(|f| f.path == "src/a.rs" && f.severity == BLOCKING)
    );
}

fn gap(name: &str, kind: &str, part: Option<usize>, reason: &str) -> Gap {
    Gap {
        name: name.to_owned(),
        kind: kind.to_owned(),
        part,
        reason: reason.to_owned(),
    }
}

#[test]
fn a_review_every_agent_reported_to_has_nothing_not_reviewed() {
    let (runs, reports) = all_answered(
        "dropped — src/a.rs:3 — fine — lang-rust\nnone — nothing to add — review-code\nSound.",
    );

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert!(d.not_reviewed.is_empty());
    assert!(
        !d.outcome.summary.contains("Not reviewed"),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn every_agent_without_an_accepted_report_is_not_reviewed_with_why() {
    let (mut runs, reports) = all_answered("Sound.");
    runs[1] = silent(
        "test-engineer-unit",
        Kind::Specialist,
        "its run ended with error_max_turns",
    );
    runs.pop();

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(
        d.not_reviewed,
        [
            gap("review-code", "generalist", None, "silent: it was not run"),
            gap(
                "test-engineer-unit",
                "specialist",
                None,
                "incomplete: its run ended with error_max_turns"
            ),
        ]
    );
    assert!(
        d.outcome
            .summary
            .contains(" Not reviewed: review-code, test-engineer-unit."),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn a_part_that_did_not_answer_is_the_part_not_reviewed() {
    let (mut runs, reports) = all_answered(
        "dropped — src/a.rs:3 — fine — lang-rust\nnone — nothing to add — review-code\nSound.",
    );
    runs[0].failure = Some("part 2 of 3: its run ended with deadline".to_owned());
    runs[0].failed_parts = vec![(2, "its run ended with deadline".to_owned())];
    runs[0].parts = 3;

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(
        d.not_reviewed,
        [gap(
            "lang-rust",
            "specialist",
            Some(2),
            "silent: its run ended with deadline"
        )]
    );
    assert!(
        d.outcome
            .summary
            .contains(" Not reviewed: lang-rust part 2."),
        "{}",
        d.outcome.summary
    );
}

#[test]
fn a_refused_report_and_an_unread_answer_are_not_reviewed() {
    let (runs, mut reports) = all_answered("The code is sound.");
    reports.remove("lang-rust");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(
        d.not_reviewed,
        [
            gap("lang-rust", "specialist", None, "refused: has no report"),
            gap(
                "review-code",
                "generalist",
                None,
                "unread: it answered no finding, no dropped one and no `none —` line"
            ),
        ]
    );
    // An approval can never stand beside an agent not reviewed.
    assert_eq!(d.outcome.verdict, "request_changes");
    assert!(
        d.outcome
            .summary
            .contains(" Not reviewed: lang-rust, review-code."),
        "{}",
        d.outcome.summary
    );

    let (runs, reports) = all_answered(
        "src/b.rs:2 — nit — tidy — lang-rust\n\
         src/a.rs:x — blocking — an unwrap — lang-rust\nDone.",
    );
    let d = decide(&plan(DISPATCH), &runs, &reports, &known());
    assert_eq!(
        d.not_reviewed,
        [gap(
            "review-code",
            "generalist",
            None,
            "unread: it answered a finding line the review cannot read"
        )]
    );
}

#[test]
fn a_generalist_cut_off_keeps_what_it_stated_and_is_named_as_not_answering() {
    let (mut runs, reports) = all_answered("");
    runs[2] = silent(
        "review-code",
        Kind::Generalist,
        "its run ended with error_max_turns",
    );
    runs[2].log.texts = vec![
        "reading".to_owned(),
        "src/a.rs:3 — blocking — an unwrap — lang-rust\nsrc/b.rs:4 — nit — long — lang-rust"
            .to_owned(),
    ];

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    let kept: Vec<_> = d
        .outcome
        .findings
        .iter()
        .map(|f| (f.path.as_str(), f.severity.as_str()))
        .collect();
    assert_eq!(
        kept,
        [
            ("review/", "blocking"),
            ("src/a.rs", "blocking"),
            ("src/b.rs", "nit")
        ]
    );
    assert!(d.outcome.summary.contains("kept 1 blocking and 1 nit"));
    assert!(!d.consulted.contains("review-code"));
    assert_eq!(d.not_reviewed.len(), 1);
}

#[test]
fn a_generalist_that_states_its_weighing_as_it_goes_is_read_whole() {
    let (mut runs, reports) = all_answered("The code is sound.");
    runs[2].log.texts = vec!["src/a.rs:3 — nit — a long line — lang-rust".to_owned()];

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "approve");
    assert_eq!(d.outcome.findings.len(), 1);
    assert!(d.answers["review-code"].contains("The code is sound."));
}

#[test]
fn a_generalist_that_weighs_none_of_a_specialist_s_findings_is_partial_by_the_review_alone() {
    // The specialists reported src/a.rs:3; the generalist weighed another
    // file and says its scope is whole. The review, not the agent, decides.
    let (runs, reports) =
        all_answered("src/z.rs:9 — nit — long — lang-rust\nThe whole scope was weighed.");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert!(d.outcome.findings.iter().any(|f| f.body.contains(
        "review-code weighed no finding at src/a.rs:3, which its specialists reported, so its \
         weighing is partial"
    )));
    assert_eq!(
        d.not_reviewed,
        vec![gap(
            "review-code",
            "generalist",
            None,
            "unread: it weighed no finding at src/a.rs:3"
        )]
    );
}

#[test]
fn a_finding_at_the_place_or_a_line_that_names_it_exactly_weighs_it() {
    for answer in [
        "src/b.rs:1 — nit — also src/a.rs:3 and two more — lang-rust\nDone.",
        "dropped — src/a.rs:3 — fine — lang-rust\nnone — nothing to add — review-code\nDone.",
        "src/a.rs:1-5 — nit — the whole block — lang-rust\nDone.",
        "src/b.rs:1 — nit — as at (src/a.rs:2-4) — lang-rust\nDone.",
    ] {
        let (runs, reports) = all_answered(answer);

        let d = decide(&plan(DISPATCH), &runs, &reports, &known());

        assert_eq!(d.outcome.verdict, "approve", "{answer}");
    }
}

#[test]
fn another_line_of_the_file_or_a_place_that_only_starts_alike_weighs_nothing() {
    for answer in [
        "src/a.rs:40 — nit — same file, another line — lang-rust\nDone.",
        "dropped — src/a.rs:31 — fine — lang-rust\nDone.",
        "dropped — lib/src/a.rs:3 — fine — lang-rust\nDone.",
        "src/b.rs:1 — nit — see src/a.rs:30 — lang-rust\nDone.",
        "src/a.rs:4-9 — nit — the block after it — lang-rust\nDone.",
    ] {
        let (runs, reports) = all_answered(answer);

        let d = decide(&plan(DISPATCH), &runs, &reports, &known());

        assert_eq!(d.outcome.verdict, "request_changes", "{answer}");
        assert!(
            d.outcome
                .findings
                .iter()
                .any(|f| f.body.contains("weighed no finding at src/a.rs:3")),
            "{answer}"
        );
    }
}

#[test]
fn a_line_names_a_place_bounded_by_no_path_or_digit_character() {
    assert!(names("see src/a.rs:3.", "src/a.rs", 3));
    assert!(names("`src/a.rs:3`", "src/a.rs", 3));
    assert!(names("src/a.rs:2–4", "src/a.rs", 3));
    assert!(names("src/a.rs:9 and src/a.rs:3", "src/a.rs", 3));
    assert!(!names("src/a.rs:31", "src/a.rs", 3));
    assert!(!names("lib/src/a.rs:3", "src/a.rs", 3));
    assert!(!names("src/a.rs:4-9", "src/a.rs", 3));
    assert!(!names("src/a.rs:x", "src/a.rs", 3));
    assert!(!names("src/a.rs:3-x", "src/a.rs", 4));
    assert!(!names("nothing here", "src/a.rs", 3));
}

#[test]
fn a_range_wider_than_its_bound_names_its_first_line_alone() {
    let widest = format!("src/a.rs:1-{MAX_RANGE}");
    assert!(
        names(&widest, "src/a.rs", MAX_RANGE),
        "{MAX_RANGE} lines wide"
    );
    let wider = format!("src/a.rs:1-{}", MAX_RANGE + 1);
    assert!(names(&wider, "src/a.rs", 1), "its first line");
    assert!(!names(&wider, "src/a.rs", 2));
    assert!(!names("src/a.rs:1-9999", "src/a.rs", 3));
}

#[test]
fn one_line_of_a_huge_range_weighs_none_of_the_findings_inside_it() {
    let (runs, reports) =
        all_answered("src/b.rs:1 — nit — all of src/a.rs:1-9999 — lang-rust\nDone.");

    let d = decide(&plan(DISPATCH), &runs, &reports, &known());

    assert_eq!(d.outcome.verdict, "request_changes");
    assert!(
        d.outcome
            .findings
            .iter()
            .any(|f| f.body.contains("weighed no finding at src/a.rs:3"))
    );
}

#[test]
fn each_generalist_s_weighing_is_kept_answered_or_cut_off_and_none_without_a_run() {
    let (mut runs, reports) = all_answered("src/a.rs:3 — nit — x — lang-rust\nDone.");
    let answered = decide(&plan(DISPATCH), &runs, &reports, &known());
    assert_eq!(
        answered.weighings["review-code"],
        "src/a.rs:3 — nit — x — lang-rust\nDone."
    );

    runs[2] = silent(
        "review-code",
        Kind::Generalist,
        "its run ended with deadline",
    );
    runs[2].log.texts = vec!["src/a.rs:3 — blocking — y — lang-rust\nreading".to_owned()];
    let cut = decide(&plan(DISPATCH), &runs, &reports, &known());
    assert_eq!(
        cut.weighings["review-code"],
        "src/a.rs:3 — blocking — y — lang-rust"
    );
    assert!(!cut.answers.contains_key("review-code"));

    runs.pop();
    let none = decide(&plan(DISPATCH), &runs, &reports, &known());
    assert!(none.weighings.is_empty());
}

const FOUND: &str = "src/a.rs:3 — if not addressed: a long line hides a bug — I9";
const ASKED: &str = "docs/x.md:7 — request: add a row — a motive";

#[test]
fn the_places_of_reports_are_each_listed_once_sorted_findings_and_requests() {
    let one = format!("{FOUND}\nprose\n{ASKED}\noutcome: findings — two");
    let two = format!("{FOUND}\n");

    assert_eq!(
        places([one.as_str(), two.as_str()]),
        vec!["docs/x.md:7".to_owned(), "src/a.rs:3".to_owned()]
    );
    assert!(places(["outcome: none — nothing"]).is_empty());
}

#[test]
fn a_prompt_names_each_place_to_answer_and_none_without_places() {
    assert_eq!(answerable(&[]), None);
    let note = answerable(&["src/a.rs:3".to_owned(), "docs/x.md:7".to_owned()]).unwrap();

    assert!(note.contains("`src/a.rs:3`, `docs/x.md:7`"), "{note}");
    assert!(note.contains("range"), "{note}");
    assert!(note.contains("at most 20 lines"), "{note}");
}

#[test]
fn the_places_told_are_the_places_a_weighing_is_held_to() {
    let report = format!("{FOUND}\n{ASKED}\n");

    assert_eq!(missed([report.as_str()], ""), places([report.as_str()]));
}

mod paths;
mod read;
