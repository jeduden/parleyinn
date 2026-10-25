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

    let lines = measure_layer(&host, &plan(), TestLayer::Integration, Path::new("out"))
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

#[test]
fn render_writes_lines_back_as_lcov() {
    let lines = crate::lcov::parse("SF:b\nDA:2,0\nSF:a\nDA:1,1\n").unwrap();

    assert_eq!(
        render(&lines),
        "SF:a\nDA:1,1\nend_of_record\nSF:b\nDA:2,0\nend_of_record\n"
    );
}
