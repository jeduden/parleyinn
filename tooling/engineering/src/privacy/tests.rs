use super::*;
use testkit::TempDir;

fn checkout(files: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for (path, body) in files {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    }
    let c = Checkout::new(dir.path());

    (dir, c)
}

const TESTS: &str = "\
#[test]
fn it_holds() {}

#[test]
#[ignore = \"slow\"]
fn it_is_skipped() {}

fn not_a_test() {}
";

#[test]
fn a_statement_is_a_block_with_a_phrase_and_a_code_fence_is_not_prose() {
    let body = "Intro.\nIt sends NO data: telemetry never ships\nand (test: `a_test`).\n\n```\nno network here\n```\n| Row | no outbound | test: `b` |\n| ok | fine |\nplain\n";

    let found = statements("a.md", body);

    assert_eq!(
        found,
        [
            Statement {
                file: "a.md".to_owned(),
                line: 1,
                cites: vec!["a_test".to_owned()]
            },
            Statement {
                file: "a.md".to_owned(),
                line: 8,
                cites: vec!["b".to_owned()]
            },
        ]
    );
}

#[test]
fn citations_follow_test_or_tests_joined_by_commas_and_and() {
    assert_eq!(
        cites("x test: `a`, `b` and `@SEC-15`."),
        ["a", "b", "@SEC-15"]
    );
    assert_eq!(cites("tests:\n`a`,\n`b`"), ["a", "b"]);
    assert_eq!(cites("tests: `a`, and `b`; test: `c`"), ["a", "b", "c"]);
    assert_eq!(cites("test: `a` then `code`"), ["a"]);
    assert_eq!(cites("test: none, `code`"), Vec::<String>::new());
    assert_eq!(cites("no citation, `code`"), Vec::<String>::new());
}

#[test]
fn a_citation_resolves_to_a_running_test_or_a_scenario_that_is_not_pending() {
    let (_dir, c) = checkout(&[
        ("crates/t.rs", TESTS),
        ("tooling/u/tests/x.rs", "#[test]\nfn other() {}\n"),
        ("crates/notes.txt", "x"),
    ]);
    let tests = test_functions(&c).unwrap();
    let scenarios = vec![("SEC-15".to_owned(), false), ("SEC-01".to_owned(), true)];

    assert_eq!(
        tests,
        [
            ("it_holds".to_owned(), true),
            ("it_is_skipped".to_owned(), false),
            ("other".to_owned(), true),
        ]
    );
    assert!(resolves(&tests, &scenarios, "it_holds"));
    assert!(!resolves(&tests, &scenarios, "it_is_skipped"));
    assert!(!resolves(&tests, &scenarios, "not_a_test"));
    assert!(resolves(&tests, &scenarios, "@SEC-15"));
    assert!(!resolves(&tests, &scenarios, "@SEC-01"));
    assert!(!resolves(&tests, &scenarios, "@NOPE-1"));
}

#[test]
fn a_directory_or_file_that_cannot_be_read_is_named() {
    let (_dir, c) = checkout(&[("README.md", "")]);
    assert!(test_functions(&c).unwrap_err().starts_with("list "));

    let (_dir, c) = checkout(&[("crates/t.rs", TESTS), ("tooling/x.txt", "")]);
    std::os::unix::fs::symlink("/nonexistent", c.root.join("crates/dangling.rs")).unwrap();
    assert!(test_functions(&c).unwrap_err().starts_with("read "));
}

#[test]
fn documentation_is_security_md_and_the_markdown_in_the_named_directories() {
    let (dir, c) = checkout(&[
        ("SECURITY.md", "no network"),
        ("docs/a.md", "x"),
        ("docs/b.txt", "x"),
        ("docs/security/t.md", "x"),
        ("docs/srs/s.md", "no network"),
    ]);

    assert_eq!(
        documentation(&c),
        ["SECURITY.md", "docs/a.md", "docs/security/t.md"]
    );
    assert_eq!(all(&c).unwrap().len(), 1);
    assert!(all(&Checkout::new(&dir.path().join("none"))).is_err());
}

#[test]
fn every_statement_must_cite_a_test_that_exists() {
    let (dir, c) = checkout(&[
        ("crates/t.rs", TESTS),
        ("tooling/x.txt", ""),
        (
            "SECURITY.md",
            "ok: no network (test: `it_holds`)\n\nbare: sends no data\n\nbad: telemetry (test: `missing`)\n",
        ),
    ]);

    let problems = proven(&c, &[]);

    assert_eq!(
        problems,
        [
            "SECURITY.md:3: a privacy statement cites no test",
            "SECURITY.md:5: cites missing, which is no test that exists and runs",
        ]
    );
    let none = Checkout::new(&dir.path().join("none"));
    assert!(proven(&none, &[])[0].starts_with("read SECURITY.md"));
}

#[test]
fn the_scenarios_are_read_with_their_pending_state() {
    let states = scenario_states(&testkit::repo_root().join("features")).unwrap();

    assert!(states.iter().any(|(id, _)| id == "OPS-02"));
    assert!(scenario_states(&testkit::repo_root().join("nowhere")).is_err());
}

#[test]
fn the_repository_documentation_cites_its_tests() {
    let c = Checkout::new(&testkit::repo_root());
    let states = scenario_states(&c.root.join("features")).unwrap();

    assert_eq!(proven(&c, &states), Vec::<String>::new());
}
