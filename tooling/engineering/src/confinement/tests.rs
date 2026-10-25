use super::*;
use testkit::TempDir;

const CONFINED: &str = "//! Starts the core; names no std::process::Command.\n\
    use testkit::boundary::{Component, Confinement};\n\
    fn f() { Confinement::of(Component::Core).run(Path::new(env!(\"CARGO_BIN_EXE_parinn\")), &[], h); }\n";

#[test]
fn a_test_starting_a_component_through_the_confinement_is_confined() {
    assert!(!starts_unconfined(CONFINED));
}

#[test]
fn a_test_starting_no_component_is_none_of_its_business() {
    assert!(!starts_unconfined(
        "use std::process::Command;\nfn f() { Command::new(\"cargo\"); }\n"
    ));
}

#[test]
fn a_test_naming_a_command_beside_a_component_starts_it_unconfined() {
    let body = format!(
        "{CONFINED}fn g() {{ std::process::Command::new(env!(\"CARGO_BIN_EXE_parinn\")); }}\n"
    );

    assert!(starts_unconfined(&body));
}

#[test]
fn a_test_starting_a_component_without_the_confinement_starts_it_unconfined() {
    assert!(starts_unconfined(
        "fn f() { run(env!(\"CARGO_BIN_EXE_parinn\")); }\n"
    ));
}

/// A checkout with the test files `files` under `crates/`.
fn checkout(files: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for (file, body) in files {
        let path = dir.path().join("crates").join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

#[test]
fn every_test_of_a_product_crate_is_read() {
    let (_dir, c) = checkout(&[
        ("parinn/Cargo.toml", ""),
        ("parinn/tests/e2e_parinn.rs", CONFINED),
        (
            "parinn/tests/deep/e2e_more.rs",
            "fn f() { run(env!(\"CARGO_BIN_EXE_parinn\")); }\n",
        ),
        ("parinn/tests/notes.txt", "CARGO_BIN_EXE_parinn"),
        ("parinn/src/lib.rs", "CARGO_BIN_EXE_parinn"),
        ("other/Cargo.toml", ""),
    ]);

    assert_eq!(
        unconfined_starts(&c),
        [
            "crates/parinn/tests/deep/e2e_more.rs starts a component's executable outside the end-to-end suite's confinement (ENG-12)"
        ]
    );
}

#[test]
fn a_checkout_without_product_crates_is_named() {
    let dir = TempDir::new().unwrap();
    let c = Checkout::new(dir.path());

    let problems = unconfined_starts(&c);

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].starts_with("list crates: "), "{problems:?}");
}

#[test]
fn an_unreadable_test_file_is_named() {
    let (_dir, c) = checkout(&[("parinn/Cargo.toml", "")]);
    fs::create_dir_all(c.root.join("crates/parinn/tests")).unwrap();
    fs::write(c.root.join("crates/parinn/tests/bad.rs"), [0xff, 0xfe]).unwrap();

    let problems = unconfined_starts(&c);

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("read crates/parinn/tests/bad.rs: "),
        "{problems:?}"
    );
}
