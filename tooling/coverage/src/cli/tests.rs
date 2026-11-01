use super::*;
use crate::tests::FakeHost;
use std::cell::RefCell;

const METADATA: &str = r#"{
  "workspace_root": "/w",
  "packages": [{"name": "srs", "manifest_path": "/w/srs/Cargo.toml", "targets": []}],
  "metadata": {"coverage": {"floors": {"all": 100}, "pyramid": true}}
}"#;

fn host(report: &str) -> FakeHost {
    FakeHost {
        metadata: METADATA.into(),
        reports: RefCell::new([report.to_owned()].into()),
        sources: vec![
            ("/w/srs/src/tests.rs".into(), "#[test]\n".repeat(3)),
            ("/w/srs/tests/gates.rs".into(), "#[test]\n".repeat(2)),
        ],
        scenarios: 1,
        ..FakeHost::default()
    }
}

fn invoke(args: &[&str], host: &FakeHost, summary: Option<&Path>) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        args.iter().map(|a| (*a).to_owned()),
        &mut out,
        &mut err,
        host,
        summary,
        false,
    );
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn run_prints_the_table_and_writes_each_layer() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\n");

    let (code, out, err) = invoke(&["--out", "cov"], &host, Some(Path::new("summary.md")));

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("| srs | 1 | 100.0% |"), "{out}");
    let files = host.files.borrow();
    assert!(files.contains_key(Path::new("cov/unit.lcov")));
    assert!(
        files[Path::new("summary.md")].starts_with("## Line coverage per test layer\n\n| Crate |")
    );
}

#[test]
fn run_fails_on_a_crate_below_its_floor() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\nDA:2,0\n");

    let (code, _, err) = invoke(&["--out=cov"], &host, None);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains(
            "coverage: srs is at 50.0% of its lines (1/2) in every test layer together, want 100%"
        ),
        "{err}"
    );
    assert!(host.files.borrow().contains_key(Path::new("cov/all.lcov")));
}

#[test]
fn run_fails_when_the_measurement_does() {
    for needle in ["metadata", "--lib", "append"] {
        let host = FakeHost {
            fail_on: Some(needle),
            ..host("")
        };
        let (code, _, err) = invoke(&[], &host, Some(Path::new("summary.md")));
        assert_eq!(code, EXIT_FAILURE, "{needle}");
        assert!(err.starts_with("coverage: "), "{needle}: {err}");
    }

    let host = FakeHost {
        metadata: "not json".into(),
        ..host("")
    };
    assert!(invoke(&[], &host, None).2.contains("read cargo metadata"));
}

#[test]
fn run_measures_one_layer_and_writes_only_its_lcov() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\n");

    let (code, out, err) = invoke(&["--layer", "unit", "--out", "cov"], &host, None);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "wrote cov/unit.lcov\n");
    {
        let calls = host.calls.borrow();
        assert!(
            calls.contains(
                &"cargo llvm-cov --no-report --workspace --locked --lib --bins".to_owned()
            )
        );
        assert!(!calls.iter().any(|c| c.contains("--test")), "{calls:?}");
    }
    assert!(!host.files.borrow().contains_key(Path::new("cov/all.lcov")));

    let (code, out, _) = invoke(&["--layer=integration"], &host, None);
    assert_eq!(code, EXIT_OK);
    assert_eq!(out, "no integration tests to measure\n");
}

/// A host whose workspace has a `bdd` target, as the real one does.
fn host_with_bdd(report: &str) -> FakeHost {
    FakeHost {
        metadata: METADATA.replace(
            r#""targets": []"#,
            r#""targets": [{"name": "bdd", "kind": ["test"]}]"#,
        ),
        ..host(report)
    }
}

#[test]
fn run_measures_one_shard_of_the_end_to_end_layer() {
    let host = host_with_bdd("SF:/w/srs/src/lib.rs\nDA:1,1\n");

    let (code, out, err) = invoke(
        &["--out", "cov", "--layer=e2e", "--shard", "2/4"],
        &host,
        None,
    );

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "wrote cov/e2e-2-of-4.lcov\n");
    assert!(host.calls.borrow().contains(
        &"cargo llvm-cov --no-report --workspace --locked --test bdd -- --shard 2/4".to_owned()
    ));
    assert!(
        host.files
            .borrow()
            .contains_key(Path::new("cov/e2e-2-of-4.lcov"))
    );

    let (code, out, _) = invoke(&["--shard=1/2", "--layer", "e2e"], &host, None);
    assert_eq!(code, EXIT_OK);
    assert_eq!(out, "wrote target/coverage/e2e-1-of-2.lcov\n");
}

