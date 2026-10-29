use std::fs;

use testkit::TempDir;

use super::*;

const MINIMAL: &str = r#"
component = "lang-rust"
kind = "defect"
title = "an unwrap"

[[create]]
file = "crates/x/src/rogue.rs"
content = "fn f() { None::<u8>.unwrap(); }\n"

[expect]
outcome = "blocking"

[[expect.finding]]
path = "crates/x/src/rogue.rs"
severity = "blocking"
mentions = ["unwrap"]
"#;

#[test]
fn a_minimal_case_parses_with_the_default_repetition_rule() {
    let case = Case::parse("lang-rust", "unwrap", MINIMAL).unwrap();

    assert_eq!(case.id(), "lang-rust/unwrap");
    assert_eq!(case.kind, "defect");
    assert_eq!(case.title, "an unwrap");
    assert_eq!(case.consent, None);
    assert_eq!(case.edits.len(), 1);
    assert_eq!(case.edits[0].file(), "crates/x/src/rogue.rs");
    assert_eq!(case.expect.outcome, vec!["blocking"]);
    assert_eq!(case.expect.finding[0].mentions, vec!["unwrap"]);
    assert_eq!((case.repetitions, case.passes), (REPETITIONS, PASSES));
}

#[test]
fn a_full_case_parses_every_field() {
    let text = r#"
component = "review-code"
kind = "other"
title = "everything"
consent = "I consent."

[runs]
repetitions = 5
passes = 4

[[replace]]
file = "a.txt"
old = "x"
new = "y"

[[remove]]
file = "b.txt"

[[create]]
file = "c.txt"
content = "c"

[[report]]
agent = "lang-rust"
content = "outcome: sound"

[expect]
outcome = ["sound", "nits"]
absent = ["approved"]
consulted = ["lang-rust"]

[[expect.none]]
path = "a.txt"

[[expect.dropped]]
path = "a.txt"
mentions = ["false alarm"]
"#;

    let case = Case::parse("review-code", "all", text).unwrap();

    assert_eq!(case.consent.as_deref(), Some("I consent."));
    assert_eq!((case.repetitions, case.passes), (5, 4));
    // Creates first, then replacements, then removals.
    assert_eq!(
        case.edits.iter().map(Edit::file).collect::<Vec<_>>(),
        vec!["c.txt", "a.txt", "b.txt"]
    );
    assert_eq!(case.seeds[0].agent, "lang-rust");
    assert_eq!(case.expect.outcome, vec!["sound", "nits"]);
    assert_eq!(case.expect.absent, vec!["approved"]);
    assert_eq!(case.expect.consulted, vec!["lang-rust"]);
    assert_eq!(case.expect.none[0].path, "a.txt");
    assert_eq!(case.expect.dropped[0].mentions, vec!["false alarm"]);
}

#[test]
fn parse_names_each_problem() {
    let replace = |from: &str, to: &str| MINIMAL.replacen(from, to, 1);

    assert_eq!(
        Case::parse("lang-toml", "unwrap", MINIMAL).unwrap_err(),
        "case lang-toml/unwrap: names component lang-rust, but lies under lang-toml"
    );
    assert_eq!(
        Case::parse("lang-rust", "x", &replace("defect", "bug")).unwrap_err(),
        "case lang-rust/x: kind bug is none of defect, clean, injection, other"
    );
    assert_eq!(
        Case::parse("lang-rust", "x", &replace("an unwrap", " ")).unwrap_err(),
        "case lang-rust/x: has no title"
    );
    assert_eq!(
        Case::parse("lang-rust", "x", &replace("\"blocking\"\n", "\"maybe\"\n")).unwrap_err(),
        "case lang-rust/x: outcome maybe is none of sound, nits, blocking, approve, request_changes"
    );
    let no_edit = MINIMAL.replacen(
        "[[create]]\nfile = \"crates/x/src/rogue.rs\"\ncontent = \"fn f() { None::<u8>.unwrap(); }\\n\"\n",
        "",
        1,
    );
    assert_eq!(
        Case::parse("lang-rust", "x", &no_edit).unwrap_err(),
        "case lang-rust/x: makes no edit"
    );
    let bad_rule = format!("{MINIMAL}\n[runs]\nrepetitions = 2\npasses = 3\n");
    assert_eq!(
        Case::parse("lang-rust", "x", &bad_rule).unwrap_err(),
        "case lang-rust/x: must pass 3 of 2 runs, which is no rule"
    );
    let only_repetitions = Case::parse(
        "lang-rust",
        "x",
        &format!("{MINIMAL}\n[runs]\nrepetitions = 5\n"),
    )
    .unwrap();
    assert_eq!(
        (only_repetitions.repetitions, only_repetitions.passes),
        (5, PASSES)
    );
    let zero = format!("{MINIMAL}\n[runs]\npasses = 0\n");
    assert_eq!(
        Case::parse("lang-rust", "x", &zero).unwrap_err(),
        "case lang-rust/x: must pass 0 of 3 runs, which is no rule"
    );
    let unknown = Case::parse("lang-rust", "x", &format!("{MINIMAL}\nnope = 1\n")).unwrap_err();
    assert!(unknown.starts_with("case lang-rust/x: "), "{unknown}");
    assert!(unknown.contains("nope"), "{unknown}");
}

fn tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs/a.md"), "# A\n\nold text\n").unwrap();
    fs::write(dir.path().join("docs/b.md"), "# B\n").unwrap();
    dir
}

fn case_with(edits: Vec<Edit>) -> Case {
    let mut case = Case::parse("lang-rust", "x", MINIMAL).unwrap();
    case.edits = edits;
    case
}

