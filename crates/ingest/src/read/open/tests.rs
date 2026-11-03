use std::io::Cursor;

use record::digest::Digest;
use store::Position;
use testkit::TempDir;

use super::*;

const MIB: usize = 1 << 20;

/// `n` JSON lines of `len` bytes each, a line break after each.
fn lines_of(n: usize, len: usize) -> Vec<u8> {
    let mut one = format!("{{\"x\":\"{}", "y".repeat(len - 8)).into_bytes();
    one.extend_from_slice(b"\"}\n");
    one.truncate(len);
    one.push(b'\n');
    one.repeat(n)
}

fn written(dir: &TempDir, bytes: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, bytes).unwrap();
    path
}

/// The source whose ingest consumed the first `offset` of `bytes`.
fn source(bytes: &[u8], offset: u64, purged: Vec<Span>) -> Source {
    let mut hash = Zeroed::new(&purged);
    hash.update(&bytes[..offset as usize]);
    let position = Position {
        offset,
        prefix: hash.digest(),
        lines: offset / (MIB as u64 + 1),
        seen: bytes.len() as u64,
    };

    Source {
        id: 1,
        path: String::new(),
        generation: 1,
        directory: String::new(),
        position,
        purged,
        held_since: None,
        lost: false,
        marked_at: 0,
    }
}

#[test]
fn a_source_that_fits_one_window_is_read_whole_whatever_its_marker() {
    let dir = TempDir::new().unwrap();
    let bytes = lines_of(3, 20);
    let path = written(&dir, &bytes);

    let fresh = open(&path, None).unwrap();
    let marked = open(&path, Some(&source(&bytes, 21, Vec::new()))).unwrap();

    assert_eq!(fresh, marked);
    assert_eq!(fresh.window.bytes, bytes);
    assert_eq!((fresh.window.start, fresh.seen), (0, 63));
}

#[test]
fn a_source_past_64_mib_resumes_at_its_marker_while_the_prefix_holds() {
    let dir = TempDir::new().unwrap();
    let bytes = lines_of(66, MIB);
    let path = written(&dir, &bytes);
    let at = (MIB as u64 + 1) * 64;
    let known = source(&bytes, at, Vec::new());

    let first = open(&path, None).unwrap();
    let rest = open(&path, Some(&known)).unwrap();
    let mut changed = known.clone();
    changed.position.prefix = Digest::of(&[b"other"]);
    let rewritten = open(&path, Some(&changed)).unwrap();
    let at_start = open(&path, Some(&source(&bytes, 0, Vec::new()))).unwrap();

    assert_eq!(first.window.bytes.len(), (MIB + 1) * 54);
    assert!(first.window.more);
    assert_eq!(first.seen, bytes.len() as u64);
    assert_eq!(rest.window.start, at);
    assert_eq!(rest.window.bytes.len(), (MIB + 1) * 2);
    assert!(!rest.window.more);
    assert_eq!(rest.seen, bytes.len() as u64);
    assert_eq!(rewritten.window.start, 0);
    assert_eq!(at_start.window.start, 0);
}

#[test]
fn a_marker_with_a_purged_line_zeroed_still_vouches_for_its_prefix() {
    let dir = TempDir::new().unwrap();
    let bytes = lines_of(66, MIB);
    let at = (MIB as u64 + 1) * 64;
    let purged = vec![Span {
        offset: 0,
        length: MIB as u64,
    }];
    let known = source(&bytes, at, purged);
    let mut erased = bytes.clone();
    erased[..MIB].fill(0);
    let path = written(&dir, &erased);

    let got = open(&path, Some(&known)).unwrap();

    assert_eq!(got.window.start, at);
}

#[test]
fn a_window_that_ends_at_a_refused_line_is_seen_past_it() {
    let dir = TempDir::new().unwrap();
    let mut bytes = lines_of(1, 20);
    bytes.extend(vec![b'z'; (4 << 20) + 1]);
    bytes.push(b'\n');
    let path = written(&dir, &bytes);

    let got = open(&path, None).unwrap();

    assert_eq!(got.window.bytes.len(), 21);
    assert_eq!(got.window.refused.map(|r| r.offset), Some(21));
    assert_eq!(got.seen, bytes.len() as u64);
}

#[test]
fn a_transcript_that_is_no_regular_file_is_not_opened() {
    let dir = TempDir::new().unwrap();

    assert!(open(dir.path(), None).is_err());
    assert!(!continues(dir.path(), &source(b"a\n", 0, Vec::new())));
}

