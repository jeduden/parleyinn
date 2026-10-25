//! The benchmark crate's public API across its real boundaries: a real
//! program timed to its exit, and samples files written to disk,
//! concatenated the way the CI job's rounds append to them, then
//! decided by the gate.

#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]

use std::env;
use std::fs;

use benchmark::gate::{self, Status};
use benchmark::{measure, samples};
use testkit::TempDir;

/// Times this test executable, which runs no test and exits 0 when its
/// filter matches none.
#[test]
fn a_real_program_is_timed_to_its_exit() {
    let me = env::current_exe().unwrap();

    let took = measure::time_program(
        &me,
        &["--exact", "no_test_has_this_name"],
        measure::DEADLINE,
    )
    .unwrap();

    assert!(took < measure::DEADLINE);
}

#[test]
fn rounds_appended_to_one_file_are_decided_as_one_set_of_samples() {
    let dir = TempDir::new().unwrap();
    let (base, head) = (dir.path().join("base"), dir.path().join("head"));
    let mut base_text = String::new();
    let mut head_text = String::new();
    for round in 0..3_u64 {
        let base_round: Vec<u64> = (0..10).map(|i| 800_000 + round + i).collect();
        let head_round: Vec<u64> = base_round.iter().map(|ns| ns * 2).collect();
        for (text, taken) in [(&mut base_text, base_round), (&mut head_text, head_round)] {
            text.push_str(samples::HEADER);
            text.push('\n');
            text.push_str(&samples::render("core_start_up", &taken));
        }
    }
    fs::write(&base, base_text).unwrap();
    fs::write(&head, head_text).unwrap();

    let read = |path| samples::parse_strata(&fs::read_to_string(path).unwrap()).unwrap();
    let (base, head) = (read(&base), read(&head));
    let decided = gate::decide(&base, &head, &[]);

    assert_eq!(samples::pooled(&base)["core_start_up"].len(), 30);
    assert_eq!(decided.len(), 1);
    assert_eq!(decided[0].status, Status::Regressed);
}
