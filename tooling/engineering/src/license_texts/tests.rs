use super::*;
use crate::tests::checkout;

fn package(name: &str, license: Option<&str>) -> Package {
    Package {
        name: name.into(),
        version: "1.0.0".into(),
        license: license.map(str::to_owned),
        dir: PathBuf::from(format!("/registry/{name}-1.0.0")),
    }
}

fn file(name: &str, text: &str) -> (String, String) {
    (name.to_owned(), text.to_owned())
}

fn mit_file() -> (String, String) {
    file(
        "LICENSE",
        &format!(
            "MIT License\n\nCopyright (c) 2024 Someone\n\n{}",
            canonical("MIT").unwrap()
        ),
    )
}

#[test]
fn every_canonical_text_fits_itself_and_keeps_its_id() {
    for (id, text) in CANON {
        assert!(fits(text, text), "{id}");
        assert_eq!(canonical(id), Some(text));
    }
    assert_eq!(canonical("GPL-3.0-only"), None);
}

#[test]
fn normalize_ignores_case_whitespace_and_copyright_lines() {
    assert_eq!(
        normalize("Copyright (c) 2020 A\n© 2021 B\nCopyright 2023 D\nHello\n  World \n"),
        "hello world"
    );
    assert_eq!(
        normalize("the\ncopyright\nholders"),
        "the copyright holders"
    );
    assert_eq!(normalize("(c) You must retain"), "(c) you must retain");
}

#[test]
fn fits_lets_a_template_fill_in_what_it_leaves_free() {
    let bsd3 = canonical("BSD-3-Clause").unwrap();
    let text = bsd3.replace(FREE, "the University of Somewhere");

    assert!(fits(&text, bsd3));
    assert!(fits(
        &format!("BSD 3-Clause License\n\nCopyright (c) 2020 X\n\n{text}"),
        bsd3
    ));
    assert!(!fits(
        &text.replace("nor the names of its", "or anyone"),
        bsd3
    ));
    assert!(!fits("Permission granted.", canonical("MIT").unwrap()));
}

#[test]
fn fits_refuses_a_clause_inside_a_gap_the_template_fills() {
    let bsd3 = canonical("BSD-3-Clause").unwrap();
    let clause = "anyone, who may use this software for non commercial purposes only and must not sell it or any work derived from it, ";

    assert!(!fits(&bsd3.replace(FREE, clause), bsd3));
    assert!(fits(
        &bsd3.replace(FREE, "the University of Somewhere"),
        bsd3
    ));
}

#[test]
fn fits_refuses_a_term_added_after_the_text_or_a_long_preamble() {
    let mit = canonical("MIT").unwrap();

    assert!(!fits(
        &format!("{mit}\nYou may not use this commercially."),
        mit
    ));
    assert!(!fits(
        &format!("Use is limited to non commercial work, as agreed by all parties here.\n{mit}"),
        mit
    ));
}

#[test]
fn a_clause_starting_c_is_license_text_not_a_copyright_line() {
    let apache = canonical("Apache-2.0").unwrap();
    let clause = "(c) You must retain, in the Source form";
    let altered = apache.replace(clause, "(c) You may drop, in the Source form");

    assert!(apache.contains(clause));
    assert!(!fits(&altered, apache));
}

#[test]
fn the_apache_appendix_is_optional_and_nothing_else_may_follow() {
    let apache = canonical("Apache-2.0").unwrap();
    let with = format!("{apache}\n{APACHE_APPENDIX}");
    let bracketed = with.replace('{', "[").replace('}', "]");

    assert!(fits(apache, apache));
    assert!(fits(&with, apache));
    assert!(fits(&bracketed, apache));
    assert!(!fits(&format!("{apache}\nAnd you owe us a fee."), apache));
    assert!(!fits(&format!("{with}\nAnd you owe us a fee."), apache));
}

#[test]
fn only_apache_may_add_the_appendix() {
    let mit = canonical("MIT").unwrap();

    assert!(!fits(&format!("{mit}\n{APACHE_APPENDIX}"), mit));
    assert!(!fits(
        &format!(
            "{mit}\nAPPENDIX: How to apply the Apache License to your work. Not for commercial use."
        ),
        mit
    ));
}

