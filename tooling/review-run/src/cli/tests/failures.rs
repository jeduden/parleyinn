//! The failures of a review that nothing else reaches: an SRS that does
//! not read, a tree that cannot be scrubbed, a resume that cannot copy
//! what it keeps.

use super::*;

#[test]
fn an_srs_that_does_not_read_fails_the_review_before_any_agent_runs() {
    let dir = workdir();
    let srs = dir.path().join("docs/srs");
    fs::create_dir_all(&srs).unwrap();
    fs::write(
        srs.join("10-x.md"),
        "| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n| eng-1 | P0 | Bad. | T | I2 |\n",
    )
    .unwrap();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    assert!(out.starts_with("failed="), "{out}");
    assert!(err.contains("malformed requirement id"), "{err}");
    assert!(host.agents().is_empty());
}

#[test]
fn an_agent_output_that_cannot_be_scrubbed_fails_the_review_naming_it() {
    let dir = workdir();
    let looped = dir.path().join("review/logs/loop");
    let host = FakeHost::new(move |c| {
        let _ = std::os::unix::fs::symlink("loop", &looped);
        fake::either(c)
    });

    let (code, _, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("review-run: read ") && err.contains("loop"),
        "{err}"
    );
}

#[test]
fn a_resume_that_cannot_copy_what_it_keeps_fails_naming_the_file() {
    let earlier = TempDir::new().unwrap();
    first_review(earlier.path());
    let dir = workdir();
    fs::create_dir_all(dir.path().join("review/reports/test-engineer-unit.md")).unwrap();
    let host = FakeHost::new(fake::either);
    let gaps = earlier.path().join("gaps.json");

    let (code, _, err) = run_with(
        &host,
        dir.path(),
        &[
            "--head-sha",
            HEAD,
            "--only",
            gaps.to_str().unwrap(),
            "--from",
            earlier.path().to_str().unwrap(),
        ],
    );

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("review-run: copy "), "{err}");
    assert!(host.agents().is_empty());
}

#[test]
fn a_kept_log_that_is_not_utf8_is_told_as_the_resume_goes() {
    use std::os::unix::ffi::OsStrExt;
    let earlier = TempDir::new().unwrap();
    first_review(earlier.path());
    let name = std::ffi::OsStr::from_bytes(b"test-engineer-unit.\xff.jsonl");
    fs::write(earlier.path().join("logs").join(name), "x").unwrap();

    let (_, _, err) = resume_from(earlier.path(), HEAD, &|_| {});

    assert!(
        err.contains("test-engineer-unit: log test-engineer-unit.") && err.contains("not UTF-8"),
        "{err}"
    );
}
