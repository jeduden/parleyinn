//! Ingest's later tests: resumption, forks, repeats and hook observations.
//! The rest of the module's tests, moved from `tests.rs` to keep each file in its limit.

use super::*;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use store::Store;
use testkit::TempDir;

#[test]
fn an_unterminated_last_line_waits_until_it_is_whole() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "{\"a\":1}\n{\"b\":");

    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    std::fs::write(&path, "{\"a\":1}\n{\"b\":2}\n").unwrap();
    let rest = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((first.events, rest.events), (1, 1));
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(first.writer).unwrap();
    assert_eq!(log[1].content.as_ref().unwrap()["text"], "{\"b\":2}");
}

#[test]
fn a_shrunk_or_rewritten_source_starts_a_new_generation_keeping_every_event() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, &numbered("s", 1, 40));
    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    std::fs::write(&path, numbered("s", 1, 10)).unwrap();
    let shrunk = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let mut rewritten = String::from("{\"other\":\"first line\"}\n");
    rewritten.push_str(&numbered("s", 2, 40));
    std::fs::write(&path, &rewritten).unwrap();
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let unchanged = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert!(shrunk.new_generation && shrunk.generation == 2);
    assert_eq!(shrunk.events, 0, "its 10 lines equal the earlier ones");
    assert!(again.new_generation && again.generation == 3);
    assert_eq!(again.events, 40);
    assert!(!unchanged.new_generation && unchanged.events == 0);
    assert_eq!(again.writer, first.writer);
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.log(first.writer).unwrap().len(), 80);
    verifies(&home);
}

#[test]
fn a_new_generation_compares_with_the_lines_its_predecessor_skipped_too() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, &numbered("s", 1, 10));
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    std::fs::write(&path, numbered("s", 1, 5)).unwrap();
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let mut grown = numbered("s", 1, 3);
    grown.push_str("{\"new\":1}\n");
    std::fs::write(&path, grown).unwrap();

    let third = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    // Generation 2 holds lines 1 to 5 as generation 1's; generation 3
    // shares lines 1 to 3 with it and adds one.
    assert_eq!((third.generation, third.events), (3, 1));
}

#[test]
fn a_transcript_splits_by_run_each_subagent_tied_to_its_parent() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let main = dir.path().join("s1.jsonl");
    std::fs::write(&main, numbered("s1", 1, 2)).unwrap();
    let sub_dir = dir.path().join("s1/subagents");
    std::fs::create_dir_all(&sub_dir).unwrap();
    let sub = sub_dir.join("agent-Explore.jsonl");
    let lines = [
        serde_json::json!({"type": "user", "uuid": "a", "sessionId": "s1", "agentId": "Explore"}),
        serde_json::json!({"type": "last-prompt"}),
        serde_json::json!({"type": "user", "uuid": "b", "sessionId": "s1", "agentId": "Other"}),
    ];
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    std::fs::write(&sub, text).unwrap();

    let parent = ingest_path(&home, &main, &redactor(), &mut OsEntropy).unwrap();
    let ingested = ingest_path(&home, &sub, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(parent.runs.len(), 1);
    assert_eq!(parent.runs[0].agent_id, None);
    assert_eq!(parent.runs[0].parent, None);
    assert_eq!(ingested.runs.len(), 2);
    let explore = &ingested.runs[0];
    assert_eq!(explore.writer, ingested.writer);
    assert_eq!(explore.agent_id.as_deref(), Some("Explore"));
    assert_eq!(explore.parent, Some(parent.writer));
    assert_eq!(explore.events, 2);
    assert_eq!(ingested.runs[1].agent_id.as_deref(), Some("Other"));
    assert_eq!(ingested.runs[1].events, 1);
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.ingested_runs().unwrap().len(), 3);
    verifies(&home);
}

