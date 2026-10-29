use testkit::TempDir;

use super::*;
use crate::case::{Edit, Expect, Expected};
use crate::world::tests::repository;

/// The repository of the world tests, with the agents every component
/// of it needs and a written case directory.
fn checkout() -> TempDir {
    let dir = TempDir::new().unwrap();
    repository(dir.path());
    let write = |rel: &str, body: &str| {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    };
    let names = component::names(dir.path()).unwrap();
    for name in names.iter().filter(|n| *n != "review") {
        let (tools, model, turns) = if name.starts_with("review-") {
            ("Read, Grep, Glob", "sonnet", "")
        } else {
            ("Read, Grep, Glob, Write", "haiku", "maxTurns: 20\n")
        };
        write(
            &format!(".claude/agents/{name}.md"),
            &format!(
                "---\nname: {name}\ndescription: d\ntools: {tools}\nmodel: {model}\neffort: low\n{turns}---\n# {name}\n"
            ),
        );
    }
    dir
}

const UNWRAP: &str = r#"
component = "lang-rust"
kind = "defect"
title = "an unwrap"

[[create]]
file = "crates/x/src/rogue.rs"
content = "pub fn f() { None::<u8>.unwrap(); }\n"

[expect]
outcome = "blocking"
"#;

fn write_case(dir: &Path, component: &str, name: &str, kind: &str, body: &str) {
    let path = dir
        .join(crate::CASES)
        .join(component)
        .join(format!("{name}.toml"));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        body.replacen("kind = \"defect\"", &format!("kind = \"{kind}\""), 1),
    )
    .unwrap();
}

#[test]
fn a_component_without_its_three_kinds_is_named() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(dir.path(), "lang-rust", "unwrap", "defect", UNWRAP);

    let got = problems(dir.path(), dir.path(), &cases, None);

    assert!(
        got.contains(&"component lang-rust has no clean case".to_owned()),
        "{got:?}"
    );
    assert!(
        got.contains(&"component lang-rust has no injection case".to_owned()),
        "{got:?}"
    );
    assert!(
        !got.contains(&"component lang-rust has no defect case".to_owned()),
        "{got:?}"
    );
    assert!(
        got.contains(&"component review has no defect case".to_owned()),
        "{got:?}"
    );
    // Derived cases cover the section and model specialists.
    assert!(
        !got.iter().any(|p| p.starts_with("component srs-")),
        "{got:?}"
    );
    assert!(
        !got.iter().any(|p| p.starts_with("component domain-model-")),
        "{got:?}"
    );
    // A deep instance has no cases of its own.
    assert!(
        !got.iter().any(|p| p.contains("review-code-deep has no")),
        "{got:?}"
    );
}

#[test]
fn a_case_that_does_not_dispatch_its_component_is_named() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(
        dir.path(),
        "lang-toml",
        "unwrap",
        "defect",
        &UNWRAP.replace("lang-rust", "lang-toml"),
    );
    write_case(
        dir.path(),
        "lang-rust",
        "nothing",
        "clean",
        &UNWRAP.replace("crates/x/src/rogue.rs", "research/note.txt"),
    );

    let got = problems(dir.path(), dir.path(), &cases, None);

    assert!(
        got.contains(&"case lang-toml/unwrap does not dispatch lang-toml; the bundle dispatches lang-rust, performance, review-code, test-engineer-unit".to_owned()),
        "{got:?}"
    );
    assert!(
        got.contains(
            &"case lang-rust/nothing does not dispatch lang-rust; the bundle dispatches no agent"
                .to_owned()
        ),
        "{got:?}"
    );
}

#[test]
fn a_case_of_an_unknown_component_a_deep_instance_or_a_generalist_s_outcome_is_named() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(
        dir.path(),
        "lang-cobol",
        "unwrap",
        "defect",
        &UNWRAP.replace("lang-rust", "lang-cobol"),
    );
    write_case(
        dir.path(),
        "review-code-deep",
        "unwrap",
        "defect",
        &UNWRAP.replace("lang-rust", "review-code-deep"),
    );
    write_case(
        dir.path(),
        "review-code",
        "unwrap",
        "defect",
        &UNWRAP.replace("lang-rust", "review-code"),
    );

    let got = problems(dir.path(), dir.path(), &cases, None);

    assert!(
        got.contains(&"case lang-cobol/unwrap names no component".to_owned()),
        "{got:?}"
    );
    assert!(
        got.contains(
            &"case review-code-deep/unwrap names a deep instance; cases belong to review-code"
                .to_owned()
        ),
        "{got:?}"
    );
    assert!(
        got.contains(
            &"case review-code/unwrap expects an outcome of a generalist, which gives none"
                .to_owned()
        ),
        "{got:?}"
    );
}

