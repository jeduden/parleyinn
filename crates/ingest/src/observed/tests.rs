use store::Observation;

use super::*;

fn held(id: i64, hook: &str, session: &str) -> Held {
    Held {
        id,
        observation: Observation {
            hook: hook.into(),
            harness_session: session.into(),
            transcript_path: format!("/t/{session}.jsonl"),
            cwd: "/w".into(),
            offset: Some(10),
            source: None,
            prompt_id: None,
            attended: false,
            ancestry: None,
            harness: None,
            at: 1,
        },
        appended: false,
        taken: false,
    }
}

#[test]
fn a_session_start_with_source_fork_is_a_fork_observation() {
    let mut fork = held(1, "SessionStart", "f");
    fork.observation.source = Some("fork".into());
    let mut startup = held(2, "SessionStart", "s");
    startup.observation.source = Some("startup".into());
    let mut prompt = held(3, "UserPromptSubmit", "u");
    prompt.observation.source = Some("fork".into());

    let observed = Observed::of(vec![fork, startup, prompt]);

    assert_eq!(observed.forks(), BTreeSet::from(["f".to_owned()]));
}

#[test]
fn a_session_is_observed_once_a_session_start_read_its_ancestry() {
    let unread = held(1, "SessionStart", "s");
    let mut read = held(2, "SessionStart", "s");
    read.observation.ancestry = Some((1, false));
    let mut nested = held(3, "SessionStart", "n");
    nested.observation.ancestry = Some((2, true));
    let mut prompt = held(4, "UserPromptSubmit", "p");
    prompt.observation.ancestry = Some((1, false));

    let observed = Observed::of(vec![unread.clone(), read, nested, prompt]);

    assert_eq!(
        observed.session("s"),
        Session {
            ancestry_observed: true,
            nested: false
        }
    );
    assert_eq!(
        observed.session("n"),
        Session {
            ancestry_observed: true,
            nested: true
        }
    );
    assert_eq!(observed.session("p"), Session::default());
    assert_eq!(Observed::of(vec![unread]).session("s"), Session::default());
}

#[test]
fn the_attended_mark_is_read_from_the_prompts_own_hook_observation() {
    let mut attended = held(1, "UserPromptSubmit", "s");
    attended.observation.prompt_id = Some("p1".into());
    attended.observation.attended = true;
    let mut unattended = held(2, "UserPromptSubmit", "s");
    unattended.observation.prompt_id = Some("p2".into());
    let mut start = held(3, "SessionStart", "s");
    start.observation.prompt_id = Some("p3".into());
    start.observation.attended = true;

    let observed = Observed::of(vec![attended, unattended, start]);

    assert!(observed.attended("s", "p1"));
    assert!(!observed.attended("s", "p2"));
    assert!(!observed.attended("s", "p3"));
    assert!(!observed.attended("t", "p1"));
}

#[test]
fn pending_observations_are_a_sessions_not_yet_appended() {
    let mut appended = held(1, "SessionStart", "s");
    appended.appended = true;
    let observed = Observed::of(vec![
        appended,
        held(2, "UserPromptSubmit", "s"),
        held(3, "SessionStart", "t"),
    ]);

    let ids: Vec<i64> = observed.pending("s").iter().map(|h| h.id).collect();

    assert_eq!(ids, [2]);
}

#[test]
fn an_observation_event_holds_only_structural_fields() {
    let run = WriterId::derive(&[1; 32]);
    let mut start = held(1, "SessionStart", "s");
    start.observation.source = Some("fork".into());
    start.observation.ancestry = Some((1, false));
    assert_eq!(
        content(&start, run),
        json!({"hook": "SessionStart", "harness_session": "s", "run": run.to_string(),
               "offset": 10, "source": "fork", "harness_processes": 1, "nested": false})
    );
    let mut prompt = held(2, "UserPromptSubmit", "s");
    prompt.observation.prompt_id = Some("p".into());
    prompt.observation.offset = None;
    assert_eq!(
        content(&prompt, run),
        json!({"hook": "UserPromptSubmit", "harness_session": "s", "run": run.to_string(),
               "offset": null, "prompt_id": "p", "attended": false})
    );
}
