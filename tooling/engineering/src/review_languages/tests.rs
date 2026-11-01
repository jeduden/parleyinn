use super::*;

fn problems(files: &[(&str, &str)]) -> Vec<String> {
    let (_d, c) = crate::tests::checkout(files);
    every_language_has_its_specialist(&c)
}

#[test]
fn source_and_configuration_in_a_known_language_pass() {
    assert_eq!(
        problems(&[
            ("a/b.rs", ""),
            ("Cargo.toml", ""),
            (".mdsmith.yml", ""),
            (".github/rulesets/main.json", ""),
            ("README.md", ""),
            ("LICENSE", ""),
            ("tooling/eval/recall-v1.txt", ""),
            ("testdata/x.snap", ""),
            ("plan/x/spike.go", ""),
        ]),
        Vec::<String>::new()
    );
}

#[test]
fn a_file_in_no_specialist_s_language_is_named() {
    assert_eq!(
        problems(&[("scripts/x.py", ""), ("a/b.rs", "")]),
        ["scripts/x.py is in no language a review specialist reads"]
    );
}

#[test]
fn what_git_does_not_track_is_left_out() {
    assert_eq!(
        problems(&[
            (
                ".gitignore",
                "# local\n/parinn\n*.profraw\n.env\n/dist/\n!keep.py\n"
            ),
            ("parinn", ""),
            ("x/default.profraw", ""),
            ("x/.env", ""),
            ("dist/a.py", ""),
            ("target/a.py", ""),
            (".git/hooks/a.py", ""),
            (".claude/worktrees/w/a.py", ""),
            ("sub/.gitignore", "local/\n"),
            ("sub/local/a.py", ""),
        ]),
        Vec::<String>::new()
    );
}

#[test]
fn a_git_file_of_a_worktree_is_left_out() {
    assert_eq!(
        problems(&[(".git", "gitdir: /elsewhere\n"), ("a/b.rs", "")]),
        Vec::<String>::new()
    );
}

#[test]
fn an_ignore_in_a_subdirectory_applies_below_it_alone() {
    assert_eq!(
        problems(&[("sub/.gitignore", "a.py\n"), ("sub/a.py", ""), ("a.py", "")]),
        ["a.py is in no language a review specialist reads"]
    );
}

#[test]
fn a_dir_only_pattern_leaves_a_file_of_that_name() {
    let rule = &ignores("", "dist/\n")[0];
    assert!(ignored(rule, "dist", true));
    assert!(!ignored(rule, "dist", false));
    let rule = &ignores("sub", "/a/b\n")[0];
    assert!(ignored(rule, "sub/a/b", false));
    assert!(!ignored(rule, "a/b", false));
    assert!(!ignored(rule, "other/a/b", false));
}

#[test]
fn a_glob_stars_any_run_but_never_a_slash() {
    assert!(glob("*.profraw", "a.profraw"));
    assert!(glob("parinn-*", "parinn-x86"));
    assert!(glob("a*b*c", "aXbYc"));
    assert!(!glob("*.profraw", "a/b.profraw"));
    assert!(!glob("a*", "b"));
    assert!(!glob("abc", "abd"));
    assert!(glob("é*", "éa"));
}

#[test]
fn a_directory_that_cannot_be_read_is_named() {
    let (_d, c) = crate::tests::checkout(&[]);
    let gone = Checkout::new(&c.root.join("missing"));

    let got = every_language_has_its_specialist(&gone);

    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].starts_with("read : "), "{got:?}");
}

#[cfg(unix)]
#[test]
fn a_name_that_is_not_utf8_is_named() {
    use std::os::unix::ffi::OsStrExt;
    let (_d, c) = crate::tests::checkout(&[("a/b.rs", "")]);
    std::fs::write(
        c.root
            .join("a")
            .join(std::ffi::OsStr::from_bytes(b"\xff.rs")),
        "",
    )
    .unwrap();

    assert_eq!(
        every_language_has_its_specialist(&c),
        ["a/\u{fffd}.rs is not UTF-8"]
    );
}
