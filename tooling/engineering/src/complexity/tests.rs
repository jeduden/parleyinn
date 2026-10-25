use super::*;
use std::fs;
use testkit::TempDir;

/// The workspace manifest that denies every complexity lint.
fn denying() -> String {
    let mut body = String::from("[workspace]\nmembers = [\"crates/*\", \"tools/one\"]\n\n");
    body.push_str("[workspace.lints.clippy]\nunwrap_used = \"deny\"\n");
    for lint in LINTS {
        body.push_str(&format!("{lint} = \"deny\"\n"));
    }
    body
}

/// A clippy.toml holding every limit at its ceiling.
fn holding() -> String {
    LIMITS
        .iter()
        .map(|(key, max)| format!("{key} = {max}\n"))
        .collect()
}

const INHERITS: &str = "[package]\nname = \"x\"\n\n[lints]\nworkspace = true\n";

/// A checkout whose workspace is `root`, with the member crates
/// `crates` by their directory and manifest, and `clippy` as the root's
/// clippy.toml.
fn checkout(root: &str, crates: &[(&str, &str)], clippy: &str) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), root).unwrap();
    fs::write(dir.path().join("clippy.toml"), clippy).unwrap();
    fs::create_dir_all(dir.path().join("crates")).unwrap();
    for (path, manifest) in crates {
        fs::create_dir_all(dir.path().join(path)).unwrap();
        fs::write(dir.path().join(path).join("Cargo.toml"), manifest).unwrap();
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

#[test]
fn the_workspace_lints_deny_every_complexity_lint() {
    assert!(workspace_denies(&denying()).is_empty());

    let forbid = denying().replace(
        "too_many_lines = \"deny\"",
        "too_many_lines = { level = \"forbid\", priority = 1 }",
    );
    assert!(workspace_denies(&forbid).is_empty());
}

#[test]
fn a_complexity_lint_left_out_or_only_warned_fails() {
    let root = denying()
        .replace("too_many_lines = \"deny\"\n", "")
        .replace(
            "excessive_nesting = \"deny\"",
            "excessive_nesting = \"warn\" # deny",
        );

    assert_eq!(
        workspace_denies(&root),
        [
            "the workspace lints do not deny clippy::too_many_lines",
            "the workspace lints do not deny clippy::excessive_nesting",
        ]
    );
}

#[test]
fn threshold_reads_a_top_level_integer_key() {
    let body = "# too-many-lines-threshold = 1\ntoo-many-lines-threshold = 80 # code lines\nx = [\n  { path = \"a\" },\n]\nmax-fn-params-bools=3\nexcessive-nesting-threshold = \"5\"\n";

    assert_eq!(threshold(body, "too-many-lines-threshold"), Some(80));
    assert_eq!(threshold(body, "max-fn-params-bools"), Some(3));
    assert_eq!(threshold(body, "excessive-nesting-threshold"), None);
    assert_eq!(threshold(body, "cognitive-complexity-threshold"), None);
    assert_eq!(
        threshold(
            "too-many-lines-threshold-x = 1\n",
            "too-many-lines-threshold"
        ),
        None
    );
}

#[test]
fn a_clippy_toml_holding_every_limit_passes() {
    assert!(holds_limits("clippy.toml", &holding()).is_empty());

    let lower = holding().replace("= 80", "= 60");
    assert!(holds_limits("clippy.toml", &lower).is_empty());
}

#[test]
fn a_limit_missing_or_raised_is_named_with_its_file() {
    let body = holding()
        .replace(
            "too-many-lines-threshold = 80",
            "too-many-lines-threshold = 200",
        )
        .replace("excessive-nesting-threshold = 5\n", "");

    assert_eq!(
        holds_limits("crates/a/clippy.toml", &body),
        [
            "crates/a/clippy.toml sets too-many-lines-threshold to 200, above 80",
            "crates/a/clippy.toml sets no excessive-nesting-threshold",
        ]
    );
}

#[test]
fn every_crate_holds_the_limits_through_the_file_clippy_reads() {
    let crates = [
        ("crates/a", INHERITS),
        ("crates/b", INHERITS),
        ("tools/one", INHERITS),
    ];
    let (d, c) = checkout(&denying(), &crates, &holding());
    assert!(
        crates_hold_limits(&c).is_empty(),
        "{:?}",
        crates_hold_limits(&c)
    );

    fs::write(d.path().join("crates/b/clippy.toml"), "").unwrap();
    let got = crates_hold_limits(&c);
    assert_eq!(got.len(), LIMITS.len());
    assert_eq!(
        got[0],
        "crates/b/clippy.toml sets no cognitive-complexity-threshold"
    );
}

#[test]
fn a_crate_with_its_own_lints_escapes_the_limits_and_fails() {
    let own = "[package]\nname = \"a\"\n";
    let (_d, c) = checkout(
        &denying(),
        &[("crates/a", own), ("tools/one", INHERITS)],
        &holding(),
    );

    assert_eq!(
        crates_hold_limits(&c),
        ["crate crates/a does not inherit the workspace lints, so the complexity lints miss it"]
    );
}

#[test]
fn the_unsafe_shim_holds_the_limits_through_its_own_lints() {
    let shim = "[package]\nname = \"platform\"\n\n[lints.clippy]\ncognitive_complexity = \"deny\"\ntoo_many_lines = \"deny\"\nexcessive_nesting = \"deny\"\nfn_params_excessive_bools = \"deny\"\n";
    let shim_crate = crate::static_analysis::UNSAFE_SHIM;
    let (_d, c) = checkout(
        &denying(),
        &[(shim_crate, shim), ("tools/one", INHERITS)],
        &holding(),
    );
    assert!(
        crates_hold_limits(&c).is_empty(),
        "{:?}",
        crates_hold_limits(&c)
    );

    let loose = shim.replace("too_many_lines = \"deny\"\n", "");
    let (_d, c) = checkout(
        &denying(),
        &[(shim_crate, &loose), ("tools/one", INHERITS)],
        &holding(),
    );
    assert_eq!(
        crates_hold_limits(&c),
        ["crate crates/platform, the one unsafe shim, does not deny clippy::too_many_lines"]
    );
}

#[test]
fn the_workspace_failing_to_deny_a_lint_fails_the_crates() {
    let root = denying().replace("cognitive_complexity = \"deny\"\n", "");
    let (_d, c) = checkout(&root, &[("tools/one", INHERITS)], &holding());

    assert_eq!(
        crates_hold_limits(&c),
        ["the workspace lints do not deny clippy::cognitive_complexity"]
    );
}

#[test]
fn what_it_cannot_read_is_named() {
    let dir = TempDir::new().unwrap();
    let c = Checkout::new(dir.path());
    assert!(crates_hold_limits(&c)[0].starts_with("read Cargo.toml: "));

    let (d, c) = checkout(&denying(), &[("tools/one", INHERITS)], &holding());
    fs::remove_file(d.path().join("tools/one/Cargo.toml")).unwrap();
    assert!(crates_hold_limits(&c)[0].starts_with("read tools/one/Cargo.toml: "));

    let (d, c) = checkout(&denying(), &[("tools/one", INHERITS)], &holding());
    fs::remove_file(d.path().join("clippy.toml")).unwrap();
    assert!(crates_hold_limits(&c)[0].starts_with("read clippy.toml: "));

    let root = denying().replace("\"crates/*\", \"tools/one\"", "\"gone/*\"");
    let (_d, c) = checkout(&root, &[], &holding());
    assert!(crates_hold_limits(&c)[0].starts_with("list gone: "));
}

#[test]
fn a_nesting_ceiling_of_zero_switches_the_lint_off_and_fails() {
    let body = holding().replace(
        "excessive-nesting-threshold = 5",
        "excessive-nesting-threshold = 0",
    );

    assert_eq!(
        holds_limits("clippy.toml", &body),
        ["clippy.toml sets excessive-nesting-threshold to 0, which switches the lint off"]
    );

    let zero_lines = holding().replace("max-fn-params-bools = 3", "max-fn-params-bools = 0");
    assert!(holds_limits("clippy.toml", &zero_lines).is_empty());
}

#[test]
fn threshold_reads_only_the_keys_before_the_first_table() {
    let body = "too-many-lines-threshold = 80\n[later]\nmax-fn-params-bools = 3\n";

    assert_eq!(threshold(body, "too-many-lines-threshold"), Some(80));
    assert_eq!(threshold(body, "max-fn-params-bools"), None);
}
