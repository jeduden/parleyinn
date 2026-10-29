use std::path::Path;
use std::time::Duration;

use super::*;

#[test]
fn a_command_carries_its_program_arguments_and_directory() {
    let c = Command::new("git", &["status", "--short"], Path::new("/work"));

    assert_eq!(c.program, "git");
    assert_eq!(c.args, vec!["status", "--short"]);
    assert_eq!(c.cwd, Path::new("/work"));
    assert_eq!(c.env, Vec::<(String, String)>::new());
    assert_eq!(c.stdout_to, None);
    assert_eq!(c.deadline, None);
    assert_eq!(c.line(), "git status --short");
}

#[test]
fn a_command_logs_to_a_file_within_a_deadline() {
    let c = Command::new("claude", &[], Path::new("."))
        .logging_to(Path::new("log.jsonl"))
        .within(Duration::from_secs(5));

    assert_eq!(c.stdout_to.as_deref(), Some(Path::new("log.jsonl")));
    assert_eq!(c.deadline, Some(Duration::from_secs(5)));
}

#[test]
fn an_output_is_ok_only_on_exit_0_within_its_deadline() {
    let exit_0 = Output {
        code: Some(0),
        ..Output::default()
    };
    let exit_1 = Output {
        code: Some(1),
        ..Output::default()
    };
    let killed = Output {
        code: None,
        timed_out: true,
        ..Output::default()
    };

    assert!(exit_0.ok());
    assert!(!exit_1.ok());
    assert!(!killed.ok());
    assert!(!Output::default().ok());
}
