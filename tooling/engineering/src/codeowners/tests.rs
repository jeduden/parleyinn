use super::*;

const INDEX: &str = "| **Stakeholder** | @boss, also the security reviewer |\n";

const ENG: &str = "\
| ID | Pri | Requirement | Ver |
|---|---|---|---|
| ENG-21 | P0 | Changes to the specification (`docs/srs/` and the decision records), to the scenarios (`features/`), or to the process (CI, the build, lint and coverage configuration, the agent instructions and `CODEOWNERS` itself) need the stakeholder. | I |
";

const OWNERS: &str = "\
# a comment
/docs/srs/        @boss
/docs/adr/        @boss
/features/        @boss
/crates/*/clippy.toml @boss
/crates/*/Cargo.toml  @boss
/tooling/*/Cargo.toml @boss
/.github/         @boss # trailing
/scripts/         @boss
/Cargo.toml       @boss
/CLAUDE.md        @boss
/.claude/         @boss
/SECURITY.md      @boss
";

const FILES: &[&str] = &[
    "features/a.feature",
    "tooling/x/Cargo.toml",
    "tooling/x/src/lib.rs",
    "crates/pin/src/lib.rs",
    "scripts/a.sh",
    "Cargo.toml",
    "CLAUDE.md",
    ".claude/a",
    "crates/zed/clippy.toml",
    "crates/zed/Cargo.toml",
];

/// A checkout with the SRS pieces, `owners` as CODEOWNERS and `FILES`.
fn repo(owners: &str) -> (testkit::TempDir, Checkout) {
    let mut files = vec![
        ("docs/srs/index.md", INDEX),
        ("docs/srs/10.md", ENG),
        (FILE, owners),
    ];
    files.extend(FILES.iter().map(|f| (*f, "x")));
    crate::tests::checkout(&files)
}

fn problems(owners: &str) -> Vec<String> {
    stakeholder_owns_the_enforcing_paths(&repo(owners).1)
}

fn has(problems: &[String], want: &str) {
    assert!(
        problems.iter().any(|p| p.contains(want)),
        "{want:?} not in {problems:#?}"
    );
}

fn entry(pattern: &str, owner: &str) -> Entry {
    Entry {
        pattern: pattern.to_owned(),
        owners: vec![owner.to_owned()],
    }
}

#[test]
fn parse_skips_comments_and_cuts_a_trailing_comment() {
    let entries = parse("# c\n\n/a/ @x @y # t\n/b\n");

    assert_eq!(
        entries,
        [
            Entry {
                pattern: "/a/".to_owned(),
                owners: vec!["@x".to_owned(), "@y".to_owned()]
            },
            Entry {
                pattern: "/b".to_owned(),
                owners: vec![]
            }
        ]
    );
}

#[test]
fn the_last_matching_entry_owns_a_path() {
    let entries = [entry("/docs/", "@a"), entry("/docs/srs/", "@b")];

    assert_eq!(owners_of(&entries, "docs/srs/x.md"), ["@b"]);
    assert_eq!(owners_of(&entries, "docs/other.md"), ["@a"]);
    assert_eq!(owners_of(&entries, "src/x.rs"), [] as [String; 0]);
}

#[test]
fn patterns_anchor_name_and_glob_the_way_codeowners_does() {
    for (pattern, path, want) in [
        ("/docs/srs/", "docs/srs/", true),
        ("/docs/srs/", "docs/srs/a/b.md", true),
        ("/docs/srs/", "docs/", false),
        ("/docs/srs/", "x/docs/srs/a", false),
        ("/Cargo.toml", "Cargo.toml", true),
        ("/Cargo.toml", "crates/a/Cargo.toml", false),
        ("Cargo.toml", "crates/a/Cargo.toml", true),
        ("*.md", "docs/a.md", true),
        ("*.md", "docs/a.rs", false),
        ("/crates/*/clippy.toml", "crates/a/clippy.toml", true),
        ("/crates/*/clippy.toml", "crates/a/b/clippy.toml", false),
        ("/a/?.rs", "a/b.rs", true),
        ("/a/?.rs", "a/bc.rs", false),
    ] {
        assert_eq!(matches(pattern, path), want, "{pattern} {path}");
    }
}

#[test]
fn the_stakeholder_comes_from_the_stakeholder_row() {
    let (_d, c) = repo(OWNERS);
    assert_eq!(stakeholder(&c).unwrap(), "@boss");

    let (_d, c) = crate::tests::checkout(&[("docs/srs/index.md", "| **Stakeholder** | @boss |\n")]);
    assert_eq!(stakeholder(&c).unwrap(), "@boss");

    let (_d, c) =
        crate::tests::checkout(&[("docs/srs/index.md", "| **Stakeholder** | nobody |\n")]);
    assert_eq!(
        stakeholder(&c).unwrap_err(),
        "the Stakeholder row names no user"
    );

    let (_d, c) = crate::tests::checkout(&[("docs/srs/index.md", "x\n")]);
    assert_eq!(
        stakeholder(&c).unwrap_err(),
        "docs/srs/index.md has no Stakeholder row"
    );
}

#[test]
fn a_conforming_file_has_no_problem() {
    assert_eq!(problems(OWNERS), Vec::<String>::new());
    assert_eq!(
        no_owner_on_any_other_path(&repo(OWNERS).1),
        Vec::<String>::new()
    );
    assert_eq!(
        no_owner_on_rust_source(&repo(OWNERS).1),
        Vec::<String>::new()
    );
}

