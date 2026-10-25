use std::path::Path;

use testkit::TempDir;

use super::*;
use crate::tests::short_run;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn short(templates: &Templates, _seed: u64) -> Vec<Run> {
    vec![short_run(templates)]
}

fn outcome(depth: usize, rank: Option<usize>) -> Outcome {
    Outcome {
        seed: 1,
        depth,
        question: "q".into(),
        value: "v".into(),
        rank,
        in_context: false,
    }
}

#[test]
fn anything_but_recall_prints_the_usage() {
    let root = testkit::repo_root();
    for wrong in [&[][..], &["recall", "--published"], &["other"]] {
        assert_eq!(run(&args(wrong), &root, &[], short), (2, USAGE.to_owned()));
    }
}

#[test]
fn a_measure_runs_each_seed_in_its_own_home() {
    let (outcomes, unparsed) = measure(&testkit::repo_root(), &[1, 2], short).unwrap();
    assert_eq!(unparsed, 0);
    let seeds: Vec<u64> = outcomes.iter().map(|o| o.seed).collect();
    assert_eq!(seeds, [1, 1, 2, 2]);
}

#[test]
fn a_dataset_below_the_target_fails_and_says_which_depth() {
    let root = testkit::repo_root();
    let (code, out) = run(&args(&["recall"]), &root, &[1], short);
    assert_eq!(code, 1);
    assert!(
        out.ends_with("depth 1 is below the target of 0.95\n"),
        "{out}"
    );
}

#[test]
fn a_dataset_at_the_target_passes_and_a_line_that_did_not_parse_fails_it() {
    let met = [outcome(1, Some(1)), outcome(2, Some(3))];
    let (out, ok) = conclude(&[1], &met, 0);
    assert!(ok, "{out}");
    let (out, ok) = conclude(&[1], &met, 2);
    assert!(!ok);
    assert!(
        out.ends_with("2 lines of the dataset did not parse\n"),
        "{out}"
    );
}

#[test]
fn the_published_result_must_be_the_one_measured_now() {
    let root = testkit::repo_root();
    let dir = TempDir::new().unwrap();
    let (_, report) = run(&args(&["recall"]), &root, &[], short);
    std::fs::write(dir.path().join("same.txt"), &report).unwrap();
    std::fs::write(dir.path().join("other.txt"), "an older result\n").unwrap();
    let published = |name: &str| dir.path().join(name).display().to_string();

    assert_eq!(
        run(
            &args(&["recall", "--published", &published("same.txt")]),
            &root,
            &[],
            short
        ),
        (0, report.clone())
    );
    let (code, out) = run(
        &args(&["recall", "--published", &published("other.txt")]),
        &root,
        &[],
        short,
    );
    assert_eq!(code, 1);
    assert!(
        out.ends_with("is not the result measured now; publish this report in its place\n"),
        "{out}"
    );
    let (code, out) = run(
        &args(&["recall", "--published", &published("none.txt")]),
        &root,
        &[],
        short,
    );
    assert_eq!(code, 1);
    assert!(out.contains("parinn-eval: read "), "{out}");
}

#[test]
fn a_checkout_without_the_fixture_fails_naming_it() {
    let (code, out) = run(&args(&["recall"]), Path::new("/nonexistent"), &[1], short);
    assert_eq!(code, 1);
    assert!(out.starts_with("parinn-eval: read /nonexistent/"), "{out}");
}

#[test]
fn the_repository_publishes_its_result_beside_this_crate() {
    assert!(testkit::repo_root().join(PUBLISHED).is_file());
}
