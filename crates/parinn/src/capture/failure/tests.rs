//! The capture failures an MCP server counts instead of passing over
//! (REC-13, I6).

use key::random::OsEntropy;
use redaction::Redactor;

use super::tests::{Fixture, HARNESS, line};
use super::*;

#[test]
fn a_sibling_transcript_a_live_ingest_could_not_read_is_counted() {
    let f = Fixture::new();
    let other = f.projects().join("other.jsonl");
    std::fs::write(&other, line(1, None)).unwrap();
    let mut ingested = ingest::ingest_path(
        &f.home,
        &other,
        &Redactor::new(&[]).unwrap().0,
        &mut OsEntropy,
    )
    .unwrap();
    ingested.unread_sources = vec![ingest::UnreadSource {
        name: "sibling".to_owned(),
        why: "not a regular file".to_owned(),
    }];

    f.running(None, None).audit_ingested(&ingested);

    assert_eq!(f.counted(), ["ingest_source_unread"]);
}

#[test]
fn a_transcript_that_cannot_be_scanned_is_audited_once_and_read_again() {
    let f = Fixture::new();
    std::fs::write(f.main(), line(1, Some("toolu_m"))).unwrap();
    let redactor = Redactor::new(&[]).unwrap().0;
    ingest::ingest_path(&f.home, &f.main(), &redactor, &mut OsEntropy).unwrap();
    std::fs::remove_file(f.main()).unwrap();
    std::fs::create_dir(f.main()).unwrap();
    let mut running = f.running(Some(HARNESS), None);
    running.transcripts.insert(f.main(), 0);

    running.read_line_fields(&f.main());
    running.read_line_fields(&f.main());

    assert_eq!(f.counted(), ["live_ingest_failed"]);
    assert_eq!(running.transcripts.get(&f.main()), Some(&0));

    std::fs::remove_dir(f.main()).unwrap();
    std::fs::write(f.main(), line(1, Some("toolu_m"))).unwrap();
    running.read_line_fields(&f.main());
    assert!(running.unscanned.is_empty());
    assert_eq!(f.shared.runs_of("toolu_m").len(), 1);
}

#[test]
fn a_transcript_that_cannot_be_read_to_its_end_is_left_short() {
    let f = Fixture::new();
    std::fs::create_dir(f.main()).unwrap();
    let home = f.home.clone();
    let names = remainders(&home, [f.main()].iter());
    assert_eq!(names.len(), 1);
}
