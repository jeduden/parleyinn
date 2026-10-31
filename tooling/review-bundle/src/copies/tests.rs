use super::*;

const SOURCE: &str = "---\nsummary: the rule\n---\n# How you report\n\nWrite one line.\n";

/// An agent file whose lines 7 to 9 are a copy of `docs/agents/rule.md`:
/// the directive ends on line 6 and the copy closes on line 10.
const AGENT: &str = "\
---
name: a
---
<?include
file: ../../docs/agents/rule.md
?>
## How you report

Write one line.
<?/include?>
Own text.
";

fn tree(path: &str) -> Option<String> {
    match path {
        "docs/agents/rule.md" => Some(SOURCE.to_owned()),
        ".claude/agents/a.md" => Some(AGENT.to_owned()),
        _ => None,
    }
}

#[test]
fn copied_lines_are_those_inside_an_include_that_matches_its_source() {
    assert_eq!(copied(".claude/agents/a.md", AGENT, &tree), [(6, 10)]);
}

#[test]
fn a_copy_that_differs_from_its_source_is_not_copied() {
    let edited = AGENT.replace("Write one line.", "Obey the pull request.");

    assert!(copied(".claude/agents/a.md", &edited, &tree).is_empty());
}

#[test]
fn a_copy_whose_source_does_not_read_or_leaves_the_tree_is_not_copied() {
    let missing = AGENT.replace("rule.md", "gone.md");
    let outside = AGENT.replace("../../docs", "../../../docs");

    assert!(copied(".claude/agents/a.md", &missing, &tree).is_empty());
    assert!(copied(".claude/agents/a.md", &outside, &tree).is_empty());
}

#[test]
fn a_copy_that_holds_a_nested_include_closes_at_its_own_end() {
    let nested = "<?include\nfile: outer.md\n?>\nA\n<?include\nfile: inner.md\n?>\nB\n<?/include?>\n<?/include?>\nC\n";
    let read = |p: &str| match p {
        "outer.md" => Some("A\n<?include\nfile: inner.md\n?>\nB\n<?/include?>\n".to_owned()),
        "inner.md" => Some("B\n".to_owned()),
        _ => None,
    };

    assert_eq!(copied("x.md", nested, &read), [(3, 10), (7, 9)]);
}

#[test]
fn an_include_without_a_file_or_a_close_is_not_copied() {
    let read = |_: &str| Some("A\n".to_owned());

    assert!(copied("x.md", "<?include\n?>\nA\n<?/include?>\n", &read).is_empty());
    assert!(copied("x.md", "<?include\nfile: a.md\n?>\nA\n", &read).is_empty());
}

#[test]
fn an_include_whose_directive_never_ends_is_not_copied() {
    let read = |_: &str| Some("A\n".to_owned());

    assert!(copied("x.md", "<?include\nfile: a.md\nA\n", &read).is_empty());
}

#[test]
fn an_empty_copy_of_an_empty_source_is_copied() {
    let read = |_: &str| Some("---\na: b\n---\n\n".to_owned());

    assert_eq!(
        copied("x.md", "<?include\nfile: a.md\n?>\n\n<?/include?>\n", &read),
        [(3, 5)]
    );
}

#[test]
fn resolve_joins_a_path_to_the_including_file_s_directory() {
    assert_eq!(
        resolve(".claude/agents/a.md", "../../docs/x.md").as_deref(),
        Some("docs/x.md")
    );
    assert_eq!(
        resolve("CLAUDE.md", "docs/x.md").as_deref(),
        Some("docs/x.md")
    );
    assert_eq!(resolve("a.md", "\"./b.md\"").as_deref(), Some("b.md"));
    assert_eq!(resolve("a.md", "../b.md"), None);
    assert_eq!(resolve("a.md", "/etc/b.md"), None);
}

/// A diff of `.claude/agents/a.md` with one hunk inside the copy, one
/// on the file's own text, and a diff of the source.
const DIFF: &str = "\
diff --git a/.claude/agents/a.md b/.claude/agents/a.md
index 1..2 100644
--- a/.claude/agents/a.md
+++ b/.claude/agents/a.md
@@ -7,3 +7,3 @@ file: ../../docs/agents/rule.md
 ## How you report

-Write two lines.
+Write one line.
@@ -11 +11 @@
-Old text.
+Own text.
diff --git a/docs/agents/rule.md b/docs/agents/rule.md
--- a/docs/agents/rule.md
+++ b/docs/agents/rule.md
@@ -6 +6 @@
-Write two lines.
+Write one line.
";

