use super::*;

use std::path::Path;

const DIFF: &str = "review/diffs/lang-rust.diff";
/// The work directory the review runs in.
const WORK: &str = "/home/w";

fn read(path: &str, offset: Option<usize>, limit: Option<usize>) -> ReadCall {
    ReadCall {
        path: path.to_owned(),
        offset,
        limit,
        answered: true,
        bad_range: false,
    }
}

/// The ranges of [`DIFF`] under [`WORK`] that `reads` left unread.
fn unread_of(reads: &[ReadCall], total: usize) -> Vec<(usize, usize)> {
    unread(reads, &[Path::new(WORK)], DIFF, total)
}

#[test]
fn one_read_of_the_whole_file_leaves_nothing_unread() {
    let reads = [read(DIFF, None, None)];

    assert!(unread_of(&reads, 300).is_empty());
}

#[test]
fn reads_in_chunks_and_overlapping_chunks_cover_the_file() {
    let chunks = [
        read(DIFF, Some(1), Some(100)),
        read(DIFF, Some(101), Some(100)),
    ];
    assert!(unread_of(&chunks, 200).is_empty());

    let overlapping = [
        read(DIFF, Some(150), Some(100)),
        read(DIFF, None, Some(160)),
    ];
    assert!(unread_of(&overlapping, 249).is_empty());
}

#[test]
fn a_gap_between_reads_is_unread() {
    let reads = [
        read(DIFF, Some(201), Some(100)),
        read(DIFF, Some(1), Some(100)),
    ];

    assert_eq!(unread_of(&reads, 300), vec![(101, 200)]);
}

#[test]
fn a_read_no_result_answered_or_one_with_a_bad_range_covers_nothing() {
    let unanswered = [ReadCall {
        answered: false,
        ..read(DIFF, None, None)
    }];
    assert_eq!(unread_of(&unanswered, 10), vec![(1, 10)]);

    let bad = [ReadCall {
        bad_range: true,
        ..read(DIFF, None, None)
    }];
    assert_eq!(unread_of(&bad, 10), vec![(1, 10)]);
}

#[test]
fn a_read_past_the_end_covers_nothing_and_one_into_it_is_clipped() {
    let past = [read(DIFF, Some(50), Some(10))];
    assert_eq!(unread_of(&past, 40), vec![(1, 40)]);

    let into = [read(DIFF, Some(31), Some(100))];
    assert_eq!(unread_of(&into, 40), vec![(1, 30)]);
}

#[test]
fn a_read_without_a_limit_reads_two_thousand_lines() {
    let reads = [read(DIFF, None, None)];

    assert_eq!(unread_of(&reads, 5000), vec![(2001, 5000)]);
}

#[test]
fn offset_zero_starts_at_the_first_line_and_limit_zero_reads_nothing() {
    let zero = [read(DIFF, Some(0), Some(10))];
    assert_eq!(unread_of(&zero, 20), vec![(11, 20)]);

    let none = [read(DIFF, Some(5), Some(0))];
    assert_eq!(unread_of(&none, 20), vec![(1, 20)]);
}

#[test]
fn a_huge_offset_or_limit_does_not_overflow() {
    let reads = [
        read(DIFF, Some(usize::MAX), Some(usize::MAX)),
        read(DIFF, Some(3), Some(usize::MAX)),
    ];

    assert_eq!(unread_of(&reads, 10), vec![(1, 2)]);
}

#[test]
fn a_relative_a_dotted_and_an_absolute_path_under_the_work_directory_name_the_diff() {
    for path in [
        DIFF,
        "./review/diffs/lang-rust.diff",
        "/home/w/review/diffs/lang-rust.diff",
        "review//diffs/./lang-rust.diff",
        "/home//w/./review/diffs/lang-rust.diff",
    ] {
        let reads = [read(path, None, None)];
        assert!(unread_of(&reads, 10).is_empty(), "{path}");
        assert!(
            unread(
                &reads,
                &[Path::new(WORK)],
                "./review/diffs/lang-rust.diff",
                10
            )
            .is_empty(),
            "{path}"
        );
    }
}

#[test]
fn a_path_under_any_name_of_the_work_directory_names_the_diff() {
    let reads = [read("/real/w/review/diffs/lang-rust.diff", None, None)];
    let roots = [Path::new(WORK), Path::new("/real/w")];

    assert!(unread(&reads, &roots, DIFF, 10).is_empty());
    assert_eq!(unread(&reads, &roots[..1], DIFF, 10), vec![(1, 10)]);
}

#[test]
fn another_file_is_not_the_diff() {
    for path in [
        "review/diffs/lang-rust.diff.bak",
        "review/diffs/x-lang-rust.diff",
        "lang-rust.diff",
        "diffs/lang-rust.diff",
        "review/diffs/lang-rust.1.diff",
        // The pull request's own tree may hold a file of the same name:
        // a read of it is not a read of the diff the review gave.
        "pr-head/review/diffs/lang-rust.diff",
        "/home/w/pr-head/review/diffs/lang-rust.diff",
        "/elsewhere/review/diffs/lang-rust.diff",
        // A path that climbs is never resolved: it names nothing.
        "pr-head/../review/diffs/lang-rust.diff",
        "/home/w/x/../review/diffs/lang-rust.diff",
    ] {
        let reads = [read(path, None, None)];
        assert_eq!(unread_of(&reads, 10), vec![(1, 10)], "{path}");
    }
}

#[test]
fn a_work_directory_that_climbs_names_no_diff() {
    let reads = [read(DIFF, None, None)];

    assert_eq!(
        unread(&reads, &[Path::new("/home/x/../w")], DIFF, 10),
        vec![(1, 10)]
    );
}

#[test]
fn an_empty_file_is_read_whole() {
    assert!(unread_of(&[], 0).is_empty());
    assert_eq!(unread_of(&[], 3), vec![(1, 3)]);
}

#[test]
fn a_span_runs_from_the_offset_for_the_limit_clipped_to_the_file() {
    assert_eq!(span(&read(DIFF, None, None), 5000), (1, 2000));
    assert_eq!(span(&read(DIFF, Some(10), Some(5)), 100), (10, 14));
    assert_eq!(span(&read(DIFF, Some(98), Some(5)), 100), (98, 100));
}

#[test]
fn components_drop_empty_and_current_directory_parts_and_refuse_a_climb() {
    assert_eq!(components("./a//b/./c"), Some(vec!["a", "b", "c"]));
    assert_eq!(components("/a/b"), Some(vec!["a", "b"]));
    assert_eq!(components("a/../b"), None);
}
