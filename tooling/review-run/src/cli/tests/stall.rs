//! The command's loop breaker: a specialist that fails the same way in
//! the first run and in two resumes after it stalls, is named for a
//! person in a blocking finding of its own, and no later resume runs it
//! again, the review staying failed closed.

use super::*;

/// A host on which lang-rust never starts.
fn failing() -> FakeHost {
    FakeHost::new(|c| {
        if fake::agent_of(c) == "lang-rust" {
            Err("claude: not found".to_owned())
        } else {
            fake::either(c)
        }
    })
}

/// Resumes in a fresh work directory from the earlier review directory
/// `earlier`, lang-rust failing, `extra` after; returns the host, the
/// outputs and the progress, and the new review directory kept in
/// `keep`.
fn resumed(earlier: &Path, keep: &Path, extra: &[&str]) -> (FakeHost, String, String) {
    let dir = workdir();
    let host = failing();
    let gaps = earlier.join("gaps.json");
    let mut list = vec![
        "--head-sha",
        HEAD,
        "--breaker-fraction=1",
        "--only",
        gaps.to_str().unwrap(),
        "--from",
        earlier.to_str().unwrap(),
    ];
    list.extend_from_slice(extra);
    let (code, out, err) = run_with(&host, dir.path(), &list);
    assert_eq!(code, EXIT_OK, "{err}");
    copy_tree(&dir.path().join("review"), keep);
    (host, out, err)
}

#[test]
fn a_gap_that_recurs_in_two_resumes_stalls_and_is_not_run_again() {
    let runs = TempDir::new().unwrap();
    let dir = |i: usize| runs.path().join(format!("run{i}"));
    first_review(&dir(0));
    let first = read(&dir(0), "stalls.json");
    assert_eq!(first[0]["times"], 1);

    let (_, out1, _) = resumed(&dir(0), &dir(1), &[]);
    let (_, out2, _) = resumed(&dir(1), &dir(2), &[]);
    let (host, out3, err3) = resumed(&dir(2), &dir(3), &[]);

    assert!(out1.contains("\nstalled=0\n"), "{out1}");
    assert!(out2.contains("\nstalled=1\n"), "{out2}");
    let outcome = read(&dir(2), "outcome.json");
    assert!(
        outcome["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["body"].as_str().unwrap().starts_with(
                "stalled: lang-rust failed 3 times the same way; needs a person: silent:"
            )),
        "{outcome}"
    );
    // The resume after it runs the generalist that weighs it, not it.
    assert_eq!(host.agents(), ["review-code"], "{err3}");
    assert!(out3.starts_with("verdict=request_changes\n"), "{out3}");
    assert!(out3.contains("\nstalled=1\n"), "{out3}");
    assert!(
        err3.contains("review-run: lang-rust: not run again: stalled: lang-rust failed 3 times"),
        "{err3}"
    );
    let gaps = read(&dir(3), "gaps.json");
    assert!(
        gaps[0]["reason"]
            .as_str()
            .unwrap()
            .starts_with("stalled: lang-rust failed 3 times"),
        "{gaps}"
    );
    assert_eq!(read(&dir(3), "stalls.json"), read(&dir(2), "stalls.json"));
}

#[test]
fn more_resumes_allowed_stall_later() {
    let runs = TempDir::new().unwrap();
    let dir = |i: usize| runs.path().join(format!("run{i}"));
    first_review(&dir(0));

    let (_, _, _) = resumed(&dir(0), &dir(1), &["--max-resumes=3"]);
    let (_, out2, _) = resumed(&dir(1), &dir(2), &["--max-resumes=3"]);
    let (_, out3, _) = resumed(&dir(2), &dir(3), &["--max-resumes=3"]);

    assert!(out2.contains("\nstalled=0\n"), "{out2}");
    assert!(out3.contains("\nstalled=1\n"), "{out3}");
}

#[test]
fn an_earlier_stalls_file_that_does_not_read_fails_the_resume() {
    let runs = TempDir::new().unwrap();
    let earlier = runs.path().join("run0");
    first_review(&earlier);
    fs::write(earlier.join("stalls.json"), "{").unwrap();
    let dir = workdir();
    let host = failing();
    let gaps = earlier.join("gaps.json");

    let (code, _, err) = run_with(
        &host,
        dir.path(),
        &[
            "--only",
            gaps.to_str().unwrap(),
            "--from",
            earlier.to_str().unwrap(),
        ],
    );

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("stalls.json: malformed stalls"), "{err}");
    assert!(host.agents().is_empty());
}
