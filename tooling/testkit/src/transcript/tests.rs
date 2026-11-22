use harness_contract::{recording, replay, shapes};
use redaction::Redactor;
use transcript::shape::Shapes;

use super::*;
use crate::TempDir;

/// The problems the core's own parser and classifier find replaying
/// `lines` as a transcript.
fn problems(lines: &[Value]) -> Vec<String> {
    let body: String = lines.iter().map(|l| format!("{l}\n")).collect();
    replay::replay(&body, &Shapes::supported(), &Redactor::new(&[]).unwrap().0)
}

fn assert_replays_clean(lines: &[Value]) {
    assert_eq!(problems(lines), Vec::<String>::new());
}

#[test]
fn the_recordings_read_as_harness_contract_reads_them() {
    let version = repo_root().join("testdata/claude-code").join(VERSION);
    let expected: Vec<Vec<Value>> = shapes::transcripts(&version)
        .unwrap()
        .iter()
        .map(|p| recording::lines(p).unwrap())
        .collect();

    assert!(!expected.is_empty());
    assert_eq!(recorded_transcripts().unwrap(), expected.as_slice());
}

#[test]
fn a_transcript_listing_fails_on_a_missing_directory() {
    let dir = TempDir::new().unwrap();

    let err = transcript_files(&dir.path().join("none")).unwrap_err();

    assert!(err.starts_with("list "), "{err}");
}

#[test]
fn only_jsonl_files_under_a_transcripts_directory_are_transcripts() {
    let dir = TempDir::new().unwrap();
    let kept = dir.path().join("a/transcripts/b");
    fs::create_dir_all(&kept).unwrap();
    fs::create_dir_all(dir.path().join("a/hooks")).unwrap();
    for path in [
        kept.join("one.jsonl"),
        kept.join("note.txt"),
        dir.path().join("a/hooks/two.jsonl"),
    ] {
        fs::write(path, "{}\n").unwrap();
    }

    let found = transcript_files(dir.path()).unwrap();

    assert_eq!(found, [kept.join("one.jsonl")]);
}

#[test]
fn a_transcript_with_a_line_that_is_not_json_names_its_line() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    fs::write(&path, "{}\n\n{x\n").unwrap();

    let err = read_lines(&path).unwrap_err();

    assert!(err.contains("t.jsonl: line 3: "), "{err}");
    assert!(
        read_lines(&dir.path().join("none"))
            .unwrap_err()
            .starts_with("read ")
    );
}

#[test]
fn a_line_no_recording_holds_is_named() {
    assert_eq!(
        recorded("a line of no kind", |_| false).unwrap_err(),
        "no recorded line is a line of no kind"
    );
    assert_eq!(
        recorded_call("NoSuchTool").unwrap_err(),
        "no recording holds a call of NoSuchTool with its result"
    );
    assert!(
        recorded_where("user", "promptSource", &json!("nowhere"))
            .unwrap_err()
            .starts_with("no recorded line is a user line with promptSource")
    );
}

#[test]
fn a_recorded_call_comes_with_the_line_holding_its_result() {
    let (call, result) = recorded_call("Bash").unwrap();

    assert_eq!(call["message"]["content"][0]["name"], "Bash");
    assert_eq!(
        result["message"]["content"][0]["tool_use_id"],
        call["message"]["content"][0]["id"]
    );
    assert_replays_clean(&[call, result]);
}

#[test]
fn a_recorded_attachment_and_a_typed_line_replay_clean() {
    let attachment = attachment("date").unwrap();
    let typed = typed().unwrap();

    assert_eq!(attachment["attachment"]["type"], "date");
    assert_eq!(typed["promptSource"], "typed");
    assert_replays_clean(&[attachment, typed]);
}

