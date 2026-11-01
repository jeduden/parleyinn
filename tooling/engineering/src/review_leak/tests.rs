use super::*;
use crate::tests::checkout;

use testkit::TempDir;

fn repo() -> Checkout {
    Checkout::new(&testkit::repo_root())
}

#[test]
fn the_checkout_s_review_run_leaves_a_leaked_token_in_nothing_it_writes_or_prints() {
    let work = TempDir::new().unwrap();

    let problems = review_run_scrubs_what_it_writes(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn the_checkout_s_review_run_fails_closed_on_a_leaked_token() {
    let work = TempDir::new().unwrap();

    let problems = review_run_fails_closed_on_a_leak(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn the_checkout_s_review_run_refuses_every_agent_the_staged_directory_by_name() {
    let work = TempDir::new().unwrap();

    let problems = review_run_refuses_the_credential(&repo(), work.path());

    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn an_agent_not_refused_the_staged_directory_by_name_is_named() {
    let dir = Path::new("/run/cred");
    let refused = Command::new(
        "claude",
        &[
            "--agent",
            "lang-rust",
            "--disallowedTools",
            "Bash,Read(//run/cred/**),Read(//run/cred)",
        ],
        Path::new("/w"),
    );
    let open = Command::new(
        "claude",
        &[
            "--agent",
            "review-code",
            "--disallowedTools",
            "Bash,Read(//run/cred/x)",
        ],
        Path::new("/w"),
    );

    assert!(confinement_problems(&[refused.clone()], dir).is_empty());
    assert_eq!(
        confinement_problems(&[refused, open], dir),
        [
            "agent review-code is not refused the staged credential's directory /run/cred by name: its --disallowedTools lacks Read(//run/cred/**) or Read(//run/cred)"
        ]
    );
    assert_eq!(
        confinement_problems(&[], dir),
        ["review-run started no agent, so none was shown refused the staged credential"]
    );
}

#[test]
fn a_token_left_in_a_file_or_printed_and_no_redaction_are_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review/logs")).unwrap();
    fs::write(work.path().join("review/logs/a.jsonl"), TOKEN).unwrap();

    let problems = scrub_problems(work.path(), &format!("failed={TOKEN}\n"));

    assert_eq!(
        problems,
        [
            "review-run left the token in review/logs/a.jsonl",
            "review-run printed the token",
            "no file under review/ holds the token redacted, so the leak was never scrubbed",
        ]
    );
}

#[test]
fn a_file_under_review_that_does_not_read_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review")).unwrap();
    fs::write(work.path().join("review/a.md"), REDACTED).unwrap();
    // A link to nothing is a file that does not read, even for root.
    std::os::unix::fs::symlink("/nonexistent/x", work.path().join("review/b.md")).unwrap();

    let problems = scrub_problems(work.path(), "");

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with(
            "review-run left review/b.md unreadable, so it was never shown scrubbed: "
        ),
        "{problems:?}"
    );
}

#[test]
fn a_review_that_approves_a_leak_and_exits_as_decided_is_named() {
    let work = TempDir::new().unwrap();
    fs::create_dir_all(work.path().join("review")).unwrap();
    fs::write(
        work.path().join("review/outcome.json"),
        r#"{"verdict": "approve", "summary": "s", "findings": []}"#,
    )
    .unwrap();
    let ran = Ran {
        code: review_run::cli::EXIT_OK,
        printed: "verdict=approve\n".to_owned(),
        commands: Vec::new(),
    };

    let problems = fail_closed_problems(work.path(), &ran);

    assert_eq!(
        problems,
        [
            "review-run exited 0 on a leak, so the workflow would post what it decided",
            "review-run printed a verdict on a leak",
            "the review does not request changes on a leak: approve",
            "no blocking finding names the leak",
        ]
    );
    let missing = TempDir::new().unwrap();
    assert_eq!(
        fail_closed_problems(missing.path(), &ran)[2..],
        ["review-run wrote no review outcome that reads"]
    );
}

#[test]
fn a_checkout_without_the_definitions_is_named() {
    let (_dir, c) = checkout(&[]);
    let work = TempDir::new().unwrap();

    for problems in [
        review_run_scrubs_what_it_writes(&c, work.path()),
        review_run_fails_closed_on_a_leak(&c, work.path()),
    ] {
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].starts_with("read .claude/agents/lang-rust.md: "),
            "{problems:?}"
        );
    }
}

#[test]
fn a_credential_that_cannot_be_staged_is_named() {
    let work = TempDir::new().unwrap();
    fs::write(work.path().join(CREDENTIAL), "a file").unwrap();

    let err = run_leaking(&repo(), work.path()).err().unwrap();

    assert!(err.starts_with("make "), "{err}");
}

#[test]
fn the_stand_in_names_a_run_without_its_credential_or_log() {
    let mut command = Command::new("claude", &["--agent", "lang-rust"], Path::new("/w"));

    assert_eq!(
        LeakHost::default().run(&command).unwrap_err(),
        "the run was pointed at no credential"
    );
    let work = TempDir::new().unwrap();
    review_eval::credential::stage(work.path(), TOKEN).unwrap();
    command.env.push((
        review_eval::credential::CONFIG_VAR.to_owned(),
        work.path().display().to_string(),
    ));
    assert_eq!(
        LeakHost::default().run(&command).unwrap_err(),
        "the run logs to no file"
    );
    command.stdout_to = Some(work.path().join("missing/log.jsonl"));
    assert!(
        LeakHost::default()
            .run(&command)
            .unwrap_err()
            .starts_with("write ")
    );
}
