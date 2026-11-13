use super::*;

const SOAK_SOURCE: &str = "const WRITERS: u8 = 50;\nconst EVENTS: usize = 10_000;\n";

const NIGHTLY_TEXT: &str = r#"name: Nightly
on:
  schedule:
    - cron: "17 3 * * *"
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - run: cargo fuzz run redactor
  # The soak: cargo test -p store --test soak
  soak:
    runs-on: ubuntu-latest
    steps:
      - name: Append 1,000,000 events from 50 writers at once
        env:
          PARINN_SOAK_EVENTS: "1000000"
        run: |
          set -o pipefail
          cargo test --locked --release -p store --test soak -- --nocapture | tee -a "$GITHUB_STEP_SUMMARY"
"#;

const CI_TEXT: &str = r#"name: CI
on:
  pull_request:
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --workspace --locked
  miri:
    runs-on: ubuntu-latest
    steps:
      - run: cargo "+$MIRI_NIGHTLY" miri test -p platform --locked
"#;

fn held(text: &str) -> Job {
    Job {
        name: "soak".to_owned(),
        text: text.to_owned(),
    }
}

#[test]
fn a_job_is_found_by_its_name_with_its_own_lines_and_no_comment() {
    let soak = job(NIGHTLY_TEXT, "soak").unwrap();
    assert_eq!(soak.name, "soak");
    assert!(soak.text.contains("--test soak"));
    assert!(!soak.text.contains("cargo fuzz"));

    let fuzz = job(NIGHTLY_TEXT, "fuzz").unwrap();
    assert!(fuzz.text.contains("cargo fuzz run redactor"));
    assert!(!fuzz.text.contains("--test soak"), "{}", fuzz.text);

    assert_eq!(job(NIGHTLY_TEXT, "miri"), None);
}

#[test]
fn a_line_holds_its_parts_only_as_whole_words_and_all_on_one_line() {
    let job = held("  run: cargo test -p store --test soak\n  env: none\n");
    assert!(has_line(&job, &["cargo test", "-p store", "--test soak"]));
    assert!(has_line(&job, &["--test soak"]));
    assert!(has_line(&job, &["run:"]));

    let other = held("  run: cargo test -p store --test soak_fast\n");
    assert!(!has_line(&other, &["--test soak"]));
    let prefixed = held("  run: cargo test -p restore --test soak\n");
    assert!(!has_line(&prefixed, &["-p store"]));

    let split = held("  run: cargo test -p store\n  also: --test soak\n");
    assert!(!has_line(&split, &["-p store", "--test soak"]));
    assert!(!has_line(&held(""), &["cargo"]));
}

#[test]
fn the_nightly_soak_job_as_written_has_no_problem() {
    assert_eq!(
        nightly_problems(NIGHTLY_TEXT, SOAK_SOURCE),
        Vec::<String>::new()
    );
}

#[test]
fn a_soak_that_declares_other_than_fifty_writers_is_a_problem() {
    let soak = SOAK_SOURCE.replace("u8 = 50", "u8 = 5");
    assert_eq!(
        nightly_problems(NIGHTLY_TEXT, &soak),
        [
            "crates/store/tests/soak.rs does not declare \"const WRITERS: u8 = 50;\": the soak must run 50 writers"
        ]
    );
}

#[test]
fn a_nightly_workflow_without_a_soak_job_is_a_problem_that_names_the_command() {
    let nightly = NIGHTLY_TEXT.replace("  soak:\n", "  drench:\n");
    assert_eq!(
        nightly_problems(&nightly, SOAK_SOURCE),
        [
            ".github/workflows/nightly.yml has no job \"soak\": add it, running \"cargo test --locked --release -p store --test soak\""
        ]
    );
    // The missing job is reported beside the soak's own problem.
    assert_eq!(
        nightly_problems(&nightly, ""),
        [
            "crates/store/tests/soak.rs does not declare \"const WRITERS: u8 = 50;\": the soak must run 50 writers",
            ".github/workflows/nightly.yml has no job \"soak\": add it, running \"cargo test --locked --release -p store --test soak\""
        ]
    );
}

#[test]
fn a_soak_job_that_runs_another_test_is_a_problem() {
    let want = [
        "the soak job of .github/workflows/nightly.yml does not run \"cargo test -p store --test soak\": the soak test is the evidence",
    ];
    for other in ["s12_rebuild", "soak_fast"] {
        let nightly = NIGHTLY_TEXT.replace("--test soak --", &format!("--test {other} --"));
        assert_eq!(nightly_problems(&nightly, SOAK_SOURCE), want, "{other}");
    }
    // The comment above the job names the command and is not the job.
    let commented = NIGHTLY_TEXT.replace("          cargo test --locked", "          # cargo test");
    assert_eq!(nightly_problems(&commented, SOAK_SOURCE), want);
}

#[test]
fn a_soak_job_set_to_fewer_events_is_a_problem() {
    let want = [
        "the soak job of .github/workflows/nightly.yml does not set PARINN_SOAK_EVENTS to \"1000000\": set it, or the 1000000-event claim has no evidence",
    ];
    for events in ["\"100000\"", "\"10000000\"", "1000000"] {
        let nightly = NIGHTLY_TEXT.replace("\"1000000\"", events);
        assert_eq!(nightly_problems(&nightly, SOAK_SOURCE), want, "{events}");
    }
}

#[test]
fn the_miri_job_as_written_has_no_problem() {
    assert_eq!(miri_problems(CI_TEXT), Vec::<String>::new());
}

#[test]
fn a_pull_request_workflow_without_a_miri_job_is_a_problem() {
    let ci = CI_TEXT.replace("  miri:\n", "  interpret:\n");
    assert_eq!(
        miri_problems(&ci),
        [
            ".github/workflows/ci.yml has no job \"miri\": add it, running \"cargo miri test -p platform\""
        ]
    );
}

#[test]
fn a_miri_job_run_over_another_crate_is_a_problem() {
    let want = [
        "the miri job of .github/workflows/ci.yml does not run \"cargo miri test -p platform\": the platform crate holds the unsafe code Miri must run",
    ];
    for other in ["-p key", "-p platform_x", "--workspace"] {
        let ci = CI_TEXT.replace("-p platform", other);
        assert_eq!(miri_problems(&ci), want, "{other}");
    }
    // The test job's cargo test of the workspace is not Miri's.
    let no_miri = CI_TEXT.replace("miri test", "test");
    assert_eq!(miri_problems(&no_miri), want);
}
