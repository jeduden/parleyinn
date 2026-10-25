use record::commitment::commit;
use record::event::{EventKind, Provenance};
use serde_json::json;
use testkit::TempDir;

use super::*;

fn open(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("store.db")).unwrap()
}

fn observed(hook: &str, session: &str, at: u64) -> Observation {
    Observation {
        hook: hook.to_owned(),
        harness_session: session.to_owned(),
        transcript_path: format!("/t/{session}.jsonl"),
        cwd: "/w".to_owned(),
        offset: Some(12),
        source: Some("startup".to_owned()),
        prompt_id: None,
        attended: false,
        ancestry: Some((1, false)),
        harness: Some(40),
        at,
    }
}

fn committed(session: &str, prompt_id: &str, text: &str) -> PromptCommitment {
    let key = CommitmentKey::from_bytes([7; 32]);
    PromptCommitment {
        harness_session: session.to_owned(),
        prompt_id: prompt_id.to_owned(),
        commitment: commit(&key, &json!(text)).unwrap(),
        key,
        used: false,
    }
}

#[test]
fn a_held_observation_reads_back_with_its_fields_not_appended() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let start = observed("SessionStart", "s1", 100);
    let mut prompt = observed("UserPromptSubmit", "s1", 101);
    prompt.prompt_id = Some("p1".into());
    prompt.attended = true;
    prompt.ancestry = None;
    prompt.offset = None;
    prompt.source = None;

    let first = store.hold_observation(&start, None).unwrap();
    let second = store
        .hold_observation(&prompt, Some(&committed("s1", "p1", "hi")))
        .unwrap();

    let held = store.hook_observations().unwrap();
    assert_eq!(
        held,
        [
            Held {
                id: first,
                observation: start,
                appended: false,
                taken: false,
            },
            Held {
                id: second,
                observation: prompt,
                appended: false,
                taken: false,
            }
        ]
    );
}

#[test]
fn a_prompt_commitment_is_kept_once_and_used_up_with_the_events_it_names() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let mut prompt = observed("UserPromptSubmit", "s1", 1);
    prompt.prompt_id = Some("p1".into());
    let id = store
        .hold_observation(&prompt, Some(&committed("s1", "p1", "first")))
        .unwrap();
    store
        .hold_observation(&prompt, Some(&committed("s1", "p1", "second")))
        .unwrap();
    assert_eq!(
        store.prompt_commitment("s1", "p1").unwrap(),
        Some(committed("s1", "p1", "first"))
    );
    assert_eq!(store.prompt_commitment("s1", "p2").unwrap(), None);

    let writer = WriterId::derive(&[1; 32]);
    let event = NewEvent {
        kind: EventKind::HookObservation,
        provenance: Provenance::Structural,
        content: json!({"hook": "UserPromptSubmit"}),
        commitment_key: CommitmentKey::from_bytes([2; 32]),
        transcript_timestamp: None,
    };
    let appended = store
        .append_observed(writer, &[event], &[id], &[("s1".into(), "p1".into())])
        .unwrap();

    assert_eq!(appended.len(), 1);
    assert_eq!(store.log(writer).unwrap().len(), 1);
    let held = store.hook_observations().unwrap();
    assert_eq!(
        held.iter().map(|h| h.appended).collect::<Vec<_>>(),
        [true, false]
    );
    assert!(store.prompt_commitment("s1", "p1").unwrap().unwrap().used);
    assert_eq!(
        store.append_observed(writer, &[], &[], &[]).unwrap(),
        Vec::new()
    );
}

#[test]
fn an_untaken_session_binding_is_counted_by_one_reader_only() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let old = store
        .hold_observation(&observed("SessionStart", "s1", 10), None)
        .unwrap();
    store
        .hold_observation(&observed("SessionStart", "s2", 50), None)
        .unwrap();
    store
        .hold_observation(&observed("UserPromptSubmit", "s1", 5), None)
        .unwrap();

    let untaken = store.untaken_bindings(41).unwrap();
    assert_eq!(untaken.iter().map(|h| h.id).collect::<Vec<_>>(), [old]);
    assert!(store.count_binding(old).unwrap());
    assert!(!store.count_binding(old).unwrap());
    assert_eq!(store.untaken_bindings(41).unwrap(), Vec::new());
}