#[test]
fn an_enforcing_path_without_the_stakeholder_is_named_with_its_class() {
    let owners = OWNERS.replace("/CLAUDE.md        @boss\n", "");
    has(
        &problems(&owners),
        "CLAUDE.md (the agent instructions) is owned by [], want @boss",
    );

    let owners = OWNERS.replace("/docs/srs/        @boss", "/docs/srs/        @boss @other");
    has(
        &problems(&owners),
        "docs/srs/ (`docs/srs/`) is owned by [@boss @other], want @boss",
    );

    let owners = OWNERS.replace(
        "/crates/*/clippy.toml @boss",
        "/crates/*/clippy.toml @other",
    );
    has(&problems(&owners), "crates/zed/clippy.toml");

    let owners = OWNERS.replace("/features/        @boss\n", "");
    has(
        &problems(&owners),
        "features/ (the scenarios (`features/`)) is owned by [], want @boss",
    );

    let owners = OWNERS.replace("/tooling/*/Cargo.toml @boss\n", "");
    has(
        &problems(&owners),
        "tooling/x/Cargo.toml (the build, lint and coverage configuration) is owned by []",
    );
}

#[test]
fn a_class_the_row_stops_naming_is_reported() {
    let (_d, c) = crate::tests::checkout(&[
        ("docs/srs/index.md", INDEX),
        (
            "docs/srs/10.md",
            ENG.replace("the agent instructions", "the rules").as_str(),
        ),
        (FILE, OWNERS),
    ]);

    has(
        &stakeholder_owns_the_enforcing_paths(&c),
        "the ENG-21 row no longer says \"the agent instructions\"",
    );
}

#[test]
fn missing_inputs_are_named() {
    let (_d, c) = crate::tests::checkout(&[("docs/srs/10.md", ENG)]);
    assert!(stakeholder_owns_the_enforcing_paths(&c).len() >= 2);

    let (_d, c) = crate::tests::checkout(&[("docs/srs/index.md", INDEX), (FILE, OWNERS)]);
    has(
        &stakeholder_owns_the_enforcing_paths(&c),
        "no requirement ENG-21 in the SRS",
    );

    let (_d, c) = crate::tests::checkout(&[("docs/srs/10.md", ENG)]);
    assert!(no_owner_on_any_other_path(&c)[0].starts_with("read .github/CODEOWNERS"));
    assert!(no_owner_on_rust_source(&c)[0].starts_with("read .github/CODEOWNERS"));
}

#[test]
fn an_owner_on_any_other_path_is_named() {
    for extra in [
        "/crates/parinn/src/ @boss",
        "/crates/pin/ @boss",
        "/tooling/ @boss",
        "* @boss",
        "/docs/ @boss",
        "/docs/adr/x/ @boss",
    ] {
        let (_d, c) = repo(&format!("{OWNERS}{extra}\n"));
        let found = no_owner_on_any_other_path(&c);
        if extra == "/docs/adr/x/ @boss" {
            assert!(found.is_empty(), "{found:?}");
        } else {
            let pattern = extra.split(' ').next().unwrap();
            has(
                &found,
                &format!("{pattern} names an owner on a path outside"),
            );
        }
    }

    let (_d, c) = repo(&format!("{OWNERS}/docs/misc/\n"));
    assert!(
        no_owner_on_any_other_path(&c).is_empty(),
        "no owner is no claim"
    );
}

#[test]
fn syntax_the_reader_cannot_judge_is_reported() {
    let (_d, c) = repo(&format!("{OWNERS}/docs/**/x @boss\n"));

    has(
        &no_owner_on_any_other_path(&c),
        "uses CODEOWNERS syntax this check does not read",
    );
    has(
        &stakeholder_owns_the_enforcing_paths(&c),
        "/docs/**/x uses CODEOWNERS syntax",
    );
}

#[test]
fn an_owner_on_rust_source_is_named() {
    for extra in ["/crates/pin/ @boss", "*.rs @boss", "/tooling/x/src/ @boss"] {
        let (_d, c) = repo(&format!("{OWNERS}{extra}\n"));
        let found = no_owner_on_rust_source(&c);
        assert!(!found.is_empty(), "{extra}");
        assert!(
            found
                .iter()
                .all(|p| p.ends_with(".rs is Rust source owned by [@boss]")),
            "{extra}: {found:?}"
        );
    }

    let (_d, c) = repo(&format!("{OWNERS}/crates/pin/ @boss\n"));
    assert_eq!(
        no_owner_on_rust_source(&c),
        ["crates/pin/src/lib.rs is Rust source owned by [@boss]"]
    );
}

#[test]
fn rust_source_under_target_and_git_is_not_read() {
    let (_d, c) = crate::tests::checkout(&[
        (FILE, "* @boss\n"),
        ("target/debug/build/x.rs", "x"),
        (".git/x.rs", "x"),
        ("a.txt", "x"),
    ]);

    assert_eq!(no_owner_on_rust_source(&c), Vec::<String>::new());
}

#[test]
fn a_directory_symlink_is_not_followed() {
    let (_d, c) = crate::tests::checkout(&[(FILE, "* @boss\n"), ("a/x.txt", "x")]);
    std::os::unix::fs::symlink(c.root.join("a"), c.root.join("a/loop")).unwrap();

    assert_eq!(no_owner_on_rust_source(&c), Vec::<String>::new());
}

#[test]
fn a_directory_that_cannot_be_read_is_reported() {
    let (_d, c) = crate::tests::checkout(&[(FILE, "* @boss\n")]);

    let got = rust_sources(&c, "missing");

    assert_eq!(got.sources, Vec::<String>::new());
    assert_eq!(got.unread.len(), 1, "{:?}", got.unread);
    assert!(
        got.unread[0].starts_with("read missing: "),
        "{:?}",
        got.unread
    );
}
