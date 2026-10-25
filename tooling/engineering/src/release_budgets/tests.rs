use super::*;

use testkit::TempDir;

#[test]
fn a_release_benchmark_build_and_run_pass() {
    let text = "\
      - run: cargo build --release --locked -p benchmark -p parinn
      - run: target/release/benchmark run --core target/release/parinn
      - run: cargo run --release -p benchmark -- gate --base a --head b
";

    assert!(debug_benchmark_lines("ci.yml", text).is_empty());
}

#[test]
fn a_benchmark_build_without_release_is_named_with_its_line() {
    let text = "name: x\n      - run: cargo build --locked -p benchmark -p parinn\n";

    assert_eq!(
        debug_benchmark_lines("ci.yml", text),
        ["ci.yml:2 builds the benchmark suite without --release: a benchmark runs a release build"]
    );
}

#[test]
fn every_cargo_verb_that_builds_the_suite_needs_release() {
    for verb in ["run", "test", "bench", "build"] {
        let text = format!("cargo {verb} -p benchmark -- run\n");
        let found = debug_benchmark_lines("s.sh", &text);

        assert_eq!(found.len(), 1, "{verb}: {found:?}");
    }
    let spelled = "cargo build --package benchmark\n";

    assert_eq!(debug_benchmark_lines("s.sh", spelled).len(), 1);
}

#[test]
fn a_benchmark_pointed_at_a_debug_core_is_named() {
    let text = "benchmark run --core target/debug/parinn --samples 3\n";

    assert_eq!(
        debug_benchmark_lines("nightly.yml", text),
        ["nightly.yml:1 points the benchmark at a debug build: a benchmark runs a release build"]
    );
}

#[test]
fn a_comment_and_other_packages_are_left_alone() {
    let text = "\
# cargo build -p benchmark is the debug build
cargo build -p parinn
cargo test --workspace --locked
echo target/debug/parinn
";

    assert!(debug_benchmark_lines("ci.yml", text).is_empty());
}

#[test]
fn a_timed_function_starting_the_development_core_is_named() {
    let source = "\
fn untimed(w: &mut World) {
    run_core(w, &[\"status\"], None);
}

fn timed(w: &mut World) {
    let started = Instant::now();
    run_core_with(w, &[\"hook\"], None, &[]);
    let took = started.elapsed();
}
";

    assert_eq!(
        debug_core_in_timed_fns("bdd/x.rs", source),
        [
            "bdd/x.rs:5 fn timed reads a clock and starts the development core with run_core_with(: a step that times the core runs the release build"
        ]
    );
}

#[test]
fn a_timed_function_starting_the_release_core_passes() {
    let source = "\
pub fn timed(w: &mut World) {
    let started = Instant::now();
    run_release_core(w, &[\"hook\"], None);
    let exe = testkit::release_core_executable();
    let took = started.elapsed();
}
";

    assert!(debug_core_in_timed_fns("bdd/x.rs", source).is_empty());
}

#[test]
fn every_way_to_start_the_development_core_is_named_in_a_timed_function() {
    for call in DEBUG_CORE {
        let source = format!("fn t() {{\n    let a = Instant::now();\n    {call}x);\n}}\n");

        assert_eq!(debug_core_in_timed_fns("t.rs", &source).len(), 1, "{call}");
    }
}

#[test]
fn a_deadline_wait_that_starts_no_core_passes() {
    let source = "fn wait() {\n    let d = Instant::now();\n    sleep();\n    d.elapsed();\n}\n";

    assert!(debug_core_in_timed_fns("t.rs", source).is_empty());
}

fn tree(files: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for (path, text) in files {
        let at = dir.path().join(path);
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(at, text).unwrap();
    }
    let checkout = Checkout::new(dir.path());

    (dir, checkout)
}

#[test]
fn the_checkout_scan_reads_workflows_scripts_tests_and_benches() {
    let debug = "fn t() {\n    let a = Instant::now();\n    run_core(w, &[], None);\n}\n";
    let (_dir, checkout) = tree(&[
        (".github/workflows/a.yml", "cargo build -p benchmark\n"),
        ("scripts/b.sh", "cargo run -p benchmark\n"),
        ("tooling/x/tests/bdd/c.rs", debug),
        ("crates/y/benches/d.rs", debug),
        ("crates/y/tests/e.rs", "fn ok() {}\n"),
        ("crates/y/src/tests.rs", debug),
        ("tooling/x/tests/notes.txt", debug),
    ]);

    let found = problems(&checkout);

    assert_eq!(found.len(), 4, "{found:?}");
    for file in [
        ".github/workflows/a.yml:1",
        "scripts/b.sh:1",
        "tooling/x/tests/bdd/c.rs:1",
        "crates/y/benches/d.rs:1",
    ] {
        assert!(
            found.iter().any(|p| p.starts_with(file)),
            "{file}: {found:?}"
        );
    }
}

#[test]
fn the_checkout_scan_of_a_clean_or_empty_tree_finds_nothing() {
    let (_dir, checkout) = tree(&[]);

    assert!(problems(&checkout).is_empty());
}
