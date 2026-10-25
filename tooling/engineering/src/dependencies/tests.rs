use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) fn dep_adr(
    id: &str,
    status: &str,
    alternatives: &str,
    modules: Vec<adr::Module>,
) -> Adr {
    Adr {
        path: PathBuf::from(format!("docs/adr/{id}-x.md")),
        id: id.into(),
        title: "t".into(),
        status: status.into(),
        summary: "s".into(),
        sections: [("Alternatives".to_owned(), alternatives.to_owned())].into(),
        modules,
        ..Adr::default()
    }
}

fn module(name: &str, license: &str) -> adr::Module {
    adr::Module {
        name: name.into(),
        purpose: "p".into(),
        license: license.into(),
        maintenance: "m".into(),
    }
}

fn strings(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| (*x).to_owned()).collect()
}

#[test]
fn deps_named_once_and_modules_are_deps() {
    let deps = strings(&["a", "b", "c"]);
    let adrs = vec![
        dep_adr(
            "ADR-01",
            adr::ACCEPTED,
            "x",
            vec![module("a", "MIT"), module("stale", "MIT")],
        ),
        dep_adr("ADR-02", adr::ACCEPTED, "x", vec![module("b", "MIT")]),
        dep_adr("ADR-03", adr::ACCEPTED, "x", vec![module("b", "MIT")]),
        dep_adr("ADR-04", adr::SUPERSEDED, "x", vec![module("c", "MIT")]),
    ];

    assert_eq!(
        deps_named_once(&deps, &adrs),
        [
            "b is named by 2 accepted ADRs [ADR-02 ADR-03], want exactly one",
            "c is named by 0 accepted ADRs [], want exactly one",
        ]
    );
    assert_eq!(
        modules_are_deps(&deps, &adrs),
        ["ADR-01 names stale, which is not a direct dependency"]
    );

    assert!(deps_named_once(&strings(&["b"]), &adrs[1..2]).is_empty());
    assert!(modules_are_deps(&strings(&["b"]), &adrs[1..2]).is_empty());
}

#[test]
fn deps_justified_wants_every_column_and_alternatives() {
    let partial = adr::Module {
        name: "a".into(),
        license: "MIT".into(),
        maintenance: "m".into(),
        ..Default::default()
    };
    let adrs = vec![
        dep_adr("ADR-01", adr::ACCEPTED, "", vec![partial]),
        dep_adr("ADR-02", adr::ACCEPTED, "", vec![]),
        dep_adr(
            "ADR-03",
            adr::PROPOSED,
            "",
            vec![adr::Module {
                name: "b".into(),
                ..Default::default()
            }],
        ),
        Adr {
            sections: BTreeMap::new(),
            ..dep_adr("ADR-05", adr::ACCEPTED, "", vec![module("c", "MIT")])
        },
    ];

    assert_eq!(
        deps_justified(&adrs),
        [
            "ADR-01 weighs no alternatives",
            r#"ADR-01 leaves "Purpose" empty for a"#,
            "ADR-05 weighs no alternatives",
        ]
    );
    assert!(
        deps_justified(&[dep_adr(
            "ADR-04",
            adr::ACCEPTED,
            "x",
            vec![module("a", "MIT")]
        )])
        .is_empty()
    );
}

#[test]
fn licenses_and_target_come_from_the_requirement_text() {
    let eng18 = "Licenses MUST be on an allow-list (MIT, BSD). The target is ≤ 2 direct dependencies linked into a shipped executable.";
    let allowed = allow_list(eng18).unwrap();
    assert_eq!(allowed, ["MIT", "BSD"]);
    assert_eq!(dependency_target(eng18).unwrap(), 2);

    let ok = [dep_adr(
        "ADR-01",
        adr::ACCEPTED,
        "x",
        vec![module("a", "MIT"), module("b", "BSD-3-Clause")],
    )];
    assert!(licenses_allowed(&ok, &allowed, &BTreeMap::new()).is_empty());

    let bad = [dep_adr(
        "ADR-01",
        adr::ACCEPTED,
        "x",
        vec![module("a", "GPL-3.0"), module("b", "BSDish")],
    )];
    assert_eq!(
        licenses_allowed(&bad, &allowed, &BTreeMap::new()),
        [
            r#"a is licensed "GPL-3.0", not on the allow-list [MIT BSD]"#,
            r#"b is licensed "BSDish", not on the allow-list [MIT BSD]"#,
        ]
    );

    assert!(within_target(&strings(&["a", "b"]), 2).is_ok());
    assert_eq!(
        within_target(&strings(&["a", "b", "c"]), 2).unwrap_err(),
        "3 direct dependencies linked into a shipped executable, want at most 2: [a b c]"
    );
}

