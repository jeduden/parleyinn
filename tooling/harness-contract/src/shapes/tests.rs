use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::*;

/// A recording of version `1.0.0`: one transcript under `transcripts`,
/// and a hook input elsewhere that is no transcript.
fn recordings(root: &Path) {
    let transcripts = root.join("1.0.0/transcripts/p");
    fs::create_dir_all(&transcripts).unwrap();
    fs::write(
        transcripts.join("s.jsonl"),
        "{\"type\":\"mode\",\"mode\":\"normal\"}\n[1]\n",
    )
    .unwrap();
    let hooks = root.join("1.0.0/hooks");
    fs::create_dir_all(&hooks).unwrap();
    fs::write(hooks.join("h.jsonl"), "{\"type\":\"other\"}\n").unwrap();
}

#[test]
fn a_versions_transcripts_are_the_jsonl_files_under_a_transcripts_directory() {
    let dir = TempDir::new().unwrap();
    recordings(dir.path());

    let found = transcripts(&dir.path().join("1.0.0")).unwrap();
    assert_eq!(found, [dir.path().join("1.0.0/transcripts/p/s.jsonl")]);
}

#[test]
fn generating_reads_each_transcripts_objects_under_the_versions_name() {
    let dir = TempDir::new().unwrap();
    recordings(dir.path());

    let shapes = generate(&dir.path().join("1.0.0")).unwrap();
    assert_eq!(shapes.version, "1.0.0");
    assert_eq!(shapes.shapes.keys().collect::<Vec<_>>(), ["mode"]);

    fs::write(dir.path().join("1.0.0/transcripts/p/bad.jsonl"), "{x\n").unwrap();
    assert!(generate(&dir.path().join("1.0.0")).is_err());
}

#[test]
fn regenerating_writes_each_version_and_then_finds_nothing_differs() {
    let dir = TempDir::new().unwrap();
    let rec = dir.path().join("rec");
    recordings(&rec);
    let shapes = dir.path().join("shapes");

    assert_eq!(
        regenerate(&rec, &shapes, true).unwrap(),
        Vec::<String>::new()
    );
    let written = fs::read_to_string(file(&shapes, "1.0.0")).unwrap();
    assert!(written.contains("\"mode\""), "{written}");
    assert_eq!(
        regenerate(&rec, &shapes, false).unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn regenerating_names_a_file_that_differs_is_missing_or_has_no_version() {
    let dir = TempDir::new().unwrap();
    let rec = dir.path().join("rec");
    recordings(&rec);
    let shapes = dir.path().join("shapes");
    fs::create_dir_all(shapes.join(HARNESS)).unwrap();

    let missing = regenerate(&rec, &shapes, false).unwrap();
    assert_eq!(missing.len(), 1);
    assert!(
        missing[0].contains("is missing for the recorded version 1.0.0"),
        "{missing:?}"
    );

    fs::write(file(&shapes, "1.0.0"), "{}\n").unwrap();
    fs::write(file(&shapes, "0.9.0"), "{}\n").unwrap();
    let problems = regenerate(&rec, &shapes, false).unwrap();
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(
        problems[0].contains("differs from the shapes 1.0.0 records"),
        "{problems:?}"
    );
    assert!(
        problems[1].contains("0.9.0.json has no recorded version"),
        "{problems:?}"
    );
}

#[test]
fn regenerating_fails_when_it_cannot_write_or_read() {
    let dir = TempDir::new().unwrap();
    let rec = dir.path().join("rec");
    recordings(&rec);
    let blocked = dir.path().join("file");
    fs::write(&blocked, "").unwrap();

    assert!(regenerate(&rec, &blocked, true).is_err());
    let shapes = dir.path().join("shapes");
    fs::create_dir_all(file(&shapes, "1.0.0")).unwrap();
    assert!(regenerate(&rec, &shapes, true).is_err());
    assert!(regenerate(&dir.path().join("none"), &shapes, false).is_err());
}

#[test]
fn a_versions_hook_input_shapes_key_each_input_by_its_hook() {
    let dir = TempDir::new().unwrap();
    let rec = dir.path().join("rec");
    let flow = rec.join("1.0.0/hooks/startup");
    fs::create_dir_all(&flow).unwrap();
    fs::write(
        flow.join("01-SessionStart.json"),
        r#"{"hook_event_name":"SessionStart","session_id":"s"}"#,
    )
    .unwrap();
    fs::write(flow.join("02-x.json"), r#"{"no_hook":1}"#).unwrap();

    let generated = generate_hooks(&rec.join("1.0.0")).unwrap();
    assert_eq!(generated.version, "1.0.0");
    assert_eq!(
        generated.shapes.keys().collect::<Vec<_>>(),
        ["SessionStart"]
    );
    let shapes = dir.path().join("shapes");
    assert_eq!(
        regenerate_hooks(&rec, &shapes, true).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(
        regenerate_hooks(&rec, &shapes, false).unwrap(),
        Vec::<String>::new()
    );
    fs::write(flow.join("03-bad.json"), "{x").unwrap();
    assert!(generate_hooks(&rec.join("1.0.0")).is_err());
}
