//! The body the gate renders: its sections, headings and code spans.

use super::*;

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
