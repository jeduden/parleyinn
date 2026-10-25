use super::*;
use testkit::TempDir;

const LOCK: &str = "\
[[package]]
name = \"serde_json\"
version = \"1.0.0\"

[[package]]
name = \"sentry-core\"
version = \"0.3.0\"

[[package]]
name = \"hyper\"

[[package]]
name = \"hyperfine-like\"

[[package]]
name = \"self_update\"
";

#[test]
fn the_lockfile_names_its_crates() {
    assert_eq!(
        locked(LOCK),
        [
            "serde_json",
            "sentry-core",
            "hyper",
            "hyperfine-like",
            "self_update"
        ]
    );
}

#[test]
fn a_crate_that_reports_updates_or_reaches_a_host_is_denied_by_name_or_family() {
    let found = denied_crates("Cargo.lock", LOCK);

    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].starts_with("Cargo.lock resolves sentry-core"));
    assert!(found[1].contains("resolves hyper,"));
    assert!(found[2].contains("resolves self_update"));
    assert_eq!(
        denied_crates("fuzz/Cargo.lock", "name = \"serde_json\"\n"),
        Vec::<String>::new()
    );
}

#[test]
fn every_lockfile_the_repository_commits_is_read() {
    assert_eq!(LOCKFILES, ["Cargo.lock", "fuzz/Cargo.lock"]);
    assert!(denied_crates("fuzz/Cargo.lock", LOCK)[0].starts_with("fuzz/Cargo.lock resolves"));
}

#[test]
fn a_web_address_in_code_is_named_by_file_and_line() {
    assert_eq!(
        addresses("a.rs", "fn x() {}\nlet u = \"https://x.test\";\n").len(),
        1
    );
    assert_eq!(
        addresses("a.rs", "ok\nlet u = \"http://x.test\";\n")[0],
        "a.rs:2: names a web address: let u = \"http://x.test\";"
    );
    assert_eq!(addresses("a.rs", "no address"), Vec::<String>::new());
}

fn product(files: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for (path, body) in files {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    }
    let c = Checkout::new(dir.path());

    (dir, c)
}

#[test]
fn shipped_code_names_no_address_and_tests_are_left_out() {
    let (_dir, c) = product(&[
        ("crates/a/src/lib.rs", "pub fn f() {}"),
        ("crates/a/src/tests.rs", "// https://ok.test"),
        ("crates/a/tests/e2e.rs", "// https://ok.test"),
        ("crates/a/src/tests/more.rs", "// https://ok.test"),
        ("crates/a/README.md", "https://ok.test"),
    ]);
    assert_eq!(shipped_addresses(&c), Vec::<String>::new());

    let (_dir, c) = product(&[("crates/a/src/net.rs", "const U: &str = \"https://x.test\";")]);
    assert_eq!(shipped_addresses(&c).len(), 1);
    assert!(shipped_addresses(&c)[0].starts_with("crates/a/src/net.rs:1:"));
}

#[test]
fn an_unlistable_crates_directory_is_named() {
    let (_dir, c) = product(&[("README.md", "")]);

    assert!(
        shipped_addresses(&c)[0].starts_with("list "),
        "{:?}",
        shipped_addresses(&c)
    );
}

#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_read_is_named() {
    let (_dir, c) = product(&[("crates/a/src/lib.rs", "")]);
    std::os::unix::fs::symlink("/nonexistent", c.root.join("crates/a/src/dangling.rs")).unwrap();

    let found = shipped_addresses(&c);

    assert_eq!(found.len(), 1);
    assert!(
        found[0].starts_with("read crates/a/src/dangling.rs:"),
        "{found:?}"
    );
}

fn row(name: &str, component: &str, requirements: &str) -> Row {
    Row {
        id: "X5".to_owned(),
        name: name.to_owned(),
        component: component.to_owned(),
        boundary: "—".to_owned(),
        reach: String::new(),
        default: "—".to_owned(),
        requirements: requirements.to_owned(),
    }
}

#[test]
fn the_register_must_reject_update_checks_and_telemetry_under_sec_15() {
    let good = row(
        "Price-table fetch, update checks, telemetry",
        "rejected",
        "SEC-15",
    );
    assert_eq!(
        register_rejects(std::slice::from_ref(&good)),
        Vec::<String>::new()
    );

    for bad in [
        row(
            "Price-table fetch, update checks, telemetry",
            "core",
            "SEC-15",
        ),
        row(
            "Price-table fetch, update checks, telemetry",
            "rejected",
            "I4",
        ),
        row("Update checks", "rejected", "SEC-15"),
    ] {
        assert_eq!(
            register_rejects(&[bad]),
            ["no rejected register row names update checks and telemetry under SEC-15"]
        );
    }
}

#[test]
fn the_check_reads_the_lockfile_the_code_and_the_register() {
    let (_dir, c) = product(&[]);
    let problems = check(&c);
    assert!(problems[0].starts_with("read Cargo.lock"), "{problems:?}");
    assert!(
        problems[1].starts_with("read fuzz/Cargo.lock"),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|p| p.starts_with("read docs/srs")),
        "{problems:?}"
    );

    assert_eq!(
        check(&Checkout::new(&testkit::repo_root())),
        Vec::<String>::new()
    );
}