#[test]
fn an_mcp_server_takes_its_own_harness_processs_bindings_once_and_none_is_counted_untaken() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let own = store
        .hold_observation(&observed("SessionStart", "s1", 10), None)
        .unwrap();
    let mut other = observed("SessionStart", "s2", 10);
    other.harness = Some(41);
    store.hold_observation(&other, None).unwrap();
    let mut unread = observed("SessionStart", "s3", 10);
    unread.harness = None;
    store.hold_observation(&unread, None).unwrap();
    let mut prompt = observed("UserPromptSubmit", "s1", 10);
    prompt.prompt_id = Some("p1".into());
    store.hold_observation(&prompt, None).unwrap();

    let taken = store.take_bindings(40).unwrap();

    assert_eq!(taken.iter().map(|h| h.id).collect::<Vec<_>>(), [own]);
    assert!(taken[0].taken);
    assert_eq!(store.take_bindings(40).unwrap(), Vec::new());
    let untaken: Vec<String> = store
        .untaken_bindings(100)
        .unwrap()
        .into_iter()
        .map(|h| h.observation.harness_session)
        .collect();
    assert_eq!(untaken, ["s2", "s3"]);
    assert!(
        store
            .hook_observations()
            .unwrap()
            .iter()
            .any(|h| h.id == own && h.taken)
    );
}

#[test]
fn a_prompt_commitment_whose_key_does_not_parse_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    store
        .hold_observation(
            &observed("UserPromptSubmit", "s1", 1),
            Some(&committed("s1", "p1", "x")),
        )
        .unwrap();
    store
        .conn
        .execute("UPDATE prompt_commitments SET commitment_key = 'zz'", [])
        .unwrap();
    assert!(matches!(
        store.prompt_commitment("s1", "p1"),
        Err(StoreError::Corrupt { .. })
    ));
    store
        .conn
        .execute(
            &format!(
                "UPDATE prompt_commitments SET commitment_key = '{}', commitment = 'zz'",
                CommitmentKey::from_bytes([7; 32]).to_hex()
            ),
            [],
        )
        .unwrap();
    assert!(matches!(
        store.prompt_commitment("s1", "p1"),
        Err(StoreError::Corrupt { .. })
    ));
}

#[test]
fn counts_and_offsets_stay_within_what_sqlite_holds() {
    assert_eq!(count(-1), 0);
    assert_eq!(count(5), 5);
    assert_eq!(sql_count(u64::MAX), i64::MAX);
}

#[test]
fn holding_and_counting_fail_when_sqlite_does() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    let id = store
        .hold_observation(&observed("SessionStart", "s1", 1), None)
        .unwrap();
    store
        .conn
        .execute_batch("DROP TABLE hook_observations")
        .unwrap();

    assert!(matches!(
        store.hold_observation(&observed("SessionStart", "s2", 2), None),
        Err(StoreError::Sqlite(_))
    ));
    assert!(matches!(
        store.count_binding(id),
        Err(StoreError::Sqlite(_))
    ));
}

#[test]
fn session_bindings_are_counted_taken_and_untaken_and_a_prompt_is_neither() {
    let dir = TempDir::new().unwrap();
    let mut store = open(&dir);
    assert_eq!(store.binding_counts().unwrap(), (0, 0));
    store
        .hold_observation(&observed("SessionStart", "s1", 10), None)
        .unwrap();
    let mut other = observed("SessionStart", "s2", 10);
    other.harness = Some(41);
    store.hold_observation(&other, None).unwrap();
    let mut prompt = observed("UserPromptSubmit", "s1", 10);
    prompt.prompt_id = Some("p1".into());
    store.hold_observation(&prompt, None).unwrap();
    assert_eq!(store.binding_counts().unwrap(), (0, 2));

    store.take_bindings(40).unwrap();

    assert_eq!(store.binding_counts().unwrap(), (1, 1));
}
