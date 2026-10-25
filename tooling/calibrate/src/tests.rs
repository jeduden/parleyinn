use super::*;

/// A fresh directory under the system's temporary directory.
fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("calibrate-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn mixing_no_rounds_leaves_the_seed() {
    assert_eq!(mix(SEED, 0), SEED);
}

#[test]
fn mixing_is_the_same_every_time() {
    assert_eq!(mix(SEED, 1), 0xe220_a839_7b1d_cdaf);
    assert_eq!(mix(SEED, 1000), 0x6597_9e09_8215_94d5);
}

#[test]
fn the_files_sum_what_they_wrote_and_leave_nothing_behind() {
    let dir = scratch("files");

    let sum = touch_files(&dir, 3).unwrap();

    // Each round reads CHUNK bytes of its own round number and sees
    // the file CHUNK bytes long.
    let want: u64 = (0..3u64).map(|r| r * CHUNK as u64 + CHUNK as u64).sum();
    assert_eq!(sum, want);
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    std::fs::remove_dir(&dir).unwrap();
}

#[test]
fn no_rounds_make_no_file_and_still_succeed() {
    let dir = scratch("none");

    assert_eq!(touch_files(&dir, 0).unwrap(), 0);
    assert_eq!(run_with(&dir, SEED, 0, 0).unwrap(), SEED);
    std::fs::remove_dir(&dir).unwrap();
}

#[test]
fn a_directory_that_is_not_there_fails() {
    let dir = std::env::temp_dir().join("calibrate-missing/none");

    assert!(touch_files(&dir, 1).is_err());
    assert!(run_with(&dir, SEED, 1, 1).is_err());
}

#[test]
fn the_frozen_workload_gives_the_result_it_was_recorded_with() {
    let dir = scratch("frozen");

    let got = run(&dir).unwrap();

    assert_eq!(got, 0x6abb_56e1_e7d0_b6ba);
    std::fs::remove_dir(&dir).unwrap();
}

#[test]
fn the_workload_combines_the_mix_and_the_files() {
    let dir = scratch("run");

    let got = run_with(&dir, SEED, 5, 2).unwrap();

    assert_eq!(got, mix(SEED, 5) ^ touch_files(&dir, 2).unwrap());
    std::fs::remove_dir(&dir).unwrap();
}
