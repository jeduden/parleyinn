use std::io::{self, Cursor, Read, Seek, SeekFrom};

use super::*;

fn small(line: usize, source: usize) -> Limits {
    Limits { line, source }
}

fn cut(bytes: &[u8], from: u64, limits: Limits) -> Window {
    window(Cursor::new(bytes.to_vec()), from, limits).unwrap()
}

/// `n` lines of `len` bytes each, every one ended by a line break.
fn lines_of(n: usize, len: usize) -> Vec<u8> {
    let mut one = vec![b'x'; len];
    one.push(b'\n');
    one.repeat(n)
}

#[test]
fn the_ceilings_are_4_mib_a_line_and_64_mib_a_source() {
    let limits = Limits::default();

    assert_eq!((limits.line, limits.source), (4 << 20, 64 << 20));
    assert_eq!((LINE_LIMIT, SOURCE_LIMIT), (4 << 20, 64 << 20));
}

#[test]
fn a_source_within_the_ceilings_is_one_window_of_all_its_bytes() {
    let got = cut(b"a\nbc\n", 0, Limits::default());

    assert_eq!(got.bytes, b"a\nbc\n");
    assert_eq!((got.start, got.end()), (0, 5));
    assert_eq!((got.refused, got.more), (None, false));
    assert_eq!(got.high_water, 5);
}

#[test]
fn an_empty_source_is_an_empty_window() {
    let got = cut(b"", 0, Limits::default());

    assert_eq!((got.bytes.len(), got.refused, got.more), (0, None, false));
}

#[test]
fn a_last_line_without_its_line_break_and_a_crlf_line_are_kept_as_they_are() {
    let got = cut(b"a\r\nlast", 0, Limits::default());

    assert_eq!(got.bytes, b"a\r\nlast");
    assert!(!got.more);
}

#[test]
fn a_line_of_exactly_4_mib_passes_and_one_byte_more_is_refused() {
    let mut at = vec![b'x'; LINE_LIMIT];
    at.push(b'\n');
    let mut over = vec![b'x'; LINE_LIMIT + 1];
    over.push(b'\n');

    let passes = cut(&at, 0, Limits::default());
    let refused = cut(&over, 0, Limits::default());

    assert_eq!((passes.bytes.len(), passes.refused), (LINE_LIMIT + 1, None));
    assert!(refused.bytes.is_empty());
    assert_eq!(
        refused.refused,
        Some(Refused {
            offset: 0,
            length: LINE_LIMIT as u64 + 1
        })
    );
    assert!(refused.more);
}

#[test]
fn a_refused_line_ends_the_window_before_it_and_the_lines_after_it_stay_unread() {
    let mut text = b"one\ntwo\n".to_vec();
    text.extend(vec![b'y'; 50]);
    text.extend(b"\nthree\n");

    let got = cut(&text, 0, small(10, 1000));

    assert_eq!(got.bytes, b"one\ntwo\n");
    assert_eq!(
        got.refused,
        Some(Refused {
            offset: 8,
            length: 50
        })
    );
    assert_eq!(got.end(), 8);
    assert!(got.more);
    assert!(got.refused_ended, "its line break is in the source");
}

#[test]
fn a_refused_line_is_counted_across_reads_without_being_held() {
    let mut text = b"a\n".to_vec();
    text.extend(vec![b'y'; CHUNK * 3 + 7]);
    text.extend(b"\nafter\n");

    let got = cut(&text, 0, small(100, 1 << 20));

    assert_eq!(got.bytes, b"a\n");
    assert_eq!(
        got.refused,
        Some(Refused {
            offset: 2,
            length: (CHUNK * 3 + 7) as u64
        })
    );
    assert!(got.high_water <= 100);
}

#[test]
fn a_refused_line_the_source_ends_in_is_counted_to_its_end() {
    let mut text = b"a\n".to_vec();
    text.extend(vec![b'y'; 30]);

    let got = cut(&text, 0, small(10, 1000));

    assert_eq!(
        got.refused,
        Some(Refused {
            offset: 2,
            length: 30
        })
    );
    assert_eq!(got.bytes, b"a\n");
    assert!(!got.refused_ended, "the harness may still be writing it");
    assert!(!cut(b"a\n", 0, Limits::default()).refused_ended);
}

#[test]
fn a_window_from_an_offset_reads_past_the_bytes_before_it() {
    let text = lines_of(3, 4);

    let got = cut(&text, 5, Limits::default());

    assert_eq!(got.bytes, lines_of(2, 4));
    assert_eq!((got.start, got.end()), (5, 15));
}