#[test]
fn with_no_fork_observation_a_repeated_line_refers_to_its_first_recorder() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 5));
    let fork = dir.path().join("fork.jsonl");
    let mut text = numbered("f", 1, 3);
    text.push_str(&numbered("f", 100, 101));
    std::fs::write(&fork, text).unwrap();

    let owner = ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    let repeating = ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((repeating.events, repeating.references), (5, 3));
    assert_eq!(repeating.owners_chosen, 1);
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(repeating.writer).unwrap();
    assert_eq!(log[0].link.header.kind, EventKind::RepeatReference);
    let owned = store.log(owner.writer).unwrap();
    assert_eq!(
        log[0].content.as_ref().unwrap()["repeats"],
        owned[0].link.header.address.to_string()
    );
    assert_eq!(
        log[0].link.header.provenance,
        owned[0].link.header.provenance
    );
    assert_eq!(log[3].link.header.kind, EventKind::UnparsedLine);
    let device = administration::node::Node::find(&home, &store).unwrap();
    let chosen: Vec<_> = store
        .log(device.device_writer())
        .unwrap()
        .into_iter()
        .filter(|e| e.link.header.kind == EventKind::RepeatOwner)
        .collect();
    assert_eq!(chosen.len(), 1);
    assert_eq!(
        chosen[0].content.as_ref().unwrap()["owner"],
        owner.writer.to_string()
    );
    verifies(&home);
}

#[test]
fn a_uuid_repeated_with_other_content_is_its_own_event_and_a_conflict() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 1));
    let planted = dir.path().join("planted.jsonl");
    std::fs::write(&planted, "{\"type\":\"x\",\"uuid\":\"u1\",\"n\":99}\n").unwrap();

    ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    let ingested = ingest_path(&home, &planted, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((ingested.conflicts, ingested.references), (1, 0));
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    assert_ne!(log[0].link.header.kind, EventKind::RepeatReference);
}

#[test]
fn an_ingest_leaves_its_ingest_marker_at_the_time_on_its_clock() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, &numbered("s", 1, 3));
    let at = |now_ms| Inputs {
        now_ms,
        ..Inputs::default()
    };
    ingest_with(&home, &path, &redactor(), &at(5_000), &mut OsEntropy).unwrap();
    let marked = || {
        Store::open(&home.store())
            .unwrap()
            .sources()
            .unwrap()
            .into_iter()
            .map(|s| s.marked_at)
            .collect::<Vec<_>>()
    };
    assert_eq!(marked(), [5_000]);

    std::fs::write(&path, numbered("s", 1, 4)).unwrap();
    ingest_with(&home, &path, &redactor(), &at(9_000), &mut OsEntropy).unwrap();

    assert_eq!(marked(), [9_000]);
}

/// Inputs naming the harness session `fork` a fork, at `now_ms`.
fn forked(fork: &str, now_ms: u64) -> Inputs {
    Inputs {
        forks: [fork.to_owned()].into(),
        now_ms,
        ..Inputs::default()
    }
}

#[test]
fn a_fork_read_first_is_held_until_its_owner_is_ingested() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 5));
    let fork = dir.path().join("fork.jsonl");
    let mut text = String::from("{\"type\":\"mode\"}\n");
    text.push_str(&numbered("f", 1, 3));
    text.push_str(&numbered("f", 100, 100));
    std::fs::write(&fork, text).unwrap();
    let r = redactor();

    let held = ingest_with(&home, &fork, &r, &forked("fork", 1_000), &mut OsEntropy).unwrap();
    let owner = ingest_with(&home, &parent, &r, &forked("fork", 2_000), &mut OsEntropy).unwrap();
    let released = ingest_with(&home, &fork, &r, &forked("fork", 3_000), &mut OsEntropy).unwrap();

    assert!(held.held && held.hold_started);
    assert_eq!(held.events, 1, "its own line before the repeated ones");
    assert_eq!(owner.events, 5);
    assert!(!released.held);
    assert_eq!((released.events, released.references), (4, 3));
    assert_eq!(released.owners_chosen, 0, "the fork observation chose");
    let store = Store::open(&home.store()).unwrap();
    assert!(
        store
            .source(&fork.to_string_lossy())
            .unwrap()
            .unwrap()
            .held_since
            .is_none()
    );
    verifies(&home);
}

