use super::*;
use serde_json::json;

#[test]
fn a_main_transcript_s_lines_belong_to_its_harness_session_s_main_run() {
    let defaults = Defaults::of(Path::new("/p/-w-a/s1.jsonl"));

    let main = RunKey {
        agent_id: None,
        harness_session: "s1".into(),
    };
    assert_eq!(defaults.own(), main);
    assert_eq!(defaults.key(&json!({"type": "mode"})), main);
    assert_eq!(defaults.key(&json!({"agentId": ""})), main);
}

#[test]
fn a_subagent_transcript_s_lines_belong_to_its_agent_s_run() {
    let defaults = Defaults::of(Path::new("/p/-w-a/s1/subagents/agent-a1.jsonl"));

    let sub = |a: &str| RunKey {
        agent_id: Some(a.into()),
        harness_session: "s1".into(),
    };
    assert_eq!(defaults.own(), sub("a1"));
    assert_eq!(defaults.key(&json!({"type": "last-prompt"})), sub("a1"));
    // A transcript holding several agents splits by run.
    assert_eq!(defaults.key(&json!({"agentId": "a2"})), sub("a2"));
}

#[test]
fn only_a_subagents_directory_names_a_subagent() {
    let elsewhere = Defaults::of(Path::new("/p/agent-a1.jsonl"));
    let unnamed = Defaults::of(Path::new("/p/s1/subagents/agent-.jsonl"));

    assert_eq!(elsewhere.own().agent_id, None);
    assert_eq!(elsewhere.own().harness_session, "agent-a1");
    assert_eq!(unnamed.own().agent_id, None);
    assert_eq!(Defaults::of(Path::new("")).own().harness_session, "");
}
