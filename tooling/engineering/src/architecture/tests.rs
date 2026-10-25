use super::*;
use serde_json::{Value, json};

/// One package of `cargo metadata --no-deps`: `meta` is its
/// `[package.metadata.parinn]` table, `deps` its dependencies as
/// `(name, kind)`.
fn package(name: &str, dir: &str, meta: &Value, deps: &[(&str, Option<&str>)]) -> Value {
    json!({
        "name": name,
        "manifest_path": format!("/repo/{dir}/Cargo.toml"),
        "targets": [{ "kind": ["lib"] }, { "kind": ["test"] }],
        "metadata": { "parinn": meta },
        "dependencies": deps
            .iter()
            .map(|(n, k)| json!({ "name": n, "kind": k, "path": format!("/repo/{n}") }))
            .chain([json!({ "name": "serde", "kind": null, "source": "registry+x" })])
            .collect::<Vec<_>>(),
    })
}

fn metadata(packages: &[Value]) -> String {
    json!({ "workspace_root": "/repo", "packages": packages }).to_string()
}

fn declares(responsibility: &str, may: &[&str]) -> Value {
    json!({ "responsibility": responsibility, "may-depend-on": may })
}

/// A product crate `core` and the tooling crates `kit` and `gate`.
fn sample() -> Vec<WorkspaceCrate> {
    let packages = [
        package(
            "core",
            "crates/core",
            &json!({
                "responsibility": "runs the hooks",
                "may-depend-on": [],
                "may-dev-depend-on": ["kit"],
            }),
            &[("kit", Some("dev"))],
        ),
        package("kit", "tooling/kit", &declares("helps tests", &[]), &[]),
        package(
            "gate",
            "tooling/gate",
            &declares("holds a gate", &["kit"]),
            &[("kit", None)],
        ),
    ];

    workspace(&metadata(&packages)).unwrap()
}

fn member<'a>(members: &'a mut [WorkspaceCrate], name: &str) -> &'a mut WorkspaceCrate {
    members.iter_mut().find(|m| m.name == name).unwrap()
}

fn link(on: &str, dev: bool) -> Link {
    Link {
        on: on.to_owned(),
        dev,
    }
}

#[test]
fn workspace_reads_each_crate_and_its_declarations() {
    let packages = [
        package(
            "gate",
            "tooling/gate",
            &json!({
                "responsibility": "holds a gate",
                "may-depend-on": ["kit"],
                "may-dev-depend-on": ["probe"],
            }),
            &[
                ("kit", None),
                ("probe", Some("dev")),
                ("gen", Some("build")),
            ],
        ),
        json!({
            "name": "bare",
            "manifest_path": "/repo/crates/bare/Cargo.toml",
            "targets": [{ "kind": ["bin"] }],
            "metadata": null,
            "dependencies": [],
        }),
    ];

    let got = workspace(&metadata(&packages)).unwrap();

    assert_eq!(
        got,
        [
            WorkspaceCrate {
                name: "bare".into(),
                dir: "crates/bare".into(),
                ..WorkspaceCrate::default()
            },
            WorkspaceCrate {
                name: "gate".into(),
                dir: "tooling/gate".into(),
                library: true,
                responsibility: vec!["holds a gate".into()],
                may_depend_on: Some(vec!["kit".into()]),
                may_dev_depend_on: vec!["probe".into()],
                links: vec![link("kit", false), link("probe", true), link("gen", false)],
            },
        ]
    );
}

#[test]
fn workspace_reads_a_responsibility_given_as_a_list() {
    let packages = [package(
        "gate",
        "tooling/gate",
        &json!({ "responsibility": ["a", "b"] }),
        &[],
    )];

    let got = workspace(&metadata(&packages)).unwrap();

    assert_eq!(got[0].responsibility, ["a", "b"]);
}