#[test]
fn appendix_is_exactly_the_apache_boilerplate() {
    let words = normalize(APACHE_APPENDIX);

    assert!(appendix(&words));
    assert!(appendix(&format!("{words} ")));
    assert!(!appendix(&format!("{words} and more")));
    assert!(!appendix("appendix: how to apply the apache license"));
    assert!(!appendix(""));
}

#[test]
fn unmet_names_what_keeps_an_expression_from_being_satisfied() {
    let met = |id: &str| id == "MIT";
    let expr = |s: &str| parse(s).unwrap();

    assert_eq!(unmet(&expr("MIT"), &met), None);
    assert_eq!(unmet(&expr("ISC"), &met), Some(vec!["ISC".to_owned()]));
    assert_eq!(unmet(&expr("MIT OR ISC"), &met), None);
    assert_eq!(
        unmet(&expr("ISC OR NCSA"), &met),
        Some(vec!["ISC".to_owned(), "NCSA".to_owned()])
    );
    assert_eq!(
        unmet(&expr("MIT AND ISC"), &met),
        Some(vec!["ISC".to_owned()])
    );
    assert_eq!(unmet(&expr("MIT AND (ISC OR MIT)"), &met), None);
}

#[test]
fn license_files_also_reads_a_reuse_style_licenses_directory() {
    let (dir, _) = checkout(&[
        ("LICENSE", "top"),
        ("LICENSES/MIT.txt", "m"),
        ("LICENSES/Apache-2.0.txt", "a"),
        ("src/LICENSE", "deep"),
    ]);

    let names: Vec<String> = license_files(dir.path())
        .unwrap()
        .into_iter()
        .map(|f| f.0)
        .collect();

    assert_eq!(
        names,
        ["LICENSE", "LICENSES/Apache-2.0.txt", "LICENSES/MIT.txt"]
    );
}

fn id(s: &str) -> Expr {
    Expr::Id(s.to_owned())
}

#[test]
fn parse_reads_or_and_with_and_the_legacy_slash() {
    let either = Expr::Or(vec![id("MIT"), id("Apache-2.0")]);

    assert_eq!(parse("MIT OR Apache-2.0").unwrap(), either);
    assert_eq!(parse("MIT/Apache-2.0").unwrap(), either);
    assert_eq!(
        parse("Apache-2.0 WITH LLVM-exception OR MIT").unwrap(),
        Expr::Or(vec![id("Apache-2.0"), id("MIT")])
    );
    assert_eq!(parse("MIT").unwrap(), id("MIT"));
}

#[test]
fn parse_binds_and_tighter_than_or_and_honours_parentheses() {
    assert_eq!(
        parse("(MIT OR Apache-2.0) AND Unicode-3.0").unwrap(),
        Expr::And(vec![
            Expr::Or(vec![id("MIT"), id("Apache-2.0")]),
            id("Unicode-3.0")
        ])
    );
    assert_eq!(
        parse("Apache-2.0 OR (MIT AND Unicode-3.0)").unwrap(),
        Expr::Or(vec![
            id("Apache-2.0"),
            Expr::And(vec![id("MIT"), id("Unicode-3.0")])
        ])
    );
    assert_eq!(
        parse("A OR B AND C").unwrap(),
        Expr::Or(vec![id("A"), Expr::And(vec![id("B"), id("C")])])
    );
}