#[test]
fn a_hold_releases_its_lines_after_30_s_or_once_the_owner_is_gone() {
    for gone in [false, true] {
        let dir = TempDir::new().unwrap();
        let home = home(&dir);
        let parent = transcript(&dir, &numbered("p", 1, 2));
        let fork = dir.path().join("fork.jsonl");
        std::fs::write(&fork, numbered("f", 1, 2)).unwrap();
        let r = redactor();

        let held = ingest_with(&home, &fork, &r, &forked("fork", 1_000), &mut OsEntropy).unwrap();
        let still = ingest_with(&home, &fork, &r, &forked("fork", 30_999), &mut OsEntropy).unwrap();
        let later = if gone {
            std::fs::remove_file(&parent).unwrap();
            2_000
        } else {
            31_000
        };
        let released =
            ingest_with(&home, &fork, &r, &forked("fork", later), &mut OsEntropy).unwrap();

        assert!(held.held && held.hold_started && held.events == 0);
        assert!(still.held && !still.hold_started);
        assert!(!released.held, "gone: {gone}");
        assert_eq!((released.events, released.released), (2, 2));
        assert_eq!(released.references, 0);
    }
}

#[test]
fn a_fork_s_line_whose_owner_was_released_stays_the_owner_s_own() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 2));
    let fork = dir.path().join("fork.jsonl");
    std::fs::write(&fork, numbered("f", 1, 2)).unwrap();
    let r = redactor();
    ingest_with(&home, &fork, &r, &forked("fork", 0), &mut OsEntropy).unwrap();
    ingest_with(&home, &fork, &r, &forked("fork", 40_000), &mut OsEntropy).unwrap();

    let owner = ingest_with(&home, &parent, &r, &forked("fork", 41_000), &mut OsEntropy).unwrap();

    assert_eq!((owner.events, owner.references), (2, 0));
}

#[test]
fn after_a_node_identity_change_a_run_continues_in_a_new_ingest_seat() {
    let dir = TempDir::new().unwrap();
    let id = dir.path().join("machine-id");
    std::fs::write(&id, "first").unwrap();
    let home =
        home(&dir).with_identity(administration::node_identity::IdentitySource::Files(vec![
            id.clone(),
        ]));
    let path = transcript(&dir, &numbered("s", 1, 2));
    let before = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    std::fs::write(&id, "second").unwrap();
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(&numbered("s", 3, 3));
    std::fs::write(&path, text).unwrap();

    let after = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_ne!(after.writer, before.writer);
    assert_eq!(after.events, 1);
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.log(before.writer).unwrap().len(), 2);
    verifies(&home);
}

#[test]
fn a_repeated_line_refers_through_a_repeat_reference_when_a_generation_compares() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = transcript(&dir, &numbered("p", 1, 2));
    let fork = dir.path().join("fork.jsonl");
    std::fs::write(&fork, numbered("f", 1, 2)).unwrap();
    ingest_path(&home, &parent, &redactor(), &mut OsEntropy).unwrap();
    ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();

    std::fs::write(&fork, numbered("f", 1, 1)).unwrap();
    let shrunk = ingest_path(&home, &fork, &redactor(), &mut OsEntropy).unwrap();

    // Its one line equals, by content, the line its repeat reference
    // names.
    assert!(shrunk.new_generation);
    assert_eq!(shrunk.events, 0);
}

/// A recorded Claude Code 2.1.296 line typed at the harness's input,
/// carrying `text` as its prompt `prompt_id` marked `source`.
fn typed(text: &str, prompt_id: &str, source: &str) -> String {
    let recorded = std::fs::read_to_string(
        testkit::repo_root()
            .join("testdata/claude-code/2.1.296/transcripts/rewind/1-before-rewind.jsonl"),
    )
    .unwrap();
    let mut line: serde_json::Value = recorded
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|l| l["promptSource"] == "typed")
        .unwrap();
    line["message"]["content"] = serde_json::json!(text);
    line["promptId"] = serde_json::json!(prompt_id);
    line["promptSource"] = serde_json::json!(source);
    format!("{line}\n")
}