#[test]
fn a_case_that_does_not_apply_is_named() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(
        dir.path(),
        "lang-rust",
        "gone",
        "defect",
        &UNWRAP
            .replace("[[create]]", "[[remove]]")
            .replace("content = \"pub fn f() { None::<u8>.unwrap(); }\\n\"\n", ""),
    );

    let got = problems(dir.path(), dir.path(), &cases, None);

    assert!(
        got.contains(&"lang-rust/gone: crates/x/src/rogue.rs: does not exist".to_owned()),
        "{got:?}"
    );
}

#[test]
fn problems_name_what_cannot_be_read() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    fs::create_dir_all(cases.join("lang-rust")).unwrap();
    fs::write(cases.join("lang-rust/bad.toml"), "kind = 1\n").unwrap();
    let got = problems(dir.path(), dir.path(), &cases, None);
    assert_eq!(got.len(), 1);
    assert!(got[0].starts_with("case lang-rust/bad: "), "{got:?}");

    fs::remove_file(cases.join("lang-rust/bad.toml")).unwrap();
    fs::remove_file(dir.path().join(".claude/agents/lang-rust.md")).unwrap();
    let got = problems(dir.path(), dir.path(), &cases, None);
    assert_eq!(got.len(), 1);
    assert!(
        got[0].starts_with("read .claude/agents/lang-rust.md: "),
        "{got:?}"
    );

    let empty = TempDir::new().unwrap();
    let got = problems(empty.path(), empty.path(), &cases, None);
    assert_eq!(got.len(), 1);
    assert!(got[0].starts_with("read docs/srs: "), "{got:?}");
}

#[test]
fn problems_name_a_tree_the_bundle_cannot_read() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    fs::remove_file(dir.path().join("docs/srs/invariants.md")).unwrap();

    let got = problems(dir.path(), dir.path(), &cases, None);

    assert_eq!(got.len(), 1);
    assert!(got[0].starts_with("read the tree: "), "{got:?}");
}

#[test]
fn a_derived_change_to_a_section_is_read_through_a_scratch_copy() {
    let dir = checkout();
    let base = Snapshot::read(dir.path()).unwrap();
    let component = component::load(dir.path(), "srs-05").unwrap();
    let cases = derived::cases(dir.path()).unwrap();
    let row = cases
        .iter()
        .find(|c| c.id() == "srs-05/defect-row-without-scenario")
        .unwrap();

    assert_eq!(dispatches(dir.path(), &base, row, &component), Ok(None));

    let orchestrator = component::load(dir.path(), "review").unwrap();
    assert_eq!(dispatches(dir.path(), &base, row, &orchestrator), Ok(None));
    let generalist = component::load(dir.path(), "review-spec-deep").unwrap();
    assert_eq!(dispatches(dir.path(), &base, row, &generalist), Ok(None));
    let other = component::load(dir.path(), "review-code").unwrap();
    assert!(
        dispatches(dir.path(), &base, row, &other)
            .unwrap()
            .is_some()
    );
}

#[test]
fn a_head_snapshot_fails_when_a_changed_section_does_not_read() {
    let dir = checkout();
    let base = Snapshot::read(dir.path()).unwrap();
    let changes = vec![Change {
        path: "docs/srs/invariants.md".to_owned(),
        old: Some("# 1.3 Invariants\n\nI1 holds.\n".to_owned()),
        new: None,
    }];

    let err = head_snapshot(dir.path(), &base, &changes).unwrap_err();

    assert!(err.starts_with("read the changed tree: "), "{err}");
    let changes = vec![Change {
        path: "features/record.feature".to_owned(),
        old: None,
        new: Some("Feature: broken\n  Scenario: x\n    Given y\n".to_owned()),
    }];
    let err = head_snapshot(dir.path(), &base, &changes).unwrap_err();
    assert!(err.starts_with("read the changed tree: "), "{err}");
    // A section whose path is a directory in the tree cannot be written.
    fs::create_dir_all(dir.path().join("docs/srs/odd.md")).unwrap();
    let changes = vec![Change {
        path: "docs/srs/odd.md".to_owned(),
        old: None,
        new: Some("# odd\n".to_owned()),
    }];
    let err = head_snapshot(dir.path(), &base, &changes).unwrap_err();
    assert!(err.starts_with("write docs/srs/odd.md: "), "{err}");
}

