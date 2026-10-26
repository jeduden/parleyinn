use super::*;
use std::path::PathBuf;

fn req(id: &str, path: &str, traces: &[&str]) -> srs::Requirement {
    srs::Requirement {
        id: id.into(),
        path: PathBuf::from(path),
        traces: traces.iter().map(|t| (*t).to_owned()).collect(),
        ..srs::Requirement::default()
    }
}

fn names(set: &[&str]) -> BTreeSet<String> {
    set.iter().map(|s| (*s).to_owned()).collect()
}

fn of(paths: &[&str]) -> Dispatch {
    let paths: Vec<String> = paths.iter().map(|p| (*p).to_owned()).collect();
    dispatch(&paths, &[], &["pin".to_owned()])
}

#[test]
fn a_section_specialist_is_named_for_its_file_without_its_number() {
    for (path, want) in [
        ("docs/srs/06-security.md", Some("srs-security")),
        (
            "docs/srs/05b-lane-requirements.md",
            Some("srs-lane-requirements"),
        ),
        (
            "docs/srs/appendix-a-concern-traceability.md",
            Some("srs-appendix-a-concern-traceability"),
        ),
        ("docs/srs/invariants.md", Some("srs-invariants")),
        ("docs/srs/index.md", None),
        ("docs/srs/sub/x.md", None),
        ("docs/srs/x.txt", None),
        ("docs/srs/x\n## y.md", None),
        ("docs/srs/X.md", None),
        ("docs/other.md", None),
    ] {
        assert_eq!(section_specialist(path).as_deref(), want, "{path}");
    }
}

#[test]
fn rust_routes_to_its_language_and_its_test_layer() {
    assert_eq!(
        of(&["crates/store/src/lib.rs"]),
        Dispatch::from([(
            CODE.to_owned(),
            names(&["lang-rust", PERFORMANCE, "test-engineer-unit"])
        )])
    );
    assert_eq!(
        of(&["crates/store/tests/disk.rs"])[CODE],
        names(&["lang-rust", "test-engineer-integration"])
    );
    for e2e in [
        "crates/parinn/tests/e2e_hook.rs",
        "tooling/scenario/tests/bdd/record.rs",
    ] {
        assert_eq!(
            of(&[e2e])[CODE],
            names(&["lang-rust", "test-engineer-end-to-end"]),
            "{e2e}"
        );
    }
}

#[test]
fn configuration_scripts_and_workflows_route_to_code() {
    assert_eq!(of(&["crates/a/Cargo.toml"])[CODE], names(&["lang-toml"]));
    assert_eq!(of(&["Cargo.lock"])[CODE], names(&["lang-toml"]));
    assert_eq!(of(&["mise.lock"])[CODE], names(&["lang-toml"]));
    assert_eq!(of(&["scripts/a.sh"])[CODE], names(&["lang-shell"]));
    for workflow in [
        ".github/workflows/ci.yml",
        ".github/actions/setup/action.yaml",
    ] {
        assert_eq!(
            of(&[workflow])[CODE],
            names(&["ci-engineer", "lang-github-actions"]),
            "{workflow}"
        );
    }
}

#[test]
fn the_specification_and_the_model_route_to_spec() {
    let got = of(&["docs/srs/06-security.md", "docs/domain-model/record.md"]);
    assert_eq!(
        got[SPEC],
        names(&[
            "domain-model-hub",
            "domain-model-record",
            "lang-markdown",
            "srs-security"
        ])
    );
    assert_eq!(
        of(&["docs/domain-model/index.md"])[SPEC],
        names(&["domain-model-hub", "lang-markdown"])
    );
    let scenarios = of(&["features/record.feature"]);
    assert_eq!(
        scenarios[SPEC],
        names(&["domain-model-hub", "lang-gherkin"])
    );
    assert_eq!(scenarios[CODE], names(&["test-engineer-end-to-end"]));
}

