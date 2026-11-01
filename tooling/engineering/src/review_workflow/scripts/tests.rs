use super::*;

fn script(path: &str) -> Result<String, String> {
    match path {
        "scripts/a.sh" => Ok("#!/bin/bash\nset -e\necho a\n".to_owned()),
        _ => Err(format!("read {path}: no such file")),
    }
}

#[test]
fn a_step_that_runs_a_script_holds_its_text() {
    let body = "jobs:\n  j:\n    steps:\n      - name: x\n        run: scripts/a.sh\n      - run: b\n      - run: scripts/a.sh\n";

    assert_eq!(
        inline_scripts(body, script).unwrap(),
        "jobs:\n  j:\n    steps:\n      - name: x\n        run: |\n          #!/bin/bash\n          set -e\n          echo a\n      - run: b\n      - run: |\n          #!/bin/bash\n          set -e\n          echo a\n"
    );
}

#[test]
fn a_script_that_cannot_be_read_or_takes_arguments_is_refused() {
    assert_eq!(
        inline_scripts("  run: scripts/b.sh\n", script).unwrap_err(),
        "read scripts/b.sh: no such file"
    );
    assert_eq!(
        inline_scripts("  run: scripts/a.sh x\n", script).unwrap_err(),
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
        "on:\n  push:\njobs:\n  a:\n    steps:\n      - run: scripts/a.sh\n",
    );
    write("scripts/a.sh", "echo from-the-script\n");
    let checkout = Checkout::new(root.path());

    let flow = load(&checkout, "w.yml").unwrap();

    assert!(flow.jobs[0].text.contains("echo from-the-script"));
    assert!(load(&checkout, "missing.yml").is_err());
}