/// What a hook handler of harness session `session` observed.
fn observation(hook: &str, session: &str) -> store::Observation {
    store::Observation {
        hook: hook.to_owned(),
        harness_session: session.to_owned(),
        transcript_path: format!("/t/{session}.jsonl"),
        cwd: "/w".to_owned(),
        offset: Some(0),
        source: Some("startup".to_owned()),
        prompt_id: None,
        attended: false,
        ancestry: None,
        harness: None,
        at: 1,
    }
}

/// Holds, in `home`'s store, a `SessionStart` hook observation of
/// `session` that read its process ancestry, nested or not.
fn observe_start(home: &Home, session: &str, nested: bool) {
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let mut start = observation("SessionStart", session);
    start.ancestry = Some((1 + u64::from(nested), nested));
    store.hold_observation(&start, None).unwrap();
}

/// Holds a `UserPromptSubmit` hook observation of `session` for the
/// prompt `prompt_id` committing to `text`.
fn observe_prompt(home: &Home, session: &str, prompt_id: &str, text: &str, attended: bool) {
    let mut store = Store::open(&home.store()).unwrap();
    let mut prompt = observation("UserPromptSubmit", session);
    prompt.prompt_id = Some(prompt_id.to_owned());
    prompt.attended = attended;
    let key = record::commitment::CommitmentKey::from_bytes([9; 32]);
    let commitment = store::PromptCommitment {
        harness_session: session.to_owned(),
        prompt_id: prompt_id.to_owned(),
        commitment: record::commitment::commit(&key, &serde_json::json!(text)).unwrap(),
        key,
        used: false,
    };
    store.hold_observation(&prompt, Some(&commitment)).unwrap();
}

/// The provenance of each event of `writer`.
fn classes(home: &Home, writer: WriterId) -> Vec<String> {
    let store = Store::open(&home.store()).unwrap();
    store
        .log(writer)
        .unwrap()
        .iter()
        .map(|e| e.link.header.provenance.name())
        .collect()
}

#[test]
fn a_typed_line_is_user_only_once_its_ancestry_is_observed_and_not_nested() {
    for (observed, nested, class) in [
        (false, false, "harness_text"),
        (true, false, "user"),
        (true, true, "harness_text"),
    ] {
        let dir = TempDir::new().unwrap();
        let home = home(&dir);
        if observed {
            observe_start(&home, "s1", nested);
        }
        let path = dir.path().join("s1.jsonl");
        std::fs::write(&path, typed("hello", "p1", "typed")).unwrap();

        let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

        assert_eq!(
            classes(&home, ingested.writer),
            [class],
            "{observed} {nested}"
        );
    }
}

