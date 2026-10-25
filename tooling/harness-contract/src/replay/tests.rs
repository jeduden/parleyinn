use std::fs;

use serde_json::json;
use testkit::TempDir;
use transcript::shape::RecordedShapes;

use super::*;

fn shapes() -> Shapes {
    let mode = json!({"type": "mode", "mode": "normal"});
    Shapes::of(vec![RecordedShapes::generate(
        "1.0.0",
        &[mode.as_object().cloned().unwrap()],
    )])
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

#[test]
fn a_transcript_that_fits_its_shapes_replays_clean() {
    let body = "{\"type\":\"mode\",\"mode\":\"plan\"}\n\n{\"type\":\"mode\",\"mode\":\"normal\"}\n";

    assert_eq!(replay(body, &shapes(), &redactor()), Vec::<String>::new());
}

#[test]
fn replaying_names_each_unparsed_and_drifting_line() {
    let body = "{x\n{\"type\":\"future_kind\"}\n{\"type\":\"mode\"}\n{\"type\":\"mode\",\"mode\":\"n\",\"extra\":1}\n";

    assert_eq!(
        replay(body, &shapes(), &redactor()),
        [
            "line 1: unparsed",
            "line 2: format drift claude-code:unversioned:future_kind:unrecorded-type",
            "line 3: format drift claude-code:unversioned:mode:missing:mode",
            "line 4: format drift claude-code:unversioned:mode:beyond:extra",
        ]
    );
}

#[test]
fn replaying_a_file_leads_each_problem_with_its_path() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    fs::write(&path, "{x\n").unwrap();

    let problems = replay_file(&path, &shapes(), &redactor()).unwrap();
    assert_eq!(problems, [format!("{}: line 1: unparsed", path.display())]);
    assert!(replay_file(&dir.path().join("none"), &shapes(), &redactor()).is_err());
}

#[test]
fn a_hook_input_replays_through_the_cores_decoder() {
    let dir = TempDir::new().unwrap();
    let write = |name: &str, body: &str| {
        let path = dir.path().join(name);
        fs::write(&path, body).unwrap();
        path
    };
    let fits = write(
        "fits.json",
        r#"{"session_id":"s","transcript_path":"/t","cwd":"/w","hook_event_name":"SessionStart","source":"startup"}"#,
    );
    let drifts = write(
        "drifts.json",
        r#"{"session_id":"s","transcript_path":"/t","cwd":"/w","hook_event_name":"SessionStart"}"#,
    );
    let invalid = write(
        "invalid.json",
        r#"{"session_id":1,"hook_event_name":"SessionStart"}"#,
    );
    let other = write("other.json", r#"{"hook_event_name":"Stop"}"#);

    assert_eq!(replay_hook_input(&fits).unwrap(), Vec::<String>::new());
    assert_eq!(replay_hook_input(&other).unwrap(), Vec::<String>::new());
    assert_eq!(
        replay_hook_input(&drifts).unwrap(),
        [format!(
            "{}: format drift claude-code:2.1.296:SessionStart:missing:source",
            drifts.display()
        )]
    );
    assert_eq!(
        replay_hook_input(&invalid).unwrap(),
        [format!(
            "{}: hook input failed schema validation",
            invalid.display()
        )]
    );
    assert!(replay_hook_input(&dir.path().join("none")).is_err());
}
