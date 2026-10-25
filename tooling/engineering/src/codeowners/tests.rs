use super::*;

const INDEX: &str = "| **Stakeholder** | @boss, also the security reviewer |\n";

const ENG: &str = "\
| ID | Pri | Requirement | Ver |
|---|---|---|---|
| ENG-11 | P0 | Coverage for security-sensitive crates (`pin`, `search`) and more. | A |
| ENG-21 | P0 | Changes to `docs/srs/`, or to what enforces it (the requirement–scenario gate, CI, the lint and coverage configuration, the agent instructions and `CODEOWNERS` itself) need the stakeholder. | I |
";

const OWNERS: &str = "\
# a comment
/docs/srs/        @boss
/docs/adr/        @boss
/tooling/         @boss
/crates/*/clippy.toml @boss
/crates/*/Cargo.toml  @boss
/.github/         @boss # trailing
/scripts/         @boss
/Cargo.toml       @boss
/CLAUDE.md        @boss
/.claude/         @boss
/SECURITY.md      @boss
/crates/pin/      @boss
/crates/search/   @boss
";

const FILES: &[&str] = &[
    "tooling/x/lib.rs",
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
fn roles_come_from_the_stakeholder_row() {
    let (_d, c) = repo(OWNERS);
    assert_eq!(
        roles(&c).unwrap(),
        Roles {
            stakeholder: "@boss".to_owned(),
            security_reviewer: "@boss".to_owned()
        }
    );

    let (_d, c) = crate::tests::checkout(&[("docs/srs/index.md", "| **Stakeholder** | @boss |\n")]);
    assert_eq!(roles(&c).unwrap().security_reviewer, "");

    let (_d, c) =
        crate::tests::checkout(&[("docs/srs/index.md", "| **Stakeholder** | nobody |\n")]);
    assert_eq!(roles(&c).unwrap_err(), "the Stakeholder row names no user");

    let (_d, c) = crate::tests::checkout(&[("docs/srs/index.md", "x\n")]);
    assert_eq!(
        roles(&c).unwrap_err(),
        "docs/srs/index.md has no Stakeholder row"
    );
}

#[test]
fn the_security_crates_come_from_the_eng_11_row() {
    let (_d, c) = repo(OWNERS);
    assert_eq!(security_crates(&c).unwrap(), ["pin", "search"]);

    for (row, want) in [
        ("Text.", "ENG-11 names no security-sensitive crates"),
        (
            "security-sensitive crates (`a`",
            "ENG-11's crate list is not closed",
        ),
        (
            "security-sensitive crates ()",
            "ENG-11 lists no security-sensitive crate",
        ),
    ] {
        let md = format!(
            "| ID | Pri | Requirement | Ver |\n|---|---|---|---|\n| ENG-11 | P0 | {row} | A |\n"
        );
        let (_d, c) = crate::tests::checkout(&[("docs/srs/10.md", md.as_str())]);
        assert_eq!(security_crates(&c).unwrap_err(), want);
    }
}

#[test]
fn a_conforming_file_has_no_problem() {
    assert_eq!(problems(OWNERS), Vec::<String>::new());
    assert_eq!(
        no_owner_on_any_other_path(&repo(OWNERS).1),
        Vec::<String>::new()
    );
    assert_eq!(
        security_crates_owned_by_the_security_reviewer(&repo(OWNERS).1),
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
    assert_eq!(
        no_owner_on_any_other_path(&c),
        ["no requirement ENG-11 in the SRS"]
    );

    let (_d, c) = crate::tests::checkout(&[("docs/srs/10.md", ENG)]);
    assert!(no_owner_on_any_other_path(&c)[0].starts_with("read .github/CODEOWNERS"));
    assert!(security_crates_owned_by_the_security_reviewer(&c).len() >= 2);
}

#[test]
fn an_owner_on_any_other_path_is_named() {
    for extra in [
        "/crates/parinn/src/ @boss",
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
fn the_security_crates_belong_to_the_security_reviewer() {
    let owners = OWNERS.replace("/crates/search/   @boss\n", "");
    let (_d, c) = repo(&owners);

    assert_eq!(
        security_crates_owned_by_the_security_reviewer(&c),
        ["crates/search/ is not owned by @boss"]
    );

    let (_d, c) = crate::tests::checkout(&[
        ("docs/srs/index.md", "| **Stakeholder** | @boss |\n"),
        ("docs/srs/10.md", ENG),
        (FILE, OWNERS),
    ]);
    assert_eq!(
        security_crates_owned_by_the_security_reviewer(&c),
        ["the SRS index names no security reviewer"]
    );
}
