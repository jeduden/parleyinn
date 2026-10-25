use super::*;
use store::{Seq, WriterId};

fn segment(n: u8, first: u64, last: u64, version: u32) -> Segment {
    Segment {
        writer: WriterId::derive(&[n; 32]),
        first: Seq::new(first).unwrap(),
        last: Seq::new(last).unwrap(),
        format_version: version,
        closed: false,
    }
}

#[test]
fn every_segment_closed_and_none_newer_shows_no_line() {
    let status = SegmentsStatus {
        open: Vec::new(),
        newer: Vec::new(),
    };

    assert!(status.lines().is_empty());
    assert_eq!(status.json(), json!({"open": [], "newer": []}));
}

#[test]
fn open_and_newer_segments_are_each_named() {
    let (a, b) = (segment(1, 1, 2, 1), segment(2, 4, 5, 2));
    let status = SegmentsStatus {
        open: vec![a.clone(), segment(3, 7, 7, 1)],
        newer: vec![b.clone()],
    };

    assert_eq!(
        status.lines(),
        [
            format!(
                "writer chains ended without a closed segment: {} (seq 1 to 2), {} (seq 7 to 7)",
                a.writer,
                WriterId::derive(&[3; 32])
            ),
            format!("refused: {}", StoreError::NewerSegment(b.clone())),
        ]
    );
    assert_eq!(
        status.json()["newer"],
        json!([{"writer": b.writer.to_string(), "first": 4, "last": 5, "format_version": 2}])
    );
    assert_eq!(
        open_json(std::slice::from_ref(&a)),
        json!([{"writer": a.writer.to_string(), "first": 1, "last": 2}])
    );
}