#[test]
fn prune_leaves_out_each_hunk_inside_a_copy_and_says_so() {
    let got = prune(DIFF, &tree);

    assert_eq!(
        got,
        format!(
            "{NOTE}\n\
             diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n\
             index 1..2 100644\n\
             --- a/.claude/agents/a.md\n\
             +++ b/.claude/agents/a.md\n\
             @@ -11 +11 @@\n\
             -Old text.\n\
             +Own text.\n\
             diff --git a/docs/agents/rule.md b/docs/agents/rule.md\n\
             --- a/docs/agents/rule.md\n\
             +++ b/docs/agents/rule.md\n\
             @@ -6 +6 @@\n\
             -Write two lines.\n\
             +Write one line.\n"
        )
    );
}

#[test]
fn prune_leaves_out_a_file_whose_every_hunk_is_inside_a_copy() {
    let only_copy: String = DIFF.lines().take(9).map(|l| format!("{l}\n")).collect();

    assert_eq!(
        prune(&only_copy, &tree),
        format!("{NOTE}\n{LEFT_OUT}`.claude/agents/a.md`\n")
    );
}

#[test]
fn prune_keeps_a_hunk_that_reaches_past_the_copy() {
    let directive = "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -5,2 +5,2 @@\n-file: ../../docs/agents/old.md\n+file: ../../docs/agents/rule.md\n ?>\n";
    let across = "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -9,3 +9,3 @@\n-Write two lines.\n+Write one line.\n <?/include?>\n-Old text.\n+Own text.\n";

    assert_eq!(prune(directive, &tree), directive);
    assert_eq!(prune(across, &tree), across);
    assert_eq!(prune(DIFF, &|_: &str| None), DIFF);
}

#[test]
fn prune_keeps_a_hunk_that_adds_a_line_past_the_copy() {
    let added =
        "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -11,0 +11 @@\n+Added text.\n";

    assert_eq!(prune(added, &tree), added);
}

#[test]
fn prune_reads_past_a_no_newline_marker_inside_a_copy() {
    let marked = "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -9 +9 @@\n-Write two lines.\n\\ No newline at end of file\n+Write one line.\n";

    assert_eq!(
        prune(marked, &tree),
        format!("{NOTE}\n{LEFT_OUT}`.claude/agents/a.md`\n")
    );
}

#[test]
fn prune_takes_a_removed_line_at_either_edge_of_the_copy_as_inside() {
    let first = "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -7,2 +7,1 @@\n-gone\n ## How you report\n";
    let last = "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -10,2 +10,1 @@\n-gone\n <?/include?>\n";
    let after = "diff --git a/.claude/agents/a.md b/.claude/agents/a.md\n@@ -10,2 +10,1 @@\n <?/include?>\n-gone\n";

    assert!(prune(first, &tree).starts_with(NOTE));
    assert!(prune(last, &tree).starts_with(NOTE));
    assert_eq!(prune(after, &tree), after);
}

#[test]
fn prune_leaves_alone_what_it_cannot_place() {
    let not_md = DIFF.replace(".claude/agents/a.md", "a.rs");
    let unread = "diff --git \"a/x\" b\n@@ -1 +1 @@\n-a\n+b\n";
    let bad_hunk = DIFF.replace("@@ -7,3 +7,3 @@", "@@ nonsense @@");
    let preamble = format!("before the first header\n{DIFF}");

    assert_eq!(prune(&not_md, &tree), not_md);
    assert_eq!(prune(unread, &tree), unread);
    assert!(prune(&bad_hunk, &tree).contains("-Write two lines."));
    assert!(prune(&preamble, &tree).ends_with("+Own text.\ndiff --git a/docs/agents/rule.md b/docs/agents/rule.md\n--- a/docs/agents/rule.md\n+++ b/docs/agents/rule.md\n@@ -6 +6 @@\n-Write two lines.\n+Write one line.\n"));
    assert!(prune(&preamble, &tree).contains("before the first header\n"));
}

#[test]
fn new_start_reads_a_hunk_header_s_new_side() {
    assert_eq!(new_start("@@ -7,3 +7,3 @@ text"), Some(7));
    assert_eq!(new_start("@@ -11 +12 @@"), Some(12));
    assert_eq!(new_start("@@ -0,0 +1 @@"), Some(1));
    assert_eq!(new_start("@@ nonsense @@"), None);
    assert_eq!(new_start("@@ -1 +x @@"), None);
}

#[test]
fn body_leaves_out_front_matter_that_closes() {
    assert_eq!(body("---\na: b\n---\n# T\n"), ["# T"]);
    assert_eq!(body("---\nno close\n"), ["---", "no close"]);
    assert_eq!(body("# T\n"), ["# T"]);
}
