use super::*;

/// A diff of the size list with `lines`, each with its sign.
fn diff_of(lines: &str) -> String {
    format!("diff --git a/{SIZE_LIST} b/{SIZE_LIST}\n--- a/{SIZE_LIST}\n+++ b/{SIZE_LIST}\n{lines}")
}

#[test]
fn an_entry_reads_as_a_file_and_its_lines() {
    assert_eq!(
        entry("crates/a/src/lib.rs 1200"),
        Some(("crates/a/src/lib.rs", 1200))
    );
    assert_eq!(
        entry("  crates/a/src/lib.rs   7 "),
        Some(("crates/a/src/lib.rs", 7))
    );
    for no_entry in ["", "# a 12", "#a 12", "a", "a b", "a 1 2"] {
        assert_eq!(entry(no_entry), None, "{no_entry:?}");
    }
}

#[test]
fn listed_keeps_the_most_lines_of_each_file_of_the_size_list() {
    let changed = [
        (SIZE_LIST.to_owned(), "a.rs 1100"),
        (SIZE_LIST.to_owned(), "a.rs 1300"),
        (SIZE_LIST.to_owned(), "a.rs 1200"),
        (SIZE_LIST.to_owned(), "# never add a line"),
        ("docs/other.txt".to_owned(), "b.rs 2000"),
    ];

    assert_eq!(listed(&changed), BTreeMap::from([("a.rs", 1300)]));
}

#[test]
fn a_file_added_to_the_size_list_or_raised_in_it_lengthens_it() {
    let diff = diff_of("+new.rs 1001\n-raised.rs 1100\n+raised.rs 1101\n");

    assert_eq!(lengthened(&diff), ["new.rs", "raised.rs"]);
}

#[test]
fn a_file_lowered_moved_or_removed_does_not_lengthen_the_size_list() {
    let diff = diff_of(
        "-lowered.rs 1100\n+lowered.rs 1099\n-moved.rs 1500\n+moved.rs 1500\n-gone.rs 1200\n+# a comment 12\n",
    );

    assert!(lengthened(&diff).is_empty());
    assert!(lengthened("diff --git a/x.txt b/x.txt\n+new.rs 1001\n").is_empty());
}

#[test]
fn a_lengthened_size_list_blocks_until_a_comment_names_it() {
    let diff = diff_of("+new.rs 1001\n-raised.rs 1100\n+raised.rs 1101\n");
    let finding = |severity, body: &str| {
        Some(Finding {
            path: SIZE_LIST.to_owned(),
            line: 0,
            severity,
            body: body.to_owned(),
        })
    };

    assert_eq!(
        finding(
            BLOCKING,
            "this change lengthens the size list (ENG-34): it adds or raises new.rs, \
             raised.rs, and no stakeholder comment names tooling/engineering/oversize.txt, \
             consenting to it"
        ),
        super::finding(&diff, false)
    );
    assert_eq!(
        finding(
            NIT,
            "this change lengthens the size list (ENG-34): it adds or raises new.rs, \
             raised.rs, and a stakeholder comment names tooling/engineering/oversize.txt, \
             consenting to it"
        ),
        super::finding(&diff, true)
    );
}

#[test]
fn a_size_list_that_only_shrinks_raises_no_finding() {
    let diff = diff_of("-lowered.rs 1100\n+lowered.rs 1099\n");

    assert_eq!(finding(&diff, false), None);
    assert_eq!(finding(&diff, true), None);
}
