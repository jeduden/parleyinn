use std::fs;

use testkit::TempDir;

use super::*;

fn artifact(id: u64, name: &str, at: &str, run: u64, expired: bool) -> String {
    format!(
        r#"{{"id":{id},"name":"{name}","expired":{expired},"created_at":"{at}","workflow_run":{{"id":{run}}}}}"#
    )
}

fn listing(items: &[String]) -> String {
    format!(
        r#"{{"total_count":{},"artifacts":[{}]}}"#,
        items.len(),
        items.join(",")
    )
}

fn runs(items: &[(u64, &str, &str)]) -> String {
    let rows: Vec<String> = items
        .iter()
        .map(|(id, path, event)| format!(r#"{{"id":{id},"path":"{path}","event":"{event}"}}"#))
        .collect();
    format!("[{}]", rows.join(","))
}

const NAME: &str = "review-findings-pr-7";
const REVIEW: &str = ".github/workflows/review.yml";

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
fn the_name_is_the_pull_requests_number_and_nothing_else() {
    assert_eq!(name("7").unwrap(), NAME);
    assert_eq!(name("1204").unwrap(), "review-findings-pr-1204");
    for bad in ["", "7a", "-7", "7/8", "a b", "../7"] {
        assert!(name(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn the_newest_artifact_from_main_s_review_workflow_is_taken() {
    let a = listing(&[
        artifact(10, NAME, "2026-11-01T10:00:00Z", 100, false),
        artifact(12, NAME, "2026-11-01T12:00:00Z", 120, false),
        artifact(11, NAME, "2026-11-01T11:00:00Z", 110, false),
    ]);
    let r = runs(&[
        (100, REVIEW, "workflow_run"),
        (110, REVIEW, "issue_comment"),
        (120, REVIEW, "workflow_run"),
    ]);
    assert_eq!(
        select("7", &a, &r).unwrap(),
        Selection {
            artifact: Some(12),
            skipped: 0
        }
    );
}

#[test]
fn an_artifact_of_the_name_from_another_workflow_is_skipped_and_counted() {
    let a = listing(&[
        artifact(12, NAME, "2026-11-01T12:00:00Z", 120, false),
        artifact(11, NAME, "2026-11-01T11:00:00Z", 110, false),
    ]);
    let r = runs(&[
        (120, ".github/workflows/ci.yml", "workflow_run"),
        (110, REVIEW, "workflow_run"),
    ]);
    assert_eq!(
        select("7", &a, &r).unwrap(),
        Selection {
            artifact: Some(11),
            skipped: 1
        }
    );
}

#[test]
fn a_run_on_the_pull_requests_own_definition_is_skipped() {
    // The pull request may rename its own workflow to review.yml; its
    // event is `pull_request`, never one main's definition runs on.
    let a = listing(&[artifact(12, NAME, "2026-11-01T12:00:00Z", 120, false)]);
    for event in ["pull_request", "push", "workflow_dispatch", "schedule"] {
        let r = runs(&[(120, REVIEW, event)]);
        assert_eq!(
            select("7", &a, &r).unwrap(),
            Selection {
                artifact: None,
                skipped: 1
            },
            "{event}"
        );
    }
}

#[test]
fn a_path_with_a_ref_suffix_is_read_by_the_path_before_it() {
    let a = listing(&[artifact(12, NAME, "2026-11-01T12:00:00Z", 120, false)]);
    let ok = runs(&[(
        120,
        ".github/workflows/review.yml@refs/heads/main",
        "workflow_run",
    )]);
    assert_eq!(select("7", &a, &ok).unwrap().artifact, Some(12));
    let other = runs(&[(
        120,
        ".github/workflows/review.yml.evil@main",
        "workflow_run",
    )]);
    assert_eq!(select("7", &a, &other).unwrap().artifact, None);
}

#[test]
fn an_artifact_whose_run_is_unknown_is_skipped() {
    let a = listing(&[artifact(12, NAME, "2026-11-01T12:00:00Z", 120, false)]);
    assert_eq!(
        select("7", &a, "[]").unwrap(),
        Selection {
            artifact: None,
            skipped: 1
        }
    );
    let no_run = r#"{"artifacts":[{"id":5,"name":"review-findings-pr-7","created_at":"2026-11-01T12:00:00Z"}]}"#;
    assert_eq!(select("7", no_run, "[]").unwrap().skipped, 1);
}

#[test]
fn an_expired_artifact_and_another_pull_requests_are_not_taken_nor_counted() {
    let a = listing(&[
        artifact(12, NAME, "2026-11-01T12:00:00Z", 120, true),
        artifact(
            11,
            "review-findings-pr-8",
            "2026-11-01T11:00:00Z",
            110,
            false,
        ),
        artifact(10, "review-reports", "2026-11-01T10:00:00Z", 100, false),
    ]);
    let r = runs(&[
        (120, REVIEW, "workflow_run"),
        (110, REVIEW, "workflow_run"),
        (100, REVIEW, "workflow_run"),
    ]);
    assert_eq!(
        select("7", &a, &r).unwrap(),
        Selection {
            artifact: None,
            skipped: 0
        }
    );
}

#[test]
fn none_qualifying_is_a_selection_with_no_artifact() {
    assert_eq!(
        select("7", &listing(&[]), "[]").unwrap(),
        Selection {
            artifact: None,
            skipped: 0
        }
    );
}

#[test]
fn a_newer_artifact_wins_over_a_higher_id_with_an_older_time() {
    let a = listing(&[
        artifact(99, NAME, "2026-11-01T09:00:00Z", 100, false),
        artifact(5, NAME, "2026-11-01T12:00:00Z", 120, false),
    ]);
    let r = runs(&[(100, REVIEW, "workflow_run"), (120, REVIEW, "workflow_run")]);
    assert_eq!(select("7", &a, &r).unwrap().artifact, Some(5));
}

#[test]
fn inputs_that_do_not_read_are_errors() {
    assert!(
        select("7", "not json", "[]")
            .unwrap_err()
            .contains("artifacts")
    );
    assert!(
        select("7", &listing(&[]), "{")
            .unwrap_err()
            .contains("runs")
    );
    assert!(select("x", &listing(&[]), "[]").is_err());
}

#[test]
fn the_lines_name_the_artifact_and_the_count_for_the_job_s_outputs() {
    let found = Selection {
        artifact: Some(12),
        skipped: 2,
    };
    assert_eq!(found.lines(), "previous_artifact=12\nprevious_skipped=2\n");
    let none = Selection {
        artifact: None,
        skipped: 0,
    };
    assert_eq!(none.lines(), "previous_artifact=\nprevious_skipped=0\n");
}

#[test]
fn a_findings_file_that_reads_passes_and_one_that_does_not_is_named() {
    check("[]").unwrap();
    assert!(check("{").unwrap_err().contains("malformed findings"));
    let big = format!("[{}]", " ".repeat(MAX_BYTES));
    assert!(check(&big).unwrap_err().contains("past"));
}

#[test]
fn a_findings_file_of_exactly_the_size_limit_passes() {
    let at = format!("[{}]", " ".repeat(MAX_BYTES - 2));
    assert_eq!(at.len(), MAX_BYTES);
    check(&at).unwrap();
    let past = format!("[{}]", " ".repeat(MAX_BYTES - 1));
    assert_eq!(past.len(), MAX_BYTES + 1);
    assert!(check(&past).unwrap_err().contains("past"));
}

#[test]
fn the_previous_command_is_found_past_the_summary_flag() {
    assert_eq!(
        subcommand(&args(&["--summary=s", "previous", "check", "f"])),
        Some(args(&["check", "f"]))
    );
    assert_eq!(subcommand(&args(&["previous"])), Some(Vec::new()));
    assert_eq!(subcommand(&args(&["--workdir", "previous"])), None);
    assert_eq!(subcommand(&[]), None);
}

#[test]
fn select_prints_the_outputs_and_exits_ok() {
    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a.json");
    let r = dir.path().join("r.json");
    fs::write(
        &a,
        listing(&[artifact(12, NAME, "2026-11-01T12:00:00Z", 120, false)]),
    )
    .unwrap();
    fs::write(&r, runs(&[(120, REVIEW, "workflow_run")])).unwrap();
    let (code, out, err) = run_with(&[
        "select",
        "--pull",
        "7",
        "--artifacts",
        a.to_str().unwrap(),
        "--runs",
        r.to_str().unwrap(),
    ]);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert_eq!(out, "previous_artifact=12\nprevious_skipped=0\n");
}

#[test]
fn select_fails_naming_a_file_that_does_not_read() {
    let (code, out, err) = run_with(&[
        "select",
        "--pull",
        "7",
        "--artifacts",
        "/nonexistent/a",
        "--runs",
        "/nonexistent/r",
    ]);
    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(err.contains("/nonexistent/a"), "{err}");
}

#[test]
fn check_exits_ok_on_findings_and_names_a_broken_file() {
    let dir = TempDir::new().unwrap();
    let good = dir.path().join("good.json");
    let bad = dir.path().join("bad.json");
    fs::write(&good, "[]").unwrap();
    fs::write(&bad, "{").unwrap();
    assert_eq!(run_with(&["check", good.to_str().unwrap()]).0, EXIT_OK);
    let (code, _, err) = run_with(&["check", bad.to_str().unwrap()]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains(bad.to_str().unwrap()), "{err}");
    assert!(err.contains("malformed findings"), "{err}");
    let (code, _, err) = run_with(&["check", "/nonexistent/f"]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("/nonexistent/f"), "{err}");
}

#[test]
fn a_command_line_that_is_wrong_is_a_usage_error() {
    for list in [
        &["select"][..],
        &["select", "--pull", "7"],
        &["select", "--bogus"],
        &["select", "--pull"],
        &["check"],
        &["check", "a", "b"],
        &["unknown"],
        &[],
    ] {
        let (code, _, err) = run_with(list);
        assert_eq!(code, EXIT_USAGE, "{list:?}");
        assert!(err.contains("usage: review-run previous"), "{err}");
    }
}

/// A writer that refuses everything.
struct Refuses;

impl Write for Refuses {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("no room"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn an_output_that_cannot_be_written_is_a_failure_that_says_so() {
    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a.json");
    let r = dir.path().join("r.json");
    fs::write(&a, listing(&[])).unwrap();
    fs::write(&r, "[]").unwrap();
    let mut err = Vec::new();
    let code = run(
        &args(&[
            "select",
            "--pull",
            "7",
            "--artifacts",
            a.to_str().unwrap(),
            "--runs",
            r.to_str().unwrap(),
        ]),
        &mut Refuses,
        &mut err,
    );
    assert_eq!(code, EXIT_FAILURE);
    assert!(String::from_utf8(err).unwrap().contains("no room"));
}

#[test]
fn an_artifact_with_no_time_is_read_and_ranks_last() {
    let a = r#"{"artifacts":[
        {"id":5,"name":"review-findings-pr-7","created_at":null,"workflow_run":{"id":50}},
        {"id":4,"name":"review-findings-pr-7","created_at":"2026-11-01T09:00:00Z","workflow_run":{"id":40}}]}"#;
    let r = runs(&[(50, REVIEW, "workflow_run"), (40, REVIEW, "workflow_run")]);
    assert_eq!(select("7", a, &r).unwrap().artifact, Some(4));
}

#[test]
fn an_artifact_from_an_issue_comment_run_is_taken_when_it_alone_qualifies() {
    let a = listing(&[artifact(7, NAME, "2026-11-01T10:00:00Z", 70, false)]);
    let r = runs(&[(70, REVIEW, "issue_comment")]);
    assert_eq!(
        select("7", &a, &r).unwrap(),
        Selection {
            artifact: Some(7),
            skipped: 0
        }
    );
}
