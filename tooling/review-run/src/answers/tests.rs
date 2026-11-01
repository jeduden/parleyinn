use std::fs;

use testkit::TempDir;

use super::*;

const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|a| (*a).to_owned()).collect()
}

fn run_with(list: &[&str]) -> (u8, String, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(&args(list), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn the_answers_command_is_found_past_the_summary_flag() {
    assert_eq!(
        subcommand(&args(&["--summary=s", "answers", "drop", KEY])),
        Some(args(&["drop", KEY]))
    );
    assert_eq!(subcommand(&args(&["answers"])), Some(Vec::new()));
    assert_eq!(subcommand(&args(&["--workdir", "answers"])), None);
    assert_eq!(subcommand(&[]), None);
}

#[test]
fn drop_takes_the_answer_out_of_the_store_and_says_so() {
    let dir = TempDir::new().unwrap();
    let store = dir.path().join("answers");
    fs::create_dir_all(store.join(KEY)).unwrap();
    fs::write(store.join(KEY).join("log.jsonl"), "x").unwrap();
    let other = "f".repeat(64);
    fs::create_dir_all(store.join(&other)).unwrap();
    let at = store.display().to_string();

    let (code, out, err) = run_with(&["drop", "--answers", &at, KEY]);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, format!("dropped {KEY} from {at}\n"));
    assert!(!store.join(KEY).exists());
    assert!(store.join(other).exists());
}

#[test]
fn drop_names_a_key_that_is_not_there_or_is_no_key() {
    let dir = TempDir::new().unwrap();
    let at = format!("--answers={}", dir.path().display());

    let (code, _, err) = run_with(&["drop", KEY, &at]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("no answer is stored under"), "{err}");

    let (code, _, err) = run_with(&["drop", "../etc", &at]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("is no answer key"), "{err}");
}

#[test]
fn a_wrong_answers_command_line_shows_the_usage() {
    for (list, want) in [
        (vec![], "the one answers command is drop"),
        (vec!["list"], "the one answers command is drop"),
        (vec!["drop", KEY], "--answers needs a value"),
        (vec!["drop", KEY, "--answers"], "--answers needs a value"),
        (vec!["drop", "--answers=d"], "drop needs the KEY"),
        (vec!["drop", KEY, KEY, "--answers=d"], "unexpected argument"),
        (vec!["drop", KEY, "--force"], "unknown flag"),
    ] {
        let (code, out, err) = run_with(&list);
        assert_eq!(code, EXIT_USAGE, "{list:?}");
        assert!(out.is_empty());
        assert!(err.contains(want), "{list:?}: {err}");
        assert!(err.contains(USAGE), "{err}");
    }
}

/// Standard output that takes no write.
struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("closed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_drop_that_cannot_say_so_fails() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(KEY)).unwrap();
    let mut err = Vec::new();
    let at = format!("--answers={}", dir.path().display());

    let code = run(&args(&["drop", KEY, &at]), &mut Closed, &mut err);

    assert_eq!(code, EXIT_FAILURE);
    assert!(String::from_utf8(err).unwrap().contains("but say so"));
}
