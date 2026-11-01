use super::*;

fn script(path: &str) -> Result<String, String> {
    match path {
        "scripts/a.sh" => Ok("#!/bin/bash\nset -e\necho a\n".to_owned()),
        _ => Err(format!("read {path}: no such file")),
    }
}

/// A job with a checkout of main's tree, then the step `line`.
fn in_a_job(line: &str) -> String {
    format!("jobs:\n  j:\n    steps:\n      - uses: actions/checkout@v7\n      - {line}\n")
}

#[test]
fn a_step_that_runs_a_script_holds_its_text() {
    let body = "jobs:\n  j:\n    steps:\n      - uses: actions/checkout@v7\n      - name: x\n        run: scripts/a.sh\n      - run: b\n      - run: scripts/a.sh\n";

    assert_eq!(
        inline_scripts(body, script).unwrap(),
        "jobs:\n  j:\n    steps:\n      - uses: actions/checkout@v7\n      - name: x\n        run: |\n          #!/bin/bash\n          set -e\n          echo a\n      - run: b\n      - run: |\n          #!/bin/bash\n          set -e\n          echo a\n"
    );
}

#[test]
fn a_script_that_cannot_be_read_or_takes_arguments_is_refused() {
    assert_eq!(
        inline_scripts(&in_a_job("run: scripts/b.sh"), script).unwrap_err(),
        "read scripts/b.sh: no such file"
    );
    assert_eq!(
        inline_scripts(&in_a_job("run: scripts/a.sh x"), script).unwrap_err(),
        "run: scripts/a.sh x passes the script arguments"
    );
}

#[test]
fn a_workflow_is_parsed_with_its_scripts_inline() {
    let root = testkit::TempDir::new().unwrap();
    let write = |rel: &str, text: &str| {
        let path = root.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "w.yml",
        "on:\n  push:\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v7\n      - run: scripts/a.sh\n",
    );
    write("scripts/a.sh", "echo from-the-script\n");
    let checkout = Checkout::new(root.path());

    let flow = load(&checkout, "w.yml").unwrap();

    assert!(flow.jobs[0].text.contains("echo from-the-script"));
    assert!(load(&checkout, "missing.yml").is_err());
}

#[test]
fn a_script_step_before_a_checkout_of_mains_tree_is_refused() {
    let before =
        "jobs:\n  j:\n    steps:\n      - run: scripts/a.sh\n      - uses: actions/checkout@v7\n";
    let err = inline_scripts(before, script).unwrap_err();
    assert_eq!(
        err,
        "run: scripts/a.sh in job j runs before a checkout of main's tree"
    );

    let none = "jobs:\n  j:\n    steps:\n      - run: scripts/a.sh\n";
    assert!(inline_scripts(none, script).is_err());
}

#[test]
fn a_checkout_that_is_conditional_or_of_another_path_does_not_count() {
    for extra in [
        "        if: x == 'y'\n",
        "        with:\n          path: pr-head\n",
        "        with:\n          ref: abc\n",
    ] {
        let body = format!(
            "jobs:\n  j:\n    steps:\n      - uses: actions/checkout@v7\n{extra}      - run: scripts/a.sh\n"
        );
        assert!(inline_scripts(&body, script).is_err(), "{extra}");
    }
    let lead_if = "jobs:\n  j:\n    steps:\n      - if: x\n        uses: actions/checkout@v7\n      - run: scripts/a.sh\n";
    assert!(inline_scripts(lead_if, script).is_err());
}

#[test]
fn a_checkout_in_another_job_does_not_count() {
    let body = "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@v7\n  b:\n    steps:\n      - run: scripts/a.sh\n";
    let err = inline_scripts(body, script).unwrap_err();
    assert!(err.contains("in job b"), "{err}");
}
