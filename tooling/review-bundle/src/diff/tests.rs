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
