use std::fs;

use testkit::TempDir;

use super::*;

fn call(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(args.iter().map(|a| (*a).to_owned()), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// A review's work directory with `verdict`, under `root`.
fn review(root: &TempDir, name: &str, verdict: &str, head: &str) -> String {
    let dir = root.path().join(name);
    fs::create_dir_all(dir.join("review")).unwrap();
    fs::write(
        dir.join("review/outcome.json"),
        format!(r#"{{"verdict":"{verdict}","findings":[]}}"#),
    )
    .unwrap();
    fs::write(
        dir.join("review/run-id.json"),
        format!(r#"{{"head":"{head}","inputs":"i"}}"#),
    )
    .unwrap();
    dir.display().to_string()
}

#[test]
fn a_wrong_command_line_prints_the_usage() {
    for args in [
        &[][..],
        &["other"],
        &["review"],
        &["write"],
        &["write", "--out", "x"],
        &["stability", "--out", "x", "one"],
        &["stability", "--in", "x", "one", "two"],
        &["stability"],
        &["states"],
        &["states", "a", "b"],
    ] {
        let (code, _, err) = call(args);
        assert_eq!(code, EXIT_USAGE, "{args:?}");
        assert!(err.contains(USAGE), "{args:?}");
    }
}

#[test]
fn write_writes_the_run_list_and_the_report_of_a_work_directory() {
    let root = TempDir::new().unwrap();
    let dir = review(&root, "w", "approve", "abc");

    let (code, _, err) = call(&["write", "--workdir", &dir]);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(
        fs::read_to_string(format!("{dir}/{}", crate::AGENT_RUNS))
            .unwrap()
            .starts_with("{\"review\":")
    );
    assert!(
        fs::read_to_string(format!("{dir}/{}", crate::REPORT))
            .unwrap()
            .contains("approve")
    );
}

#[test]
fn write_names_each_file_it_could_not_write_and_fails() {
    let root = TempDir::new().unwrap();
    let dir = review(&root, "w", "approve", "abc");
    fs::create_dir_all(format!("{dir}/{}", crate::REPORT)).unwrap();

    let (code, _, err) = call(&["write", "--workdir", &dir]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("review/report.html not written: "), "{err}");
}

#[test]
fn stability_writes_its_file_and_prints_its_headline() {
    let root = TempDir::new().unwrap();
    let one = review(&root, "one", "approve", "abc");
    let two = review(&root, "two", "request_changes", "abc");
    let three = review(&root, "three", "approve", "abc");
    let out = root.path().join("stability.json").display().to_string();

    let (code, printed, err) = call(&["stability", "--out", &out, &one, &two, &three]);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert_eq!(
        printed,
        "runs=3\nflip_rate=0.333333\njaccard=1\nstable=false\nsame_state=true\nwithin_limits=false\n"
    );
    let file: serde_json::Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(file["flip_rate"], 0.333_333);
    assert_eq!(file["mode"], "approve");
}

#[test]
fn stability_warns_on_runs_of_different_states_and_names_what_did_not_read() {
    let root = TempDir::new().unwrap();
    let one = review(&root, "one", "approve", "abc");
    let two = review(&root, "two", "approve", "def");
    fs::write(format!("{two}/review/usage.json"), "not json").unwrap();
    let out = root.path().join("s.json").display().to_string();

    let (code, printed, err) = call(&["stability", "--out", &out, &one, &two]);

    assert_eq!(code, EXIT_OK);
    assert!(printed.contains("same_state=false"));
    assert!(err.contains("review/usage.json does not read"), "{err}");
    assert!(err.contains("their stability means nothing"), "{err}");
}

#[test]
fn stability_fails_naming_a_file_it_could_not_write() {
    let root = TempDir::new().unwrap();
    let one = review(&root, "one", "approve", "abc");
    let out = root.path().join("missing/s.json").display().to_string();

    let (code, _, err) = call(&["stability", "--out", &out, &one, &one]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("s.json not written: "), "{err}");
}

#[test]
fn states_prints_the_measurement_s_matrices_or_fails_naming_why() {
    let root = TempDir::new().unwrap();
    let file = root.path().join("states.toml");
    fs::write(
        &file,
        "[[state]]\nname = \"pr-99\"\npull = 99\n\
         base = \"cbf144c6e74833535424c54cbfd04a23d90b47b0\"\n\
         head = \"86ec49030e8e8fe2af2bc2fd9441a64167e12ab6\"\n",
    )
    .unwrap();
    let file = file.display().to_string();

    let (code, out, err) = call(&["states", &file]);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("states={\"include\":[{"), "{out}");
    assert!(out.contains("\nsamples={\"include\":[{"), "{out}");

    fs::write(&file, "state = []").unwrap();
    let (code, _, err) = call(&["states", &file]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("the states holds no state"), "{err}");

    let (code, _, err) = call(&["states", "/nonexistent/states.toml"]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("read /nonexistent/states.toml: "), "{err}");
}

#[test]
fn stability_says_why_failed_reviews_measure_nothing() {
    let root = TempDir::new().unwrap();
    let one = root.path().join("one").display().to_string();
    let two = root.path().join("two").display().to_string();
    let out = root.path().join("s.json").display().to_string();

    let (code, printed, err) = call(&["stability", "--out", &out, &one, &two]);

    assert_eq!(code, EXIT_OK);
    assert!(printed.contains("stable=false"), "{printed}");
    assert!(printed.contains("within_limits=false"), "{printed}");
    assert!(err.contains("2 of 2 runs have no verdict"), "{err}");
}
