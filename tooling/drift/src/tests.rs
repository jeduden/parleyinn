use super::*;
use testkit::TempDir;

fn tree() -> TempDir {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("a.md"), "one two one\n").unwrap();
    root
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn apply_replaces_the_first_occurrence() {
    let root = tree();

    apply(root.path(), &Injection::replace("a.md", "one", "1")).unwrap();

    assert_eq!(read(&root.path().join("a.md")), "1 two one\n");
}

#[test]
fn apply_removes_and_copies() {
    let root = tree();

    apply(root.path(), &Injection::copy("a.md", "b.md")).unwrap();
    assert_eq!(read(&root.path().join("b.md")), "one two one\n");

    apply(root.path(), &Injection::remove("a.md")).unwrap();
    assert!(!root.path().join("a.md").exists());
}

#[test]
fn apply_creates_a_file_and_the_directories_on_its_path() {
    let root = tree();

    apply(root.path(), &Injection::create("vendor/x/LICENSE", "text")).unwrap();

    assert_eq!(read(&root.path().join("vendor/x/LICENSE")), "text");
    let err = apply(root.path(), &Injection::create("a.md", "x")).unwrap_err();
    assert_eq!(err.to_string(), "drift: a.md: already exists");
    assert!(std::error::Error::source(&err).is_none());
    let err = apply(root.path(), &Injection::create("a.md/inner/f", "x")).unwrap_err();
    assert!(
        err.to_string().starts_with("drift: a.md/inner/f: "),
        "{err}"
    );
}

#[test]
fn validate_refuses_an_injection_that_would_inject_nothing() {
    let root = tree();
    let cases = [
        (
            Injection::replace("missing.md", "x", "y"),
            "drift: missing.md: ",
        ),
        (
            Injection::replace("a.md", "three", "3"),
            r#"drift: a.md: "three" not found"#,
        ),
        (
            Injection::replace("a.md", "", "3"),
            r#"drift: a.md: "" not found"#,
        ),
        (
            Injection::copy("a.md", "a.md"),
            "drift: a.md: copy target a.md already exists",
        ),
    ];
    for (injection, want) in cases {
        let err = validate(root.path(), &injection).unwrap_err().to_string();
        assert!(err.starts_with(want), "{err}");
        let err = apply(root.path(), &injection).unwrap_err().to_string();
        assert!(err.starts_with(want), "{err}");
    }
    assert!(validate(root.path(), &Injection::remove("a.md")).is_ok());
}

#[test]
fn errors_keep_their_cause() {
    let root = tree();

    let err = validate(root.path(), &Injection::remove("missing.md")).unwrap_err();
    assert!(std::error::Error::source(&err).is_some());

    let err = validate(root.path(), &Injection::replace("a.md", "x", "y")).unwrap_err();
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn apply_reports_a_write_that_fails() {
    let root = tree();
    fs::create_dir(root.path().join("dir")).unwrap();
    fs::write(root.path().join("dir/a.md"), "x").unwrap();

    let err = apply(root.path(), &Injection::copy("a.md", "missing/b.md")).unwrap_err();

    assert!(err.to_string().starts_with("drift: a.md: "), "{err}");
}

#[test]
fn check_constructors_name_the_command() {
    assert_eq!(
        scenario("ENG-18").to_string(),
        "cargo test --locked -p scenario --test bdd -- --tags @ENG-18"
    );
    assert_eq!(
        cargo_test("srs", "gates", "x").to_string(),
        "cargo test --locked -p srs --test gates -- --exact x"
    );
    assert_eq!(
        cargo_lib_test("testkit", "relative::tests::x").to_string(),
        "cargo test --locked -p testkit --lib -- --exact relative::tests::x"
    );
    assert_eq!(mdsmith().to_string(), "mdsmith check .");
    assert_eq!(taplo_fmt().to_string(), "taplo fmt --check");
    assert_eq!(taplo_lint().to_string(), "taplo lint");
    assert_eq!(
        setup_tools().to_string(),
        "sh scripts/setup-tools.sh --check"
    );
    assert_eq!(
        clippy("parinn").to_string(),
        "cargo clippy --locked -p parinn -- -D warnings"
    );
}
