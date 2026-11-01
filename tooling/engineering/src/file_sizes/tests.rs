use super::*;

fn sizes(list: &[(&str, usize)]) -> Sizes {
    list.iter().map(|(p, n)| ((*p).to_owned(), *n)).collect()
}

#[test]
fn parse_record_reads_a_path_and_its_lines_a_line() {
    let (got, problems) = parse_record("# why\n\n  a.rs 1200  \nb.yml 600\n");

    assert_eq!(got, sizes(&[("a.rs", 1200), ("b.yml", 600)]));
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn parse_record_names_a_line_it_cannot_read_and_a_path_listed_twice() {
    let (got, problems) = parse_record("a.rs\nb.rs many\nc.rs 1\nc.rs 2\nd.rs 1 2\n");

    assert_eq!(got, sizes(&[("c.rs", 2)]));
    assert_eq!(
        problems,
        [
            format!("{RECORD}:1: want `<path> <lines>`, got \"a.rs\""),
            format!("{RECORD}:2: \"many\" is no line count"),
            format!("{RECORD}:4: c.rs is listed twice"),
            format!("{RECORD}:5: want `<path> <lines>`, got \"d.rs 1 2\""),
        ]
    );
}

#[test]
fn compare_names_a_file_past_its_limit_that_the_record_does_not_list() {
    let got = compare(&sizes(&[("a.rs", 1001), ("b.rs", 1000)]), &Sizes::new());

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].starts_with("a.rs holds 1001 lines, past the limit of 1000 for Rust"));
    assert!(got[0].ends_with("— ENG-34"), "{got:?}");
}

#[test]
fn compare_names_a_listed_file_that_grew_but_not_one_held_or_shrunk() {
    let record = sizes(&[("a.rs", 1100), ("b.rs", 1100), ("c.rs", 1100)]);

    let got = compare(
        &sizes(&[("a.rs", 1101), ("b.rs", 1100), ("c.rs", 1050)]),
        &record,
    );

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].starts_with("a.rs grew from its recorded 1100 lines to 1101"));
}

#[test]
fn compare_asks_for_a_listed_file_that_fits_or_is_gone_to_leave_the_record() {
    let record = sizes(&[("a.rs", 1100), ("b.rs", 1100), ("README.md", 9)]);

    let got = compare(&sizes(&[("a.rs", 1000), ("README.md", 9)]), &record);

    assert_eq!(got.len(), 3, "{got:?}");
    assert!(got.contains(&format!(
        "a.rs now fits its limit with 1000 lines: remove it from {RECORD} — ENG-34"
    )));
    assert!(got.contains(&format!(
        "b.rs is listed in {RECORD} but is gone or has no limit: remove it — ENG-34"
    )));
    assert!(got[0].starts_with("README.md now fits"), "{got:?}");
}

#[test]
fn compare_leaves_a_file_without_a_limit_alone() {
    assert!(compare(&sizes(&[("notes.md", 5000)]), &Sizes::new()).is_empty());
}

#[test]
fn measure_reads_the_lines_of_each_file_with_a_limit_outside_what_is_not_searched() {
    let root = testkit::TempDir::new().unwrap();
    for (rel, body) in [
        ("a.rs", "x\ny\n"),
        ("sub/b.sh", "z"),
        ("notes.md", "no limit\n"),
        ("target/c.rs", "build output\n"),
        ("fuzz/target/d.rs", "build output\n"),
        (".git/e.rs", "git\n"),
        (".claude/worktrees/w/f.rs", "another checkout\n"),
        ("Cargo.lock", "left out\n"),
    ] {
        let path = root.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    assert_eq!(
        measure(root.path()).unwrap(),
        sizes(&[("a.rs", 2), ("sub/b.sh", 1)])
    );
}

#[test]
fn measure_fails_on_a_tree_it_cannot_read() {
    let root = testkit::TempDir::new().unwrap();
    let err = measure(&root.path().join("missing")).unwrap_err();

    assert!(err.starts_with("read "), "{err}");
}

#[test]
fn check_reads_the_record_against_the_files() {
    let root = testkit::TempDir::new().unwrap();
    let write = |rel: &str, body: &str| {
        let path = root.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    let checkout = Checkout::new(root.path());
    assert!(check(&checkout)[0].starts_with(&format!("read {RECORD}")));

    write(RECORD, "bad\n");
    write("a.rs", &"x\n".repeat(1001));
    let got = check(&checkout);
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got[0].contains("want `<path> <lines>`"));
    assert!(got[1].starts_with("a.rs holds 1001 lines"));

    write(RECORD, "a.rs 1001\n");
    assert!(check(&checkout).is_empty());
}

#[test]
fn the_fixture_gives_each_drift_its_problem_and_the_held_files_none() {
    let work = testkit::TempDir::new().unwrap();

    let found = fixture_problems(work.path()).unwrap();

    assert_eq!(found.len(), 4, "{found:?}");
    assert!(names_each_drift(&found).is_empty());
    assert!(names_no_held_file(&found).is_empty());
}

#[test]
fn the_fixture_s_checks_name_what_the_check_missed_and_what_it_wrongly_named() {
    let found = [format!("{GROWN} grew"), format!("{HELD} grew")];

    let missed = names_each_drift(&found);
    let wrong = names_no_held_file(&found);

    assert_eq!(missed.len(), 3, "{missed:?}");
    assert!(missed[0].starts_with(&format!("the size check does not name {UNLISTED}")));
    assert_eq!(
        wrong,
        [format!(
            "the size check names {HELD}, which is within its record"
        )]
    );
}

#[test]
fn check_parts_keep_a_stale_record_apart_from_a_file_past_its_limit() {
    let root = testkit::TempDir::new().unwrap();
    let record = root.path().join(RECORD);
    fs::create_dir_all(record.parent().unwrap()).unwrap();
    fs::write(root.path().join("a.rs"), "x\n".repeat(1001)).unwrap();
    fs::write(record, "b.rs 1100\n").unwrap();

    let (past, stale) = check_parts(&Checkout::new(root.path()));

    assert!(past[0].starts_with("a.rs holds 1001 lines"), "{past:?}");
    assert!(stale[0].starts_with("b.rs is listed"), "{stale:?}");
}

#[test]
fn a_tree_that_cannot_be_measured_is_a_problem_of_the_check() {
    let root = testkit::TempDir::new().unwrap();
    let record = root.path().join(RECORD);
    fs::create_dir_all(record.parent().unwrap()).unwrap();
    fs::write(record, "").unwrap();

    let (past, stale) = check_with(&Checkout::new(root.path()), |_| Err("no tree".to_owned()));

    assert_eq!(past, ["no tree"]);
    assert!(stale.is_empty());
}

#[test]
fn a_fixture_that_cannot_be_written_is_named() {
    let work = testkit::TempDir::new().unwrap();
    fs::write(work.path().join("crates"), "a file where a directory goes").unwrap();

    assert!(
        fixture_problems(work.path())
            .unwrap_err()
            .starts_with("write ")
    );
}