#[test]
fn workspace_names_what_it_cannot_read() {
    let bad = |meta: Value| {
        workspace(&metadata(&[package("gate", "tooling/gate", &meta, &[])])).unwrap_err()
    };

    assert!(
        workspace("not json")
            .unwrap_err()
            .starts_with("read cargo metadata")
    );
    assert_eq!(
        workspace(r#"{"packages": []}"#).unwrap_err(),
        "cargo metadata names no workspace root"
    );
    assert_eq!(
        workspace(r#"{"workspace_root": "/repo"}"#).unwrap_err(),
        "cargo metadata lists no packages"
    );
    assert_eq!(
        bad(json!({ "responsibility": 3 })),
        "crate gate: responsibility must be a string"
    );
    assert_eq!(
        bad(json!({ "responsibility": ["a", 3] })),
        "crate gate: responsibility must be a string"
    );
    assert_eq!(
        bad(json!({ "may-depend-on": "kit" })),
        "crate gate: may-depend-on must be a list of crate names"
    );
    assert_eq!(
        bad(json!({ "may-dev-depend-on": [1] })),
        "crate gate: may-dev-depend-on must be a list of crate names"
    );
    let outside = json!({
        "workspace_root": "/repo",
        "packages": [{ "name": "far", "manifest_path": "/elsewhere/Cargo.toml" }],
    });
    assert_eq!(
        workspace(&outside.to_string()).unwrap_err(),
        "crate far: manifest /elsewhere/Cargo.toml is outside the workspace"
    );
}

#[test]
fn directions_hold_for_the_declared_ones() {
    assert_eq!(directions(&sample()), Vec::<String>::new());
}

#[test]
fn directions_name_both_crates_of_a_forbidden_dependency() {
    let mut members = sample();
    member(&mut members, "kit").links.push(link("gate", false));
    member(&mut members, "gate").links.push(link("core", true));

    assert_eq!(
        directions(&members),
        [
            "crate gate depends on crate core (dev), which its declared directions forbid",
            "crate kit depends on crate gate, which its declared directions forbid",
        ]
    );
}

#[test]
fn directions_want_each_crate_to_declare_its_own() {
    let mut members = sample();
    member(&mut members, "kit").may_depend_on = None;

    assert_eq!(
        directions(&members),
        ["crate kit declares no may-depend-on directions"]
    );
}

#[test]
fn directions_refuse_a_name_that_is_no_workspace_crate() {
    let mut members = sample();
    member(&mut members, "gate").may_depend_on = Some(vec!["kit".into(), "ghost".into()]);
    member(&mut members, "core")
        .may_dev_depend_on
        .push("phantom".into());

    assert_eq!(
        directions(&members),
        [
            "crate core names crate phantom in may-dev-depend-on, which is no workspace crate",
            "crate gate names crate ghost in may-depend-on, which is no workspace crate",
        ]
    );
}

#[test]
fn directions_never_let_a_product_crate_depend_on_tooling() {
    let mut members = sample();
    member(&mut members, "core").may_depend_on = Some(vec!["kit".into()]);
    member(&mut members, "core").links.push(link("kit", false));
    member(&mut members, "core").links.push(link("gate", false));

    assert_eq!(
        directions(&members),
        [
            "product crate core depends on tooling crate gate, which a product crate never does",
            "product crate core depends on tooling crate kit, which a product crate never does",
            "product crate core may depend on tooling crate kit, which a product crate never does",
        ]
    );
}

#[test]
fn directions_hold_a_product_crate_to_what_it_declares_for_its_tests() {
    let mut members = sample();
    member(&mut members, "core").may_dev_depend_on.clear();

    assert_eq!(
        directions(&members),
        ["crate core depends on crate kit (dev), which its declared directions forbid"]
    );
}

#[test]
fn directions_let_a_product_crate_depend_on_a_product_crate() {
    let mut members = sample();
    members.push(WorkspaceCrate {
        name: "store".into(),
        dir: "crates/store".into(),
        library: true,
        may_depend_on: Some(vec![]),
        ..WorkspaceCrate::default()
    });
    member(&mut members, "core").may_depend_on = Some(vec!["store".into()]);
    member(&mut members, "core")
        .links
        .push(link("store", false));
    member(&mut members, "store")
        .links
        .push(link("core", false));

    assert_eq!(
        directions(&members),
        ["crate store depends on crate core, which its declared directions forbid"]
    );
}

#[test]
fn responsibilities_want_exactly_one_per_library_crate() {
    assert_eq!(responsibilities(&sample()), Vec::<String>::new());
    let mut members = sample();
    member(&mut members, "core").responsibility.clear();
    member(&mut members, "kit").responsibility = vec!["a".into(), "b".into()];
    member(&mut members, "gate").responsibility = vec!["  ".into()];

    assert_eq!(
        responsibilities(&members),
        [
            "library crate core declares no responsibility",
            "library crate gate declares an empty responsibility",
            "library crate kit declares 2 responsibilities, want exactly one",
        ]
    );
}

#[test]
fn responsibilities_are_not_shared_and_not_asked_of_a_binary() {
    let mut members = sample();
    member(&mut members, "kit").responsibility = vec!["runs the hooks".into()];
    members.push(WorkspaceCrate {
        name: "tool".into(),
        dir: "tooling/tool".into(),
        ..WorkspaceCrate::default()
    });

    assert_eq!(
        responsibilities(&members),
        ["crates core and kit declare the same responsibility: runs the hooks"]
    );
}

#[test]
fn a_forbidden_dependency_is_one_the_check_refuses_naming_both_crates() {
    let (broken, from, on) = with_forbidden_dependency(&sample()).unwrap();

    assert_eq!((from.as_str(), on.as_str()), ("core", "gate"));
    let found = directions(&broken);
    assert_eq!(names_both(&found, &from, &on), Ok(()));
    assert_eq!(directions(&sample()), Vec::<String>::new());
}

#[test]
fn a_forbidden_dependency_needs_a_product_and_a_tooling_crate() {
    let want = Err("the workspace needs a product crate and a tooling crate".to_owned());
    let tooling_only: Vec<WorkspaceCrate> =
        sample().into_iter().filter(|m| m.is_tooling()).collect();
    let product_only: Vec<WorkspaceCrate> =
        sample().into_iter().filter(|m| m.is_product()).collect();

    assert_eq!(with_forbidden_dependency(&tooling_only).map(|_| ()), want);
    assert_eq!(with_forbidden_dependency(&product_only).map(|_| ()), want);
}

#[test]
fn names_both_wants_whole_crate_names_on_one_line() {
    let found = vec!["crate core depends on crate gate, which no".to_owned()];

    assert_eq!(names_both(&found, "core", "gate"), Ok(()));
    assert!(names_both(&found, "cor", "gate").is_err());
    assert!(names_both(&found, "core", "kit").is_err());
    let apart = vec!["crate core".to_owned(), "crate gate".to_owned()];
    assert!(names_both(&apart, "core", "gate").is_err());
}

#[test]
fn problems_gather_both_checks_or_the_one_it_cannot_read() {
    let packages = [
        package("core", "crates/core", &declares("a", &["kit"]), &[]),
        package("kit", "tooling/kit", &declares("", &[]), &[("core", None)]),
    ];

    assert_eq!(
        problems(&metadata(&packages)),
        [
            "crate kit depends on crate core, which its declared directions forbid",
            "product crate core may depend on tooling crate kit, which a product crate never does",
            "library crate kit declares an empty responsibility",
        ]
    );
    assert!(problems("not json")[0].starts_with("read cargo metadata"));
}