#[test]
fn parse_refuses_what_is_not_an_expression() {
    for bad in [
        "", "MIT OR", "(MIT", "MIT)", "AND MIT", "MIT WITH", "MIT MIT",
    ] {
        assert!(parse(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn license_file_names_cover_the_usual_spellings() {
    for name in [
        "LICENSE",
        "LICENSE-MIT",
        "license-apache-2.0",
        "COPYING",
        "UNLICENSE",
        "LICENCE.md",
    ] {
        assert!(license_file_name(name), "{name}");
    }
    assert!(!license_file_name("README.md"));
}

#[test]
fn a_crate_with_a_fitting_license_file_passes() {
    assert!(check_crate(&package("a", Some("MIT")), &[mit_file()]).is_empty());
    assert!(
        check_crate(
            &package("a", Some("Unlicense OR MIT")),
            &[file("UNLICENSE", "free"), mit_file()]
        )
        .is_empty()
    );
}

#[test]
fn a_crate_with_no_license_file_fails_naming_the_crate_and_version() {
    assert_eq!(
        check_crate(&package("a", Some("MIT")), &[]),
        ["a 1.0.0 ships no license file"]
    );
}

#[test]
fn a_crate_declaring_no_license_fails() {
    assert_eq!(
        check_crate(&package("a", None), &[mit_file()]),
        ["a 1.0.0 declares no license"]
    );
}

#[test]
fn a_license_file_that_does_not_match_the_declared_license_fails() {
    let problems = check_crate(&package("a", Some("Apache-2.0")), &[mit_file()]);

    assert_eq!(
        problems,
        ["a 1.0.0 declares Apache-2.0, but no license file fits Apache-2.0"]
    );
}

#[test]
fn a_part_joined_by_and_needs_its_own_file() {
    let both = package("a", Some("MIT AND Unicode-3.0"));

    assert_eq!(
        check_crate(&both, &[mit_file()]),
        ["a 1.0.0 declares MIT AND Unicode-3.0, but no license file fits Unicode-3.0"]
    );
    assert!(
        check_crate(
            &both,
            &[
                mit_file(),
                file("LICENSE-UNICODE", canonical("Unicode-3.0").unwrap())
            ]
        )
        .is_empty()
    );
}

#[test]
fn a_license_with_no_canonical_text_cannot_be_checked() {
    assert_eq!(
        check_crate(&package("a", Some("WTFPL")), &[mit_file()]),
        [
            "a 1.0.0 declares WTFPL, but no license file fits WTFPL (the tool keeps no text for WTFPL)"
        ]
    );
}

#[test]
fn a_nested_expression_is_satisfied_by_either_side_of_its_or() {
    let nested = package("a", Some("Apache-2.0 OR (MIT AND Unicode-3.0)"));
    let apache = file("LICENSE", canonical("Apache-2.0").unwrap());

    assert!(check_crate(&nested, &[apache]).is_empty());
    assert_eq!(
        check_crate(&nested, &[mit_file()]),
        [
            "a 1.0.0 declares Apache-2.0 OR (MIT AND Unicode-3.0), but no license file fits Apache-2.0 or Unicode-3.0"
        ]
    );
}

#[test]
fn a_license_that_does_not_parse_fails() {
    assert_eq!(
        check_crate(&package("a", Some("MIT OR")), &[mit_file()]),
        ["a 1.0.0 declares a license that does not parse: expression ends early"]
    );
}

#[test]
fn packages_lists_registry_crates_by_name_and_leaves_the_workspace_out() {
    let json = r#"{"packages":[
        {"name":"zed","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/zed/Cargo.toml"},
        {"name":"own","version":"0.0.0","source":null,"license":"MIT","manifest_path":"/w/own/Cargo.toml"},
        {"name":"alpha","version":"2.0.0","source":"registry+x","manifest_path":"/r/alpha/Cargo.toml"}
    ]}"#;
    let found = packages(json).unwrap();

    assert_eq!(
        found.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["alpha", "zed"]
    );
    assert_eq!(found[0].license, None);
    assert_eq!(found[1].dir, PathBuf::from("/r/zed"));
    assert!(
        packages("nope")
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
    let nameless = r#"{"packages":[{"version":"1.0.0","source":"registry+x","manifest_path":"/r/x/Cargo.toml"}]}"#;
    assert!(
        packages(nameless)
            .unwrap_err()
            .starts_with("cargo metadata lists a crate without a name, version or manifest path")
    );
}

#[test]
fn license_files_reads_only_license_named_files_in_name_order() {
    let (dir, _) = checkout(&[
        ("LICENSE-MIT", "m"),
        ("LICENSE-APACHE", "a"),
        ("README.md", "r"),
    ]);
    fs::write(dir.path().join("COPYING"), [0xff, 0xfe]).unwrap();
    fs::create_dir(dir.path().join("LICENSES")).unwrap();

    let names: Vec<String> = license_files(dir.path())
        .unwrap()
        .into_iter()
        .map(|f| f.0)
        .collect();

    assert_eq!(names, ["COPYING", "LICENSE-APACHE", "LICENSE-MIT"]);
    assert!(
        license_files(&dir.path().join("missing"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn check_tree_reports_every_crate_that_fails_and_every_dir_it_cannot_read() {
    let json = r#"{"packages":[
        {"name":"a","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/a/Cargo.toml"},
        {"name":"b","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/b/Cargo.toml"},
        {"name":"c","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/c/Cargo.toml"}
    ]}"#;
    let problems = check_tree(json, |dir| match dir.to_str().unwrap() {
        "/r/a" => Ok(vec![mit_file()]),
        "/r/b" => Ok(vec![]),
        _ => Err("read /r/c: gone".into()),
    })
    .unwrap();

    assert_eq!(
        problems,
        ["b 1.0.0 ships no license file", "read /r/c: gone"]
    );
    assert!(check_tree("nope", |_| Ok(vec![])).is_err());
}

#[test]
fn license_files_without_a_licenses_directory_reads_the_top_only() {
    let (dir, _) = checkout(&[("LICENSE", "top")]);

    let files = license_files(dir.path()).unwrap();

    assert_eq!(files, [("LICENSE".to_owned(), "top".to_owned())]);
}

#[test]
fn canon_covers_every_license_deny_toml_allows_or_grants() {
    let deny =
        "allow = [\"MIT\", \"ISC\"]\nexceptions = [\n  { crate = \"x\", allow = [\"NCSA\"] },\n]\n";
    assert!(canon_covers(deny).is_empty());

    let uncovered =
        "allow = [\"MIT\"]\nexceptions = [\n  { crate = \"x\", allow = [\"WTFPL\"] },\n]\n";
    assert_eq!(
        canon_covers(uncovered),
        ["deny.toml grants WTFPL, which the license tool keeps no text for"]
    );
    assert!(canon_covers("").is_empty());

    let reflowed = "[licenses]\nallow = [\n  \"MIT\", # one\n  \"WTFPL\",\n]\n";
    assert_eq!(
        canon_covers(reflowed),
        ["deny.toml grants WTFPL, which the license tool keeps no text for"]
    );
}

#[test]
fn kept_texts_flags_a_license_file_the_repository_keeps_for_a_crate() {
    let (_dir, c) = checkout(&[
        (
            "licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES",
            "all",
        ),
        ("third_party/serde-1.0.0/LICENSE-MIT", "m"),
        ("vendor/syn/COPYING", "c"),
        ("vendor/syn/README.md", "r"),
        ("vendor/other/LICENSE", "o"),
        ("vendor/syn-extra-1/LICENSE", "o"),
    ]);
    let found = kept_texts(&c, &[package("serde", None), package("syn", None)]);

    assert_eq!(
        found,
        [
            "the repository keeps a license text at third_party/serde-1.0.0/LICENSE-MIT",
            "the repository keeps a license text at vendor/syn/COPYING",
        ]
    );
    let (_dir, none) = checkout(&[("README.md", "x")]);
    assert!(kept_texts(&none, &[package("serde", None)]).is_empty());
}

#[test]
fn kept_texts_reports_a_directory_it_cannot_read() {
    let (_dir, c) = checkout(&[("vendor", "a file where a directory should be")]);

    let found = kept_texts(&c, &[package("serde", None)]);

    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("read "), "{found:?}");
}

fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

/// A resolved graph: \`app\` (a binary under crates/) reaches \`serde\`
/// through a normal dependency and \`cc\` through a build dependency; a
/// tooling crate reaches \`tool-only\`; \`dev\` is only a dev dependency.
const GRAPH: &str = r#"{"workspace_root":"/w","packages":[
  {"id":"app 0","name":"app","version":"0.0.0","source":null,"manifest_path":"/w/crates/app/Cargo.toml","targets":[{"kind":["bin"]}]},
  {"id":"tool 0","name":"tool","version":"0.0.0","source":null,"manifest_path":"/w/tooling/tool/Cargo.toml","targets":[{"kind":["bin"]}]},
  {"id":"serde 1","name":"serde","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/serde/Cargo.toml","targets":[{"kind":["lib"]}]},
  {"id":"cc 1","name":"cc","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/cc/Cargo.toml","targets":[{"kind":["lib"]}]},
  {"id":"dev 1","name":"dev","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/dev/Cargo.toml","targets":[{"kind":["lib"]}]},
  {"id":"derive 1","name":"derive","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/derive/Cargo.toml","targets":[{"kind":["proc-macro"]}]},
  {"id":"syn 1","name":"syn","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/syn/Cargo.toml","targets":[{"kind":["lib"]}]},
  {"id":"util 1","name":"util","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/util/Cargo.toml","targets":[{"kind":["lib"]}]},
  {"id":"tool-only 1","name":"tool-only","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/tool-only/Cargo.toml","targets":[{"kind":["lib"]}]}
 ],"resolve":{"nodes":[
  {"id":"app 0","deps":[
    {"pkg":"serde 1","dep_kinds":[{"kind":null,"target":null}]},
    {"pkg":"util 1","dep_kinds":[{"kind":null,"target":null}]},
    {"pkg":"cc 1","dep_kinds":[{"kind":"build","target":null}]},
    {"pkg":"dev 1","dep_kinds":[{"kind":"dev","target":null}]}]},
  {"id":"tool 0","deps":[{"pkg":"tool-only 1","dep_kinds":[{"kind":null,"target":null}]}]},
  {"id":"util 1","deps":[{"pkg":"serde 1","dep_kinds":[{"kind":null,"target":null}]}]},
  {"id":"derive 1","deps":[{"pkg":"syn 1","dep_kinds":[{"kind":null,"target":null}]}]},
  {"id":"serde 1","deps":[{"pkg":"derive 1","dep_kinds":[{"kind":null,"target":null}]}]}]}}"#;

#[test]
fn linked_follows_only_normal_dependencies_from_the_products_binaries() {
    let found = linked(GRAPH, Path::new("crates/")).unwrap();

    assert_eq!(
        found.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["serde", "util"]
    );
}

#[test]
fn linked_refuses_metadata_it_cannot_follow() {
    assert!(
        linked("nope", Path::new("crates/"))
            .unwrap_err()
            .starts_with("read cargo metadata: ")
    );
    assert_eq!(
        linked(r#"{"packages":[]}"#, Path::new("crates/")).unwrap_err(),
        "cargo metadata holds no resolved graph"
    );
    assert_eq!(
        linked(
            r#"{"packages":[],"resolve":{"nodes":[]}}"#,
            Path::new("crates/")
        )
        .unwrap_err(),
        "cargo metadata names no workspace_root"
    );
}

#[test]
fn linked_reports_a_linked_crate_without_a_manifest_path() {
    let json = r#"{"workspace_root":"/w","packages":[
      {"id":"app 0","name":"app","version":"0.0.0","source":null,"manifest_path":"/w/crates/app/Cargo.toml","targets":[{"kind":["bin"]}]},
      {"id":"x 1","name":"x","version":"1.0.0","source":"registry+x"}],
      "resolve":{"nodes":[{"id":"app 0","deps":[{"pkg":"x 1","dep_kinds":[{"kind":null}]}]}]}}"#;

    assert!(
        linked(json, Path::new("crates/"))
            .unwrap_err()
            .starts_with("cargo metadata lists a crate without")
    );
}

#[test]
fn linked_for_release_joins_the_targets_once_each() {
    let two = GRAPH.to_owned();
    let found = linked_for_release(&["a".to_owned(), "b".to_owned()], |_| Ok(two.clone())).unwrap();

    assert_eq!(found.len(), 2);
    assert_eq!(
        linked_for_release(&["a".to_owned()], |t| Err(format!("no {t}"))).unwrap_err(),
        "no a"
    );
}

#[test]
fn release_targets_reads_the_rust_target_of_each_pair() {
    let workflow = "env:\n  LINUX_TARGETS: >-\n    x86_64-unknown-linux-musl:linux-amd64\n    aarch64-unknown-linux-musl:linux-arm64\n  DARWIN_TARGETS: >-\n    aarch64-apple-darwin:darwin-arm64\n  MINGW: >-\n    gcc-mingw-w64-x86-64-win32=13.2.0\n";

    assert_eq!(
        release_targets(workflow).unwrap(),
        [
            "aarch64-apple-darwin",
            "aarch64-unknown-linux-musl",
            "x86_64-unknown-linux-musl"
        ]
    );
    assert_eq!(
        release_targets("env:\n  MINGW: >-\n    x\n").unwrap_err(),
        "the build workflow names no release target"
    );
}

#[test]
fn check_packages_reports_each_failure_and_each_unreadable_directory() {
    let a = package("a", Some("MIT"));
    let b = package("b", Some("MIT"));
    let c = package("c", Some("MIT"));
    let problems = check_packages(&[a, b, c], |dir| match dir.to_str().unwrap() {
        "/registry/a-1.0.0" => Ok(vec![mit_file()]),
        "/registry/b-1.0.0" => Ok(vec![]),
        _ => Err("read c: gone".into()),
    });

    assert_eq!(problems, ["b 1.0.0 ships no license file", "read c: gone"]);
}

#[test]
fn ci_must_run_license_check_with_the_locked_tree() {
    let on = "on:\n  pull_request:\n    branches: [main]\n";
    let step = "      - run: cargo run --locked -p engineering --bin license-check\n";
    let ok = format!("{on}{step}");

    assert!(ci_runs_check(&ok).is_empty());
    for bad in [
        "      - run: cargo run -p engineering --bin license-check\n",
        "      - run: cargo run --locked -p engineering --bin protection-check\n",
        "      # - run: cargo run --locked -p engineering --bin license-check\n",
        "",
    ] {
        assert_eq!(
            ci_runs_check(&format!("{on}{bad}")),
            ["the CI workflow does not run license-check with --locked"],
            "{bad}"
        );
    }
    assert_eq!(
        ci_runs_check(&format!("on:\n  push:\n{step}")),
        ["the CI workflow does not run on every pull request"]
    );
}

#[test]
fn problems_split_into_missing_files_and_texts_that_do_not_fit() {
    let missing = check_crate(&package("a", Some("MIT")), &[]);
    let unfit = check_crate(&package("b", Some("Apache-2.0")), &[mit_file()]);
    let all = [
        missing.clone(),
        unfit.clone(),
        vec!["read /r/c: gone".into()],
    ]
    .concat();

    let (no_file, not_fit) = split_problems(all);

    assert_eq!(
        no_file,
        [missing, vec!["read /r/c: gone".to_owned()]].concat()
    );
    assert_eq!(not_fit, unfit);
}

#[cfg(unix)]
#[test]
fn cargo_metadata_for_never_lets_cargo_fetch() {
    use std::os::unix::fs::PermissionsExt;
    let (dir, _) = checkout(&[("cargo", "#!/bin/sh\necho \"$@\" > \"$0.args\"\nexit 1\n")]);
    let fake = dir.path().join("cargo");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();

    let err =
        cargo_metadata_for(fake.as_os_str(), Path::new("Cargo.toml"), dir.path(), "t").unwrap_err();

    assert!(
        err.contains("cargo fetch --locked fetches the sources"),
        "{err}"
    );
    let args = fs::read_to_string(dir.path().join("cargo.args")).unwrap();
    assert!(
        args.contains("--offline") && args.contains("--locked"),
        "{args}"
    );
}

const APP: &str = "[package]\nname = \"app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

pub(crate) const BUILD_FIXTURE: &str =
    "env:\n  LINUX_TARGETS: >-\n    x86_64-unknown-linux-musl:linux-amd64\n";

/// A workspace with one product binary, no dependencies, and the files
/// the check reads.
pub(crate) fn workspace(deny: &str) -> (testkit::TempDir, Checkout) {
    checkout(&[
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/app\"]\nresolver = \"3\"\n",
        ),
        (
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.0.0\"\n",
        ),
        ("crates/app/Cargo.toml", APP),
        ("crates/app/src/main.rs", "fn main() {}\n"),
        (".github/workflows/build.yml", BUILD_FIXTURE),
        ("deny.toml", deny),
    ])
}

