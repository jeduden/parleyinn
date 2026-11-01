use super::*;

const DIFF: &str = "\
diff --git a/features/record.feature b/features/record.feature
--- a/features/record.feature
+++ b/features/record.feature
@@ -1,3 +1,3 @@
-  @REC-01 @P0 @pending
+  @REC-01 @P0
   Scenario: REC-02 stays as it is
diff --git a/docs/old.md b/docs/new.md
similarity index 90%
rename from docs/old.md
rename to docs/new.md
";

#[test]
fn changed_paths_reads_both_sides_of_every_header() {
    let got = changed_paths(DIFF);

    assert_eq!(
        got.paths,
        ["docs/new.md", "docs/old.md", "features/record.feature"]
    );
    assert!(got.unparsed.is_empty());
    assert_eq!(changed_paths("not a diff"), Paths::default());
}

#[test]
fn mentioned_ids_reads_an_id_that_starts_a_removed_line() {
    assert_eq!(
        mentioned_ids("-REC-03 gone\n+PIN-04 new\n"),
        ["PIN-04", "REC-03"]
    );
}

#[test]
fn mentioned_ids_reads_only_changed_lines() {
    assert_eq!(mentioned_ids(DIFF), ["REC-01"]);
    assert_eq!(
        mentioned_ids("+++ b/ENG-21\n--- a/ENG-22\n+ see ENG-28, and NOT-01 or ENG-2\n"),
        ["ENG-28"]
    );
}

#[test]
fn changed_paths_unquotes_a_header_git_quoted() {
    let diff = "diff --git \"a/x\\303\\251.yml\" \"b/x\\303\\251.yml\"\n\
                diff --git \"a/q\\\"\\\\\\t\\n\\r.md\" b/plain.md\n\
                diff --git a/plain2.md \"b/t\\ty.md\"\n";

    let got = changed_paths(diff);

    assert_eq!(
        got.paths,
        [
            "plain.md",
            "plain2.md",
            "q\"\\\t\n\r.md",
            "t\ty.md",
            "xé.yml"
        ]
    );
    assert!(got.unparsed.is_empty(), "{:?}", got.unparsed);
}

#[test]
fn changed_paths_reads_invalid_utf_8_lossily() {
    let got = changed_paths("diff --git \"a/\\377\" \"b/\\377\"\n");

    assert_eq!(got.paths, ["\u{fffd}"]);
}

#[test]
fn changed_paths_lists_every_header_it_cannot_read() {
    let bad = [
        "diff --git a/only-one-side",
        "diff --git \"a/unterminated b/x",
        "diff --git \"a/x\" \"b/y",
        "diff --git \"a/x\"\"b/y\"",
        "diff --git \"a/\\q\" \"b/x\"",
        "diff --git \"a/\\12\" \"b/x\"",
        "diff --git \"a/\\9zz\" \"b/x\"",
        "diff --git \"a/x\\",
        "diff --git \"x\" \"b/x\"",
        "diff --git a/x \"y\"",
        "diff --git \"a/x\" y",
    ];
    let diff: String = bad.iter().map(|l| format!("{l}\n")).collect();

    let got = changed_paths(&diff);

    assert!(got.paths.is_empty(), "{:?}", got.paths);
    assert_eq!(got.unparsed, bad);
}

#[test]
fn unquote_reads_the_c_escapes_git_writes() {
    assert_eq!(
        unquote("a\\\\b\\\"c\\td\\ne\\rf\\101"),
        Some(b"a\\b\"c\td\ne\rfA".to_vec())
    );
    assert_eq!(unquote("\\400"), None);
    assert_eq!(unquote("\\"), None);
}

#[test]
fn added_lines_names_each_added_line_with_its_file() {
    let diff = "\
diff --git a/x.rs b/y.rs
--- a/x.rs
+++ b/y.rs
+one
-gone
 same
diff --git \"a/\\303\\251\" \"b/\\303\\251\"
+two
diff --git broken
+three
";

    assert_eq!(
        added_lines(diff),
        [
            ("y.rs".to_owned(), "one"),
            ("é".to_owned(), "two"),
            ("diff --git broken".to_owned(), "three"),
        ]
    );
    assert!(added_lines("+orphan\n").is_empty());
}

#[test]
fn changed_paths_reads_a_path_holding_b_slash_only_when_unambiguous() {
    let same = "diff --git a/crates/x b/clippy.toml b/crates/x b/clippy.toml\n";
    let renamed = "diff --git a/x b/y.md b/docs/srs/invariants.md\n";

    assert_eq!(changed_paths(same).paths, ["crates/x b/clippy.toml"]);
    let got = changed_paths(renamed);
    assert!(got.paths.is_empty(), "{got:?}");
    assert_eq!(got.unparsed, [renamed.trim_end()]);
}