#[test]
fn a_typed_line_carries_the_text_the_prompt_the_source_and_the_session() {
    let line = typed_line("say hi", "p-1", "pasted", "run-1").unwrap();

    assert_eq!(line["message"]["content"], "say hi");
    assert_eq!(line["promptId"], "p-1");
    assert_eq!(line["promptSource"], "pasted");
    assert_eq!(line["sessionId"], "run-1");
    assert_replays_clean(&[typed_line("say hi", "p-1", "typed", "run-1").unwrap()]);
}

#[test]
fn a_user_line_carries_its_text_and_its_own_uuid() {
    let line = user_line("deploy", 7).unwrap();

    assert_eq!(line["message"]["content"], "deploy");
    assert_eq!(line["uuid"], "00000000-0000-4000-8000-000000001207");
    assert_replays_clean(&[line]);
}

#[test]
fn a_uuid_is_the_fixture_line_number_in_hex() {
    assert_eq!(uuid(255), "00000000-0000-4000-8000-0000000000ff");
}

#[test]
fn a_bash_result_keeps_the_recorded_output_unless_one_is_given() {
    let (_, recorded_result) = recorded_call("Bash").unwrap();

    let kept = bash_result(None).unwrap();
    let given = bash_result(Some("sent")).unwrap();

    assert_eq!(kept[1], recorded_result);
    assert_eq!(given[1]["message"]["content"][0]["content"], "sent");
    assert_replays_clean(&kept);
    assert_replays_clean(&given);
}

#[test]
fn a_numbered_pair_links_its_call_to_its_result_with_uuids_of_their_own() {
    let lines = pair(3, "done").unwrap();

    assert_eq!(lines[0]["message"]["content"][0]["id"], "toolu_fixture3");
    assert_eq!(
        lines[1]["message"]["content"][0]["tool_use_id"],
        "toolu_fixture3"
    );
    assert_eq!(lines[1]["message"]["content"][0]["content"], "done");
    assert_eq!(lines[0]["uuid"], uuid(6));
    assert_eq!(lines[1]["uuid"], uuid(7));
    assert_replays_clean(&lines);
}

#[test]
fn a_tool_use_result_object_gets_the_output_as_its_stdout() {
    let mut with = json!({"toolUseResult": {"stdout": "old", "stderr": ""}});
    let mut without = json!({"toolUseResult": "interrupted"});

    set_stdout(&mut with, "out");
    set_stdout(&mut without, "out");

    assert_eq!(
        with["toolUseResult"],
        json!({"stdout": "out", "stderr": ""})
    );
    assert_eq!(without["toolUseResult"], "interrupted");
}

#[test]
fn a_clean_line_replays_clean_and_is_numbered() {
    let line = clean_line(4);

    assert!(line.ends_with('\n'));
    let parsed: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(parsed["atis"], "line 4");
    assert_replays_clean(&[parsed]);
}

#[test]
fn appended_lines_are_numbered_on_from_what_the_transcript_holds() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");

    append_lines(&path, 2).unwrap();
    append_lines(&path, 1).unwrap();

    let body = fs::read_to_string(&path).unwrap();
    let numbered: Vec<String> = body
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap()["atis"].to_string())
        .collect();
    assert_eq!(numbered, [r#""line 1""#, r#""line 2""#, r#""line 3""#]);
}

#[test]
fn appending_to_a_path_that_cannot_be_opened_fails() {
    let dir = TempDir::new().unwrap();

    assert!(append_lines(&dir.path().join("no/dir/t.jsonl"), 1).is_err());
}

#[test]
fn a_call_with_no_result_line_is_passed_over_for_one_with_a_result() {
    let call = |id: &str| json!({"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Bash", "id": id}]}});
    let result = json!({"message": {"content": [{"tool_use_id": "b", "content": "ok"}]}});
    let lines = [call("a"), call("b"), result.clone()];

    let (found, answer) = call_in(&lines, "Bash").unwrap();

    assert_eq!(found["message"]["content"][0]["id"], "b");
    assert_eq!(answer, result);
    assert_eq!(call_in(&lines[..1], "Bash"), None);
}
