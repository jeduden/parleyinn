use super::*;

const MANIFEST_TEXT: &str = r#"
[package]
name = "parinn-fuzz"

[[bin]]
name = "transcript_parser"
path = "fuzz_targets/transcript_parser.rs"
test = false

[[bin]]
name = "redactor"
path = "fuzz_targets/redactor.rs"
"#;

const NIGHTLY_TEXT: &str = r#"name: Nightly
on:
  schedule:
    - cron: "17 3 * * *"
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - name: Fuzz every target
        run: |
          for target in $(cargo +"$nightly" fuzz list --fuzz-dir fuzz); do
            cargo +"$nightly" fuzz run --fuzz-dir fuzz "$target" \
              "fuzz-work/$target" "fuzz/corpus/$target" \
              -- -max_total_time=900
          done
"#;

#[test]
fn each_input_eng_07_names_has_its_target() {
    assert_eq!(target_for("transcript parser"), Ok("transcript_parser"));
    assert_eq!(target_for("event_search query parser"), Ok("query_parser"));
    assert_eq!(INPUTS.len(), 7);
}

#[test]
fn an_input_eng_07_does_not_name_is_refused() {
    assert_eq!(
        target_for("image decoder"),
        Err("ENG-07 names no input \"image decoder\"".to_owned())
    );
}

#[test]
fn the_manifest_s_targets_are_read_with_their_sources() {
    assert_eq!(
        targets(MANIFEST_TEXT),
        vec![
            (
                "transcript_parser".to_owned(),
                "fuzz_targets/transcript_parser.rs".to_owned()
            ),
            ("redactor".to_owned(), "fuzz_targets/redactor.rs".to_owned()),
        ]
    );
}

#[test]
fn a_declared_target_with_a_fuzz_entry_point_has_no_problem() {
    let source = |_: &str| Ok("fuzz_target!(|data: &[u8]| {});".to_owned());
    assert!(target_problems(MANIFEST_TEXT, "redactor", source).is_empty());
}

#[test]
fn an_undeclared_target_or_one_without_an_entry_point_is_named() {
    let source = |_: &str| Ok("fn main() {}".to_owned());
    assert_eq!(
        target_problems(MANIFEST_TEXT, "query_parser", source),
        vec!["fuzz/Cargo.toml declares no target query_parser".to_owned()]
    );
    assert_eq!(
        target_problems(MANIFEST_TEXT, "redactor", source),
        vec!["fuzz/fuzz_targets/redactor.rs has no fuzz_target! entry point".to_owned()]
    );
    let missing = |_: &str| Err("read: no such file".to_owned());
    assert_eq!(
        target_problems(MANIFEST_TEXT, "redactor", missing),
        vec!["read: no such file".to_owned()]
    );
}

#[test]
fn a_corpus_needs_at_least_one_seed() {
    assert!(corpus_problems("redactor", &["aws-key".to_owned()]).is_empty());
    assert_eq!(
        corpus_problems("redactor", &[]),
        vec!["fuzz/corpus/redactor holds no seed".to_owned()]
    );
}

#[test]
fn the_nightly_loop_over_every_listed_target_fuzzes_each_from_its_corpus() {
    assert!(nightly_problems(NIGHTLY_TEXT, "redactor").is_empty());
}

#[test]
fn a_nightly_that_names_the_target_fuzzes_it() {
    let named = NIGHTLY_TEXT.replace(
        "for target in $(cargo +\"$nightly\" fuzz list --fuzz-dir fuzz); do",
        "for target in redactor; do",
    );
    assert!(nightly_problems(&named, "redactor").is_empty());
    assert_eq!(
        nightly_problems(&named, "query_parser"),
        vec!["the nightly fuzz job does not fuzz query_parser".to_owned()]
    );
}

#[test]
fn a_nightly_with_no_schedule_no_fuzz_job_or_no_corpus_is_named() {
    let unscheduled =
        NIGHTLY_TEXT.replace("  schedule:\n    - cron: \"17 3 * * *\"\n", "  push:\n");
    assert_eq!(
        nightly_problems(&unscheduled, "redactor"),
        vec!["the nightly workflow runs on no schedule".to_owned()]
    );
    let renamed = NIGHTLY_TEXT.replace("  fuzz:\n", "  other:\n");
    assert_eq!(
        nightly_problems(&renamed, "redactor"),
        vec!["the nightly workflow has no fuzz job".to_owned()]
    );
    let no_corpus = NIGHTLY_TEXT.replace(" \"fuzz/corpus/$target\"", "");
    assert_eq!(
        nightly_problems(&no_corpus, "redactor"),
        vec!["the nightly fuzz job does not fuzz redactor from fuzz/corpus/redactor".to_owned()]
    );
}
