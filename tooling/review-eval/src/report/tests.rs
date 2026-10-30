use super::*;

#[test]
fn a_report_reads_its_findings_and_outcome() {
    let text = "crates/x/src/a.rs:12 — blocking — an unwrap outside tests — ENG-16\n\
                `docs/x.md:3` - nit - a term outside the model - I2\n\n\
                outcome: blocking — the change unwraps.\n";

    let r = parse(text);

    assert_eq!(r.findings.len(), 2);
    assert_eq!(r.findings[0].path, "crates/x/src/a.rs");
    assert_eq!(r.findings[0].line, 12);
    assert_eq!(r.findings[0].severity, "blocking");
    assert_eq!(r.findings[0].text, "an unwrap outside tests — ENG-16");
    assert_eq!(r.findings[1].path, "docs/x.md");
    assert_eq!(r.findings[1].severity, "nit");
    assert_eq!(r.outcome.as_deref(), Some("blocking"));
    assert_eq!(
        r.outcome_line.as_deref(),
        Some("outcome: blocking — the change unwraps.")
    );
    assert!(r.other.is_empty());
}

#[test]
fn a_report_without_an_outcome_at_its_end_has_none() {
    let r = parse("outcome: sound — fine\nbut then more text\n");

    assert_eq!(r.outcome, None);
    assert_eq!(r.other, vec!["outcome: sound — fine", "but then more text"]);
}

#[test]
fn headings_are_not_other_lines_but_prose_is() {
    let r = parse("# lang-rust\n\nI read the diff.\n\noutcome: sound\n");

    assert_eq!(r.other, vec!["I read the diff."]);
    assert_eq!(r.outcome.as_deref(), Some("sound"));
}

#[test]
fn the_outcome_word_tolerates_markers_and_punctuation() {
    assert_eq!(outcome_word("outcome: sound"), Some("sound"));
    assert_eq!(outcome_word("**outcome: nits** — one nit"), Some("nits"));
    assert_eq!(outcome_word("- outcome: blocking."), Some("blocking"));
    assert_eq!(outcome_word("> outcome: blocking"), Some("blocking"));
    assert_eq!(outcome_word("outcome: maybe"), None);
    assert_eq!(outcome_word("outcome:"), None);
    assert_eq!(outcome_word("the outcome: sound"), None);
}

#[test]
fn a_finding_line_tolerates_list_markers_and_hyphens() {
    let f = finding("1. tooling/x.rs:7 -- Nit -- a thing").unwrap();

    assert_eq!(
        (f.path.as_str(), f.line, f.severity.as_str()),
        ("tooling/x.rs", 7, "nit")
    );
    assert_eq!(f.text, "a thing");
    assert_eq!(f.raw, "1. tooling/x.rs:7 -- Nit -- a thing");
    assert!(finding("- `a/b.md:1` – blocking – x").is_some());
    assert!(finding("a/b.md:0 — blocking").is_some());
}

#[test]
fn a_line_range_reads_as_its_first_line() {
    let f = finding("- crates/x/src/a.rs:12-20 — blocking — an unwrap — ENG-16").unwrap();
    assert_eq!((f.path.as_str(), f.line), ("crates/x/src/a.rs", 12));
    let f = finding("crates/x/src/a.rs:3–4 — nit — long").unwrap();
    assert_eq!(f.line, 3);
    assert!(finding("crates/x/src/a.rs:-4 — nit — long").is_none());
}

#[test]
fn a_line_that_is_not_a_finding_is_none() {
    assert_eq!(finding("I read the diff and found nothing."), None);
    assert_eq!(finding("a/b.md — blocking — no line"), None);
    assert_eq!(finding("a/b.md:x — blocking — no number"), None);
    assert_eq!(finding(":3 — blocking — no path"), None);
    assert_eq!(finding("a/b.md:3 — a long severity — x"), None);
    assert_eq!(finding("a/b.md:3 —  — x"), None);
}

#[test]
fn a_dropped_line_starts_with_dropped_or_gives_it_as_severity() {
    for line in [
        "dropped — src/a.rs:3 — no such line — lang-toml",
        "Dropped: x",
        "- dropped x",
        "* **DROPPED** x",
        "> _dropped_ x",
        "1. dropped — src/a.rs:3 — why — me",
        "src/d.rs:6 — dropped — not a bug",
        "`src/d.rs:6` — **Dropped** — not a bug",
    ] {
        assert!(dropped(line), "{line}");
    }
    for line in [
        "",
        "drop",
        "src/a.rs:1 — nit — dropped later",
        "src/a.rs:1 — nit — this drops the error — lang-rust",
        "src/a.rs:1 — drop — not the word",
        "kept",
        // A path that merely starts with the letters is a finding on
        // that path, not a dropped line.
        "dropped_events.rs:12 — blocking — leaks a pin — I3",
        "- `dropped.rs`:3 — blocking — leaks a pin — I3",
        "Droppedfoo/bar.rs:3 — nit — tidy — lang-rust",
    ] {
        assert!(!dropped(line), "{line}");
    }
}

#[test]
fn a_none_line_says_a_generalist_has_nothing_to_add() {
    for line in [
        "none — nothing to add — review-code",
        "- **None** - nothing to add - review-code",
        "None — nothing to add",
    ] {
        assert!(nothing(line), "{line}");
    }
    for line in [
        "",
        "none",
        "None of the findings hold.",
        "none_of.rs:3 — nit — tidy — lang-rust",
        "dropped — a.rs:1 — no bug — lang-rust",
    ] {
        assert!(!nothing(line), "{line}");
    }
}

#[test]
fn split_dashes_cuts_at_each_spaced_dash_only() {
    assert_eq!(
        split_dashes("a — b - c -- d – e"),
        vec!["a", "b", "c", "d", "e"]
    );
    assert_eq!(split_dashes("no-dash here"), vec!["no-dash here"]);
    assert_eq!(strip_number("12. x"), "x");
    assert_eq!(strip_number(". x"), ". x");
}