#[test]
fn changes_apply_every_edit_in_order_on_a_tree() {
    let dir = tree();
    let case = case_with(vec![
        Edit::Create {
            file: "docs/c.md".into(),
            content: "# C\n".into(),
        },
        Edit::Replace {
            file: "docs/a.md".into(),
            old: "old".into(),
            new: "new".into(),
        },
        Edit::Replace {
            file: "docs/a.md".into(),
            old: "new text".into(),
            new: "newer text".into(),
        },
        Edit::Remove {
            file: "docs/b.md".into(),
        },
    ]);

    assert_eq!(case.validate(dir.path()), Ok(()));
    let changes = case.apply(dir.path()).unwrap();

    assert_eq!(
        changes,
        vec![
            Change {
                path: "docs/a.md".into(),
                old: Some("# A\n\nold text\n".into()),
                new: Some("# A\n\nnewer text\n".into()),
            },
            Change {
                path: "docs/b.md".into(),
                old: Some("# B\n".into()),
                new: None,
            },
            Change {
                path: "docs/c.md".into(),
                old: None,
                new: Some("# C\n".into()),
            },
        ]
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("docs/a.md")).unwrap(),
        "# A\n\nnewer text\n"
    );
    assert!(!dir.path().join("docs/b.md").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("docs/c.md")).unwrap(),
        "# C\n"
    );
}

#[test]
fn changes_refuse_an_edit_that_does_not_apply() {
    let dir = tree();
    let fails = |edit: Edit| case_with(vec![edit]).validate(dir.path()).unwrap_err();

    assert_eq!(
        fails(Edit::Create {
            file: "docs/a.md".into(),
            content: String::new(),
        }),
        "lang-rust/x: docs/a.md: exists already"
    );
    assert_eq!(
        fails(Edit::Replace {
            file: "docs/a.md".into(),
            old: "gone".into(),
            new: String::new(),
        }),
        "lang-rust/x: docs/a.md: does not contain \"gone\""
    );
    assert_eq!(
        fails(Edit::Replace {
            file: "docs/a.md".into(),
            old: String::new(),
            new: String::new(),
        }),
        "lang-rust/x: docs/a.md: does not contain \"\""
    );
    assert_eq!(
        fails(Edit::Replace {
            file: "docs/z.md".into(),
            old: "x".into(),
            new: String::new(),
        }),
        "lang-rust/x: docs/z.md: does not exist"
    );
    assert_eq!(
        fails(Edit::Remove {
            file: "docs/z.md".into(),
        }),
        "lang-rust/x: docs/z.md: does not exist"
    );
}

#[test]
fn apply_names_a_file_it_cannot_write() {
    let dir = tree();
    // A directory where the file should go.
    fs::create_dir_all(dir.path().join("docs/d.md")).unwrap();
    let case = case_with(vec![Edit::Create {
        file: "docs/d.md".into(),
        content: "x".into(),
    }]);

    let err = case.apply(dir.path()).unwrap_err();

    assert!(err.starts_with("lang-rust/x: docs/d.md: "), "{err}");
    // A file where the directory should be.
    let under_file = case_with(vec![Edit::Create {
        file: "docs/a.md/under.md".into(),
        content: "x".into(),
    }]);
    let err = under_file.apply(dir.path()).unwrap_err();
    assert!(
        err.starts_with("lang-rust/x: docs/a.md/under.md: "),
        "{err}"
    );
}

#[test]
fn load_dir_reads_each_component_s_cases_in_order() {
    let dir = TempDir::new().unwrap();
    let cases = dir.path().join("cases");
    fs::create_dir_all(cases.join("lang-rust")).unwrap();
    fs::write(cases.join("lang-rust/b-unwrap.toml"), MINIMAL).unwrap();
    fs::write(
        cases.join("lang-rust/a-unwrap.toml"),
        MINIMAL.replacen("an unwrap", "another", 1),
    )
    .unwrap();
    fs::write(cases.join("lang-rust/notes.txt"), "not a case").unwrap();
    fs::write(cases.join("README"), "not a component").unwrap();

    let got = load_dir(&cases).unwrap();

    assert_eq!(
        got.iter().map(Case::id).collect::<Vec<_>>(),
        vec!["lang-rust/a-unwrap", "lang-rust/b-unwrap"]
    );
    assert_eq!(got[0].title, "another");
}

#[test]
fn load_dir_reads_nothing_from_a_missing_directory() {
    let dir = TempDir::new().unwrap();

    assert_eq!(load_dir(&dir.path().join("none")), Ok(Vec::new()));
}

#[test]
fn load_dir_names_a_case_that_does_not_parse() {
    let dir = TempDir::new().unwrap();
    let cases = dir.path().join("cases");
    fs::create_dir_all(cases.join("lang-rust")).unwrap();
    fs::write(cases.join("lang-rust/bad.toml"), "component = 1\n").unwrap();

    let err = load_dir(&cases).unwrap_err();

    assert!(err.starts_with("case lang-rust/bad: "), "{err}");
}

#[test]
fn load_dir_names_a_component_directory_it_cannot_read() {
    let dir = TempDir::new().unwrap();
    let cases = dir.path().join("cases");
    fs::create_dir_all(&cases).unwrap();
    fs::write(cases.join("file-not-dir"), "x").unwrap();

    // A plain file under the directory is skipped; a directory that
    // cannot be listed is named.
    assert_eq!(load_dir(&cases), Ok(Vec::new()));
    let err = load_dir(&cases.join("file-not-dir")).unwrap_err();
    assert!(err.starts_with("read "), "{err}");
}