#[test]
fn hook_observations_and_prompt_matches_reach_the_device_seat_once() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    observe_start(&home, "s1", false);
    observe_prompt(&home, "s1", "p1", "hello", false);
    observe_prompt(&home, "s1", "p2", "other", true);
    let path = dir.path().join("s1.jsonl");
    let text = [
        typed("hello", "p1", "typed"),
        typed("hello again", "p1", "typed"),
        typed("mismatch", "p2", "typed"),
        typed("no commitment", "p3", "typed"),
    ]
    .concat();
    std::fs::write(&path, &text).unwrap();
    let inputs = Inputs {
        interactive: true,
        ..Inputs::default()
    };

    let ingested = ingest_with(&home, &path, &redactor(), &inputs, &mut OsEntropy).unwrap();

    assert_eq!(
        (
            ingested.observations,
            ingested.matched,
            ingested.prompt_conflicts,
            ingested.not_attended
        ),
        (3, 1, 1, 1)
    );
    let store = Store::open(&home.store()).unwrap();
    let node = administration::node::Node::find(&home, &store).unwrap();
    let device = store.log(node.device_writer()).unwrap();
    let results: Vec<(String, String)> = device
        .iter()
        .filter(|e| e.link.header.kind == EventKind::PromptMatch)
        .map(|e| {
            let c = e.content.as_ref().unwrap();
            (
                c["prompt_id"].as_str().unwrap().to_owned(),
                c["result"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        results,
        [
            ("p1".to_owned(), "matched".to_owned()),
            ("p1".to_owned(), "conflict".to_owned()),
            ("p2".to_owned(), "unmatched".to_owned())
        ]
    );
    let first = device
        .iter()
        .find(|e| e.link.header.kind == EventKind::PromptMatch)
        .unwrap();
    let user = store.log(ingested.writer).unwrap()[0].link.header.address;
    assert_eq!(first.content.as_ref().unwrap()["event"], user.to_string());
    let observed = device
        .iter()
        .filter(|e| e.link.header.kind == EventKind::HookObservation)
        .count();
    assert_eq!(observed, 3);
    assert!(
        store
            .hook_observations()
            .unwrap()
            .iter()
            .all(|h| h.appended)
    );
    verifies(&home);

    std::fs::write(&path, format!("{text}{}", typed("later", "p1", "typed"))).unwrap();
    let again = ingest_with(&home, &path, &redactor(), &inputs, &mut OsEntropy).unwrap();
    assert_eq!(
        (again.observations, again.matched, again.prompt_conflicts),
        (0, 0, 1)
    );
}

#[test]
fn a_subagent_s_transcript_leaves_its_sessions_observations_for_the_main_run() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    observe_start(&home, "s1", false);
    let sub = dir.path().join("s1").join("subagents");
    std::fs::create_dir_all(&sub).unwrap();
    let path = sub.join("agent-a1.jsonl");
    std::fs::write(&path, "{\"a\":1}\n").unwrap();

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(ingested.observations, 0);
    let store = Store::open(&home.store()).unwrap();
    assert!(
        store
            .hook_observations()
            .unwrap()
            .iter()
            .all(|h| !h.appended)
    );
}

#[test]
fn a_fork_observation_the_store_holds_names_a_harness_fork() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let mut fork = observation("SessionStart", "fork");
    fork.source = Some("fork".into());
    store.hold_observation(&fork, None).unwrap();
    drop(store);
    let parent = dir.path().join("parent.jsonl");
    let forked = dir.path().join("fork.jsonl");
    std::fs::write(&parent, numbered("parent", 1, 3)).unwrap();
    std::fs::write(&forked, numbered("parent", 1, 3)).unwrap();

    let first = ingest_path(&home, &forked, &redactor(), &mut OsEntropy).unwrap();

    // The fork's lines wait for their owner, though no input named it.
    assert!(first.held);
}

#[test]
fn only_a_position_another_ingest_moved_is_read_again() {
    let moved = || store::StoreError::PositionMoved { source: 1 };
    let partial = IngestError::Partial {
        writer: WriterId::derive(&[1; 32]),
        landed: 1,
        lines: 2,
        source: moved(),
    };

    assert!(IngestError::Store(moved()).position_moved());
    assert!(partial.position_moved());
    assert!(!IngestError::Empty(std::path::PathBuf::new()).position_moved());
}

/// The lines `from` to `to` of a transcript of numbered lines.
fn numbered_lines(from: u64, to: u64) -> String {
    (from..=to).map(|n| format!("{{\"n\":{n}}}\n")).collect()
}

#[test]
fn a_read_from_before_another_ingest_moved_the_position_is_refused_and_starts_no_generation() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, &numbered_lines(1, 6));
    // Another ingest read the whole source and moved the position to its
    // end; this one had read only the first three lines by then.
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    let stale = reading::ingest(
        &home,
        &path,
        &numbered_lines(1, 3),
        &redactor(),
        &Inputs::default(),
        &mut reading::Recorder::Cli,
        &mut OsEntropy,
    )
    .unwrap_err();

    assert!(stale.position_moved(), "{stale}");
    // The refused read left nothing behind: the next ingest finds the
    // source read to its end, in its one generation, each line once.
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    assert_eq!((again.events, again.new_generation), (0, false));
    let store = Store::open(&home.store()).unwrap();
    let generations = store.generations(&path.to_string_lossy()).unwrap();
    assert_eq!(generations.len(), 1);
    let lines: Vec<u64> = store
        .placed(generations[0].id)
        .unwrap()
        .iter()
        .map(|p| p.place.line)
        .collect();
    assert_eq!(lines, (1..=6).collect::<Vec<_>>());
}

