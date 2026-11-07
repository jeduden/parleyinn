use super::*;
use crate::report::{assemble, report_check};

const FIVE: [&str; 5] = [
    "a.rs:1 — if not addressed: a hook panics — ENG-04",
    "b.rs:2 — if not addressed: a pin is lost — I3",
    "c.rs:3 — request: add a cap — reports grow",
    "d.rs:4 — if not addressed: a socket opens — I4, ENG-16",
    "e.rs:5 — if not addressed: a drop goes uncounted — I6",
];

/// A report of five findings, a line saying all is well, a withdrawal
/// at a placeholder and a finding line in no form.
fn mixed() -> String {
    format!(
        "{}\n{}\nf.rs:6 — if not addressed: the range matches — no finding.\n{}\n\
         withdrawn: docs/srs/… — a placeholder\n{}\na.rs:9 — nit — x\n{}\n\
         outcome: findings — five.\n",
        FIVE[0], FIVE[1], FIVE[2], FIVE[3], FIVE[4]
    )
}

#[test]
fn a_split_keeps_the_findings_and_sets_aside_what_is_no_finding_and_what_does_not_read() {
    let got = split(&mixed());

    assert_eq!(
        got.clean,
        format!("{}\noutcome: findings — five.\n", FIVE.join("\n"))
    );
    assert_eq!(
        got.ignored,
        [
            Ignored {
                line: 3,
                text: "f.rs:6 — if not addressed: the range matches — no finding.".to_owned(),
                reason: Reason::AllWell,
            },
            Ignored {
                line: 5,
                text: "withdrawn: docs/srs/… — a placeholder".to_owned(),
                reason: Reason::Placeholder,
            },
        ]
    );
    assert_eq!(
        got.unreadable,
        [Unreadable {
            line: 7,
            text: "a.rs:9 — nit — x".to_owned(),
            problem: Problem::Malformed { line: 7 },
        }]
    );
    assert_eq!(
        report_check(Some(&got.clean), Some("outcome: findings — five."), None),
        None
    );
}

#[test]
fn the_lines_of_run_1428_are_set_aside_not_refused() {
    // lang-rust says at its place that it found nothing, then why: the
    // first words after the dash are `no finding`, so it says all is well.
    let soak = "crates/store/tests/soak.rs:1 — no finding; the soak's unwrap calls sit in an integration test…";
    // A withdrawal whose place is `docs/srs/…` names no file and line:
    // the agent says itself it is a placeholder.
    let placeholder =
        "withdrawn: docs/srs/… — a placeholder left in my first message, not a finding";
    let report = format!("{soak}\n{placeholder}\noutcome: none — nothing.");

    let got = split(&report);

    assert_eq!(got.clean, "outcome: none — nothing.\n");
    assert_eq!(
        got.ignored
            .iter()
            .map(|i| (i.line, i.reason))
            .collect::<Vec<_>>(),
        [(1, Reason::AllWell), (2, Reason::Placeholder)]
    );
    assert!(got.unreadable.is_empty());
}

#[test]
fn a_split_sets_aside_the_notes_the_assembly_writes() {
    let report = assemble(
        &[
            "a.rs:1 — if not addressed: x — I2".to_owned(),
            "withdrawn: b.rs:2 — it holds".to_owned(),
            "c.rs:3 — fine".to_owned(),
        ],
        "outcome: findings — x",
        None,
    );

    let got = split(&report);

    assert_eq!(
        got.clean,
        "a.rs:1 — if not addressed: x — I2\noutcome: findings — x\n"
    );
    assert_eq!(
        got.ignored
            .iter()
            .map(|i| (i.line, i.reason))
            .collect::<Vec<_>>(),
        [(2, Reason::WithdrewNothing), (3, Reason::AllWell)]
    );
}

#[test]
fn a_split_keeps_a_revision_that_found_nothing_as_the_finding_it_states() {
    let report = assemble(
        &["revised: c.rs:1 — if not addressed: x — I2".to_owned()],
        "outcome: findings — x",
        None,
    );

    let got = split(&report);

    assert_eq!(
        got.clean,
        "c.rs:1 — if not addressed: x — I2\noutcome: findings — x\n"
    );
    assert!(got.ignored.is_empty() && got.unreadable.is_empty());
    let left = split("revised: c.rs:1 — if not addressed: x — I2\noutcome: findings — x");
    assert_eq!(
        derived_outcome(&left.clean),
        "outcome: findings — 1 finding stated in the run log"
    );
}

#[test]
fn a_split_names_each_line_that_does_not_read_with_the_gate_s_problem() {
    let report = "I read it.\n\
                  a.rs:1 if not addressed: x — I2\n\
                  a.rs:2 — if not addressed: x — I2 (provisional)\n\
                  a.rs:3 — the range matches\n\
                  `path:line` — if not addressed: x — I2\n\
                  revised: a.rs:4 — nit — x\n\
                  withdrawn: a.rs — no line\n\
                  outcome: none — x";

    let got = split(report);

    assert_eq!(got.clean, "I read it.\noutcome: none — x\n");
    assert!(got.ignored.is_empty());
    assert_eq!(
        got.unreadable
            .iter()
            .map(|u| (u.line, u.problem.clone()))
            .collect::<Vec<_>>(),
        [
            (2, Problem::Undashed { line: 2 }),
            (3, Problem::AfterIds { line: 3 }),
            (4, Problem::Unlinked { line: 4 }),
            (5, Problem::TemplatePlace { line: 5 }),
            (6, Problem::Malformed { line: 6 }),
            (7, Problem::Malformed { line: 7 }),
        ]
    );
}

