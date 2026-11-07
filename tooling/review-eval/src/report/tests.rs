use super::*;

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
fn a_finding_whose_place_is_in_markup_reads_as_one() {
    for line in [
        "**a/b.md:3** — blocking — x — lang-markdown",
        "- **`a/b.md:3`** — blocking — x — lang-markdown",
        "[a/b.md:3](a/b.md#L3) — blocking — x — lang-markdown",
    ] {
        let f = finding(line).unwrap_or_else(|| panic!("{line}"));
        assert_eq!((f.path.as_str(), f.line), ("a/b.md", 3), "{line}");
        assert!(!unread(line), "{line}");
    }
}

#[test]
fn a_finding_shaped_line_that_does_not_read_is_unread() {
    for line in [
        "a/b.md — blocking — no line — lang-markdown",
        "a/b.md:x — **nit** — no number",
        "the place — Blocking — breaks I2",
    ] {
        assert!(finding(line).is_none(), "{line}");
        assert!(unread(line), "{line}");
    }
    for line in [
        "a/b.md:3 — blocking — x",
        "dropped — a.rs:1 — blocking — gone — lang-rust",
        "none — nothing to add — review-code",
        "The code is sound — no nit — and no blocker.",
        "The code is sound.",
    ] {
        assert!(!unread(line), "{line}");
    }
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
    assert_eq!(bare("12. x"), "x");
    assert_eq!(bare(". x"), ". x");
}

#[test]
fn a_weighing_keeps_each_line_stated_before_the_final_answer_once() {
    let texts = vec![
        "reading the diff".to_owned(),
        "a/b.md:3 — blocking — breaks I2 — lang-markdown\ndropped — a/c.md:1 — sound — x"
            .to_owned(),
        "a/b.md:3 — blocking — breaks I2 — lang-markdown".to_owned(),
    ];

    let got = weighing(
        &texts,
        "a/d.md:2 — nit — long line — review-code\nScope: all.",
    );

    assert_eq!(
        got,
        "a/b.md:3 — blocking — breaks I2 — lang-markdown\ndropped — a/c.md:1 — sound — x\n\
         a/d.md:2 — nit — long line — review-code\nScope: all."
    );
}

#[test]
fn a_weighing_of_a_run_cut_off_before_its_answer_is_what_it_stated() {
    let texts = vec!["a/b.md:3 — nit — x — y\nsome prose".to_owned()];

    assert_eq!(weighing(&texts, ""), "a/b.md:3 — nit — x — y");
    assert_eq!(weighing(&[], ""), "");
}

#[test]
fn a_weighing_does_not_repeat_a_line_the_final_answer_states_again() {
    let line = "a/b.md:3 — nit — x — y";

    assert_eq!(weighing(&[line.to_owned()], line), line);
}

#[test]
fn a_later_line_at_a_place_supersedes_what_was_stated_there_before() {
    let texts = vec![
        "a.rs:5 — blocking — X — lang-rust\nb.rs:1 — nit — y — lang-rust".to_owned(),
        "dropped — a.rs:5 — not a bug after all — lang-rust".to_owned(),
        "b.rs:1 — blocking — worse than it looked — lang-rust".to_owned(),
    ];

    assert_eq!(
        weighing(&texts, "Scope: all."),
        "dropped — a.rs:5 — not a bug after all — lang-rust\n\
         b.rs:1 — blocking — worse than it looked — lang-rust\nScope: all."
    );
}

#[test]
fn the_final_answer_supersedes_a_stated_line_at_the_same_place() {
    let texts = vec!["a.rs:5 — blocking — X — lang-rust\nc.rs:2 — nit — z — y".to_owned()];

    assert_eq!(
        weighing(&texts, "dropped — a.rs:5 — fine — lang-rust\nDone."),
        "c.rs:2 — nit — z — y\ndropped — a.rs:5 — fine — lang-rust\nDone."
    );
}

#[test]
fn a_narration_line_in_a_message_is_no_unread_finding_but_a_placed_one_is() {
    let texts = vec![
        "Severity — nit — because style\nthe place — Blocking — breaks I2".to_owned(),
        "a/b.md — blocking — no line — lang-markdown".to_owned(),
    ];

    assert_eq!(
        weighing(&texts, "none — nothing to add — x"),
        "a/b.md — blocking — no line — lang-markdown\nnone — nothing to add — x"
    );
    // In the final answer every such line stays, for the review to name.
    let answer = "Severity — nit — because style";
    assert_eq!(weighing(&[], answer), answer);
}

#[test]
fn a_place_is_read_from_a_finding_and_from_either_form_of_a_dropped_line() {
    let at = |path: &str, line| Some((path.to_owned(), line));

    assert_eq!(place_of("a.rs:5 — nit — x"), at("a.rs", 5));
    assert_eq!(place_of("dropped — a.rs:5 — fine — y"), at("a.rs", 5));
    assert_eq!(place_of("a.rs:5 — dropped — fine — y"), at("a.rs", 5));
    assert_eq!(place_of("dropped — nowhere — fine"), None);
    assert_eq!(place_of("none — nothing to add — x"), None);
}

#[test]
fn a_placed_line_starts_with_a_path_like_word() {
    assert!(placed("a/b.md — blocking — x"));
    assert!(placed("**x.rs** — nit — y"));
    assert!(!placed("Severity — nit — y"));
    assert!(!placed("the place — nit — y"));
    assert!(!placed(""));
}

#[test]
fn a_later_line_at_a_place_from_another_source_keeps_the_earlier_one() {
    let texts = vec![
        "a.rs:3 — blocking — leaks a pin — lang-rust\na.rs:3 — nit — long — lang-markdown"
            .to_owned(),
        "dropped — a.rs:3 — misread — lang-markdown".to_owned(),
    ];

    assert_eq!(
        weighing(&texts, "Scope: all."),
        "a.rs:3 — blocking — leaks a pin — lang-rust\n\
         dropped — a.rs:3 — misread — lang-markdown\nScope: all."
    );
}

#[test]
fn the_final_answer_weighs_by_the_same_rule_as_the_messages() {
    let lines = "a.rs:3 — blocking — leaks a pin — lang-rust\n\
                 a.rs:3 — nit — long — lang-markdown\n\
                 a.rs:3 — nit — milder than it looked — lang-rust\n\
                 dropped — a.rs:3 — misread — lang-markdown";
    let want = "a.rs:3 — nit — milder than it looked — lang-rust\n\
                dropped — a.rs:3 — misread — lang-markdown";

    assert_eq!(weighing(&[lines.to_owned()], ""), want);
    assert_eq!(weighing(&[], lines), want);
}

#[test]
fn a_dropped_line_naming_no_source_supersedes_only_a_line_naming_none() {
    let texts = vec!["a.rs:3 — blocking — leaks a pin — lang-rust\nb.rs:1 — nit — long".to_owned()];

    assert_eq!(
        weighing(&texts, "dropped — a.rs:3 — fine\ndropped — b.rs:1 — fine"),
        "a.rs:3 — blocking — leaks a pin — lang-rust\n\
         dropped — a.rs:3 — fine\ndropped — b.rs:1 — fine"
    );
}

#[test]
fn a_source_is_the_last_part_of_a_line_of_four_or_more() {
    assert_eq!(
        source_of("a.rs:3 — nit — x — **Lang-Rust**"),
        Some("lang-rust".to_owned())
    );
    assert_eq!(
        source_of("dropped — a.rs:3 — why — y"),
        Some("y".to_owned())
    );
    assert_eq!(source_of("a.rs:3 — nit — x"), None);
    assert_eq!(source_of("prose — with — four — parts"), None);
}
