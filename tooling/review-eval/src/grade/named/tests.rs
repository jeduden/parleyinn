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

#[test]
fn the_lines_of_a_place_are_its_line_or_the_ends_of_its_range() {
    let path = "docs/x.md";
    let at = |line: &str| lines(line, path, &[path]);

    assert_eq!(at("docs/x.md:12 — blocking — it — yours"), Some((12, 12)));
    assert_eq!(at("x.md:12-20 — nit — it — yours"), Some((12, 20)));
    assert_eq!(at("`docs/x.md:3–4` — nit — it — yours"), Some((3, 4)));
    // A range written backwards covers its first line alone, and so
    // does one whose end is no number.
    assert_eq!(at("docs/x.md:9-2 — nit — it — yours"), Some((9, 9)));
    assert_eq!(at("docs/x.md:3-abc — nit — it — yours"), Some((3, 3)));
    // The pull request's tree, named with its directory.
    assert_eq!(at("pr-head/docs/x.md:7 — nit — it — yours"), Some((7, 7)));
    assert_eq!(
        at("dropped — docs/x.md:5 — main has it — lang-rust"),
        Some((5, 5))
    );
    // Another file's place, and a place the sentence alone holds.
    assert_eq!(at("docs/y.md:12 — nit — like docs/x.md:3 — yours"), None);
    assert_eq!(at("no place here"), None);
}

#[test]
fn a_place_at_a_tail_of_a_case_s_file_is_a_slip_once_a_file() {
    let files = ["docs/x.md", "docs/y.md"];
    let report = "x.md:3 — if not addressed: it — ENG-01\n\
                  x.md:9 — if not addressed: it again — ENG-01\n\
                  docs/y.md:3 — if not addressed: whole — ENG-01\n\
                  z.md:3 — if not addressed: no file of the case — ENG-01\n\
                  outcome: findings — three";

    assert_eq!(
        tail_slips(report, &files),
        vec!["names docs/x.md by its tail x.md"]
    );
    assert_eq!(
        tail_slips("outcome: none — fine", &files),
        Vec::<String>::new()
    );
    // A tail two files of the case share names neither, so it is no
    // slip of either.
    let shared = ["a/tests.rs", "b/tests.rs"];
    assert_eq!(
        tail_slips("tests.rs:4 — if not addressed: it — ENG-01", &shared),
        Vec::<String>::new()
    );
}

#[test]
fn a_tail_of_a_file_under_the_wanted_path_names_it() {
    // A case may want a finding under a module, its file or its tests.
    let module = "tooling/review-bundle/src/sizes";
    let files = [
        "tooling/review-bundle/src/sizes.rs",
        "tooling/review-bundle/src/sizes/tests.rs",
        "tooling/review-bundle/src/lib.rs",
    ];

    for place in ["sizes.rs:32", "src/sizes.rs:32", "sizes/tests.rs:4"] {
        let line = format!("{place} — if not addressed: its error arms take no test — ENG-11");
        assert!(names(&line, module, &files), "{place}");
    }
    // A file of the case outside the module, and a tail two files share.
    let outside = "lib.rs:3 — if not addressed: its error arms take no test — ENG-11";
    assert!(!names(outside, module, &files));
    let shared = "tests.rs:4 — if not addressed: its error arms take no test — ENG-11";
    let twice = [files[1], "tooling/review-bundle/src/sizes/more/tests.rs"];
    assert!(!names(shared, module, &twice));
}
