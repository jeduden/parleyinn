use super::*;
use crate::tests::FakeHost;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn plan() -> Plan {
    Plan {
        targets: BTreeMap::from([
            (TestLayer::Integration, vec!["gates".to_owned()]),
            (
                TestLayer::EndToEnd,
                vec!["bdd".to_owned(), "e2e_x".to_owned()],
            ),
        ]),
        exclude: "tests.rs$".into(),
        ..Plan::default()
    }
}

fn host(reports: &[&str]) -> FakeHost {
    FakeHost {
        reports: std::cell::RefCell::new(reports.iter().map(|r| (*r).to_owned()).collect()),
        ..FakeHost::default()
    }
}

#[test]
fn test_args_select_each_layer_s_targets() {
    let plan = plan();
    let joined = |l| test_args(&plan, l).map(|a| a.join(" "));

    assert_eq!(
        joined(TestLayer::Unit).unwrap(),
        "llvm-cov --no-report --workspace --locked --lib --bins"
    );
    assert_eq!(
        joined(TestLayer::Integration).unwrap(),
        "llvm-cov --no-report --workspace --locked --test gates"
    );
    assert_eq!(
        joined(TestLayer::EndToEnd).unwrap(),
        "llvm-cov --no-report --workspace --locked --test bdd --test e2e_x"
    );
    assert!(test_args(&Plan::default(), TestLayer::EndToEnd).is_none());
}

#[test]
fn report_args_leave_the_excluded_files_out() {
    assert_eq!(
        report_args(&plan()).join(" "),
        "llvm-cov report --lcov --ignore-filename-regex tests.rs$"
    );
    assert_eq!(
        report_args(&Plan::default()).join(" "),
        "llvm-cov report --lcov"
    );
}

#[test]
fn measure_runs_each_layer_on_fresh_profiles() {
    let host = host(&["SF:/w/a.rs\nDA:1,1\nDA:2,0\n", "SF:/w/a.rs\nDA:2,3\n", ""]);

    let layers = measure(&host, &plan(), Path::new("out")).unwrap();

    assert_eq!(
        layers.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
        TestLayer::ALL
    );
    assert_eq!(layers[1].1[&PathBuf::from("/w/a.rs")][&2], 3);
    let calls = host.calls.borrow();
    assert_eq!(calls[0], "cargo llvm-cov clean --workspace");
    assert_eq!(
        calls[1],
        "cargo llvm-cov --no-report --workspace --locked --lib --bins"
    );
    assert_eq!(
        calls[2],
        "cargo llvm-cov report --lcov --ignore-filename-regex tests.rs$"
    );
    assert_eq!(calls[3], "write out/unit.lcov");
    assert_eq!(calls[4], "cargo llvm-cov clean --profraw-only");
    assert_eq!(calls.last().unwrap(), "write out/all.lcov");
    assert_eq!(
        host.files.borrow()[&PathBuf::from("out/all.lcov")],
        "SF:/w/a.rs\nDA:1,1\nDA:2,3\nend_of_record\n"
    );
}

#[test]
fn measure_skips_a_layer_without_tests() {
    let host = host(&["SF:/w/a.rs\nDA:1,1\n"]);

    let layers = measure(&host, &Plan::default(), Path::new("out")).unwrap();

    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].0, TestLayer::Unit);
}

#[test]
fn measure_stops_at_the_first_failure() {
    let failing = FakeHost {
        fail_on: Some("--lib"),
        ..host(&[])
    };
    assert!(
        measure(&failing, &plan(), Path::new("out"))
            .unwrap_err()
            .contains("--lib --bins: exit status: 1")
    );

    let malformed = host(&["SF:a\nDA:x\n"]);
    assert!(
        measure(&malformed, &plan(), Path::new("out"))
            .unwrap_err()
            .starts_with("lcov line 2: ")
    );
}

#[test]
fn measure_layer_runs_one_layer_on_a_fresh_build() {
    let host = host(&["SF:/w/a.rs\nDA:2,3\n"]);

    let lines = measure_layer(
        &host,
        &plan(),
        TestLayer::Integration,
        None,
        Path::new("out"),
    )
    .unwrap()
    .unwrap();

    assert_eq!(lines[&PathBuf::from("/w/a.rs")][&2], 3);
    assert_eq!(
        *host.calls.borrow(),
        [
            "cargo llvm-cov clean --workspace",
            "cargo llvm-cov --no-report --workspace --locked --test gates",
            "cargo llvm-cov report --lcov --ignore-filename-regex tests.rs$",
            "write out/integration.lcov",
        ]
    );
    assert!(
        measure_layer(
            &host,
            &Plan::default(),
            TestLayer::EndToEnd,
            None,
            Path::new("out")
        )
        .unwrap()
        .is_none(),
        "a layer without tests is nothing to measure"
    );
}

