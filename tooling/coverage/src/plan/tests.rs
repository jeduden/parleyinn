use super::*;
use std::path::Path;

const METADATA: &str = r#"{
  "packages": [
    {"name": "srs", "manifest_path": "/w/tooling/srs/Cargo.toml",
     "targets": [{"name": "srs", "kind": ["lib"]}, {"name": "gates", "kind": ["test"]}]},
    {"name": "scenario", "manifest_path": "/w/tooling/scenario/Cargo.toml",
     "targets": [{"name": "bdd", "kind": ["test"]}, {"name": "gate", "kind": ["test"]}]},
    {"name": "review-gate", "manifest_path": "/w/tooling/review-gate/Cargo.toml",
     "targets": [{"name": "review-gate", "kind": ["bin"]}, {"name": "e2e_review_gate", "kind": ["test"]},
                 {"name": "gate", "kind": ["test"]}]}
  ],
  "workspace_root": "/w",
  "metadata": {"coverage": {
    "exclude": "(^|/)tests\\.rs$",
    "entry-points": ["src/main.rs"],
    "pyramid": true,
    "floors": {"all": 100, "unit": 90},
    "crates": {"srs": {"floors": {"unit": 95}}}
  }}
}"#;

#[test]
fn the_plan_sorts_test_targets_into_layers() {
    let plan = Plan::from_metadata(METADATA).unwrap();

    assert_eq!(plan.targets[&TestLayer::Integration], ["gate", "gates"]);
    assert_eq!(
        plan.targets[&TestLayer::EndToEnd],
        ["bdd", "e2e_review_gate"]
    );
    assert!(!plan.targets.contains_key(&TestLayer::Unit));
    assert_eq!(plan.exclude, r"(^|/)tests\.rs$");
    assert_eq!(
        plan.crates[0],
        Crate {
            name: "srs".into(),
            dir: "/w/tooling/srs".into()
        }
    );
}

#[test]
fn a_crate_s_floors_override_the_defaults() {
    let plan = Plan::from_metadata(METADATA).unwrap();

    assert_eq!(
        plan.floors_of("srs"),
        Floors {
            all: Some(100.0),
            unit: Some(95.0),
            ..Floors::default()
        }
    );
    assert_eq!(
        plan.floors_of("scenario"),
        Floors {
            all: Some(100.0),
            unit: Some(90.0),
            ..Floors::default()
        }
    );
}

#[test]
fn a_file_belongs_to_the_deepest_crate_holding_it() {
    let mut plan = Plan::from_metadata(METADATA).unwrap();
    plan.crates.push(Crate {
        name: "nested".into(),
        dir: "/w/tooling/srs/nested".into(),
    });

    let of = |f: &str| plan.crate_of(Path::new(f)).map(|c| c.name.as_str());
    assert_eq!(of("/w/tooling/srs/src/lib.rs"), Some("srs"));
    assert_eq!(of("/w/tooling/srs/nested/src/lib.rs"), Some("nested"));
    assert_eq!(of("/elsewhere/lib.rs"), None);
}

