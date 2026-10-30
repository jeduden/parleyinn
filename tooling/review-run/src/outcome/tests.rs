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
    AgentRun {
        name: name.to_owned(),
        kind,
        failure: None,
        log: RunLog {
            answered: true,
            answer: answer.to_owned(),
            ..RunLog::default()
        },
    }
}

fn silent(name: &str, kind: Kind, why: &str) -> AgentRun {
    AgentRun {
        failure: Some(why.to_owned()),
        ..ran(name, kind, "")
    }
}

fn outcome_line(name: &str) -> String {
    format!("outcome: nits — {name} saw a nit.")
}

/// A specialist that answered with its report's outcome line.
fn specialist(name: &str) -> (AgentRun, (String, String)) {
    let line = outcome_line(name);
    (
        ran(
            name,
            Kind::Specialist,
            &format!("{line}\nreview/reports/{name}.md"),
        ),
        (
            name.to_owned(),
            format!("a.rs:1 — nit — x — ENG-02\n\n{line}\n"),
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

    let d = decide(&plan(DISPATCH), &runs, &reports);

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
        "3 of 3 dispatched agents answered: 2 of 2 specialists and 1 of 1 generalist. \
         The generalists kept 0 blocking and 1 nit findings. \
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
    assert!(d.answers["lang-rust"].starts_with("outcome: nits"));
}

#[test]
fn a_kept_blocking_finding_requests_changes() {
    let (runs, reports) = all_answered("src/a.rs:3 — blocking — an unwrap — lang-rust\nDone.");

    let d = decide(&plan(DISPATCH), &runs, &reports);

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

    let d = decide(&plan(DISPATCH), &runs, &reports);

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(
        d.outcome.findings,
        vec![
            blocking(
                "review/",
                "review-code did not answer (it was not run), so the review is incomplete"
                    .to_owned()
            ),
            blocking(
                "review/",
                "test-engineer-unit did not answer (its run ended with deadline), so the review \
                 is incomplete"
                    .to_owned()
            ),
        ]
    );
    assert!(
        d.outcome.summary.starts_with(
            "1 of 3 dispatched agents answered: 1 of 2 specialists and 0 of 1 generalist."
        ),
        "{}",
        d.outcome.summary
    );
    assert!(
        d.outcome
            .summary
            .contains(" Did not answer: review-code, test-engineer-unit."),
        "{}",
        d.outcome.summary
    );
    assert_eq!(d.consulted, ["lang-rust".to_owned()].into());
    assert!(!d.answers.contains_key("test-engineer-unit"));
}

#[test]
fn a_report_the_gate_would_refuse_blocks_and_is_named() {
    for (report, problem) in [
        (None, "wrote no report"),
        (Some(""), "wrote an empty report"),
        (
            Some("a.rs:1 — nit — x"),
            "has a report that does not end with its outcome",
        ),
        (
            Some("outcome: sound — another."),
            "has a report whose outcome differs from its answer in the run log",
        ),
    ] {
        let (runs, mut reports) = all_answered("none — nothing to add — review-code\nSound.");
        match report {
            None => reports.remove("lang-rust"),
            Some(text) => reports.insert("lang-rust".to_owned(), text.to_owned()),
        };

        let d = decide(&plan(DISPATCH), &runs, &reports);

        assert_eq!(d.outcome.verdict, "request_changes", "{problem}");
        assert_eq!(
            d.outcome.findings,
            vec![blocking(
                "review/reports/lang-rust.md",
                format!("lang-rust {problem}")
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
fn an_empty_dispatch_is_decided_but_not_approved() {
    let d = decide(&plan("{}"), &[], &BTreeMap::new());

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

    let d = decide(&p, &runs, &[rust_report, officer_report].into());

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
}

#[test]
fn plural_names_one_and_many() {
    assert_eq!(plural(Kind::Specialist, 1), "specialist");
    assert_eq!(plural(Kind::Generalist, 0), "generalists");
}

#[test]
fn a_generalist_that_says_neither_a_finding_nor_none_requests_changes() {
    let (runs, reports) = all_answered("The code is sound.");

    let d = decide(&plan(DISPATCH), &runs, &reports);

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
    let (runs, reports) = all_answered("none — nothing to add — review-code\nThe code is sound.");

    let d = decide(&plan(DISPATCH), &runs, &reports);

    assert_eq!(d.outcome.verdict, "approve");
    assert!(d.outcome.findings.is_empty());
    assert!(
        d.outcome
            .summary
            .ends_with(" review-code: The code is sound.")
    );
}

#[test]
fn a_specialist_s_blocking_report_counts_whatever_its_generalist_answers() {
    let (mut runs, mut reports) =
        all_answered("dropped — a.rs:1 — no bug there — lang-rust\nThe code is sound.");
    let line = "outcome: blocking — lang-rust saw an unwrap.";
    runs[0] = ran(
        "lang-rust",
        Kind::Specialist,
        &format!("{line}\nreview/reports/lang-rust.md"),
    );
    reports.insert(
        "lang-rust".to_owned(),
        format!("a.rs:1 — blocking — an unwrap — ENG-16\n\n{line}\n"),
    );

    let d = decide(&plan(DISPATCH), &runs, &reports);

    assert_eq!(d.outcome.verdict, "request_changes");
    assert_eq!(
        d.outcome.findings,
        vec![blocking(
            "review/reports/lang-rust.md",
            "lang-rust's report review/reports/lang-rust.md ends with outcome: blocking".to_owned()
        )]
    );
}