#[test]
fn run_merges_the_shards_of_the_end_to_end_layer() {
    let host = host("");
    for index in 1..=2 {
        host.files.borrow_mut().insert(
            format!("cov/e2e-{index}-of-2.lcov").into(),
            format!("SF:/w/srs/src/lib.rs\nDA:{index},1\n"),
        );
    }
    host.files.borrow_mut().insert(
        "cov/e2e-others.lcov".into(),
        "SF:/w/srs/src/lib.rs\nDA:1,1\n".into(),
    );

    let (code, out, err) = invoke(&["--merge", "--out", "cov"], &host, None);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        out.contains("| srs | 2 | 0.0% | 0.0% | 100.0% | 100.0% |"),
        "{out}"
    );
    assert!(
        out.contains("end-to-end 1 (1 scenarios)"),
        "scenarios are counted once, not per shard: {out}"
    );
    assert!(host.files.borrow().contains_key(Path::new("cov/e2e.lcov")));

    host.files
        .borrow_mut()
        .remove(Path::new("cov/e2e-2-of-2.lcov"));
    let (code, _, err) = invoke(&["--merge", "--out", "cov"], &host, None);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("lack shard 2 of 2"), "{err}");

    host.files
        .borrow_mut()
        .remove(Path::new("cov/e2e-others.lcov"));
    host.files.borrow_mut().insert(
        "cov/e2e-2-of-2.lcov".into(),
        "SF:/w/srs/src/lib.rs\nDA:2,1\n".into(),
    );
    let (code, _, err) = invoke(&["--merge", "--out", "cov"], &host, None);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("lack e2e-others.lcov"), "{err}");
}

#[test]
fn run_measures_the_end_to_end_targets_beside_the_scenario_runner() {
    let host = FakeHost {
        metadata: METADATA.replace(
            r#""targets": []"#,
            r#""targets": [{"name": "bdd", "kind": ["test"]}, {"name": "e2e_x", "kind": ["test"]}]"#,
        ),
        ..host("SF:/w/srs/src/lib.rs\nDA:1,1\n")
    };

    let (code, out, err) = invoke(&["--out", "cov", "--layer", "e2e", "--others"], &host, None);

    assert_eq!(code, EXIT_OK, "{err}");
    assert_eq!(out, "wrote cov/e2e-others.lcov\n");
    let calls = host.calls.borrow();
    assert!(
        calls.contains(&"cargo llvm-cov --no-report --workspace --locked --test e2e_x".to_owned()),
        "{calls:?}"
    );
    assert!(!calls.iter().any(|c| c.contains("bdd")), "{calls:?}");
}

#[test]
fn run_refuses_the_others_it_cannot_use() {
    let host = host("");

    for (args, want) in [
        (&["--others"][..], "--others goes with --layer e2e"),
        (
            &["--layer", "unit", "--others"],
            "--others goes with --layer e2e",
        ),
        (&["--merge", "--others"], "--others goes with --layer e2e"),
        (
            &["--layer", "e2e", "--others", "--shard", "1/2"],
            "--shard and --others are one or the other",
        ),
    ] {
        let (code, _, err) = invoke(args, &host, None);
        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(err.contains(want), "{err}");
    }
    assert!(host.calls.borrow().is_empty());
}

#[test]
fn run_refuses_a_shard_it_cannot_use() {
    let host = host("");

    for (args, want) in [
        (&["--shard"][..], "--shard needs K/N"),
        (
            &["--layer", "e2e", "--shard", "5/4"],
            r#"want K/N with 1 <= K <= N, not "5/4""#,
        ),
        (
            &["--layer", "e2e", "--shard", "x"],
            r#"want K/N with 1 <= K <= N, not "x""#,
        ),
        (&["--shard", "1/2"], "--shard goes with --layer e2e"),
        (
            &["--layer", "unit", "--shard", "1/2"],
            "--shard goes with --layer e2e",
        ),
        (
            &["--merge", "--shard", "1/2"],
            "--shard goes with --layer e2e",
        ),
    ] {
        let (code, _, err) = invoke(args, &host, None);
        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(err.contains(want), "{err}");
    }
    assert!(host.calls.borrow().is_empty());
}