#[test]
fn touched_rows_bring_their_section_and_security_their_officer() {
    let rows = [
        req("REC-01", "docs/srs/05-functional-requirements.md", &["I1"]),
        req("SEC-01", "docs/srs/06-security.md", &["I4"]),
    ];
    let got = dispatch(&[], &rows.iter().collect::<Vec<_>>(), &[]);

    assert_eq!(
        got[SPEC],
        names(&["srs-functional-requirements", "srs-security"])
    );
    assert_eq!(got[SECURITY], names(&["persona-security-officer"]));

    for invariant in ["I2", "I6", "I8"] {
        let row = req("X-01", "docs/srs/index.md", &[invariant]);
        let got = dispatch(&[], &[&row], &[]);
        assert_eq!(
            got[SECURITY],
            names(&["persona-security-officer"]),
            "{invariant}"
        );
        assert!(!got.contains_key(SPEC), "{invariant}");
    }
}

#[test]
fn security_sensitive_code_and_the_invariants_bring_the_officer() {
    for path in [
        "crates/pin/src/lib.rs",
        "crates/parinn/tests/reach.rs",
        "docs/srs/invariants.md",
    ] {
        assert_eq!(
            of(&[path])[SECURITY],
            names(&["persona-security-officer"]),
            "{path}"
        );
    }
    assert!(!of(&["crates/store/src/lib.rs"]).contains_key(SECURITY));
}

#[test]
fn every_crate_stands_for_all_crates_and_nothing_else() {
    let every = [EVERY_CRATE.to_owned()];

    assert!(security_code("crates/store/src/lib.rs", &every));
    assert!(!security_code("tooling/drift/src/lib.rs", &every));
    assert!(!security_code(
        "crates/store/src/lib.rs",
        &["pin".to_owned()]
    ));
}

#[test]
fn a_model_file_whose_name_is_no_agent_name_brings_only_the_hub() {
    assert_eq!(
        of(&["docs/domain-model/a`b\n.md"])[SPEC],
        names(&["lang-markdown"])
    );
}

#[test]
fn a_path_no_specialist_reads_dispatches_nothing() {
    assert!(of(&["testdata/a.json", "LICENSE", "plan/x/spike.go"]).is_empty());
    assert!(of(&[]).is_empty());
}

#[test]
fn json_and_yaml_configuration_route_to_their_specialists() {
    assert_eq!(
        of(&[".taplo/schemas/cargo.json"])[CODE],
        names(&["lang-json"])
    );
    assert_eq!(of(&["codecov.yml"])[CODE], names(&["lang-yaml"]));
    assert_eq!(of(&[".github/dependabot.yml"])[CODE], names(&["lang-yaml"]));
    assert_eq!(of(&["fuzz/corpus/a/b.json"]), Dispatch::new());
}

#[test]
fn the_files_that_govern_approval_bring_the_officer_and_ci() {
    for path in [
        ".github/CODEOWNERS",
        ".github/rulesets/main.json",
        ".claude/settings.json",
    ] {
        let got = of(&[path]);
        assert_eq!(got[SECURITY], names(&[OFFICER]), "{path}");
        assert!(got[CODE].contains(CI), "{path}");
    }
}

#[test]
fn a_language_is_named_for_source_and_configuration_alone() {
    for (path, want) in [
        ("a/b.rs", Some(RUST)),
        ("Cargo.lock", Some(TOML)),
        ("scripts/a.sh", Some(SHELL)),
        (".github/workflows/ci.yml", Some(ACTIONS)),
        (".github/actions/x/action.yml", Some(ACTIONS)),
        (".mdsmith.yml", Some(YAML)),
        ("x.yaml", Some(YAML)),
        ("features/a.feature", Some(GHERKIN)),
        ("README.md", Some(MARKDOWN)),
        (".claude/settings.json", Some(JSON)),
        ("testdata/a.json", None),
        ("plan/a/b.md", Some(MARKDOWN)),
        ("plan/a/spike.go", None),
        ("LICENSE", None),
    ] {
        assert_eq!(language_specialist(path), want, "{path}");
    }
}

#[test]
fn only_source_and_configuration_outside_the_data_needs_a_language() {
    for (path, want) in [
        ("a/b.rs", true),
        ("x.go", true),
        (".env", true),
        ("testdata/a.json", false),
        ("fuzz/corpus/a/seed", false),
        ("research/notes/a.py", false),
        ("plan/a/spike.go", false),
        ("LICENSE", false),
        ("licenses/x/THIRD_PARTY_LICENSES", false),
        (".github/CODEOWNERS", false),
        ("tooling/eval/recall-v1.txt", false),
        ("scripts/mise.sha256", false),
        (".gitignore", false),
        ("fuzz/.gitignore", false),
        (".gitattributes", false),
    ] {
        assert_eq!(needs_a_language(path), want, "{path}");
    }
}

