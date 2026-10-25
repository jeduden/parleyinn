use std::error::Error as _;
use std::io;

use super::Error;
use crate::CommitId;

#[test]
fn an_io_error_keeps_its_cause_and_names_the_kind_of_file() {
    let e = Error::io("a loose object")(io::Error::from(io::ErrorKind::PermissionDenied));
    assert_eq!(e.to_string(), "cannot read a loose object");
    let source = e.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
}

#[test]
fn every_message_is_lowercase_without_trailing_punctuation() {
    let id = CommitId::parse(&"a".repeat(40)).unwrap();
    let all = [
        Error::io("a reference")(io::Error::from(io::ErrorKind::NotFound)),
        Error::Corrupt("a pack index"),
        Error::Missing(id),
        Error::UnsupportedObjectFormat,
        Error::Unsupported("alternates"),
        Error::Deadline,
        Error::TooLarge("an object"),
    ];
    for e in all {
        let text = e.to_string();
        assert!(!text.is_empty());
        assert_eq!(text, text.to_lowercase(), "{text}");
        assert!(
            !text.ends_with(|c: char| c.is_ascii_punctuation()),
            "{text}"
        );
    }
}

#[test]
fn each_failure_class_says_what_failed() {
    let id = CommitId::parse(&"b".repeat(40)).unwrap();
    assert_eq!(
        Error::Corrupt("a pack index").to_string(),
        "corrupt repository: a pack index"
    );
    assert_eq!(
        Error::Missing(id).to_string(),
        format!("object {} is not in the clone", "b".repeat(40))
    );
    assert_eq!(
        Error::UnsupportedObjectFormat.to_string(),
        "unsupported object format"
    );
    assert_eq!(
        Error::Unsupported("alternates").to_string(),
        "unsupported repository layout: alternates"
    );
    assert_eq!(Error::Deadline.to_string(), "deadline passed");
    assert_eq!(
        Error::TooLarge("an object").to_string(),
        "too large: an object"
    );
}

#[test]
fn only_an_io_error_has_a_cause() {
    assert!(Error::Deadline.source().is_none());
    assert!(Error::Corrupt("x").source().is_none());
}
