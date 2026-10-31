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

/// The repository's own CODEOWNERS patterns the consent tests read.
fn repository_patterns() -> Vec<String> {
    patterns(
        "/docs/srs/ @a\n/docs/agents/ @a\n/.github/ @a\n/.claude/ @a\n/scripts/ @a\n\
         /Cargo.toml @a\n/tooling/*/Cargo.toml @a\n/CLAUDE.md @a\n/AGENTS.md @a\n",
    )
}

#[test]
fn tokens_split_on_whitespace_and_punctuation_and_drop_trailing_dots() {
    let got = tokens("`a/b`, c;d:e (f) [g] \"h\" 'i' j/. k... .\n\tl");

    assert_eq!(
        got,
        ["a/b", "c", "d", "e", "f", "g", "h", "i", "j/", "k", "l"]
            .into_iter()
            .collect()
    );
}

#[test]
fn a_comment_names_the_path_itself() {
    let p = repository_patterns();

    assert!(names(&p, "Consent for CLAUDE.md.", "CLAUDE.md"));
    assert!(!names(&p, "Consent for claude.md", "CLAUDE.md"));
    assert!(!names(&p, "Consent for CLAUDE.mdx", "CLAUDE.md"));
}

#[test]
fn a_comment_names_the_pattern_that_owns_the_path() {
    let p = repository_patterns();
    let path = "tooling/review-run/Cargo.toml";

    assert!(names(&p, "ok: `tooling/*/Cargo.toml`", path));
    assert!(!names(&p, "ok: /tooling/*/Cargo.toml", path));
    assert!(!names(
        &p,
        "ok: tooling/*/Cargo.toml",
        "crates/x/Cargo.toml"
    ));
}

#[test]
fn a_comment_names_an_ancestor_directory() {
    let p = repository_patterns();
    let workflow = ".github/workflows/review.yml";

    assert!(names(&p, "ok .github/", workflow));
    assert!(names(&p, "ok .github/workflows/", workflow));
    assert!(names(&p, "ok .github/workflows", workflow));
    assert!(names(&p, "ok docs/agents", "docs/agents/a/b.md"));
    assert!(names(&p, "ok docs/agents/a", "docs/agents/a/b.md"));
    assert!(!names(&p, "ok docs", "docs/agents/a/b.md"));
    assert!(!names(&p, "ok scripts", "scripts/review-local.sh"));
    assert!(!names(&p, "ok docs/agent", "docs/agents/a/b.md"));
    assert!(!names(&p, "ok / and ./", "Cargo.toml"));
}

#[test]
fn the_stakeholder_s_comment_consents_only_to_the_paths_it_names() {
    let p = repository_patterns();
    let comment = "Consent for the SRS, scenarios, ADRs, .github/, docs/agents, .claude/ and \
                   evaluation-case changes, and a ruling on ADR-2610300031.";

    for path in [
        ".github/workflows/review.yml",
        "docs/agents/generalist-report.md",
        ".claude/agents/x.md",
    ] {
        assert!(names(&p, comment, path), "{path}");
    }
    for path in [
        "CLAUDE.md",
        "AGENTS.md",
        "Cargo.toml",
        "tooling/review-run/Cargo.toml",
        "scripts/review-local.sh",
        "docs/srs/x.md",
    ] {
        assert!(!names(&p, comment, path), "{path}");
    }
}
