use super::*;

const DIFF: &str = "review/diffs/lang-rust.diff";

fn read(path: &str, offset: Option<usize>, limit: Option<usize>) -> ReadCall {
    ReadCall {
        path: path.to_owned(),
        offset,
        limit,
        failed: false,
    }
}

#[test]
fn one_read_of_the_whole_file_leaves_nothing_unread() {
    let reads = [read(DIFF, None, None)];

    assert!(unread(&reads, DIFF, 300).is_empty());
    assert!(read_whole(&reads, DIFF, 300));
}

#[test]
fn reads_in_chunks_and_overlapping_chunks_cover_the_file() {
    let chunks = [
        read(DIFF, Some(1), Some(100)),
        read(DIFF, Some(101), Some(100)),
    ];
    assert!(unread(&chunks, DIFF, 200).is_empty());

    let overlapping = [
        read(DIFF, Some(150), Some(100)),
        read(DIFF, None, Some(160)),
    ];
    assert!(unread(&overlapping, DIFF, 249).is_empty());
}

#[test]
fn a_gap_between_reads_is_unread() {
    let reads = [
        read(DIFF, Some(201), Some(100)),
        read(DIFF, Some(1), Some(100)),
    ];

    assert_eq!(unread(&reads, DIFF, 300), vec![(101, 200)]);
    assert!(!read_whole(&reads, DIFF, 300));
}

#[test]
fn a_failed_read_covers_nothing() {
    let reads = [ReadCall {
        failed: true,
        ..read(DIFF, None, None)
    }];

    assert_eq!(unread(&reads, DIFF, 10), vec![(1, 10)]);
}

#[test]
fn a_read_past_the_end_covers_nothing_and_one_into_it_is_clipped() {
    let past = [read(DIFF, Some(50), Some(10))];
    assert_eq!(unread(&past, DIFF, 40), vec![(1, 40)]);

    let into = [read(DIFF, Some(31), Some(100))];
    assert_eq!(unread(&into, DIFF, 40), vec![(1, 30)]);
}

#[test]
fn a_read_without_a_limit_reads_two_thousand_lines() {
    let reads = [read(DIFF, None, None)];

    assert_eq!(unread(&reads, DIFF, 5000), vec![(2001, 5000)]);
}

#[test]
fn offset_zero_starts_at_the_first_line_and_limit_zero_reads_nothing() {
    let zero = [read(DIFF, Some(0), Some(10))];
    assert_eq!(unread(&zero, DIFF, 20), vec![(11, 20)]);

    let none = [read(DIFF, Some(5), Some(0))];
    assert_eq!(unread(&none, DIFF, 20), vec![(1, 20)]);
}

#[test]
fn a_huge_offset_or_limit_does_not_overflow() {
    let reads = [
        read(DIFF, Some(usize::MAX), Some(usize::MAX)),
        read(DIFF, Some(3), Some(usize::MAX)),
    ];

    assert_eq!(unread(&reads, DIFF, 10), vec![(1, 2)]);
}

#[test]
fn a_relative_a_dotted_and_an_absolute_path_name_the_same_file() {
    for path in [
        DIFF,
        "./review/diffs/lang-rust.diff",
        "/home/w/review/diffs/lang-rust.diff",
        "review//diffs/./lang-rust.diff",
    ] {
        let reads = [read(path, None, None)];
        assert!(read_whole(&reads, DIFF, 10), "{path}");
        assert!(
            read_whole(&reads, "./review/diffs/lang-rust.diff", 10),
            "{path}"
        );
    }
}

#[test]
fn another_file_is_not_the_diff() {
    for path in [
        "review/diffs/lang-rust.diff.bak",
        "review/diffs/x-lang-rust.diff",
        "lang-rust.diff",
        "diffs/lang-rust.diff",
        "review/diffs/lang-rust.1.diff",
    ] {
        let reads = [read(path, None, None)];
        assert_eq!(unread(&reads, DIFF, 10), vec![(1, 10)], "{path}");
    }
}

#[test]
fn an_empty_file_is_read_whole() {
    assert!(unread(&[], DIFF, 0).is_empty());
    assert_eq!(unread(&[], DIFF, 3), vec![(1, 3)]);
}

#[test]
fn a_span_runs_from_the_offset_for_the_limit_clipped_to_the_file() {
    assert_eq!(span(&read(DIFF, None, None), 5000), (1, 2000));
    assert_eq!(span(&read(DIFF, Some(10), Some(5)), 100), (10, 14));
    assert_eq!(span(&read(DIFF, Some(98), Some(5)), 100), (98, 100));
}

#[test]
fn components_drop_empty_and_current_directory_parts() {
    assert_eq!(components("./a//b/./c"), vec!["a", "b", "c"]);
    assert_eq!(components("/a/b"), vec!["a", "b"]);
}