#[test]
fn requirement_text_that_states_nothing_is_reported() {
    assert_eq!(
        allow_list("Nothing said here").unwrap_err(),
        "ENG-18 does not state its license allow-list"
    );
    assert_eq!(
        allow_list("an allow-list (MIT").unwrap_err(),
        "ENG-18 does not state its license allow-list"
    );
    assert_eq!(
        dependency_target("Nothing said here").unwrap_err(),
        "ENG-18 does not state its dependency target"
    );
    assert_eq!(
        dependency_target("≤ 5 crates, ≤ x direct dependencies linked into a shipped executable")
            .unwrap_err(),
        "ENG-18 does not state its dependency target"
    );
    assert_eq!(
        dependency_target("≤ 7 direct dependencies, counting test crates").unwrap_err(),
        "ENG-18 does not state its dependency target"
    );
    assert_eq!(
        dependency_target(
            "≤ 5 crates and ≤ 7 direct dependencies linked into a shipped executable"
        )
        .unwrap(),
        7
    );
}

#[test]
fn license_family_admits_only_its_permissive_variants() {
    let allowed = strings(&["Apache-2.0", "MIT", "BSD", "ISC"]);

    for license in ["BSD-2-Clause", "BSD-3-Clause", "MIT"] {
        assert!(license_allowed(license, &allowed), "{license}");
    }
    for license in [
        "BSD",
        "BSD-4-Clause",
        "BSD-Protection",
        "MIT-advertising",
        "Apache-2.0-foo",
    ] {
        assert!(!license_allowed(license, &allowed), "{license}");
    }
}

#[test]
fn an_and_expression_wants_every_conjunct_allowed() {
    let allowed = strings(&["Apache-2.0", "MIT"]);
    let both = "(MIT OR Apache-2.0) AND NCSA";

    assert!(!license_allowed(both, &allowed));
    assert!(license_allowed(both, &strings(&["MIT", "NCSA"])));
    assert!(license_allowed("MIT AND Apache-2.0", &allowed));
    assert!(!license_allowed("MIT AND GPL-3.0", &allowed));
}