#[test]
fn an_offset_past_the_end_is_an_empty_window() {
    let got = cut(b"a\n", 100, Limits::default());

    assert_eq!((got.bytes.len(), got.refused, got.more), (0, None, false));
}

#[test]
fn a_window_ends_at_the_line_that_would_pass_the_source_ceiling() {
    let text = lines_of(5, 9);

    let got = cut(&text, 0, small(20, 25));

    assert_eq!(got.bytes, lines_of(2, 9));
    assert!(got.more);
    assert_eq!(got.refused, None);
    assert!(got.high_water <= 25);
}

#[test]
fn a_line_the_ceiling_cuts_inside_a_read_is_left_whole_for_the_next_window() {
    let text = lines_of(3, CHUNK / 2 + 10);
    let limit = text.len() / 3 * 2 - 5;

    let got = cut(&text, 0, small(CHUNK, limit));

    assert_eq!(got.bytes, lines_of(1, CHUNK / 2 + 10));
    assert!(got.more);
}

#[test]
fn a_source_that_fills_the_ceiling_exactly_has_no_more() {
    let text = lines_of(2, 9);

    let got = cut(&text, 0, small(20, 20));

    assert_eq!(got.bytes, text);
    assert!(!got.more);
}

#[test]
fn a_source_over_the_ceiling_is_read_across_windows_each_resumed_at_its_end() {
    let text = lines_of(11, 9);
    let (mut from, mut got, mut windows) = (0, Vec::new(), 0);

    loop {
        let w = cut(&text, from, small(20, 35));
        assert!(w.high_water <= 35);
        got.extend_from_slice(&w.bytes);
        from = w.end();
        windows += 1;
        if !w.more {
            break;
        }
    }

    assert_eq!(got, text);
    assert_eq!(windows, 4);
}

#[test]
fn a_source_of_more_than_64_mib_is_held_at_most_64_mib_at_once() {
    let mib = 1 << 20;
    let text = lines_of(70, mib - 1);
    let (mut from, mut lines, mut peak) = (0, 0, 0);

    loop {
        let w = cut(&text, from, Limits::default());
        peak = peak.max(w.high_water);
        lines += w.bytes.iter().filter(|b| **b == b'\n').count();
        from = w.end();
        if !w.more {
            break;
        }
    }

    assert_eq!(lines, 70);
    assert_eq!(from, text.len() as u64);
    assert!(
        peak <= SOURCE_LIMIT && peak > SOURCE_LIMIT - 2 * mib,
        "{peak}"
    );
}

#[test]
fn a_failing_read_or_seek_fails_the_window() {
    struct Failing(bool);
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }
    impl Seek for Failing {
        fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
            if self.0 {
                Err(io::Error::other("seek failed"))
            } else {
                Ok(0)
            }
        }
    }

    assert!(window(Failing(false), 0, Limits::default()).is_err());
    assert!(window(Failing(true), 5, Limits::default()).is_err());
}

#[test]
fn a_window_moves_to_its_start_by_a_seek_and_never_reads_the_bytes_before_it() {
    struct Counting(Cursor<Vec<u8>>, usize);
    impl Read for Counting {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let n = self.0.read(buffer)?;
            self.1 += n;
            Ok(n)
        }
    }
    impl Seek for Counting {
        fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
            self.0.seek(to)
        }
    }
    let mut source = Counting(Cursor::new(lines_of(4, 99)), 0);

    let got = window(&mut source, 300, Limits::default()).unwrap();

    assert_eq!((got.start, got.bytes.len()), (300, 100));
    assert_eq!(source.1, 100);
}

#[test]
fn a_measure_counts_the_first_line_a_piece_at_a_time() {
    let mut measure = Measure::default();

    assert!(!measure.feed(b"abc"));
    assert!(!measure.over(3));
    assert!(!measure.feed(b"de"));
    assert!(measure.over(4));
    assert!(measure.feed(b"f\nrest"));
    assert!(measure.feed(b"more"));
    assert_eq!(measure.length(), 6);
}

#[test]
fn a_transcript_window_opens_the_file_as_ingest_does() {
    let dir = testkit::TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, "a\nbb\n").unwrap();

    let got = crate::read::transcript_window(&path, 2).unwrap();

    assert_eq!((got.bytes, got.start), (b"bb\n".to_vec(), 2));
    assert!(crate::read::transcript_window(dir.path(), 0).is_err());
}
