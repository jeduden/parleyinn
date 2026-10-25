use super::*;

#[test]
fn patterns_reads_the_first_field_of_each_rule() {
    let got = patterns("# owners\n\n/docs/srs/   @a\n/Cargo.toml @a @b\n  *.lock @c\n");

    assert_eq!(got, ["/docs/srs/", "/Cargo.toml", "*.lock"]);
}

#[test]
fn owns_reads_an_anchored_directory_as_its_contents() {
    assert!(owns("/docs/srs/", "docs/srs/index.md"));
    assert!(owns("/docs/srs/", "docs/srs/a/b.md"));
    assert!(!owns("/docs/srs/", "docs/srs"));
    assert!(!owns("/docs/srs/", "docs/srsx/index.md"));
    assert!(!owns("/docs/srs/", "x/docs/srs/index.md"));
}

#[test]
fn owns_reads_an_anchored_file() {
    assert!(owns("/Cargo.toml", "Cargo.toml"));
    assert!(!owns("/Cargo.toml", "crates/parinn/Cargo.toml"));
}

#[test]
fn owns_matches_an_unanchored_pattern_at_any_depth() {
    assert!(owns("clippy.toml", "clippy.toml"));
    assert!(owns("clippy.toml", "crates/parinn/clippy.toml"));
    assert!(!owns("clippy.toml", "crates/parinn/clippy.toml.bak"));
}

#[test]
fn owns_globs_within_one_segment() {
    assert!(owns("/crates/*/clippy.toml", "crates/pin/clippy.toml"));
    assert!(!owns("/crates/*/clippy.toml", "crates/pin/src/clippy.toml"));
    assert!(owns(
        "/crates/*/tests/reach.rs",
        "crates/parinn/tests/reach.rs"
    ));
    assert!(owns("*.lock", "Cargo.lock"));
    assert!(!owns("*.lock", "Cargo.lockx"));
}

#[test]
fn owns_nothing_for_an_empty_pattern() {
    assert!(!owns("/", "README.md"));
}

#[test]
fn glob_matches_any_run_and_nothing_more() {
    assert!(glob("a*c", "abbc"));
    assert!(glob("a*", "a"));
    assert!(glob("*é", "café"));
    assert!(!glob("a*c", "abd"));
    assert!(!glob("b*", "a"));
}

#[test]
fn owns_anchors_a_pattern_with_an_inner_slash() {
    assert!(owns("docs/srs/", "docs/srs/a.md"));
    assert!(!owns("docs/srs/", "x/docs/srs/a.md"));
    assert!(owns("crates/*/clippy.toml", "crates/pin/clippy.toml"));
    assert!(!owns("crates/*/clippy.toml", "x/crates/pin/clippy.toml"));
}

#[test]
fn owns_reads_an_unanchored_directory() {
    assert!(owns("target/", "target/x"));
    assert!(owns("target/", "a/target/x"));
    assert!(!owns("target/", "a/target"));
}

#[test]
fn glob_matches_several_stars() {
    assert!(glob("a*b*c", "axbyc"));
    assert!(!glob("a*b*c", "axc"));
    assert!(glob("**", ""));
}
