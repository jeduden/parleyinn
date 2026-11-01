use super::*;

/// A run of the obedient and the silent claude on the checkout `dir`,
/// written under `out`: their scorecards.
fn two_runs(dir: &Path, out: &Path) -> (PathBuf, PathBuf) {
    sound_cases(dir);
    let mut host = FakeHost::new(dir);
    host.claude = obedient_claude();
    let good = out.join("good");
    let (code, _, err) = run_cli(
        &host,
        &[
            "run",
            "--checkout",
            &text(dir),
            "--component",
            "lang-rust",
            "--out",
            &text(&good),
        ],
    );
    assert_eq!(code, EXIT_OK, "{err}");
    host.claude = silent_claude();
    let bad = out.join("bad");
    let (code, _, _) = run_cli(
        &host,
        &[
            "run",
            "--checkout",
            &text(dir),
            "--component",
            "lang-rust",
            "--out",
            &text(&bad),
            "--model",
            "claude-opus-5-5",
            "--repetitions",
            "1",
        ],
    );
    assert_eq!(code, EXIT_FAILURE);

    (good.join(SCORECARD_JSON), bad.join(SCORECARD_JSON))
}

#[test]
fn publish_and_compare_read_what_a_run_wrote() {
    let dir = checkout();
    let out = TempDir::new().unwrap();
    let (good, bad) = two_runs(dir.path(), out.path());
    let host = FakeHost::new(dir.path());
    let published = out.path().join("scorecard-v1.txt");

    let (code, out, _) = run_cli(
        &host,
        &[
            "publish",
            "--scorecard",
            &text(&good),
            "--into",
            &text(&published),
        ],
    );
    assert_eq!(code, EXIT_OK);
    assert_eq!(
        out,
        format!("published 1 components into {}\n", published.display())
    );
    assert!(
        fs::read_to_string(&published)
            .unwrap()
            .contains("component=lang-rust model=haiku")
    );

    let (code, out, _) = run_cli(
        &host,
        &[
            "compare",
            "--published",
            &text(&published),
            "--scorecard",
            &text(&bad),
        ],
    );
    assert_eq!(code, EXIT_OK);
    assert!(
        out.contains("| lang-rust | haiku (claude-haiku-5-5): 3/3 cases"),
        "{out}"
    );
    assert!(
        out.contains("| worse | passes 0 of 3 cases, the published model 3 |"),
        "{out}"
    );

    let (code, _, _) = run_cli(
        &host,
        &[
            "publish",
            "--scorecard",
            &text(&bad),
            "--into",
            &text(&published),
        ],
    );
    assert_eq!(code, EXIT_OK);
    assert!(
        fs::read_to_string(&published)
            .unwrap()
            .contains("model=claude-opus-5-5")
    );
}

#[test]
fn compare_and_publish_name_what_they_cannot_read() {
    let dir = checkout();
    let out = TempDir::new().unwrap();
    let (good, _) = two_runs(dir.path(), out.path());
    let host = FakeHost::new(dir.path());
    let not_a_scorecard = out.path().join("plan.json");
    fs::write(&not_a_scorecard, "[]").unwrap();

    let (code, _, err) = run_cli(
        &host,
        &[
            "compare",
            "--published",
            "/none.txt",
            "--scorecard",
            &text(&good),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-eval: read /none.txt: "), "{err}");

    let (code, _, err) = run_cli(
        &host,
        &[
            "compare",
            "--published",
            &text(&not_a_scorecard),
            "--scorecard",
            &text(&good),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("does not open with"), "{err}");

    let (code, _, err) = run_cli(
        &host,
        &[
            "publish",
            "--scorecard",
            "/none.json",
            "--into",
            &text(&out.path().join("s.txt")),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-eval: read /none.json: "), "{err}");

    let (code, _, err) = run_cli(
        &host,
        &[
            "publish",
            "--scorecard",
            &text(&good),
            "--into",
            &text(&not_a_scorecard),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("does not open with"), "{err}");

    let (code, _, err) = run_cli(
        &host,
        &[
            "publish",
            "--scorecard",
            &text(&good),
            "--into",
            &text(out.path()),
        ],
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("review-eval: read "), "{err}");
}