#[test]
fn a_per_crate_exception_admits_one_crate_alone() {
    let deny = "# { crate = \"commented\", allow = [\"NCSA\"] },\nexceptions = [\n  { crate = \"libfuzzer-sys\", allow = [\"NCSA\"] },\n  { crate = \"two\", allow = [\"A\", \"B\"] },\n]\n";
    let exceptions = license_exceptions(deny);
    assert_eq!(exceptions.len(), 2);
    assert_eq!(exceptions["libfuzzer-sys"], ["NCSA"]);
    assert_eq!(exceptions["two"], ["A", "B"]);

    let allowed = strings(&["MIT"]);
    let adrs = [dep_adr(
        "ADR-01",
        adr::ACCEPTED,
        "x",
        vec![
            module("libfuzzer-sys", "MIT AND NCSA"),
            module("other", "MIT AND NCSA"),
        ],
    )];
    assert_eq!(
        licenses_allowed(&adrs, &allowed, &exceptions),
        [r#"other is licensed "MIT AND NCSA", not on the allow-list [MIT]"#]
    );
}

#[test]
fn lists_dependency_adrs_wants_a_link_for_each_record_naming_a_module() {
    let body = "- [ADR-01](docs/adr/ADR-01-x.md)\n";
    let mut adrs = vec![
        dep_adr("ADR-01", adr::ACCEPTED, "x", vec![module("a", "MIT")]),
        dep_adr("ADR-02", adr::ACCEPTED, "x", vec![]),
    ];
    assert!(lists_dependency_adrs(body, "DEPENDENCIES.md", "docs/adr", &adrs).is_empty());

    adrs.push(dep_adr(
        "ADR-03",
        adr::SUPERSEDED,
        "x",
        vec![module("b", "MIT")],
    ));
    assert_eq!(
        lists_dependency_adrs(body, "DEPENDENCIES.md", "docs/adr", &adrs),
        ["DEPENDENCIES.md does not link docs/adr/ADR-03-x.md"]
    );
}

#[test]
fn cargo_direct_deps_reads_every_registry_dependency_of_the_members() {
    let metadata = r#"{"packages": [
        {"name": "a", "dependencies": [
            {"name": "serde", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": null},
            {"name": "b", "source": null, "path": "/w/b", "kind": null},
            {"name": "cucumber", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": "dev"}
        ]},
        {"name": "b", "dependencies": [
            {"name": "serde", "source": "registry+https://github.com/rust-lang/crates.io-index", "kind": null}
        ]}
    ]}"#;

    assert_eq!(cargo_direct_deps(metadata).unwrap(), ["cucumber", "serde"]);
    assert!(
        cargo_direct_deps("not json")
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
    assert_eq!(cargo_direct_deps("{}").unwrap(), Vec::<String>::new());
}

#[test]
fn cargo_shipped_deps_counts_what_links_into_a_product_executable() {
    let metadata = r#"{"workspace_root": "/w", "packages": [
        {"name": "parinn", "manifest_path": "/w/crates/parinn/Cargo.toml",
         "targets": [{"kind": ["bin"]}, {"kind": ["test"]}],
         "dependencies": [
            {"name": "store", "source": null, "kind": null},
            {"name": "sha2", "source": "registry+x", "kind": null, "target": "cfg(unix)"},
            {"name": "serde_json", "source": "registry+x", "kind": "dev"},
            {"name": "cc", "source": "registry+x", "kind": "build"},
            {"name": "testkit", "source": null, "kind": "dev"}
        ]},
        {"name": "store", "manifest_path": "/w/crates/store/Cargo.toml",
         "targets": [{"kind": ["lib"]}],
         "dependencies": [
            {"name": "rusqlite", "source": "registry+x", "kind": null},
            {"name": "forked", "source": "git+https://example.com/forked", "kind": null},
            {"name": "sha2", "source": "registry+x", "kind": null},
            {"name": "parinn", "source": null, "kind": null}
        ]},
        {"name": "testkit", "manifest_path": "/w/tooling/testkit/Cargo.toml",
         "targets": [{"kind": ["lib"]}],
         "dependencies": [{"name": "getrandom", "source": "registry+x", "kind": null}]},
        {"name": "review-gate", "manifest_path": "/w/tooling/review-gate/Cargo.toml",
         "targets": [{"kind": ["bin"]}],
         "dependencies": [{"name": "serde", "source": "registry+x", "kind": null}]},
        {"name": "audit", "manifest_path": "/w/crates/audit/Cargo.toml",
         "targets": [{"kind": ["lib"]}],
         "dependencies": [{"name": "toml", "source": "registry+x", "kind": null}]}
    ]}"#;
    let product = std::path::Path::new("crates");

    assert_eq!(
        cargo_shipped_deps(metadata, product).unwrap(),
        ["rusqlite", "sha2"]
    );
    assert!(
        cargo_shipped_deps("not json", product)
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
    assert_eq!(
        cargo_shipped_deps("{}", product).unwrap_err(),
        "cargo metadata names no workspace_root"
    );
    assert_eq!(
        cargo_shipped_deps(r#"{"workspace_root": "/w"}"#, product).unwrap(),
        Vec::<String>::new()
    );
    let stray = r#"{"workspace_root": "/w", "packages": [
        {"name": "parinn", "manifest_path": "/w/crates/parinn/Cargo.toml",
         "targets": [{"kind": ["bin"]}],
         "dependencies": [{"name": "vendored", "source": null, "kind": null}]}
    ]}"#;
    assert_eq!(
        cargo_shipped_deps(stray, product).unwrap_err(),
        "parinn depends by path on vendored, which is no workspace member"
    );
}

