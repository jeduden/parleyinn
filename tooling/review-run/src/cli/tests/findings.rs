//! The command across reviews: each run keeps its findings, and a run
//! given the previous one's marks them new, still open or resolved, and
//! names one stuck blocking on an unchanged file for a stakeholder.

use super::*;

/// A generalist that keeps one blocking finding on `src/a.rs`.
fn blocking(c: &review_eval::host::Command) -> Result<review_eval::host::Output, String> {
    if fake::prompt_of(c).contains("a specialist") {
        return fake::specialist(c);
    }
    let text = "src/a.rs:3 — blocking — a long line hides a bug — lang-rust\nnot sound.";
    fake::answer(c, &[text], text)
}

/// Reviews `dir` with `claude` answering as `script`, given the
/// previous review's findings in `previous`, when there are.
fn reviewed(
    dir: &Path,
    previous: Option<&Path>,
    script: fn(&review_eval::host::Command) -> Result<review_eval::host::Output, String>,
) -> (String, String) {
    let host = FakeHost::new(script);
    let at = previous.map(|p| format!("--previous={}", p.display()));
    let mut extra = vec!["--head-sha", HEAD];
    extra.extend(at.as_deref());
    let (code, out, err) = run_with(&host, dir, &extra);
    assert_eq!(code, EXIT_OK, "{err}");
    (out, err)
}

#[test]
fn a_review_keeps_its_findings_and_the_next_marks_them() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join("pr-head/src")).unwrap();
    fs::write(dir.path().join("pr-head/src/a.rs"), "fn a() {}\n").unwrap();
    reviewed(dir.path(), None, fake::either);
    let kept = read(dir.path(), crate::findings::FINDINGS);
    assert_eq!(kept[0]["agent"], "review-code");
    assert_eq!(kept[0]["seen_count"], 1);
    assert_eq!(kept[0]["first_seen_run"], HEAD);
    let previous = TempDir::new().unwrap();
    let first = previous.path().join("findings.json");
    fs::copy(dir.path().join(crate::findings::FINDINGS), &first).unwrap();

    reviewed(dir.path(), Some(&first), blocking);

    let outcome = read(dir.path(), OUTCOME);
    let summary = outcome["summary"].as_str().unwrap();
    assert!(
        summary.ends_with(
            " Since the previous review: 1 new, 0 still open and 1 resolved (src/a.rs:3)."
        ),
        "{summary}"
    );
    let notes = fs::read_to_string(dir.path().join(crate::findings::FINDINGS_NOTES)).unwrap();
    assert!(notes.contains("## Resolved\n\n- `src/a.rs:3`"), "{notes}");
}

#[test]
fn a_blocking_finding_kept_three_reviews_on_an_unchanged_file_needs_a_stakeholder() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join("pr-head/src")).unwrap();
    fs::write(dir.path().join("pr-head/src/a.rs"), "fn a() {}\n").unwrap();
    let previous = TempDir::new().unwrap();
    let file = previous.path().join("findings.json");
    let mut outs = Vec::new();
    for at in 0..3 {
        let (out, _) = reviewed(dir.path(), (at > 0).then_some(file.as_path()), blocking);
        outs.push(out);
        fs::copy(dir.path().join(crate::findings::FINDINGS), &file).unwrap();
    }

    assert!(outs[1].contains("\nstalled=0\n"), "{}", outs[1]);
    assert!(outs[2].contains("\nstalled=1\n"), "{}", outs[2]);
    let outcome = read(dir.path(), OUTCOME);
    assert_eq!(outcome["verdict"], "request_changes");
    let bodies: Vec<&str> = outcome["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["body"].as_str().unwrap())
        .collect();
    assert!(
        bodies
            .iter()
            .any(|b| b.starts_with("stalled: needs a stakeholder ruling: review-code")),
        "{bodies:?}"
    );
}

#[test]
fn a_previous_findings_file_that_does_not_read_fails_before_any_run() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);
    let bad = dir.path().join("bad.json");
    fs::write(&bad, "{").unwrap();

    for path in [bad.clone(), dir.path().join("missing.json")] {
        let at = format!("--previous={}", path.display());
        let (code, _, err) = run_with(&host, dir.path(), &[at.as_str()]);
        assert_eq!(code, EXIT_FAILURE, "{err}");
        assert!(err.contains(&path.display().to_string()), "{err}");
    }
    assert!(host.agents().is_empty());
}

#[test]
fn a_review_whose_findings_or_stalls_cannot_be_written_fails() {
    fails_writing(crate::findings::FINDINGS);
    fails_writing(crate::findings::FINDINGS_NOTES);
    fails_writing(crate::stall::STALLS);
}
