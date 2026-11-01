use super::*;

fn paths(list: &[&str]) -> Vec<String> {
    list.iter().map(|p| (*p).to_owned()).collect()
}

/// The scopes the dispatch gives `list`, as the analysis builds them.
fn scopes_of(list: &[&str]) -> dispatch::Scopes {
    dispatch::scopes(&paths(list), &[], &[], &std::collections::BTreeMap::new())
}

#[test]
fn each_kind_of_source_and_configuration_has_its_limit() {
    for (path, want) in [
        ("crates/store/src/lib.rs", Some(("Rust", RUST_LINES))),
        ("crates/store/src/tests.rs", Some(("Rust", RUST_LINES))),
        ("features/record.feature", Some(("Gherkin", FEATURE_LINES))),
        (".github/workflows/ci.yml", Some(("YAML", YAML_LINES))),
        (".github/actions/a/action.yaml", Some(("YAML", YAML_LINES))),
        ("Cargo.toml", Some(("TOML", TOML_LINES))),
        ("scripts/setup-tools.sh", Some(("shell", SHELL_LINES))),
        ("docs/development.md", None),
        (".claude/settings.json", None),
        ("Cargo.lock", None),
        ("LICENSE", None),
        ("testdata/claude-code/record/defer.sh", None),
        ("research/notes/x.rs", None),
        ("plan/x/spike/main.rs", None),
        ("fuzz/corpus/parse/seed.toml", None),
    ] {
        assert_eq!(limit_of(path), want, "{path}");
    }
}

#[test]
fn a_file_at_its_limit_passes_and_one_past_it_is_a_nit() {
    let path = "crates/store/src/lib.rs";

    assert_eq!(oversize(path, RUST_LINES), None);
    let got = oversize(path, RUST_LINES + 1).unwrap();

    assert_eq!(got.path, path);
    assert_eq!(got.line, 0);
    assert_eq!(got.severity, NIT);
    assert_eq!(
        got.body,
        format!(
            "crates/store/src/lib.rs — if not addressed: a file past the limit exceeds \
             what a reviewer reads whole and what maintainers can hold in view ({} lines, \
             past the limit of {RUST_LINES} for Rust) — ENG-34",
            RUST_LINES + 1
        )
    );
}

#[test]
fn a_file_of_no_limited_kind_is_never_oversize() {
    assert_eq!(oversize("docs/a.md", 100_000), None);
    assert_eq!(oversize("testdata/big.rs", 100_000), None);
}

#[test]
fn a_file_no_specialist_reads_is_a_nit_and_one_a_specialist_reads_is_not() {
    let list = [
        "LICENSE",
        "testdata/a.json",
        "crates/a/src/lib.rs",
        ".github/CODEOWNERS",
    ];
    let scopes = scopes_of(&list);

    let got: Vec<Finding> = list.iter().filter_map(|p| unread(p, &scopes)).collect();

    assert_eq!(
        got,
        vec![
            Finding {
                path: "LICENSE".to_owned(),
                line: 0,
                severity: NIT,
                body: "LICENSE — if not addressed: LICENSE reached the merge unread \
                       — ENG-28"
                    .to_owned(),
            },
            Finding {
                path: "testdata/a.json".to_owned(),
                line: 0,
                severity: NIT,
                body: "testdata/a.json — if not addressed: testdata/a.json reached the merge \
                       unread — ENG-28"
                    .to_owned(),
            },
        ]
    );
}

#[test]
fn lock_files_and_generated_files_are_left_out() {
    for path in [
        "Cargo.lock",
        "fuzz/Cargo.lock",
        "target/debug/build/x.rs",
        "lcov.lcov",
        "coverage/unit.lcov",
    ] {
        assert!(left_out(path), "{path}");
        assert_eq!(unread(path, &dispatch::Scopes::new()), None, "{path}");
        assert_eq!(oversize(path, 100_000), None, "{path}");
    }
    for path in ["Cargo.toml", "crates/target/src/lib.rs", "docs/lcov.md"] {
        assert!(!left_out(path), "{path}");
    }
}

#[test]
fn findings_counts_each_changed_file_that_reads_and_names_each_unread_one() {
    let list = paths(&["big.rs", "small.rs", "gone.rs", "notes.txt"]);
    let scopes = scopes_of(&["big.rs", "small.rs", "gone.rs", "notes.txt"]);
    let lines = |path: &str| match path {
        "big.rs" => Some(RUST_LINES + 5),
        "small.rs" => Some(3),
        "notes.txt" => Some(TOML_LINES * 10),
        _ => None,
    };

    let got = findings(&list, &scopes, &lines);

    let named: Vec<(&str, bool)> = got
        .iter()
        .map(|f| (f.path.as_str(), f.body.ends_with("ENG-34")))
        .collect();
    assert_eq!(named, vec![("big.rs", true), ("notes.txt", false)]);
}

#[test]
fn findings_is_empty_for_files_within_their_limits_that_specialists_read() {
    let list = paths(&["a.rs", "features/a.feature"]);
    let scopes = scopes_of(&["a.rs", "features/a.feature"]);

    assert!(findings(&list, &scopes, &|_| Some(1)).is_empty());
}