#[test]
fn a_plan_without_a_policy_has_no_floors() {
    let plan = Plan::from_metadata(r#"{"packages": []}"#).unwrap();

    assert_eq!(plan.floors, Floors::default());
    assert!(plan.exclude.is_empty() && plan.crates.is_empty());
}

#[test]
fn a_malformed_policy_is_refused() {
    let with = |coverage: &str| {
        Plan::from_metadata(&format!(
            r#"{{"packages": [{{"name": "srs", "manifest_path": "/w/srs/Cargo.toml", "targets": []}}], "metadata": {{"coverage": {coverage}}}}}"#
        ))
    };

    assert_eq!(
        with(r#"{"floors": 90}"#).unwrap_err(),
        "coverage floors: want a table of percentages"
    );
    assert_eq!(
        with(r#"{"floors": {"all": 101}}"#).unwrap_err(),
        "coverage floors: all = 101 is no percentage"
    );
    assert_eq!(
        with(r#"{"floors": {"all": "x"}}"#).unwrap_err(),
        r#"coverage floors: all = "x" is no percentage"#
    );
    assert_eq!(
        with(r#"{"floors": {"smoke": 50}}"#).unwrap_err(),
        r#"coverage floors: "smoke" is no test layer"#
    );
    assert_eq!(
        with(r#"{"crates": {"srs": {"floors": {"e2e": -1}}}}"#).unwrap_err(),
        "coverage floors of srs: e2e = -1 is no percentage"
    );
    assert_eq!(
        with(r#"{"pyramid": "true"}"#).unwrap_err(),
        r#"coverage pyramid: "true" is no boolean"#
    );
    assert_eq!(
        with(r#"{"exclude": ["tests.rs"]}"#).unwrap_err(),
        r#"coverage exclude: ["tests.rs"] is no regular expression"#
    );
    assert!(
        Plan::from_metadata("not json")
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
}

#[test]
fn every_floor_has_its_layer() {
    let floors = Floors {
        all: Some(1.0),
        unit: Some(2.0),
        integration: Some(3.0),
        e2e: Some(4.0),
    };

    assert_eq!(
        TestLayer::ALL.map(|l| floors.of(l)),
        [Some(2.0), Some(3.0), Some(4.0)]
    );
    let parsed =
        super::floors(&serde_json::json!({"all": 1, "unit": 2, "integration": 3, "e2e": 4}))
            .unwrap();
    assert_eq!(parsed, floors);
}

#[test]
fn a_layer_is_found_by_its_name() {
    assert_eq!(TestLayer::from_name("e2e"), Some(TestLayer::EndToEnd));
    assert_eq!(TestLayer::from_name("unit"), Some(TestLayer::Unit));
    assert_eq!(TestLayer::from_name("end-to-end"), None);
}

#[test]
fn layers_are_named_for_reports_and_flags() {
    assert_eq!(
        TestLayer::ALL.map(TestLayer::name),
        ["unit", "integration", "e2e"]
    );
    assert_eq!(TestLayer::EndToEnd.to_string(), "e2e");
    assert_eq!(
        TestLayer::of_test_target("e2e_review_gate"),
        TestLayer::EndToEnd
    );
    assert_eq!(TestLayer::of_test_target("gates"), TestLayer::Integration);
}

#[test]
fn the_plan_names_the_entry_points_and_the_pyramid() {
    let plan = Plan::from_metadata(METADATA).unwrap();

    assert!(plan.pyramid);
    assert_eq!(plan.root, Path::new("/w"));
    assert!(plan.is_entry_point(Path::new("/w/tooling/review-gate/src/main.rs")));
    assert!(!plan.is_entry_point(Path::new("/w/tooling/review-gate/src/domain.rs")));
}

#[test]
fn an_entry_point_must_be_a_path() {
    let err =
        Plan::from_metadata(r#"{"packages": [], "metadata": {"coverage": {"entry-points": [1]}}}"#);

    assert_eq!(err.unwrap_err(), "coverage entry-points: 1 is no path");
}

#[test]
fn a_misspelled_policy_key_or_crate_is_refused() {
    let with = |coverage: &str| {
        let metadata = format!(
            r#"{{"packages": [{{"name": "srs", "manifest_path": "/w/srs/Cargo.toml", "targets": []}}],
               "metadata": {{"coverage": {coverage}}}}}"#
        );
        Plan::from_metadata(&metadata).map(|_| ())
    };

    assert_eq!(
        with(r#"{"floor": {"unit": 99}}"#).unwrap_err(),
        r#"coverage: unknown key "floor""#
    );
    assert_eq!(
        with(r#"{"crates": {"srz": {}}}"#).unwrap_err(),
        r#"coverage crates: "srz" is no workspace crate"#
    );
    assert_eq!(
        with(r#"{"crates": {"srs": {"floor": {}}}}"#).unwrap_err(),
        r#"coverage crates.srs: unknown key "floor""#
    );
    assert_eq!(
        with(r#"{"crates": {"srs": {"floors": {"unit": 50}}}}"#),
        Ok(())
    );
}

#[test]
fn a_package_without_a_name_or_manifest_is_refused() {
    for package in [
        r#"{"name": "a", "targets": []}"#,
        r#"{"manifest_path": "/w/a/Cargo.toml"}"#,
    ] {
        let err = Plan::from_metadata(&format!(r#"{{"packages": [{package}]}}"#)).unwrap_err();
        assert!(
            err.starts_with("cargo metadata: a package without a name or manifest: "),
            "{err}"
        );
    }
}

#[test]
fn typed_reads_a_value_of_its_type_or_refuses_another() {
    let read = |v: &Value| typed(v, Value::as_bool, "pyramid", "boolean");

    assert_eq!(read(&Value::Null), Ok(None));
    assert_eq!(read(&Value::Bool(true)), Ok(Some(true)));
    assert_eq!(
        read(&Value::from(1)),
        Err("coverage pyramid: 1 is no boolean".to_owned())
    );
}