#[test]
fn agents_lists_the_generalists_and_their_specialists() {
    let got = of(&["scripts/a.sh", "docs/guide.md"]);

    assert_eq!(
        agents(&got),
        names(&["lang-markdown", "lang-shell", CODE, SPEC])
    );
}

#[test]
fn a_model_specialist_is_the_hub_for_the_index() {
    assert_eq!(model_specialist("index").as_deref(), Some(MODEL_HUB));
    assert_eq!(
        model_specialist("record").as_deref(),
        Some("domain-model-record")
    );
    assert_eq!(model_specialist("A b"), None);
}

#[test]
fn product_code_and_the_benchmarks_bring_the_performance_specialist() {
    for path in [
        "crates/store/src/lib.rs",
        "tooling/benchmark/src/lib.rs",
        "tooling/calibrate/src/main.rs",
        "tooling/benchmark/accepted.txt",
    ] {
        assert!(of(&[path])[CODE].contains(PERFORMANCE), "{path}");
    }
    for path in [
        "crates/store/tests/disk.rs",
        "crates/store/src/tests.rs",
        "crates/store/src/a/tests.rs",
        "tooling/drift/src/cases.rs",
    ] {
        assert!(
            !of(&[path])
                .get(CODE)
                .is_some_and(|s| s.contains(PERFORMANCE)),
            "{path}"
        );
    }
}

#[test]
fn a_non_functional_row_or_one_tracing_i9_brings_the_performance_specialist() {
    for row in [
        req("NFR-01", "docs/srs/07-non-functional-requirements.md", &[]),
        req("HOOK-01", "docs/srs/05-functional-requirements.md", &["I9"]),
    ] {
        let got = dispatch(&[], &[&row], &[]);
        assert_eq!(got[CODE], names(&[PERFORMANCE]), "{}", row.id);
    }
}

#[test]
fn every_specialist_has_one_generalist() {
    for (specialist, want) in [
        ("lang-rust", Some(CODE)),
        ("lang-toml", Some(CODE)),
        ("lang-shell", Some(CODE)),
        ("lang-github-actions", Some(CODE)),
        ("ci-engineer", Some(CODE)),
        ("performance", Some(CODE)),
        ("test-engineer-unit", Some(CODE)),
        ("test-engineer-integration", Some(CODE)),
        ("test-engineer-end-to-end", Some(CODE)),
        ("lang-gherkin", Some(SPEC)),
        ("lang-markdown", Some(SPEC)),
        ("domain-model-hub", Some(SPEC)),
        ("domain-model-record", Some(SPEC)),
        ("srs-security", Some(SPEC)),
        ("persona-security-officer", Some(SECURITY)),
        ("review-code", None),
        ("persona-agent", None),
    ] {
        assert_eq!(generalist_of(specialist), want, "{specialist}");
    }
    for (specialist, generalist) in FIXED_SPECIALISTS {
        assert_eq!(generalist_of(specialist), Some(generalist), "{specialist}");
    }
}

#[test]
fn a_deep_review_hands_each_generalist_s_specialists_to_its_deep_instance() {
    let normal: Dispatch = [
        (CODE.to_owned(), ["lang-rust".to_owned()].into()),
        (SECURITY.to_owned(), [OFFICER.to_owned()].into()),
    ]
    .into();

    let got = deepened(normal);

    assert_eq!(
        got,
        [
            (CODE_DEEP.to_owned(), ["lang-rust".to_owned()].into()),
            (SECURITY_DEEP.to_owned(), [OFFICER.to_owned()].into()),
        ]
        .into()
    );
    let other: Dispatch = [("other".to_owned(), BTreeSet::new())].into();
    assert_eq!(deepened(other.clone()), other);
    for (generalist, deep) in GENERALISTS.into_iter().zip(DEEP_GENERALISTS) {
        assert_eq!(deep, format!("{generalist}-deep"));
    }
}