#[test]
fn cargo_metadata_for_resolves_a_workspace_for_a_target_and_names_a_failure() {
    let (dir, c) = workspace("");
    let home = dir.path().join(".parinn");
    let manifest = c.root.join("Cargo.toml");

    let json = cargo_metadata_for(&cargo(), &manifest, &home, "x86_64-unknown-linux-musl").unwrap();
    assert!(json.contains("\"name\":\"app\""), "{json}");

    let err = cargo_metadata_for(
        &cargo(),
        &c.root.join("missing/Cargo.toml"),
        &home,
        "x86_64-unknown-linux-musl",
    )
    .unwrap_err();
    assert!(
        err.starts_with("cargo metadata for x86_64-unknown-linux-musl failed ("),
        "{err}"
    );
    let err = cargo_metadata_for(
        std::ffi::OsStr::new("/nonexistent/cargo"),
        &manifest,
        &home,
        "t",
    );
    assert!(err.unwrap_err().starts_with("run cargo metadata: "));
}

#[test]
fn product_links_check_names_a_product_crate_that_depends_on_the_check() {
    let json = r#"{"workspace_root":"/w","packages":[
      {"name":"core","manifest_path":"/w/crates/core/Cargo.toml","dependencies":[{"name":"engineering","kind":null}]},
      {"name":"clean","manifest_path":"/w/crates/clean/Cargo.toml","dependencies":[{"name":"serde","kind":null}]},
      {"name":"dev","manifest_path":"/w/crates/dev/Cargo.toml","dependencies":[{"name":"engineering","kind":"dev"}]},
      {"name":"tool","manifest_path":"/w/tooling/tool/Cargo.toml","dependencies":[{"name":"engineering","kind":null}]}]}"#;

    assert_eq!(
        product_links_check(json).unwrap(),
        ["core depends on the engineering crate, which would ship the license check"]
    );
    assert!(product_links_check("nope").is_err());
}

