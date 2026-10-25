use super::*;
use crate::review_workflow::parse;
use testkit::TempDir;

/// A CI workflow whose jobs run `runs`, one job each, and whose `ci`
/// job needs `needs` and fails unless they all passed.
fn workflow(runs: &[(&str, &str)], needs: &str) -> Workflow {
    let mut body = String::from("on:\n  push:\njobs:\n");
    for (job, run) in runs {
        body.push_str(&format!(
            "  {job}:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {run}\n"
        ));
    }
    body.push_str(&format!(
        "  ci:\n    if: ${{{{ always() }}}}\n    needs: [{needs}]\n    steps:\n      - env:\n          RESULTS: ${{{{ join(needs.*.result, ' ') }}}}\n        run: |\n          for r in $RESULTS; do\n            [ \"$r\" = \"success\" ] || exit 1\n          done\n"
    ));
    parse(&body)
}

/// The workflow that gates on every static check.
fn gating() -> Workflow {
    workflow(
        &[
            ("test", "cargo test --workspace --locked"),
            ("lint", "cargo fmt --all --check"),
            (
                "clippy",
                "cargo clippy --workspace --all-targets --locked -- -D warnings",
            ),
            ("deny", "cargo deny check licenses bans sources"),
            ("audit", "cargo audit"),
        ],
        "test, lint, clippy, deny, audit",
    )
}

#[test]
fn needs_reads_an_inline_list_and_a_single_job() {
    let flow =
        parse("jobs:\n  ci:\n    needs: [a, b-c]\n  one:\n    needs: a\n  none:\n    runs-on: x\n");

    assert_eq!(needs(&flow.jobs[0]), ["a", "b-c"]);
    assert_eq!(needs(&flow.jobs[1]), ["a"]);
    assert!(needs(&flow.jobs[2]).is_empty());
}

#[test]
fn runs_matches_a_whole_command_on_a_run_line() {
    let flow = parse(
        "jobs:\n  a:\n    steps:\n      - run: cargo audit\n      - run: >-\n          cargo test --workspace --locked --quiet\n  b:\n    steps:\n      - run: cargo install cargo-audit@1 && cargo auditx\n",
    );

    assert!(runs(&flow.jobs[0], "cargo audit"));
    assert!(runs(&flow.jobs[0], "cargo test --workspace --locked"));
    assert!(!runs(&flow.jobs[1], "cargo audit"));
}

#[test]
fn without_assignments_drops_the_assignments_before_a_command() {
    assert_eq!(without_assignments("cargo audit"), "cargo audit");
    assert_eq!(without_assignments("A=1 B_2=x cargo audit"), "cargo audit");
    assert_eq!(without_assignments("HOME=\"a b\" cargo"), "cargo");
    assert_eq!(without_assignments("H='x' cargo"), "cargo");
    assert_eq!(without_assignments("A=1"), "");
    assert_eq!(
        without_assignments("HOME=\"open cargo"),
        "HOME=\"open cargo"
    );
    assert_eq!(without_assignments("1A=x cargo"), "1A=x cargo");
    assert_eq!(without_assignments("echo A=x cargo"), "echo A=x cargo");
}

#[test]
fn runs_matches_a_command_behind_environment_assignments() {
    let flow = parse(
        "jobs:\n  a:\n    steps:\n      - run: |\n          HOME=\"$decoy\" A=1 cargo test --workspace --locked\n  b:\n    steps:\n      - run: echo HOME=x cargo audit\n  c:\n    steps:\n      - run: HOME=\"a b\" cargo audit\n",
    );

    assert!(runs(&flow.jobs[0], "cargo test --workspace --locked"));
    assert!(!runs(&flow.jobs[1], "cargo audit"));
    assert!(runs(&flow.jobs[2], "cargo audit"));
}

#[test]
fn a_workflow_running_every_static_check_under_the_aggregator_gates() {
    assert!(ci_gates(&gating()).is_empty(), "{:?}", ci_gates(&gating()));
}

#[test]
fn a_static_check_no_job_runs_fails() {
    let flow = workflow(&[("test", "cargo test --workspace --locked")], "test");

    let problems = ci_gates(&flow);

    assert!(problems.contains(&"no CI job runs cargo-audit: cargo audit".to_owned()));
    assert_eq!(problems.len(), 4, "{problems:?}");
}

#[test]
fn a_static_check_outside_the_aggregator_fails() {
    let mut flow = gating();
    flow.jobs.last_mut().unwrap().text = flow.jobs.last().unwrap().text.replace(", audit]", "]");

    assert_eq!(
        ci_gates(&flow),
        ["job audit runs cargo-audit, but the ci job does not need it"]
    );
}