#[test]
fn ships_wants_a_binary_target_under_the_product_root() {
    let product = std::path::Path::new("/w/crates");
    let package = |manifest: &str, kind: &str| serde_json::json!({"manifest_path": manifest, "targets": [{"kind": ["lib"]}, {"kind": [kind]}]});

    assert!(ships(
        &package("/w/crates/parinn/Cargo.toml", "bin"),
        product
    ));
    assert!(!ships(
        &package("/w/crates/store/Cargo.toml", "test"),
        product
    ));
    assert!(!ships(
        &package("/w/tooling/review-gate/Cargo.toml", "bin"),
        product
    ));
    assert!(!ships(
        &package("/w/crates-old/x/Cargo.toml", "bin"),
        product
    ));
    assert!(!ships(&serde_json::json!({}), product));
}

#[test]
fn normal_keeps_neither_dev_nor_build_dependencies() {
    let package = serde_json::json!({"dependencies": [
        {"name": "a", "kind": null},
        {"name": "b", "kind": "dev"},
        {"name": "c", "kind": "build"},
        {"name": "d"}
    ]});

    let names: Vec<_> = normal(&package)
        .filter_map(|d| d["name"].as_str())
        .collect();
    assert_eq!(names, ["a", "d"]);
    assert_eq!(normal(&serde_json::json!({})).count(), 0);
}

#[test]
fn an_spdx_or_expression_is_allowed_when_one_choice_is() {
    let allowed = strings(&["Apache-2.0", "MIT", "BSD", "ISC"]);

    assert!(license_allowed("MIT OR Apache-2.0", &allowed));
    assert!(license_allowed("GPL-3.0 OR BSD-3-Clause", &allowed));
    assert!(!license_allowed("GPL-3.0 OR LGPL-2.1", &allowed));
    assert!(!license_allowed("MIT AND GPL-3.0", &allowed));
}

fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

#[test]
fn cargo_metadata_reads_a_workspace_and_names_a_failure() {
    let (dir, _) = crate::tests::checkout(&[
        (
            "Cargo.toml",
            "[package]\nname = \"t\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        ),
        ("src/lib.rs", ""),
    ]);
    let home = dir.path().join(".parinn");

    let json = cargo_metadata(&cargo(), &dir.path().join("Cargo.toml"), &home).unwrap();
    assert!(json.contains("\"name\":\"t\""), "{json}");

    let err = cargo_metadata(&cargo(), &dir.path().join("missing/Cargo.toml"), &home).unwrap_err();
    assert!(err.starts_with("cargo metadata failed: "), "{err}");
    let err = cargo_metadata(
        std::ffi::OsStr::new("/nonexistent/cargo"),
        &dir.path().join("Cargo.toml"),
        &home,
    );
    assert!(err.unwrap_err().starts_with("run cargo metadata: "));
}

#[test]
fn direct_deps_reads_each_manifest_with_its_reader() {
    let (_dir, checkout) = crate::tests::checkout(&[]);
    let shipped = checkout.root.join("crates/parinn/Cargo.toml");
    let json = format!(
        r#"{{"workspace_root": {}, "packages": [
            {{"manifest_path": {}, "targets": [{{"kind": ["bin"]}}],
              "dependencies": [{{"name": "sha2", "source": "registry+x", "kind": null}}]}},
            {{"dependencies": [{{"name": "serde", "source": "registry+x"}}]}}
        ]}}"#,
        serde_json::Value::from(checkout.root.to_string_lossy()),
        serde_json::Value::from(shipped.to_string_lossy())
    );
    let metadata = |manifest: &std::path::Path| {
        assert!(manifest.ends_with("Cargo.toml"));
        Ok(json.clone())
    };

    let deps = direct_deps(&checkout, "Cargo.toml", metadata).unwrap();
    assert_eq!(deps.all, ["serde", "sha2"]);
    assert_eq!(deps.shipped, ["sha2"]);
    // The fuzz workspace's dependencies need their ADRs but never ship,
    // even from a binary target, since it is no crate under the product.
    let fuzz = direct_deps(&checkout, "fuzz/Cargo.toml", metadata).unwrap();
    assert_eq!(fuzz.all, ["serde", "sha2"]);
    assert!(fuzz.shipped.is_empty());
    assert_eq!(
        direct_deps(&checkout, "package.json", metadata).unwrap_err(),
        "no reader for the dependencies in package.json"
    );
    assert!(
        direct_deps(&checkout, "Cargo.toml", |_| Err("boom".to_owned())).unwrap_err() == "boom"
    );
}