#[cfg(unix)]
fn script(body: &str) -> (testkit::TempDir, std::process::Child) {
    use std::os::unix::fs::PermissionsExt;
    let (dir, _) = checkout(&[("run.sh", body)]);
    let path = dir.path().join("run.sh");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let child = Command::new(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    (dir, child)
}

#[cfg(unix)]
#[test]
fn finish_within_collects_the_output_of_a_child_that_finishes() {
    let (_dir, child) = script("#!/bin/sh\necho out\necho err >&2\nexit 3\n");

    let out = finish_within(child, Duration::from_secs(30)).unwrap();

    assert_eq!(out.status.code(), Some(3));
    assert_eq!(
        (out.stdout, out.stderr),
        (b"out\n".to_vec(), b"err\n".to_vec())
    );
}

#[cfg(unix)]
#[test]
fn finish_within_kills_a_child_still_running_at_the_deadline() {
    let (_dir, child) = script("#!/bin/sh\nsleep 30\n");

    let err = finish_within(child, Duration::from_millis(100)).unwrap_err();

    assert_eq!(err, "cargo did not finish within 100ms");
}

#[test]
fn a_wrapped_sentence_with_a_number_is_not_a_copyright_line() {
    assert!(!copyright_line(
        "copyright holder, and you cure the violation prior to 30 days after"
    ));
    assert!(!copyright_line(
        "copyright holders be liable for 12 reasons"
    ));
    for line in [
        "Copyright (c) 2024 A",
        "Copyright 1999 B",
        "Copyright © C",
        "  COPYRIGHT 2010-2012 D",
        "Copyright {yyyy} {name}",
    ] {
        assert!(copyright_line(line), "{line}");
    }
    assert!(has_year("in 2024") && has_year("1999") && !has_year("30 days") && !has_year("3000"));
}
