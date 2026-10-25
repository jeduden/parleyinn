//! ENG-07, LANE-02: the `git_reader` fuzz target's checks over its
//! committed seed corpus, so they run where cargo-fuzz and its nightly
//! do not.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

#[path = "support/git_reader_harness.rs"]
mod harness;

use testkit::TempDir;

#[test]
fn the_seed_corpus_holds_the_fuzz_target_checks() {
    let corpus =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/git_reader");
    let dir = TempDir::new().unwrap();
    let mut seen = 0;
    for entry in std::fs::read_dir(corpus).unwrap() {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap();
        harness::check(&dir.path().join("run"), &bytes);
        seen += 1;
    }
    assert!(seen >= 8, "the seed corpus is committed");
}

#[test]
fn an_input_past_the_cap_is_cut_and_the_separators_split_it_in_five() {
    let dir = TempDir::new().unwrap();
    let mut input = Vec::new();
    for part in 0..6 {
        input.extend_from_slice(&[b'a' + part; 3]);
        input.extend_from_slice(harness::SEPARATOR);
    }
    input.resize(harness::MAX_INPUT + 10, 0);

    harness::check(&dir.path().join("run"), &input);
    harness::check(&dir.path().join("run"), b"");
}
