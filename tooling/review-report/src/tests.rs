use std::cell::RefCell;
use std::fs;

use testkit::TempDir;

use super::*;

#[test]
fn a_review_that_could_not_run_still_gets_its_trace_and_page() {
    let dir = TempDir::new().unwrap();

    let problems = write(dir.path(), Some("plan: review/dispatch.json does not read"));

    assert!(problems.is_empty(), "{problems:?}");
    let trace = fs::read_to_string(dir.path().join(TRACE)).unwrap();
    let header: serde_json::Value = serde_json::from_str(trace.lines().next().unwrap()).unwrap();
    assert_eq!(header["trace"]["verdict"], "failed");
    assert_eq!(
        header["trace"]["failure"],
        "plan: review/dispatch.json does not read"
    );
    let page = fs::read_to_string(dir.path().join(REPORT)).unwrap();
    assert!(page.contains("The review could not run: plan: review/dispatch.json does not read"));
}

#[test]
fn each_file_not_written_is_named_and_counted() {
    let dir = TempDir::new().unwrap();
    // A file where the review directory belongs: neither can be written.
    fs::write(dir.path().join("review"), "").unwrap();
    let lines = RefCell::new(Vec::new());

    write_reported(dir.path(), None, &|line| {
        lines.borrow_mut().push(line.to_owned())
    });

    let lines = lines.into_inner();
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert!(lines[0].starts_with("report: review/trace.jsonl not written: "));
    assert!(lines[1].starts_with("report: review/report.html not written: "));
    assert_eq!(lines[2], "report: 2 of 2 report files not written");
}

#[test]
fn a_review_whose_files_are_written_tells_nothing() {
    let dir = TempDir::new().unwrap();
    let lines = RefCell::new(Vec::new());

    write_reported(dir.path(), None, &|line| {
        lines.borrow_mut().push(line.to_owned())
    });

    assert!(lines.into_inner().is_empty());
    assert!(dir.path().join(REPORT).is_file());
}
