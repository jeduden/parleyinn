use super::*;

use std::fs;
use testkit::TempDir;

/// A checkout holding the workload's source files.
fn checkout(files: &[(&str, &str)]) -> TempDir {
    let root = TempDir::new().unwrap();
    let dir = root.path().join(SOURCE_DIR);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    root
}

#[test]
fn fnv1a_is_the_published_hash() {
    assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
}

#[test]
fn the_source_hash_covers_names_contents_and_order_of_names() {
    let base = source_hash(checkout(&[("a.rs", "x"), ("b.rs", "y")]).path()).unwrap();

    let made_in_reverse = checkout(&[("b.rs", "y"), ("a.rs", "x")]);
    assert_eq!(source_hash(made_in_reverse.path()).unwrap(), base);
    let edited = checkout(&[("a.rs", "x"), ("b.rs", "z")]);
    assert_ne!(source_hash(edited.path()).unwrap(), base);
    let renamed = checkout(&[("a.rs", "x"), ("c.rs", "y")]);
    assert_ne!(source_hash(renamed.path()).unwrap(), base);
    let swapped = checkout(&[("a.rs", "y"), ("b.rs", "x")]);
    assert_ne!(source_hash(swapped.path()).unwrap(), base);
    let extra = checkout(&[("a.rs", "x"), ("b.rs", "y"), ("c.rs", "")]);
    assert_ne!(source_hash(extra.path()).unwrap(), base);
}

#[test]
fn a_source_directory_that_is_not_there_is_an_error() {
    let root = TempDir::new().unwrap();

    let err = source_hash(root.path()).unwrap_err();

    assert!(err.contains(SOURCE_DIR), "{err}");
}

#[test]
fn a_workload_that_matches_the_record_has_no_problem() {
    let root = checkout(&[("a.rs", "x")]);
    let hash = source_hash(root.path()).unwrap();

    assert!(problems(root.path(), hash, RECORDED_CONSTANTS).is_empty());
}

#[test]
fn a_source_that_differs_from_the_record_is_named_with_both_hashes() {
    let root = checkout(&[("a.rs", "x")]);
    let hash = source_hash(root.path()).unwrap();

    let found = problems(root.path(), hash ^ 1, RECORDED_CONSTANTS);

    assert_eq!(
        found,
        [format!(
            "the calibration workload's source hashes to {hash:016x}, not the recorded {:016x}: \
             the workload is frozen, and one calibration unit is 10 ms by definition",
            hash ^ 1
        )]
    );
}

#[test]
fn each_constant_that_differs_from_the_record_is_named() {
    let root = checkout(&[("a.rs", "x")]);
    let hash = source_hash(root.path()).unwrap();
    let (seed, mixing, files) = RECORDED_CONSTANTS;

    let found = problems(root.path(), hash, (seed + 1, mixing + 1, files + 1));

    assert_eq!(
        found,
        [
            format!(
                "the calibration seed is {:#x}, not the recorded {seed:#x}",
                seed + 1
            ),
            format!(
                "the calibration mixing rounds are {}, not the recorded {mixing}",
                mixing + 1
            ),
            format!(
                "the calibration file rounds are {}, not the recorded {files}",
                files + 1
            ),
        ]
    );
}

#[test]
fn an_unreadable_source_is_a_problem() {
    let root = TempDir::new().unwrap();

    let found = problems(root.path(), 0, RECORDED_CONSTANTS);

    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains(SOURCE_DIR), "{found:?}");
}

#[test]
fn the_checkout_s_workload_is_the_recorded_one() {
    assert_eq!(frozen(&testkit::repo_root()), Vec::<String>::new());
}
