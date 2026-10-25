//! ENG-07, LANE-02: the core's own git reader on a hostile clone. The
//! input is split into the files of a clone (a branch tip, a loose
//! object, a pack and its index, a linked worktree's `gitdir:` file),
//! which the reader reads under a deadline counted in polls, not time.
//! It must neither panic nor stall, and its answers must agree with
//! each other. The checks are in the file the corpus replay test shares.
#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../crates/repository/tests/support/git_reader_harness.rs"]
mod harness;

fuzz_target!(|data: &[u8]| {
    let root = std::env::temp_dir().join(format!("parinn-fuzz-git-reader-{}", std::process::id()));
    harness::check(&root, data);
});