#[test]
fn ids_in_code_leave_out_what_markdown_quotes() {
    let diff = "\
+REC-09 before any header
diff --git a/.claude/agents/a.md b/.claude/agents/a.md
+ restore blocks (INJ-01, INJ-02)
diff --git a/crates/pin/src/lib.rs b/crates/pin/src/lib.rs
+++ b/crates/pin/src/lib.rs
+// PIN-03 holds
-// PIN-04 held
diff --git a/crates/pin/src/lib/tests.rs b/crates/pin/src/lib/tests.rs
+ fixture(\"PIN-05\")
diff --git a/crates/pin/tests/e2e.rs b/crates/pin/tests/e2e.rs
+ fixture(\"PIN-06\")
";

    assert_eq!(ids_in_code(diff), ["PIN-03", "PIN-04"]);
}

#[test]
fn only_keeps_the_files_named_and_every_header_that_does_not_read() {
    let diff = "\
lead-in before any header
diff --git a/a.rs b/a.rs
+one
diff --git a/old.md b/new.md
rename from old.md
diff --git nonsense
+?
diff --git a/c.toml b/c.toml
+three
";
    let keep = |names: &[&str]| names.iter().map(|n| (*n).to_owned()).collect();

    assert_eq!(
        only(diff, &keep(&["a.rs", "new.md"])),
        "diff --git a/a.rs b/a.rs\n+one\n\
         diff --git a/old.md b/new.md\nrename from old.md\n\
         diff --git nonsense\n+?\n"
    );
    assert_eq!(
        only(diff, &keep(&["old.md"])),
        "diff --git a/old.md b/new.md\nrename from old.md\ndiff --git nonsense\n+?\n"
    );
    assert_eq!(only(diff, &keep(&[])), "diff --git nonsense\n+?\n");
    assert_eq!(only("", &keep(&["a.rs"])), "");
}

#[test]
fn ids_by_file_names_each_file_s_changed_ids() {
    let diff = "\
+REC-09 before any header
diff --git a/features/x.feature b/features/x.feature
+++ b/features/x.feature
+  @REC-01 @P0
   @REC-02 unchanged
diff --git a/docs/a.md b/docs/a.md
-REC-03 and PIN-04
diff --git a/b.rs b/b.rs
 nothing changed
";

    assert_eq!(
        ids_by_file(diff),
        BTreeMap::from([
            (
                "docs/a.md".to_owned(),
                ["PIN-04", "REC-03"].map(str::to_owned).into()
            ),
            (
                "features/x.feature".to_owned(),
                ["REC-01".to_owned()].into()
            ),
        ])
    );
}

/// A file's section of a diff, `size` bytes long, for `path`.
fn section(path: &str, size: usize) -> String {
    let head = format!("diff --git a/{path} b/{path}\n");
    let body = "+".repeat(size - head.len() - 1);
    format!("{head}{body}\n")
}

fn files(parts: &[Part]) -> Vec<Vec<&str>> {
    parts
        .iter()
        .map(|p| p.files.iter().map(String::as_str).collect())
        .collect()
}

#[test]
fn split_packs_whole_files_in_path_order_up_to_the_limit() {
    let diff = [
        section("src/c.rs", 40),
        section("src/a.rs", 40),
        section("src/b.rs", 40),
        section("src/d.rs", 40),
    ]
    .concat();

    let parts = split(&diff, 100);

    assert_eq!(
        files(&parts),
        [vec!["src/a.rs", "src/b.rs"], vec!["src/c.rs", "src/d.rs"]]
    );
    assert_eq!(
        parts[0].text,
        [section("src/a.rs", 40), section("src/b.rs", 40)].concat()
    );
    assert_eq!(
        parts.iter().map(|p| p.text.len()).sum::<usize>(),
        diff.len()
    );
}

#[test]
fn split_never_splits_a_file_and_a_file_past_the_limit_goes_alone() {
    let diff = [
        section("a.rs", 30),
        section("b.rs", 250),
        section("c.rs", 30),
    ]
    .concat();

    let parts = split(&diff, 100);

    assert_eq!(files(&parts), [vec!["a.rs"], vec!["b.rs"], vec!["c.rs"]]);
    assert_eq!(parts[1].text, section("b.rs", 250));
}

#[test]
fn split_fills_a_part_to_exactly_the_limit() {
    let diff = [
        section("a.rs", 50),
        section("b.rs", 50),
        section("c.rs", 30),
    ]
    .concat();

    assert_eq!(
        files(&split(&diff, 100)),
        [vec!["a.rs", "b.rs"], vec!["c.rs"]]
    );
    assert_eq!(
        files(&split(&diff, 99)),
        [vec!["a.rs"], vec!["b.rs", "c.rs"]]
    );
}

#[test]
fn split_of_an_empty_diff_has_no_part_and_a_preamble_leads_the_first() {
    assert!(split("", 100).is_empty());

    let diff = format!("preamble\n{}", section("a.rs", 40));
    let parts = split(&diff, 100);

    assert_eq!(files(&parts), [vec!["a.rs"]]);
    assert_eq!(parts[0].text, diff);
    // A diff of no file at all is one part naming none, so nothing of
    // it is lost.
    assert_eq!(
        split("+x\n", 100),
        [Part {
            files: Vec::new(),
            text: "+x\n".to_owned()
        }]
    );
}

#[test]
fn split_names_a_header_that_does_not_read_as_itself() {
    let diff = format!("diff --git nonsense\n+x\n{}", section("a.rs", 40));

    let parts = split(&diff, 30);

    assert_eq!(files(&parts), [vec!["a.rs"], vec!["diff --git nonsense"]]);
}

#[test]
fn split_is_deterministic_whatever_order_the_diff_lists_its_files_in() {
    let one = [
        section("b.rs", 60),
        section("a.rs", 60),
        section("c.rs", 30),
    ]
    .concat();
    let other = [
        section("c.rs", 30),
        section("a.rs", 60),
        section("b.rs", 60),
    ]
    .concat();

    assert_eq!(split(&one, 100), split(&other, 100));
    assert_eq!(
        files(&split(&one, 100)),
        [vec!["a.rs"], vec!["b.rs", "c.rs"]]
    );
}
