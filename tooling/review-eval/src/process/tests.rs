use std::fs;
use std::path::Path;
use std::time::Duration;

use testkit::TempDir;

use super::*;

fn sh(script: &str, cwd: &Path) -> Command {
    Command::new("sh", &["-c", script], cwd)
}

#[test]
fn a_command_without_a_log_has_its_output_captured() {
    let dir = TempDir::new().unwrap();

    let out = ProcessHost
        .run(&sh("echo out; echo oops >&2; exit 3", dir.path()))
        .unwrap();

    assert_eq!(out.code, Some(3));
    assert_eq!(out.stdout, "out\n");
    assert_eq!(out.stderr, "oops\n");
    assert!(!out.timed_out);
}

#[test]
fn a_command_runs_in_its_directory() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("here.txt"), "here\n").unwrap();

    let out = ProcessHost.run(&sh("cat here.txt", dir.path())).unwrap();

    assert_eq!(out.stdout, "here\n");
}

#[test]
fn a_logged_command_writes_its_output_and_errors_beside_each_other() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("run.jsonl");

    let out = ProcessHost
        .run(&sh("echo line; echo err >&2", dir.path()).logging_to(&log))
        .unwrap();

    assert!(out.ok());
    assert_eq!(out.stdout, "");
    assert_eq!(fs::read_to_string(&log).unwrap(), "line\n");
    assert_eq!(
        fs::read_to_string(dir.path().join("run.stderr")).unwrap(),
        "err\n"
    );
}

#[test]
fn a_log_that_cannot_be_created_fails_the_start() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("missing").join("run.jsonl");

    let err = ProcessHost
        .run(&sh("true", dir.path()).logging_to(&log))
        .unwrap_err();

    assert!(err.starts_with("create "), "{err}");
}

#[test]
fn an_error_file_that_cannot_be_created_fails_the_start() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("run.jsonl");
    fs::create_dir(dir.path().join("run.stderr")).unwrap();

    let err = ProcessHost
        .run(&sh("true", dir.path()).logging_to(&log))
        .unwrap_err();

    assert!(err.starts_with("create "), "{err}");
}

#[test]
fn a_command_past_its_deadline_is_killed() {
    let dir = TempDir::new().unwrap();

    let out = ProcessHost
        .run(&sh("sleep 30", dir.path()).within(Duration::from_millis(200)))
        .unwrap();

    assert!(out.timed_out);
    assert!(!out.ok());
}

#[test]
fn a_program_that_does_not_exist_fails_to_start() {
    let dir = TempDir::new().unwrap();

    let err = ProcessHost
        .run(&Command::new("no-such-program-here", &[], dir.path()))
        .unwrap_err();

    assert!(err.starts_with("start no-such-program-here: "), "{err}");
}

#[test]
fn a_command_gets_its_own_variables_but_never_a_nested_sessions() {
    let dir = TempDir::new().unwrap();
    let mut command = sh(
        "echo \"$REVIEW_VAR ${CLAUDE_CODE_CHILD_SESSION:-unset}\"",
        dir.path(),
    );
    command.env = vec![
        ("REVIEW_VAR".to_owned(), "set".to_owned()),
        ("CLAUDE_CODE_CHILD_SESSION".to_owned(), "1".to_owned()),
    ];

    let out = ProcessHost.run(&command).unwrap();

    assert_eq!(out.stdout, "set unset\n");
    assert!(NESTED_SESSION_VARS.contains(&"CLAUDE_AUTO_BACKGROUND_TASKS"));
}
