use super::*;

const PATH: &str = "docs/domain-model/harness-facts.md";

#[test]
fn a_line_names_the_path_it_holds() {
    assert!(names(
        "docs/domain-model/harness-facts.md:19 — if not addressed: x — ASM-03",
        PATH,
        &[PATH]
    ));
    assert!(names(
        "pr-head/docs/domain-model/harness-facts.md:19 — blocking — x — yours",
        PATH,
        &[]
    ));
    assert!(!names(
        "docs/srs/02-context.md:19 — nit — x — yours",
        PATH,
        &[PATH]
    ));
}

#[test]
fn a_place_that_is_a_tail_of_the_path_names_it() {
    for place in ["harness-facts.md:19", "domain-model/harness-facts.md:19"] {
        let line = format!("{place} — if not addressed: it cites ASM-99 — ASM-03");
        assert!(names(&line, PATH, &[PATH]), "{place}");
    }
    assert!(names(
        "`harness-facts.md:19` — blocking — x — yours",
        PATH,
        &[]
    ));
    assert!(names(
        "- **harness-facts.md:19-21** — nit — x — yours",
        PATH,
        &[]
    ));
}

#[test]
fn a_tail_is_cut_at_a_directory_and_never_inside_a_name() {
    assert!(!names(
        "facts.md:19 — if not addressed: x — ASM-03",
        PATH,
        &[PATH]
    ));
    assert!(!names(
        "model/harness-facts.md:19 — nit — x — yours",
        PATH,
        &[PATH]
    ));
}

#[test]
fn a_dropped_or_changed_line_names_the_path_by_the_place_after_its_word() {
    assert!(names(
        "dropped — harness-facts.md:19 — a style point — lang-markdown",
        PATH,
        &[PATH]
    ));
    assert!(names(
        "withdrawn: harness-facts.md:19 — it moved",
        PATH,
        &[PATH]
    ));
    assert!(names(
        "revised: harness-facts.md:19 — if not addressed: x — ASM-03",
        PATH,
        &[PATH]
    ));
}

#[test]
fn a_tail_named_only_in_the_prose_names_nothing() {
    assert!(!names(
        "docs/srs/02-context.md:3 — if not addressed: it disagrees with harness-facts.md:19 — ASM-03",
        PATH,
        &[PATH]
    ));
}

#[test]
fn a_tail_two_files_of_the_case_share_names_neither() {
    let files = [
        "tooling/x/src/shape/tests.rs",
        "tooling/x/src/tally/tests.rs",
    ];
    let line = "tests.rs:4 — nit — one test covers two functions — test-engineer-unit";
    assert!(!names(line, files[0], &files));
    assert!(!names(line, files[1], &files));
    assert!(names(
        "shape/tests.rs:4 — nit — x — yours",
        files[0],
        &files
    ));
    assert!(!names(
        "shape/tests.rs:4 — nit — x — yours",
        files[1],
        &files
    ));
}