#[test]
fn a_case_whose_edits_do_not_apply_does_not_dispatch() {
    let dir = checkout();
    let base = Snapshot::read(dir.path()).unwrap();
    let component = component::load(dir.path(), "lang-rust").unwrap();
    let case = Case {
        component: "lang-rust".to_owned(),
        name: "x".to_owned(),
        kind: "defect".to_owned(),
        title: "x".to_owned(),
        consent: None,
        edits: vec![Edit::Remove {
            file: "crates/x/src/none.rs".to_owned(),
        }],
        seeds: Vec::new(),
        expect: Expect {
            finding: vec![Expected::default()],
            ..Expect::default()
        },
        repetitions: 3,
        passes: 2,
    };

    assert_eq!(
        dispatches(dir.path(), &base, &case, &component).unwrap_err(),
        "lang-rust/x: crates/x/src/none.rs: does not exist"
    );
}

#[test]
fn copy_path_copies_files_and_directories_and_skips_what_is_missing() {
    let dir = TempDir::new().unwrap();
    let from = dir.path().join("from");
    fs::create_dir_all(from.join("sub")).unwrap();
    fs::write(from.join("sub/a.txt"), "a").unwrap();
    fs::write(from.join("b.txt"), "b").unwrap();
    let to = dir.path().join("to");

    copy_path(&from, &to).unwrap();
    copy_path(&dir.path().join("none"), &dir.path().join("none-copy")).unwrap();

    assert_eq!(fs::read_to_string(to.join("sub/a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(to.join("b.txt")).unwrap(), "b");
    assert!(!dir.path().join("none-copy").exists());
    let err = copy_path(&from.join("b.txt"), &to).unwrap_err();
    assert!(err.starts_with("copy "), "{err}");
    let err = copy_path(&from.join("b.txt"), &to.join("b.txt/under")).unwrap_err();
    assert!(err.starts_with("copy "), "{err}");
}

#[test]
fn all_cases_join_the_written_and_the_derived_in_order() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(dir.path(), "lang-rust", "unwrap", "defect", UNWRAP);

    let got = all_cases(dir.path(), &cases).unwrap();

    let ids: Vec<String> = got.iter().map(Case::id).collect();
    assert!(ids.contains(&"lang-rust/unwrap".to_owned()));
    assert!(ids.contains(&"srs-05/clean-sentence".to_owned()));
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
    assert!(
        all_cases(dir.path(), &dir.path().join("docs/srs/05.md"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn a_selection_checks_only_its_suites() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(dir.path(), "lang-rust", "unwrap", "defect", UNWRAP);
    write_case(dir.path(), "lang-rust", "clean", "clean", UNWRAP);
    write_case(dir.path(), "lang-rust", "note", "injection", UNWRAP);
    let rust = ["lang-rust".to_owned()];
    let deep = ["review-code-deep".to_owned()];

    assert_eq!(
        problems(dir.path(), dir.path(), &cases, Some(&rust)),
        Vec::<String>::new()
    );
    assert_eq!(
        problems(dir.path(), dir.path(), &cases, Some(&deep)),
        vec![
            "component review-code has no clean case".to_owned(),
            "component review-code has no defect case".to_owned(),
            "component review-code has no injection case".to_owned(),
        ]
    );
    let got = problems(
        dir.path(),
        dir.path(),
        &cases,
        Some(&["lang-cobol".to_owned()]),
    );
    assert_eq!(got.len(), 1);
    assert!(
        got[0].starts_with("read .claude/agents/lang-cobol.md: "),
        "{got:?}"
    );
}

#[test]
fn problems_come_in_kinds_and_cases() {
    let dir = checkout();
    let cases = dir.path().join(crate::CASES);
    write_case(dir.path(), "lang-rust", "unwrap", "defect", UNWRAP);
    write_case(
        dir.path(),
        "lang-toml",
        "stray",
        "defect",
        &UNWRAP.replace("lang-rust", "lang-toml"),
    );

    let examined = examine(dir.path(), dir.path(), &cases, None).unwrap();

    assert!(
        examined
            .kinds
            .contains(&"component lang-rust has no clean case".to_owned())
    );
    assert!(
        examined
            .cases
            .iter()
            .any(|c| c.starts_with("case lang-toml/stray does not dispatch"))
    );
    assert_eq!(
        examined.all().len(),
        examined.kinds.len() + examined.cases.len()
    );
    assert_eq!(
        kind_problems(dir.path(), dir.path(), &cases),
        examined.kinds
    );
    assert_eq!(
        case_problems(dir.path(), dir.path(), &cases),
        examined.cases
    );
    let empty = TempDir::new().unwrap();
    assert!(kind_problems(empty.path(), empty.path(), &cases)[0].starts_with("read docs/srs: "));
    assert!(case_problems(empty.path(), empty.path(), &cases)[0].starts_with("read docs/srs: "));
}
