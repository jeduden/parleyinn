//! ENG-07, SEC-06: the `envelope_encoder` fuzz target's checks over its
//! committed seed corpus, so they run where cargo-fuzz and its nightly
//! do not.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

mod support;

#[test]
fn every_seed_of_the_corpus_passes() {
    let dir = format!(
        "{}/../../fuzz/corpus/envelope_encoder",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut seeds = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        support::encode(&std::fs::read(entry.unwrap().path()).unwrap());
        seeds += 1;
    }

    assert!(seeds >= 8, "{seeds} seeds");
}