#[test]
fn a_source_read_shorter_than_its_position_and_found_shorter_still_starts_a_new_generation() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, &numbered_lines(1, 6));
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    // The source was rewritten shorter: the read is no stale one, and
    // REC-07 starts a generation, whether or not the file reads again.
    std::fs::write(&path, numbered_lines(7, 8)).unwrap();
    let rewritten = reading::ingest(
        &home,
        &path,
        &numbered_lines(7, 8),
        &redactor(),
        &Inputs::default(),
        &mut reading::Recorder::Cli,
        &mut OsEntropy,
    )
    .unwrap();
    assert!(rewritten.new_generation);

    std::fs::remove_file(&path).unwrap();
    let gone = reading::ingest(
        &home,
        &path,
        &numbered_lines(9, 9),
        &redactor(),
        &Inputs::default(),
        &mut reading::Recorder::Cli,
        &mut OsEntropy,
    )
    .unwrap();
    assert!(gone.new_generation);
}

/// A writer id larger than `writer` that shares its slot in the search
/// index: the same first five bytes of its digest.
fn sharing_the_slot_of(writer: record::id::WriterId) -> record::id::WriterId {
    let mut bytes = *writer.digest().as_bytes();
    bytes[5..].fill(0xff);

    record::id::WriterId::parse(&record::digest::Digest::from_bytes(bytes).to_hex()).unwrap()
}

#[test]
fn an_ingest_reports_the_search_index_gaps_not_yet_reported_and_only_those() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "{\"a\":1}\n");
    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    assert!(first.index_gaps.is_empty());
    assert_eq!(first.index_gaps_unread, None);

    let other = sharing_the_slot_of(first.writer);
    assert!(first.writer < other);
    let mut store = Store::open(&home.store()).unwrap();
    store
        .add_seat(&store::StoredSeat {
            writer: other,
            seat: record::id::SeatId::derive(&first.room, other.digest().as_bytes()),
            room: first.room,
            certificate: serde_json::json!({}),
        })
        .unwrap();
    store
        .append(
            other,
            &[store::NewEvent {
                kind: EventKind::UnparsedLine,
                provenance: Provenance::Unparsed,
                content: serde_json::json!({"line": 1, "text": "left out"}),
                commitment_key: record::commitment::CommitmentKey::from_bytes([7; 32]),
                transcript_timestamp: None,
            }],
        )
        .unwrap();
    drop(store);
    let second = dir.path().join("b.jsonl");
    std::fs::write(&second, "{\"b\":2}\n").unwrap();
    let second = ingest_path(&home, &second, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(
        second.index_gaps,
        [store::IndexGap {
            writer: other,
            class: store::GapClass::WriterUnindexed
        }]
    );
    let store = Store::open(&home.store()).unwrap();
    store
        .mark_search_index_gaps_reported(&second.index_gaps)
        .unwrap();
    let third = dir.path().join("c.jsonl");
    std::fs::write(&third, "{\"c\":3}\n").unwrap();
    let third = ingest_path(&home, &third, &redactor(), &mut OsEntropy).unwrap();
    assert!(third.index_gaps.is_empty());
}

#[test]
fn a_drift_description_that_reaches_the_scan_limit_is_counted() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let key = format!("AIza{}", "a".repeat(35));
    let line = format!(r#"{{"type":"user","version":"{}"}}"#, key.repeat(5_000));
    let path = transcript(&dir, &format!("{line}\n"));
    let (redactor, _) = Redactor::new(&[]).unwrap();

    let ingested = ingest_path(&home, &path, &redactor, &mut OsEntropy).unwrap();

    assert_eq!(
        ingested.scan_limited, 2,
        "the line and its drift description"
    );
    assert!(
        ingested.drifts[0].contains("redaction_scan_limit"),
        "{:?}",
        ingested.drifts
    );
}