#[test]
fn a_prefix_is_held_while_the_file_begins_with_it() {
    let dir = TempDir::new().unwrap();
    let bytes = lines_of(2, 20);
    let path = written(&dir, &bytes);
    let known = source(&bytes, 21, Vec::new());

    assert!(continues(&path, &known));
    assert!(holds(Cursor::new(bytes.clone()), &known).unwrap());
    assert!(!holds(Cursor::new(b"short".to_vec()), &known).unwrap());
    assert!(!holds(Cursor::new(lines_of(2, 21)), &known).unwrap());
    let other = written(&dir, b"other bytes entirely, longer than the prefix\n");
    assert!(!continues(&other, &known));
}

#[test]
fn a_running_hash_takes_the_bytes_after_its_prefix_and_zeroes_purged_lines() {
    let bytes = b"aaaa\nbbbb\ncccc\n";
    let spans = [Span {
        offset: 5,
        length: 4,
    }];

    let mut hash = zeroed(Cursor::new(bytes.to_vec()), 10, &spans)
        .unwrap()
        .unwrap();
    hash.update(&bytes[10..]);
    let mut whole = Zeroed::new(&spans);
    whole.update(bytes);

    assert_eq!(hash.digest(), whole.digest());
    assert!(
        zeroed(Cursor::new(b"ab".to_vec()), 3, &[])
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_failing_read_fails_the_running_hash() {
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    assert!(zeroed(Failing, 5, &[]).is_err());
}

#[test]
fn each_line_up_to_an_offset_is_given_a_window_at_a_time() {
    let dir = TempDir::new().unwrap();
    let mut bytes = lines_of(65, MIB);
    bytes.extend_from_slice(b"last\n");
    let path = written(&dir, &bytes);
    let (mut all, mut early) = (0, Vec::new());

    for_each_line(&path, u64::MAX, |_| all += 1).unwrap();
    for_each_line(&path, 12, |t| early.push(t.to_owned())).unwrap();
    let mut middle = Vec::new();
    for_each_line_between(&path, (MIB + 1) as u64, 3 * (MIB + 1) as u64, |t| {
        middle.push(t.len())
    })
    .unwrap();
    let lined = written(&dir, b"a\nbb\nccc\n");
    for_each_line(&lined, 5, |t| early.push(t.to_owned())).unwrap();

    assert_eq!(all, 66);
    assert_eq!(early, ["a", "bb"]);
    assert_eq!(middle, [MIB, MIB]);
    assert!(for_each_line(dir.path(), 0, |_| ()).is_err());
}

#[test]
fn a_file_prefix_is_hashed_a_chunk_at_a_time_with_purged_spans_zeroed() {
    let dir = TempDir::new().unwrap();
    let bytes = lines_of(2, 20);
    let path = written(&dir, &bytes);
    let spans = [Span {
        offset: 4,
        length: 8,
    }];
    let mut expected = Zeroed::new(&spans);
    expected.update(&bytes);

    assert_eq!(
        prefix(&path, bytes.len() as u64, &spans).unwrap(),
        Some(expected.digest())
    );
    assert_eq!(prefix(&path, bytes.len() as u64 + 1, &spans).unwrap(), None);
}

#[test]
fn a_line_over_the_ceiling_is_stepped_over_and_a_line_not_utf_8_read_lossily() {
    let dir = TempDir::new().unwrap();
    let mut bytes = b"a\n".to_vec();
    bytes.extend(vec![b'z'; (4 << 20) + 5]);
    bytes.extend_from_slice(b"\nb\n\xff\n");
    let path = written(&dir, &bytes);
    let mut seen = Vec::new();

    for_each_line(&path, u64::MAX, |t| seen.push(t.to_owned())).unwrap();

    assert_eq!(seen, ["a", "b", "\u{fffd}"]);
}

#[test]
fn a_refused_line_at_an_offset_is_measured_by_seeking_to_it() {
    let dir = TempDir::new().unwrap();
    let mut bytes = b"a\n".to_vec();
    bytes.extend(vec![b'z'; (4 << 20) + 1]);
    bytes.push(b'\n');
    bytes.extend_from_slice(b"b\n");
    let path = written(&dir, &bytes);

    let over = refused_at(&path, 2).unwrap();
    let fits = refused_at(&path, 0).unwrap();
    let ends = refused_at(&path, bytes.len() as u64).unwrap();

    assert_eq!(
        over,
        Some(Refused {
            offset: 2,
            length: (4 << 20) + 1
        })
    );
    assert_eq!((fits, ends), (None, None));
    assert!(refused_at(dir.path(), 0).is_err());
}

#[test]
fn only_a_source_longer_than_one_window_needs_its_marker() {
    let dir = TempDir::new().unwrap();
    let small = written(&dir, &lines_of(2, 20));
    let big = dir.path().join("big.jsonl");
    std::fs::write(&big, lines_of(65, MIB)).unwrap();

    assert!(!needs_marker(&small));
    assert!(needs_marker(&big));
    assert!(!needs_marker(&dir.path().join("none")));
}
