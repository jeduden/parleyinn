use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|a| (*a).to_owned()).collect()
}

/// A work directory as the workflow prepares it, for one generalist
/// and two specialists.
fn workdir() -> TempDir {
    let dir = TempDir::new().unwrap();
    let review = dir.path().join("review");
    fs::create_dir_all(&review).unwrap();
    fs::write(
        review.join("dispatch.json"),
        r#"{"review-code": ["lang-rust", "test-engineer-unit"]}"#,
    )
    .unwrap();
    fs::write(dir.path().join("class.txt"), "class=standard\ntouched=1\n").unwrap();
    for name in ["lang-rust", "test-engineer-unit", "review-code"] {
        fake::define(dir.path(), name, 20);
    }
    dir
}

fn run_in(host: &FakeHost, dir: &Path) -> (u8, String, String) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        host,
        args(&["--workdir", dir.to_str().unwrap()]),
        &mut out,
        &mut err,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn read(dir: &Path, rel: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(dir.join(rel)).unwrap()).unwrap()
}

#[test]
fn a_review_writes_every_file_the_gate_reads() {
    let dir = workdir();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(
        out,
        "verdict=approve\ndispatched=3\nconsulted=3\nfindings=1\ncost=0.0300\n"
    );
    assert_eq!(err.lines().count(), 3, "{err}");
    let outcome = fs::read_to_string(dir.path().join("review/outcome.json")).unwrap();
    let outcome = review_gate::parse_outcome(&outcome).unwrap();
    assert_eq!(outcome.verdict, "approve");
    let consulted = fs::read_to_string(dir.path().join("review/consulted.json")).unwrap();
    assert_eq!(
        review_gate::parse_consulted(&consulted).unwrap(),
        ["lang-rust", "review-code", "test-engineer-unit"]
            .map(String::from)
            .into()
    );
    let answers = fs::read_to_string(dir.path().join("review/answers.json")).unwrap();
    assert_eq!(review_gate::parse_answers(&answers).unwrap().len(), 3);
    let usage = read(dir.path(), "review/usage.json");
    assert_eq!(usage.as_array().unwrap().len(), 3);
    assert_eq!(usage[0]["calls"], 1);
    assert!(dir.path().join("review/logs/review-code.jsonl").exists());
}

#[test]
fn a_review_whose_specialist_fails_is_still_decided() {
    let dir = workdir();
    let host = FakeHost::new(|c| {
        if fake::agent_of(c) == "lang-rust" {
            Err("claude: not found".to_owned())
        } else {
            fake::either(c)
        }
    });

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_OK);
    assert!(out.starts_with("verdict=request_changes\n"), "{out}");
    assert!(
        err.contains("review-run: lang-rust (specialist): did not answer: claude: not found"),
        "{err}"
    );
    let consulted = read(dir.path(), "review/consulted.json");
    assert_eq!(
        consulted,
        serde_json::json!(["review-code", "test-engineer-unit"])
    );
}

#[test]
fn a_review_without_its_plan_cannot_run() {
    let dir = TempDir::new().unwrap();
    let host = FakeHost::new(fake::either);

    let (code, out, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "");
    assert!(err.starts_with("review-run: read "), "{err}");
    assert!(host.agents().is_empty());
}

#[test]
fn a_review_whose_outcome_cannot_be_written_fails() {
    let dir = workdir();
    fs::create_dir_all(dir.path().join("review/outcome.json")).unwrap();
    let host = FakeHost::new(fake::either);

    let (code, _, err) = run_in(&host, dir.path());

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("review-run: write "), "{err}");
}

fn refusal(bad: &[&str]) -> String {
    parse(args(bad)).unwrap_err()
}

#[test]
fn the_work_directory_alone_takes_the_default_options() {
    assert_eq!(
        parse(args(&["--workdir", "w"])).unwrap(),
        ("w".to_owned(), Options::default())
    );
}

#[test]
fn the_flags_read_inline_or_as_the_next_argument() {
    assert_eq!(
        parse(args(&["--workdir=w", "--parallel=2", "--deadline", "30"])).unwrap(),
        (
            "w".to_owned(),
            Options {
                parallel: 2,
                deadline: Duration::from_secs(30),
            }
        )
    );
}

#[test]
fn a_command_line_without_the_work_directory_is_refused() {
    assert_eq!(refusal(&[]), "--workdir is required");
}

#[test]
fn an_argument_that_is_no_flag_is_refused() {
    assert_eq!(refusal(&["w"]), "unexpected argument \"w\"");
}

#[test]
fn an_unknown_flag_is_refused() {
    assert_eq!(refusal(&["--out", "x"]), "unknown flag \"--out\"");
}

#[test]
fn a_flag_without_a_value_is_refused() {
    assert_eq!(refusal(&["--workdir"]), "--workdir needs a value");
    assert_eq!(refusal(&["--workdir="]), "--workdir needs a value");
}

#[test]
fn a_zero_bound_is_refused() {
    assert_eq!(
        refusal(&["--workdir", "w", "--parallel", "0"]),
        "--parallel \"0\" is not a number above zero"
    );
}

#[test]
fn a_deadline_that_is_no_number_is_refused() {
    assert_eq!(
        refusal(&["--workdir", "w", "--deadline", "soon"]),
        "--deadline \"soon\" is not a number above zero"
    );
}

#[test]
fn a_wrong_command_line_prints_the_usage() {
    let host = FakeHost::new(fake::either);
    let mut out = Vec::new();
    let mut err = Vec::new();

    let code = run(&host, args(&["--nope"]), &mut out, &mut err);

    assert_eq!(code, EXIT_USAGE);
    let err = String::from_utf8(err).unwrap();
    assert!(err.starts_with("review-run: unknown flag"), "{err}");
    assert!(err.ends_with(&format!("{USAGE}\n")), "{err}");
}

#[test]
fn a_value_that_does_not_encode_is_an_error_naming_its_file() {
    let dir = TempDir::new().unwrap();
    let value: BTreeMap<(u8, u8), u8> = BTreeMap::from([((1, 2), 3)]);

    let err = write_json(dir.path(), "out.json", &value).unwrap_err();

    assert!(err.starts_with("encode "), "{err}");
    assert!(err.contains("out.json: "), "{err}");
    assert!(!dir.path().join("out.json").exists());
}