#[test]
fn run_merges_the_layers_written_and_holds_the_floors() {
    let host = host("");
    host.files.borrow_mut().insert(
        "cov/unit.lcov".into(),
        "SF:/w/srs/src/lib.rs\nDA:1,1\nDA:2,0\n".into(),
    );
    host.files.borrow_mut().insert(
        "cov/e2e.lcov".into(),
        "SF:/w/srs/src/lib.rs\nDA:2,1\n".into(),
    );

    let (code, out, err) = invoke(&["--merge", "--out", "cov"], &host, None);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        out.contains("| srs | 2 | 50.0% | 0.0% | 50.0% | 100.0% |"),
        "{out}"
    );
    assert!(
        !host.calls.borrow().iter().any(|c| c.contains("llvm-cov")),
        "a merge runs no test"
    );

    let (code, _, err) = invoke(&["--merge"], &host, None);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("no <layer>.lcov under target/coverage"),
        "{err}"
    );
}

#[test]
fn run_explains_its_command_line() {
    let host = host("");

    let (code, out, _) = invoke(&["--help"], &host, None);
    assert_eq!(code, EXIT_OK);
    assert!(out.starts_with("usage: coverage [--out DIR]"));

    for (args, want) in [
        (&["--bogus"][..], r#"unexpected argument "--bogus""#),
        (&["--out"], "--out needs a directory"),
        (&["--layer"], "--layer needs a name"),
        (&["--layer", "end-to-end"], r#"no test layer "end-to-end""#),
        (&["--layer", "unit", "--merge"], "one or the other, once"),
        (&["--merge", "--merge"], "one or the other, once"),
    ] {
        let (code, _, err) = invoke(args, &host, None);
        assert_eq!(code, EXIT_USAGE);
        assert!(err.contains(want), "{err}");
    }
    assert!(
        host.calls.borrow().is_empty(),
        "a wrong command line runs nothing"
    );
}

#[test]
fn run_measures_into_target_coverage_by_default() {
    let host = host("");

    invoke(&[], &host, None);

    assert!(
        host.files
            .borrow()
            .contains_key(Path::new("target/coverage/unit.lcov"))
    );
}

#[test]
fn run_reports_the_pyramid_s_shape() {
    let host = host("SF:/w/srs/src/lib.rs\nDA:1,1\n");

    let (code, out, err) = invoke(&[], &host, None);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        out.contains("Tests per test layer: unit 3, integration 2, end-to-end 1 (1 scenarios)"),
        "{out}"
    );
    assert!(
        host.calls
            .borrow()
            .contains(&"scenarios /w/features".to_owned())
    );
}

#[test]
fn run_fails_an_inverted_pyramid() {
    let host = FakeHost {
        scenarios: 5,
        ..host("SF:/w/srs/src/lib.rs\nDA:1,1\n")
    };

    let (code, _, err) = invoke(&[], &host, None);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("coverage: the test pyramid is inverted: unit 3, integration 2, end-to-end 5"),
        "{err}"
    );
}

#[test]
fn run_fails_when_the_tests_cannot_be_counted() {
    for needle in ["sources", "scenarios"] {
        let host = FakeHost {
            fail_on: Some(needle),
            ..host("SF:/w/srs/src/lib.rs\nDA:1,1\n")
        };
        let (code, _, err) = invoke(&[], &host, None);
        assert_eq!(code, EXIT_FAILURE, "{needle}");
        assert!(err.contains(&format!("{needle} /w")), "{needle}: {err}");
    }
}

#[test]
fn run_refuses_to_measure_inside_a_measurement() {
    let host = host("");
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(Vec::<String>::new(), &mut out, &mut err, &host, None, true);

    assert_eq!(code, EXIT_USAGE);
    assert!(
        String::from_utf8(err)
            .unwrap()
            .contains("refusing to recurse")
    );
    assert!(host.calls.borrow().is_empty());
}