#[test]
fn a_split_with_the_known_ids_sets_aside_a_finding_citing_an_unknown_one() {
    let report = "a.rs:1 — if not addressed: x — ENG-04\n\
                  b.rs:2 — if not addressed: y — PIN-99\n\
                  revised: c.rs:3 — if not addressed: z — ENG-98\n\
                  d.rs:4 — request: add a cap — why\n\
                  outcome: findings — x";
    let known = ["ENG-04".to_owned()].into_iter().collect();

    let got = split_with(report, Some(&known));

    assert_eq!(
        got.clean,
        "a.rs:1 — if not addressed: x — ENG-04\nd.rs:4 — request: add a cap — why\noutcome: findings — x\n"
    );
    assert_eq!(
        got.unreadable
            .iter()
            .map(|u| u.problem.clone())
            .collect::<Vec<_>>(),
        [
            Problem::UnknownId {
                line: 2,
                id: "PIN-99".to_owned()
            },
            Problem::UnknownId {
                line: 3,
                id: "ENG-98".to_owned()
            },
        ]
    );
    assert!(split(report).unreadable.is_empty());
}

#[test]
fn says_all_well_reads_all_is_well_first_or_last_after_a_place() {
    for text in [
        "a.rs:1 — fine",
        "a.rs:1 — No findings: the row holds",
        "a.rs:1 — none. The row holds — as read",
        "a.rs:1 — if not addressed: x — sound",
        "**a.rs:1** — no finding",
    ] {
        assert!(says_all_well(text), "{text:?}");
    }
    for text in [
        "a.rs:1 — none of the hooks fail open",
        "a.rs:1 — finely tuned",
        "a.rs:1 — if not addressed: none — x",
        "a b.rs:1 — fine; really",
        "see — fine; really",
        "a.rs:1",
    ] {
        assert!(!says_all_well(text), "{text:?}");
    }
}

#[test]
fn placeholder_reads_a_revision_or_withdrawal_at_no_real_place() {
    for text in [
        "withdrawn: docs/srs/… — x",
        "withdrawn: docs/srs/...:12 x",
        "revised: <path>:<line> — if not addressed: x — I2",
        "Withdrawn: path:line — x",
    ] {
        assert!(placeholder(text), "{text:?}");
    }
    for text in [
        "withdrawn: a.rs — no line",
        "docs/srs/… — if not addressed: x — I2",
        "revised: a.rs:4 — nit — x",
    ] {
        assert!(!placeholder(text), "{text:?}");
    }
}

#[test]
fn every_reason_reads_as_a_sentence_about_the_line() {
    for (reason, want) in [
        (Reason::AllWell, "says all is well, which is not a finding"),
        (
            Reason::WithdrewNothing,
            "notes a withdrawal that removed nothing",
        ),
        (
            Reason::Placeholder,
            "names a placeholder where its place belongs, not a file and line",
        ),
    ] {
        assert_eq!(reason.to_string(), want);
    }
}

#[test]
fn a_derived_outcome_counts_the_findings_and_requests_the_lines_state() {
    assert_eq!(
        derived_outcome(""),
        "outcome: none — nothing stated in the run log"
    );
    assert_eq!(
        derived_outcome("I read it.\nwithdrawn: a.rs:1 — x\noutcome: findings — x"),
        "outcome: none — nothing stated in the run log"
    );
    assert_eq!(
        derived_outcome(&FIVE.join("\n")),
        "outcome: findings — 5 findings stated in the run log"
    );
}

#[test]
fn with_outcome_keeps_an_agreeing_answer_and_puts_the_derived_line_in_place_of_any_other() {
    let findings = format!("{}\noutcome: none — stale\n", FIVE[0]);
    for (clean, answer, want) in [
        (
            findings.as_str(),
            "I am done.\noutcome: findings — one hook.",
            "outcome: findings — one hook.",
        ),
        (
            findings.as_str(),
            "outcome: none — all holds.",
            "outcome: findings — 1 finding stated in the run log",
        ),
        (
            findings.as_str(),
            "I am done.",
            "outcome: findings — 1 finding stated in the run log",
        ),
        (
            "I read it.",
            "outcome: none — all holds.",
            "outcome: none — all holds.",
        ),
        (
            "I read it.",
            "outcome: findings — many",
            "outcome: none — nothing stated in the run log",
        ),
    ] {
        let got = with_outcome(clean, answer);

        let lines = clean
            .lines()
            .filter(|l| !l.starts_with("outcome:"))
            .chain([want])
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        assert_eq!(got, lines, "{answer:?}");
        assert_eq!(stated_check(&got, None), None, "{answer:?}");
    }
}

#[test]
fn a_stated_check_still_names_a_line_that_does_not_read() {
    assert_eq!(
        stated_check("a.rs:1 — nit — x\noutcome: findings — x", None),
        Some(Problem::Malformed { line: 1 })
    );
}