#[test]
fn a_static_check_allowed_to_fail_fails() {
    let mut flow = gating();
    flow.jobs[1].text.push_str("    continue-on-error: true\n");
    flow.jobs[2].text.push_str("    continue-on-error: false\n");

    assert_eq!(
        ci_gates(&flow),
        ["job lint runs cargo fmt, but may fail without failing CI: continue-on-error"]
    );
}

#[test]
fn an_aggregator_that_passes_a_failed_job_fails() {
    let mut flow = gating();
    let ci = flow.jobs.last_mut().unwrap();
    ci.text = ci.text.replace("if: ${{ always() }}\n", "");

    assert_eq!(
        ci_gates(&flow),
        ["the ci job does not run after a failed job and fail unless every job it needs succeeded"]
    );
}

#[test]
fn a_workflow_without_an_aggregator_fails() {
    let flow = parse("jobs:\n  audit:\n    steps:\n      - run: cargo audit\n");

    assert!(
        ci_gates(&flow).contains(&"the CI workflow has no ci job".to_owned()),
        "{:?}",
        ci_gates(&flow)
    );
}

/// One crate: its directory, its manifest and its `src` files.
type Crate<'a> = (&'a str, &'a str, &'a [(&'a str, &'a str)]);

/// A checkout with a root manifest `root` and the crates `crates`.
fn checkout(root: &str, crates: &[Crate<'_>]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), root).unwrap();
    for (path, manifest, src) in crates {
        let at = dir.path().join(path);
        fs::create_dir_all(at.join("src")).unwrap();
        fs::write(at.join("Cargo.toml"), manifest).unwrap();
        for (file, body) in *src {
            fs::write(at.join("src").join(file), body).unwrap();
        }
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

const ROOT: &str = "[workspace]\nmembers = [\"crates/*\", \"tools/one\"]\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n";

const INHERITS: &str = "[package]\nname = \"x\"\n\n[lints]\nworkspace = true\n";

#[test]
fn the_workspace_lints_forbid_unsafe_code() {
    assert_eq!(workspace_forbids_unsafe(ROOT), Ok(()));
    assert_eq!(
        workspace_forbids_unsafe(
            "[workspace.lints.rust]\nunsafe_code = { level = \"forbid\", priority = 1 }\n"
        ),
        Ok(())
    );
    assert_eq!(
        workspace_forbids_unsafe("[workspace.lints.rust]\nunsafe_code = \"deny\"\n"),
        Err("the workspace lints do not forbid unsafe_code".to_owned())
    );
    assert_eq!(
        workspace_forbids_unsafe("[workspace]\n"),
        Err("the workspace lints do not forbid unsafe_code".to_owned())
    );
}

#[test]
fn a_crate_inherits_the_workspace_lints_only_from_its_lints_table() {
    assert!(inherits_workspace_lints(INHERITS));
    assert!(inherits_workspace_lints("[lints]\nworkspace=true # yes\n"));
    assert!(!inherits_workspace_lints(
        "[lints.rust]\nworkspace = true\n"
    ));
    assert!(!inherits_workspace_lints(
        "[package]\nworkspace = true\n[lints]\n"
    ));
    assert!(!inherits_workspace_lints("[lints]\nworkspace = false\n"));
}

#[test]
fn members_expands_each_glob_to_the_crates_under_it() {
    let (_dir, c) = checkout(
        ROOT,
        &[
            ("crates/b", INHERITS, &[]),
            ("crates/a", INHERITS, &[]),
            ("tools/one", INHERITS, &[]),
        ],
    );
    fs::create_dir_all(c.root.join("crates/notes")).unwrap();

    assert_eq!(
        members(&c, ROOT),
        Ok(vec![
            "crates/a".to_owned(),
            "crates/b".to_owned(),
            "tools/one".to_owned()
        ])
    );
    assert!(members(&c, "[workspace]\nmembers = [\"gone/*\"]\n").is_err());
}

#[test]
fn every_crate_inheriting_the_lints_forbids_unsafe_code() {
    let (_dir, c) = checkout(
        ROOT,
        &[("crates/a", INHERITS, &[]), ("tools/one", INHERITS, &[])],
    );

    assert!(
        crates_forbid_unsafe(&c).is_empty(),
        "{:?}",
        crates_forbid_unsafe(&c)
    );
}

#[test]
fn a_crate_with_its_own_lints_forbids_unsafe_in_each_crate_root() {
    let own = "[package]\nname = \"a\"\n";
    let forbid = "//! a\n#![forbid(unsafe_code)]\n";
    let (_dir, c) = checkout(
        ROOT,
        &[
            ("crates/a", own, &[("lib.rs", forbid), ("main.rs", forbid)]),
            (
                "crates/b",
                own,
                &[("lib.rs", forbid), ("main.rs", "fn main() {}\n")],
            ),
            ("crates/c", own, &[]),
            ("tools/one", INHERITS, &[]),
        ],
    );

    assert_eq!(
        crates_forbid_unsafe(&c),
        [
            "crate crates/b neither inherits the workspace lints nor forbids unsafe code in src/main.rs",
            "crate crates/c neither inherits the workspace lints nor forbids unsafe code: it has no src/lib.rs or src/main.rs",
        ]
    );
}

#[test]
fn the_one_unsafe_shim_denies_unsafe_code_in_its_own_lints() {
    let shim = "[package]\nname = \"platform\"\n\n[lints.rust]\nunsafe_code = \"deny\"\n";
    let (_dir, c) = checkout(
        ROOT,
        &[(UNSAFE_SHIM, shim, &[]), ("tools/one", INHERITS, &[])],
    );
    assert!(
        crates_forbid_unsafe(&c).is_empty(),
        "{:?}",
        crates_forbid_unsafe(&c)
    );

    let allowing = shim.replace("\"deny\"", "\"allow\"");
    let (_dir, c) = checkout(
        ROOT,
        &[(UNSAFE_SHIM, &allowing, &[]), ("tools/one", INHERITS, &[])],
    );
    assert_eq!(
        crates_forbid_unsafe(&c),
        ["crate crates/platform, the one unsafe shim, does not deny unsafe code in its own lints"]
    );
}

#[test]
fn a_workspace_that_stops_forbidding_unsafe_code_fails() {
    let root = ROOT.replace("\"forbid\"", "\"warn\"");
    let (_dir, c) = checkout(
        &root,
        &[("crates/a", INHERITS, &[]), ("tools/one", INHERITS, &[])],
    );

    assert_eq!(
        crates_forbid_unsafe(&c),
        ["the workspace lints do not forbid unsafe_code"]
    );
}

#[test]
fn an_unreadable_workspace_is_named() {
    let dir = TempDir::new().unwrap();
    let c = Checkout::new(dir.path());

    assert_eq!(crates_forbid_unsafe(&c).len(), 1);
    assert!(crates_forbid_unsafe(&c)[0].starts_with("read Cargo.toml: "));
}

#[test]
fn an_unreadable_member_is_named() {
    let (_dir, c) = checkout(ROOT, &[("crates/a", INHERITS, &[])]);

    let problems = crates_forbid_unsafe(&c);

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("read tools/one/Cargo.toml: "),
        "{problems:?}"
    );
}

/// A checkout holding the core's crate with the clippy.toml `clippy`
/// and, when `reach` is set, the reach test with that body.
fn core(clippy: &str, reach: Option<&str>) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    let at = dir.path().join("crates/parinn");
    fs::create_dir_all(at.join("tests")).unwrap();
    fs::write(at.join("Cargo.toml"), INHERITS).unwrap();
    fs::write(at.join("clippy.toml"), clippy).unwrap();
    if let Some(body) = reach {
        fs::write(at.join("tests/reach.rs"), body).unwrap();
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

/// A clippy.toml carrying every ban.
fn every_ban() -> String {
    let ty = REACH_BANS.iter().chain(&DETERMINISM_TYPES);
    let types: Vec<String> = ty.map(|b| format!("{{ path = \"{b}\" }}")).collect();
    let methods: Vec<String> = DETERMINISM_METHODS
        .iter()
        .map(|b| format!("{{ path = \"{b}\" }}"))
        .collect();
    format!(
        "disallowed-types = [{}]\ndisallowed-methods = [{}]\n",
        types.join(", "),
        methods.join(", ")
    )
}

const REACH: &str = "#[test]\nfn the_core_executable_links_only_crates_its_reach_allows() {}\n";

#[test]
fn the_core_s_reach_evidence_is_its_bans_its_reach_test_and_their_ci_jobs() {
    let (_dir, c) = core(&every_ban(), Some(REACH));

    assert!(
        core_reach_evidence(&c, &gating()).is_empty(),
        "{:?}",
        core_reach_evidence(&c, &gating())
    );
}

#[test]
fn the_core_s_reach_evidence_missing_its_reach_test_fails() {
    let (_dir, c) = core(&every_ban(), None);

    let problems = core_reach_evidence(&c, &gating());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with(
            "the core's reach evidence is missing: read crates/parinn/tests/reach.rs: "
        ),
        "{problems:?}"
    );
}

#[test]
fn the_core_s_reach_test_must_hold_its_test() {
    let (_dir, c) = core(&every_ban(), Some("fn other() {}\n"));

    assert_eq!(
        core_reach_evidence(&c, &gating()),
        [
            "the core's reach evidence is missing: crates/parinn/tests/reach.rs has no test the_core_executable_links_only_crates_its_reach_allows"
        ]
    );
}

#[test]
fn the_core_s_reach_evidence_needs_every_reach_ban() {
    let clippy = every_ban().replace("{ path = \"std::process::Command\" }", "");
    let (_dir, c) = core(&clippy, Some(REACH));

    assert_eq!(
        core_reach_evidence(&c, &gating()),
        [
            "the core's reach evidence is missing: crates/parinn/clippy.toml does not ban std::process::Command"
        ]
    );
}

#[test]
fn the_core_s_reach_evidence_needs_ci_to_run_it() {
    let (_dir, c) = core(&every_ban(), Some(REACH));
    let flow = workflow(&[("audit", "cargo audit")], "audit");

    assert_eq!(
        core_reach_evidence(&c, &flow),
        [
            "no CI job runs cargo clippy: cargo clippy --workspace --all-targets --locked -- -D warnings",
            "no CI job runs the custom checks: cargo test --workspace --locked",
        ]
    );
}

#[test]
fn derived_code_is_refused_a_clock_and_a_random_order_at_build_time() {
    let (_dir, c) = core(&every_ban(), Some(REACH));

    assert!(derived_code_gated(&c, &gating()).is_empty());
}

#[test]
fn derived_code_may_not_lose_a_determinism_ban() {
    let clippy = every_ban().replace("{ path = \"std::time::Instant::now\" }", "");
    let (_dir, c) = core(&clippy, Some(REACH));
    let flow = workflow(&[("audit", "cargo audit")], "audit");

    assert_eq!(
        derived_code_gated(&c, &flow),
        [
            "code that computes derived artifacts may read a clock or randomness: crate parinn: crates/parinn/clippy.toml does not ban std::time::Instant::now in disallowed-methods",
            "no CI job runs cargo clippy: cargo clippy --workspace --all-targets --locked -- -D warnings",
        ]
    );
}

#[test]
fn derived_code_names_a_checkout_without_crates() {
    let dir = TempDir::new().unwrap();
    let c = Checkout::new(dir.path());

    let problems = derived_code_gated(&c, &gating());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].starts_with("list crates: "), "{problems:?}");
}