#[test]
fn merge_layers_reads_what_was_written_and_skips_a_missing_layer() {
    let host = host(&[]);
    host.files.borrow_mut().insert(
        PathBuf::from("out/unit.lcov"),
        "SF:/w/a.rs\nDA:1,1\nDA:2,0\n".into(),
    );
    host.files
        .borrow_mut()
        .insert(PathBuf::from("out/e2e.lcov"), "SF:/w/a.rs\nDA:2,3\n".into());

    let layers = merge_layers(&host, Path::new("out")).unwrap();

    assert_eq!(
        layers.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
        [TestLayer::Unit, TestLayer::EndToEnd]
    );
    assert_eq!(
        host.files.borrow()[&PathBuf::from("out/all.lcov")],
        "SF:/w/a.rs\nDA:1,1\nDA:2,3\nend_of_record\n"
    );
    assert!(
        !host.calls.borrow().iter().any(|c| c.starts_with("cargo")),
        "a merge runs no cargo"
    );
}

#[test]
fn merge_layers_refuses_nothing_to_merge_and_a_bad_file() {
    let empty = host(&[]);
    assert_eq!(
        merge_layers(&empty, Path::new("out")).unwrap_err(),
        "no <layer>.lcov under out"
    );

    let malformed = host(&[]);
    malformed
        .files
        .borrow_mut()
        .insert(PathBuf::from("out/unit.lcov"), "SF:a\nDA:x\n".into());
    assert!(
        merge_layers(&malformed, Path::new("out"))
            .unwrap_err()
            .starts_with("lcov line 2: ")
    );

    let unreadable = FakeHost {
        fail_on: Some("read out/unit.lcov"),
        ..host(&[])
    };
    assert!(
        merge_layers(&unreadable, Path::new("out"))
            .unwrap_err()
            .contains("read out/unit.lcov")
    );
}

fn shard(index: u64, total: u64) -> Shard {
    Shard { index, total }
}

#[test]
fn a_shard_reads_k_of_n_and_names_its_file() {
    assert_eq!("2/4".parse::<Shard>().unwrap(), shard(2, 4));
    for bad in [
        "", "2", "0/4", "5/4", "1/0", "+1/4", "-1/4", "a/4", "1/b", "1/",
    ] {
        assert_eq!(
            bad.parse::<Shard>().unwrap_err(),
            format!("{bad:?} is no shard: want K/N with 1 <= K <= N")
        );
    }
    assert_eq!(
        shard(2, 4).file_name(TestLayer::EndToEnd),
        "e2e-2-of-4.lcov"
    );
}

#[test]
fn a_shard_file_name_is_read_back_for_its_layer_only() {
    let e2e = TestLayer::EndToEnd;

    assert_eq!(
        Shard::from_file_name(e2e, "e2e-3-of-4.lcov"),
        Some(shard(3, 4))
    );
    for other in [
        "e2e.lcov",
        "e2e-3-of-4",
        "e2e-0-of-4.lcov",
        "e2e-5-of-4.lcov",
        "e2e-x-of-4.lcov",
        "e2e-3.lcov",
        "unit-3-of-4.lcov",
        "all.lcov",
    ] {
        assert_eq!(Shard::from_file_name(e2e, other), None, "{other}");
    }
}

