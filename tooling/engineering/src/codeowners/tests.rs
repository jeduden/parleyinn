use super::*;

const INDEX: &str = "| **Stakeholder** | @boss, also the security reviewer |\n";

const ENG: &str = "\
| ID | Pri | Requirement | Ver |
|---|---|---|---|
| ENG-21 | P0 | Only a change to what guides the review needs the stakeholder: the agent instructions, the review's code, cases and scorecard, CI and the merge rules (`CODEOWNERS` itself among them), the bans and acceptances CI holds the code to and the linter configuration (`.mdsmith.yml`). | I |
";

const OWNERS: &str = "\
# a comment
/CLAUDE.md        @boss
/.claude/         @boss
/docs/agents/     @boss
/tooling/review-*/ @boss
/.github/         @boss # trailing
/scripts/         @boss
/crates/*/clippy.toml @boss
/deny.toml        @boss
/.mdsmith.yml     @boss
";

const FILES: &[&str] = &[
    "features/a.feature",
    "tooling/x/src/lib.rs",
    "tooling/review-gate/src/lib.rs",
    "tooling/review-run/Cargo.toml",
    "tooling/reviewer.txt",
    "crates/pin/src/lib.rs",
    "docs/agents/a.md",
    "docs/srs/a.md",
    "scripts/a.sh",
    "Cargo.toml",
    "CLAUDE.md",
    ".claude/a",
    ".mdsmith.yml",
    "crates/zed/clippy.toml",
    "deny.toml",
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

    let owners = OWNERS.replace("/scripts/         @boss", "/scripts/         @boss @other");
    has(
        &problems(&owners),
        "scripts/ (CI and the merge rules) is owned by [@boss @other], want @boss",
    );

    let owners = OWNERS.replace("/tooling/review-*/ @boss", "/tooling/review-run/ @boss");
    has(
        &problems(&owners),
        "tooling/review-gate/ (the review's code, cases and scorecard) is owned by [], want @boss",
    );
    assert_eq!(problems(&owners).len(), 1, "{:?}", problems(&owners));

    let owners = OWNERS.replace(
        "/crates/*/clippy.toml @boss",
        "/crates/*/clippy.toml @other",
    );
    has(
        &problems(&owners),
        "crates/zed/clippy.toml (the bans and acceptances CI holds the code to) is owned by [@other]",
    );

    let owners = OWNERS.replace("/.mdsmith.yml     @boss\n", "");
    has(
        &problems(&owners),
        ".mdsmith.yml (the linter configuration) is owned by [], want @boss",
    );
}

#[test]
fn a_starred_name_stands_for_each_entry_it_matches() {
    let (_d, c) = repo(OWNERS);

    assert_eq!(
        expand(&c, "tooling/review-*/"),
        ["tooling/review-gate/", "tooling/review-run/"]
    );
    assert_eq!(expand(&c, "*.md"), ["CLAUDE.md"]);
    assert_eq!(
        expand(&c, "tooling/*/Cargo.toml"),
        ["tooling/review-run/Cargo.toml"]
    );
    assert_eq!(expand(&c, "missing/*/x"), Vec::<String>::new());
    assert_eq!(expand(&c, "docs/agents/"), ["docs/agents/"]);
    assert_eq!(expand(&c, "docs/none/"), Vec::<String>::new());
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
        "/docs/srs/ @boss",
        "/features/ @boss",
        "/Cargo.toml @boss",
        "/crates/*/Cargo.toml @boss",
        "/tooling/engineering/oversize.txt @boss",
        "/SECURITY.md @boss",
        "/tooling/review-eval/cases/ @boss",
        "/docs/agents/x/ @boss",
    ] {
        let (_d, c) = repo(&format!("{OWNERS}{extra}\n"));
        let found = no_owner_on_any_other_path(&c);
        if extra == "/docs/agents/x/ @boss" {
            assert!(found.is_empty(), "{found:?}");
        } else {
            let pattern = extra.split(' ').next().unwrap();
            has(
                &found,
                &format!("{pattern} names an owner on a path that does not guide the review"),
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
fn the_review_s_crates_alone_hold_owned_rust_source() {
    let (_d, c) = repo(&format!("{OWNERS}/tooling/ @boss\n"));

    assert_eq!(
        no_owner_on_rust_source(&c),
        ["tooling/x/src/lib.rs is Rust source owned by [@boss]"]
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
fn another_checkout_under_claude_worktrees_is_not_read() {
    let (_d, c) = crate::tests::checkout(&[
        (FILE, "/.claude/ @boss\n"),
        (".claude/worktrees/w/crates/a/src/lib.rs", "x"),
        ("x/.claude/worktrees/b.rs", "x"),
    ]);

    assert_eq!(no_owner_on_rust_source(&c), Vec::<String>::new());

    let (_d, c) =
        crate::tests::checkout(&[(FILE, "/.claude/ @boss\n"), (".claude/other/a.rs", "x")]);
    assert_eq!(
        no_owner_on_rust_source(&c),
        [".claude/other/a.rs is Rust source owned by [@boss]"]
    );
}

#[cfg(unix)]
#[test]
fn a_directory_symlink_is_not_followed() {
    let (_d, c) = crate::tests::checkout(&[(FILE, "* @boss\n"), ("a/x.txt", "x")]);
    std::os::unix::fs::symlink(c.root.join("a"), c.root.join("a/loop")).unwrap();

    assert_eq!(no_owner_on_rust_source(&c), Vec::<String>::new());
}

#[test]
fn under_joins_a_name_to_its_directory_or_the_root() {
    assert_eq!(under("", "a.rs"), "a.rs");
    assert_eq!(under("a/b", "c.rs"), "a/b/c.rs");
}

#[cfg(unix)]
#[test]
fn a_file_name_that_is_not_utf8_is_reported() {
    use std::os::unix::ffi::OsStrExt;
    let (_d, c) = crate::tests::checkout(&[(FILE, "* @boss\n"), ("a/b.rs", "")]);
    std::fs::write(
        c.root
            .join("a")
            .join(std::ffi::OsStr::from_bytes(b"\xff.rs")),
        "",
    )
    .unwrap();

    let got = rust_sources(&c, "a");

    assert_eq!(got.sources, ["a/b.rs"]);
    assert_eq!(got.unread, ["a/\u{fffd}.rs is not UTF-8"]);
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