#[test]
fn a_core_without_a_clippy_configuration_names_the_one_it_reads() {
    let (_dir, c) = core(&every_ban(), Some(REACH));
    fs::remove_file(c.root.join("crates/parinn/clippy.toml")).unwrap();

    let reach = core_reach_evidence(&c, &gating());
    let derived = derived_code_gated(&c, &gating());

    assert_eq!(reach.len(), 1, "{reach:?}");
    assert!(
        reach[0].starts_with("the core's reach evidence is missing: read clippy.toml: "),
        "{reach:?}"
    );
    assert_eq!(derived.len(), 1, "{derived:?}");
    assert!(derived[0].starts_with("read clippy.toml: "), "{derived:?}");
}

#[test]
fn a_member_glob_that_cannot_be_listed_is_named() {
    let (_dir, c) = checkout(
        "[workspace]\nmembers = [\"gone/*\"]\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n",
        &[],
    );

    assert_eq!(crates_forbid_unsafe(&c).len(), 1);
    assert!(crates_forbid_unsafe(&c)[0].starts_with("list gone: "));
}

#[test]
fn ci_gating_on_both_toml_checks_passes() {
    let flow = workflow(
        &[("toml", "taplo fmt --check"), ("schemas", "taplo lint")],
        "toml, schemas",
    );

    assert!(toml_gates(&flow).is_empty(), "{:?}", toml_gates(&flow));
}

#[test]
fn a_toml_check_not_run_or_not_needed_is_named() {
    let flow = workflow(
        &[("toml", "taplo fmt --check"), ("schemas", "taplo lint")],
        "toml",
    );
    assert_eq!(
        toml_gates(&flow),
        ["job schemas runs taplo lint, but the ci job does not need it"]
    );

    let flow = workflow(&[("toml", "taplo lint")], "toml");
    assert_eq!(
        toml_gates(&flow),
        ["no CI job runs taplo fmt: taplo fmt --check"]
    );
}
