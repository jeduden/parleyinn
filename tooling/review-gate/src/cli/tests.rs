use super::*;
use crate::{APPROVE, Review};
use std::collections::BTreeMap;

const HEAD: &str = "206b0a76a81abb510e53d87ef187e83b5aacbcbb";

fn inputs() -> BTreeMap<String, String> {
    [
        (
            "outcome.json",
            r#"{"verdict":"approve","summary":"Fine.","findings":[]}"#,
        ),
        (
            "check-runs.jsonl",
            r#"{"name":"CI","status":"completed","conclusion":"success"}"#,
        ),
        ("checked.json", "[]"),
        ("dispatch.json", r#"{"review-code":["lang-rust"]}"#),
        ("consulted.json", r#"["lang-rust","review-code"]"#),
        ("reports/lang-rust.md", "Outcome: sound.\n"),
        (
            "answers.json",
            r#"{"lang-rust":"Outcome: sound.","review-code":"r"}"#,
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect()
}

fn args(current: &str) -> Vec<String> {
    [
        "--outcome",
        "outcome.json",
        "--check-runs",
        "check-runs.jsonl",
        "--reviewed",
        HEAD,
        "--head",
        current,
        "--checked",
        "checked.json",
        "--dispatch",
        "dispatch.json",
        "--consulted",
        "consulted.json",
        "--reports",
        "reports",
        "--answers",
        "answers.json",
    ]
    .map(str::to_owned)
    .to_vec()
}

/// Runs the command over `files`, returning its code, stdout and
/// stderr.
fn invoke(args: Vec<String>, files: &BTreeMap<String, String>) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    };
    let code = run(args, &mut out, &mut err, &read);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn run_prints_the_review_to_post() {
    let (code, out, err) = invoke(args(HEAD), &inputs());

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.ends_with('\n'));
    let review: Review = serde_json::from_str(&out).unwrap();
    assert_eq!(review.event, APPROVE);
    assert_eq!(review.commit_id, HEAD);
}

#[test]
fn the_usage_names_every_flag_as_required() {
    let (code, _, err) = invoke(vec!["--nope".to_owned()], &inputs());

    assert_eq!(code, EXIT_USAGE);
    assert!(
        err.ends_with(
            "usage: review-gate --outcome FILE --check-runs FILE --reviewed SHA --head SHA --checked FILE --dispatch FILE --consulted FILE --reports DIR --answers FILE\n"
        ),
        "{err}"
    );
}

#[test]
fn run_takes_inline_flag_values() {
    let mut args = args(HEAD);
    args.splice(0..2, ["--outcome=outcome.json".to_owned()]);

    assert_eq!(invoke(args, &inputs()).0, EXIT_OK);
}

#[test]
fn run_prints_nothing_for_a_head_that_moved() {
    let (code, out, err) = invoke(args("490efa734617788d1dc1f831b26d0e23888e90fa"), &inputs());

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.is_empty());
    assert!(err.contains("moved past"), "{err}");
}

/// One way the gate fails closed: its name, the arguments, the input
/// files, what standard error must say and the exit code.
type FailCase = (
    &'static str,
    Vec<String>,
    BTreeMap<String, String>,
    &'static str,
    u8,
);

/// Requires each of `cases` to exit with its code, write nothing to
/// standard output and say why on standard error.
fn fails_closed(cases: Vec<FailCase>) {
    for (name, args, files, want, code) in cases {
        let (got, out, err) = invoke(args, &files);
        assert_eq!(got, code, "{name}: {err}");
        assert!(out.is_empty(), "{name}");
        assert!(err.contains(want), "{name}: {err}");
    }
}

#[test]
fn run_fails_closed_on_a_wrong_command_line() {
    let mut stray = args(HEAD);
    stray.push("extra".into());
    fails_closed(vec![
        (
            "an unknown flag",
            vec!["--approve".to_owned()],
            inputs(),
            r#"unknown flag "--approve""#,
            EXIT_USAGE,
        ),
        (
            "a flag with no value",
            vec!["--head".to_owned()],
            inputs(),
            "--head needs a value",
            EXIT_USAGE,
        ),
        (
            "a missing flag",
            args(HEAD)[..6].to_vec(),
            inputs(),
            "--head is required",
            EXIT_USAGE,
        ),
        (
            "no --checked",
            args(HEAD)[..8].to_vec(),
            inputs(),
            "--checked is required",
            EXIT_USAGE,
        ),
        (
            "no --dispatch",
            [&args(HEAD)[..10], &args(HEAD)[12..]].concat(),
            inputs(),
            "--dispatch is required",
            EXIT_USAGE,
        ),
        (
            "no --consulted",
            args(HEAD)[..12].to_vec(),
            inputs(),
            "--consulted is required",
            EXIT_USAGE,
        ),
        (
            "an empty reviewed head",
            {
                let mut a = args(HEAD);
                a[5] = String::new();
                a
            },
            inputs(),
            "--reviewed needs a value",
            EXIT_USAGE,
        ),
        (
            "an empty inline head",
            vec![
                "--outcome=outcome.json".to_owned(),
                "--check-runs=check-runs.jsonl".to_owned(),
                format!("--reviewed={HEAD}"),
                "--head=".to_owned(),
                "--checked=checked.json".to_owned(),
                "--dispatch=dispatch.json".to_owned(),
                "--consulted=consulted.json".to_owned(),
            ],
            inputs(),
            "--head needs a value",
            EXIT_USAGE,
        ),
        (
            "a stray argument",
            stray,
            inputs(),
            "unexpected argument",
            EXIT_USAGE,
        ),
    ]);
}

/// The inputs with `name` set to `body`.
fn with(name: &str, body: &str) -> BTreeMap<String, String> {
    let mut m = inputs();
    m.insert(name.to_owned(), body.to_owned());
    m
}

/// The inputs without `name`.
fn without(name: &str) -> BTreeMap<String, String> {
    let mut m = inputs();
    m.remove(name);
    m
}

#[test]
fn run_fails_closed_on_a_missing_or_malformed_dispatch_or_a_skipped_agent() {
    fails_closed(vec![
        (
            "no dispatch file",
            args(HEAD),
            without("dispatch.json"),
            "read dispatch.json",
            EXIT_FAILURE,
        ),
        (
            "a malformed dispatch",
            args(HEAD),
            with("dispatch.json", "[]"),
            "malformed dispatch",
            EXIT_FAILURE,
        ),
        (
            "no consulted file",
            args(HEAD),
            without("consulted.json"),
            "read consulted.json",
            EXIT_FAILURE,
        ),
        (
            "a malformed consulted list",
            args(HEAD),
            with("consulted.json", "{}"),
            "malformed consulted agents",
            EXIT_FAILURE,
        ),
        (
            "a dispatched agent left out",
            args(HEAD),
            with("consulted.json", r#"["review-code"]"#),
            "dispatched but not consulted: lang-rust",
            EXIT_FAILURE,
        ),
        (
            "a specialist with no report",
            args(HEAD),
            without("reports/lang-rust.md"),
            "lang-rust wrote no report",
            EXIT_FAILURE,
        ),
        (
            "a report another agent wrote",
            args(HEAD),
            with("reports/lang-rust.md", "Outcome: blocking.\n"),
            "lang-rust has a report whose outcome differs from its answer in the run log",
            EXIT_FAILURE,
        ),
        (
            "no answers file",
            args(HEAD),
            without("answers.json"),
            "read answers.json",
            EXIT_FAILURE,
        ),
        (
            "malformed answers",
            args(HEAD),
            with("answers.json", "[]"),
            "malformed agents' answers",
            EXIT_FAILURE,
        ),
        (
            "a dispatched name that is a path",
            args(HEAD),
            with("dispatch.json", r#"{"review-code":["../x"]}"#),
            "agent name",
            EXIT_FAILURE,
        ),
    ]);
}

#[test]
fn run_fails_closed_on_a_missing_or_malformed_input() {
    fails_closed(vec![
        (
            "no outcome file",
            args(HEAD),
            BTreeMap::new(),
            "read outcome.json",
            EXIT_FAILURE,
        ),
        (
            "a malformed outcome",
            args(HEAD),
            with("outcome.json", "{}"),
            "malformed review outcome",
            EXIT_FAILURE,
        ),
        (
            "no check runs file",
            args(HEAD),
            without("check-runs.jsonl"),
            "read check-runs.jsonl",
            EXIT_FAILURE,
        ),
        (
            "a malformed check run",
            args(HEAD),
            with("check-runs.jsonl", "nope"),
            "check run line 1",
            EXIT_FAILURE,
        ),
        (
            "CI not passed at head",
            args(HEAD),
            with("check-runs.jsonl", ""),
            "CI has not passed",
            EXIT_FAILURE,
        ),
    ]);
}

/// A stdout that refuses every write, as a closed pipe does.
struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn run_reports_a_failed_write() {
    let files = inputs();
    let read = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    };
    let mut err = Vec::new();

    let code = run(args(HEAD), &mut Closed, &mut err, &read);

    assert_eq!(code, EXIT_FAILURE);
    assert!(String::from_utf8(err).unwrap().contains("closed"));
    assert!(Closed.flush().is_ok());
}

#[test]
fn run_merges_the_deterministic_findings() {
    let mut files = inputs();
    files.insert(
        "checked.json".to_owned(),
        r#"[{"path":"features/record.feature","line":0,"severity":"blocking","body":"re-pended"}]"#
            .to_owned(),
    );

    let (code, out, err) = invoke(args(HEAD), &files);

    assert_eq!(code, EXIT_OK, "{err}");
    let review: Review = serde_json::from_str(&out).unwrap();
    assert_eq!(review.event, crate::REQUEST_CHANGES);
}

#[test]
fn run_fails_closed_on_a_missing_or_malformed_checked_file() {
    let mut files = inputs();
    files.remove("checked.json");
    let (code, out, err) = invoke(args(HEAD), &files);
    assert_eq!(code, EXIT_FAILURE, "{err}");
    assert!(out.is_empty());
    assert!(err.contains("read checked.json"), "{err}");

    files.insert("checked.json".to_owned(), "nope".to_owned());
    let (code, _, err) = invoke(args(HEAD), &files);
    assert_eq!(code, EXIT_FAILURE, "{err}");
    assert!(err.contains("malformed deterministic findings"), "{err}");
}

#[test]
fn run_wants_the_reports_and_the_answers() {
    fails_closed(vec![
        (
            "no --reports",
            [&args(HEAD)[..14], &args(HEAD)[16..]].concat(),
            inputs(),
            "--reports is required",
            EXIT_USAGE,
        ),
        (
            "no --answers",
            args(HEAD)[..16].to_vec(),
            inputs(),
            "--answers is required",
            EXIT_USAGE,
        ),
    ]);
}
