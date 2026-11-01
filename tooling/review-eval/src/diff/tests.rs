use super::*;

#[test]
fn a_new_file_is_all_additions() {
    let got = unified("x.txt", None, Some("a\nb\n"));

    assert_eq!(
        got,
        "diff --git a/x.txt b/x.txt\nnew file mode 100644\n--- /dev/null\n+++ b/x.txt\n@@ -0,0 +1,2 @@\n+a\n+b\n"
    );
}

#[test]
fn a_deleted_file_is_all_removals() {
    let got = unified("x.txt", Some("a\n"), None);

    assert_eq!(
        got,
        "diff --git a/x.txt b/x.txt\ndeleted file mode 100644\n--- a/x.txt\n+++ /dev/null\n@@ -1,1 +0,0 @@\n-a\n"
    );
}

#[test]
fn an_edit_in_the_middle_keeps_three_lines_of_context() {
    let old = "1\n2\n3\n4\n5\n6\n7\n8\n9\n";
    let new = "1\n2\n3\n4\nfive\n6\n7\n8\n9\n";

    let got = unified("n.txt", Some(old), Some(new));

    assert_eq!(
        got,
        "diff --git a/n.txt b/n.txt\n--- a/n.txt\n+++ b/n.txt\n@@ -2,7 +2,7 @@\n 2\n 3\n 4\n-5\n+five\n 6\n 7\n 8\n"
    );
}

#[test]
fn two_far_apart_edits_make_two_hunks() {
    let old: String = (1..=20).map(|n| format!("{n}\n")).collect();
    let new = old
        .replacen("\n2\n", "\ntwo\n", 1)
        .replacen("\n19\n", "\nnineteen\n", 1);

    let got = unified("n.txt", Some(&old), Some(&new));

    assert_eq!(got.matches("@@ ").count(), 2, "{got}");
    assert!(
        got.contains("@@ -1,5 +1,5 @@\n 1\n-2\n+two\n 3\n 4\n 5\n"),
        "{got}"
    );
    assert!(
        got.contains("@@ -16,5 +16,5 @@\n 16\n 17\n 18\n-19\n+nineteen\n 20\n"),
        "{got}"
    );
}

#[test]
fn two_close_edits_share_a_hunk() {
    let old: String = (1..=10).map(|n| format!("{n}\n")).collect();
    let new = old
        .replacen("\n3\n", "\nthree\n", 1)
        .replacen("\n7\n", "\nseven\n", 1);

    let got = unified("n.txt", Some(&old), Some(&new));

    assert_eq!(got.matches("@@ ").count(), 1, "{got}");
}

#[test]
fn an_insertion_and_a_removal_are_marked() {
    let got = unified("n.txt", Some("a\nb\nc\n"), Some("a\nc\nd\n"));

    assert_eq!(
        got,
        "diff --git a/n.txt b/n.txt\n--- a/n.txt\n+++ b/n.txt\n@@ -1,3 +1,3 @@\n a\n-b\n c\n+d\n"
    );
}

#[test]
fn a_missing_final_newline_is_noted() {
    let got = unified("n.txt", Some("a\nb"), Some("a\nb\n"));

    assert_eq!(
        got,
        "diff --git a/n.txt b/n.txt\n--- a/n.txt\n+++ b/n.txt\n@@ -1,2 +1,2 @@\n a\n-b\n\\ No newline at end of file\n+b\n"
    );
}

#[test]
fn the_same_text_on_both_sides_is_no_diff() {
    assert_eq!(unified("n.txt", Some("a\n"), Some("a\n")), "");
    assert_eq!(unified("n.txt", None, None), "");
}

#[test]
fn an_emptied_file_shows_its_removed_lines() {
    let got = unified("n.txt", Some("a\n"), Some(""));

    assert_eq!(
        got,
        "diff --git a/n.txt b/n.txt\n--- a/n.txt\n+++ b/n.txt\n@@ -1,1 +0,0 @@\n-a\n"
    );
}

#[test]
fn all_writes_the_files_in_path_order() {
    let changes = vec![
        Change {
            path: "b.txt".into(),
            old: None,
            new: Some("b\n".into()),
        },
        Change {
            path: "a.txt".into(),
            old: None,
            new: Some("a\n".into()),
        },
    ];

    let got = all(&changes);

    let a = got.find("diff --git a/a.txt").unwrap_or(usize::MAX);
    let b = got.find("diff --git a/b.txt").unwrap_or(usize::MAX);
    assert!(a < b, "{got}");
}

#[test]
fn the_middle_prefers_to_delete_before_inserting() {
    let got = middle(&["x\n"], &["y\n"]);

    assert_eq!(got, vec![Op::Delete(0), Op::Insert(0)]);
}

#[test]
fn an_edit_script_with_no_change_is_all_equal() {
    assert_eq!(
        edits(&["a\n", "b\n"], &["a\n", "b\n"]),
        vec![Op::Equal(0, 0), Op::Equal(1, 1)]
    );
    assert!(hunks(&edits(&["a\n"], &["a\n"])).is_empty());
}
