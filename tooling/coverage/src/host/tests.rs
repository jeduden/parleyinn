use super::*;
use std::fs;
use testkit::TempDir;

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn cargo_runs_and_returns_its_stdout() {
    let cargo = Cargo::from_env();

    let version = cargo.cargo_output(&args(&["--version"])).unwrap();
    assert!(
        version.starts_with("cargo ") && version.ends_with('\n'),
        "{version}"
    );
    assert_eq!(cargo.cargo(&args(&["--version"])), Ok(()));
}

#[test]
fn a_failing_cargo_names_its_command() {
    let cargo = Cargo::from_env();

    assert!(
        cargo
            .cargo(&args(&["no-such-command"]))
            .unwrap_err()
            .starts_with("cargo no-such-command: exit status: ")
    );
    assert!(
        cargo
            .cargo_output(&args(&["no-such-command"]))
            .unwrap_err()
            .starts_with("cargo no-such-command: exit status: ")
    );
}

#[test]
fn a_cargo_that_cannot_start_says_so() {
    let missing = Cargo {
        program: "/nonexistent/cargo".into(),
    };

    assert!(
        missing
            .cargo(&args(&["x"]))
            .unwrap_err()
            .starts_with("run cargo x: ")
    );
    assert!(
        missing
            .cargo_output(&args(&["x"]))
            .unwrap_err()
            .starts_with("run cargo x: ")
    );
}

#[test]
fn write_creates_its_directory_and_append_adds() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cov/unit.lcov");
    let cargo = Cargo::from_env();

    cargo.write(&path, "a").unwrap();
    cargo.append(&path, "b").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "ab");
    assert!(
        cargo
            .write(&path.join("x"), "a")
            .unwrap_err()
            .starts_with("write ")
    );
    assert!(
        cargo
            .append(&dir.path().join("missing/x"), "a")
            .unwrap_err()
            .starts_with("append to ")
    );
}

#[test]
fn read_gives_a_file_s_text_or_none_for_a_missing_one() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("unit.lcov");
    fs::write(&path, "a").unwrap();
    let cargo = Cargo::from_env();

    assert_eq!(cargo.read(&path), Ok(Some("a".into())));
    assert_eq!(cargo.read(&dir.path().join("missing.lcov")), Ok(None));
    assert!(cargo.read(dir.path()).unwrap_err().starts_with("read "));
}

#[test]
fn sources_reads_every_rust_file_but_build_output() {
    let dir = TempDir::new().unwrap();
    for (name, text) in [
        ("src/lib.rs", "a"),
        ("src/x/tests.rs", "b"),
        ("target/gen.rs", "c"),
        ("README.md", "d"),
    ] {
        let path = dir.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    let mut got = Cargo::from_env().sources(dir.path()).unwrap();
    got.sort();

    assert_eq!(
        got,
        [
            (dir.path().join("src/lib.rs"), "a".into()),
            (dir.path().join("src/x/tests.rs"), "b".into())
        ]
    );
    assert!(
        Cargo::from_env()
            .sources(&dir.path().join("missing"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[cfg(unix)]
#[test]
fn sources_reports_a_file_it_cannot_read() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("bad.rs"), [0xff, 0xfe]).unwrap();

    assert!(
        Cargo::from_env()
            .sources(dir.path())
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn bound_scenarios_counts_the_ones_not_pending() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.feature"),
        "Feature: F\n\n  @REC-01\n  Scenario: a\n    Given x\n\n  @REC-02 @pending\n  Scenario: b\n    Given x\n",
    )
    .unwrap();

    assert_eq!(Cargo::from_env().bound_scenarios(dir.path()), Ok(1));
    assert!(
        Cargo::from_env()
            .bound_scenarios(&dir.path().join("missing"))
            .unwrap_err()
            .starts_with("scenario: ")
    );
}

#[test]
fn every_cargo_started_carries_the_measuring_mark() {
    let command = Cargo::from_env().command(&args(&["x"]));

    assert!(
        command
            .get_envs()
            .any(|(k, v)| k == crate::MEASURING && v == Some("1".as_ref()))
    );
}