#[test]
fn test_runs_without_a_shard_are_the_layer_s_one_command() {
    let plan = plan();

    assert_eq!(
        test_runs(&plan, TestLayer::EndToEnd, None).unwrap(),
        [test_args(&plan, TestLayer::EndToEnd).unwrap()]
    );
    assert!(
        test_runs(&Plan::default(), TestLayer::EndToEnd, None)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_runs_give_the_first_shard_the_other_targets_and_the_rest_their_build() {
    let plan = plan();
    let joined = |index| {
        test_runs(&plan, TestLayer::EndToEnd, Some(shard(index, 4)))
            .unwrap()
            .iter()
            .map(|a| a.join(" "))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        joined(1),
        [
            "llvm-cov --no-report --workspace --locked --test e2e_x",
            "llvm-cov --no-report --workspace --locked --test bdd -- --shard 1/4"
        ]
    );
    for index in 2..=4 {
        assert_eq!(
            joined(index),
            [
                "llvm-cov --no-report --workspace --locked --test e2e_x -- --list".to_owned(),
                format!(
                    "llvm-cov --no-report --workspace --locked --test bdd -- --shard {index}/4"
                )
            ]
        );
    }
}

#[test]
fn test_runs_with_only_bdd_run_it_alone_in_the_first_shard() {
    let plan = Plan {
        targets: BTreeMap::from([(TestLayer::EndToEnd, vec!["bdd".to_owned()])]),
        ..Plan::default()
    };

    let runs = test_runs(&plan, TestLayer::EndToEnd, Some(shard(1, 2))).unwrap();

    assert_eq!(runs.len(), 1);
}

#[test]
fn test_runs_refuse_a_shard_that_has_nothing_to_divide() {
    let sharded = Some(shard(1, 2));

    assert_eq!(
        test_runs(&plan(), TestLayer::Integration, sharded).unwrap_err(),
        "no bdd target in the integration test layer to shard"
    );
    assert_eq!(
        test_runs(&Plan::default(), TestLayer::EndToEnd, sharded).unwrap_err(),
        "no bdd target in the e2e test layer to shard"
    );
    let no_bdd = Plan {
        targets: BTreeMap::from([(TestLayer::EndToEnd, vec!["e2e_x".to_owned()])]),
        ..Plan::default()
    };
    assert!(test_runs(&no_bdd, TestLayer::EndToEnd, sharded).is_err());
}

#[test]
fn measure_layer_writes_a_shard_to_its_own_file() {
    let host = host(&["SF:/w/a.rs\nDA:2,3\n"]);

    measure_layer(
        &host,
        &plan(),
        TestLayer::EndToEnd,
        Some(shard(2, 4)),
        Path::new("out"),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        *host.calls.borrow(),
        [
            "cargo llvm-cov clean --workspace",
            "cargo llvm-cov --no-report --workspace --locked --test e2e_x -- --list",
            "cargo llvm-cov --no-report --workspace --locked --test bdd -- --shard 2/4",
            "cargo llvm-cov report --lcov --ignore-filename-regex tests.rs$",
            "write out/e2e-2-of-4.lcov",
        ]
    );
}

#[test]
fn measure_layer_refuses_a_shard_of_another_layer_before_running_it() {
    let host = host(&[]);

    let err = measure_layer(
        &host,
        &plan(),
        TestLayer::Unit,
        Some(shard(1, 2)),
        Path::new("out"),
    )
    .unwrap_err();

    assert_eq!(err, "no bdd target in the unit test layer to shard");
    assert_eq!(*host.calls.borrow(), ["cargo llvm-cov clean --workspace"]);
}

fn with_shards(total: u64, present: &[u64]) -> FakeHost {
    let host = host(&[]);
    for index in present {
        host.files.borrow_mut().insert(
            PathBuf::from(format!("out/e2e-{index}-of-{total}.lcov")),
            format!("SF:/w/a.rs\nDA:{index},{index}\nDA:9,1\n"),
        );
    }
    host
}

#[test]
fn merge_layers_adds_the_shards_of_a_layer_up() {
    let host = with_shards(3, &[3, 1, 2]);
    host.files.borrow_mut().insert(
        PathBuf::from("out/unit.lcov"),
        "SF:/w/a.rs\nDA:1,1\n".into(),
    );
    // A merge an earlier run left is replaced, not read.
    host.files.borrow_mut().insert(
        PathBuf::from("out/e2e.lcov"),
        "SF:/w/stale.rs\nDA:1,1\n".into(),
    );

    let layers = merge_layers(&host, Path::new("out")).unwrap();

    assert_eq!(
        layers.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
        [TestLayer::Unit, TestLayer::EndToEnd]
    );
    let merged = "SF:/w/a.rs\nDA:1,1\nDA:2,2\nDA:3,3\nDA:9,3\nend_of_record\n";
    assert_eq!(host.files.borrow()[&PathBuf::from("out/e2e.lcov")], merged);
    assert_eq!(layers[1].1[&PathBuf::from("/w/a.rs")][&9], 3);
    assert_eq!(layers[1].1.len(), 1, "the stale merge is no part of it");
}

#[test]
fn merge_layers_refuses_shards_that_are_not_all_there() {
    let missing = with_shards(4, &[1, 2, 4]);
    assert_eq!(
        merge_layers(&missing, Path::new("out")).unwrap_err(),
        "the e2e shards under out lack shard 3 of 4"
    );

    let mixed = with_shards(4, &[1, 2, 3, 4]);
    mixed.files.borrow_mut().insert(
        PathBuf::from("out/e2e-1-of-2.lcov"),
        "SF:/w/a.rs\nDA:1,1\n".into(),
    );
    assert!(
        merge_layers(&mixed, Path::new("out"))
            .unwrap_err()
            .contains("disagree on their count: ")
    );

    let malformed = with_shards(1, &[1]);
    malformed
        .files
        .borrow_mut()
        .insert(PathBuf::from("out/e2e-1-of-1.lcov"), "SF:a\nDA:x\n".into());
    assert!(
        merge_layers(&malformed, Path::new("out"))
            .unwrap_err()
            .starts_with("lcov line 2: ")
    );
}

#[test]
fn render_writes_lines_back_as_lcov() {
    let lines = crate::lcov::parse("SF:b\nDA:2,0\nSF:a\nDA:1,1\n").unwrap();

    assert_eq!(
        render(&lines),
        "SF:a\nDA:1,1\nend_of_record\nSF:b\nDA:2,0\nend_of_record\n"
    );
}
