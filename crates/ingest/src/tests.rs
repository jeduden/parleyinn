use super::*;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use std::ffi::OsStr;
use store::Store;
use testkit::TempDir;

fn home(dir: &TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

fn transcript(dir: &TempDir, text: &str) -> std::path::PathBuf {
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn every_line_becomes_one_unparsed_event_of_a_new_ingest_seat() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "{\"a\":1}\n{\"b\":2}\n");

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(ingested.events, 2);
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    assert_eq!(log.len(), 2);
    for (i, (stored, text)) in log.iter().zip(["{\"a\":1}", "{\"b\":2}"]).enumerate() {
        assert_eq!(stored.link.header.kind, EventKind::UnparsedLine);
        assert_eq!(stored.link.header.provenance, Provenance::Unparsed);
        assert_eq!(
            stored.content,
            Some(serde_json::json!({"line": i + 1, "text": text}))
        );
    }
    let seat = store.seat(ingested.writer).unwrap().unwrap();
    assert_eq!((seat.seat, seat.room), (ingested.seat, ingested.room));
    let device =
        key::signing::DeviceKey::fetch(home.key_store().as_ref(), &key::key_store::Entry::device())
            .unwrap();
    assert!(administration::verify::verify(&store, &device.public()).is_ok());
}

#[test]
fn each_transcript_s_run_has_its_own_seat_in_the_same_personal_room() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, "x\n");
    let other = dir.path().join("other.jsonl");
    std::fs::write(&other, "y\n").unwrap();

    let a = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let b = ingest_path(&home, &other, &redactor(), &mut OsEntropy).unwrap();

    // Ingesting the same transcript source again appends nothing, to the
    // same run's writer (REC-03).
    assert_eq!((again.writer, again.events), (a.writer, 0));
    assert_ne!(a.writer, b.writer);
    assert_eq!(a.room, b.room);
}

#[test]
fn an_unreadable_empty_or_non_utf8_transcript_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);

    let missing =
        ingest_path(&home, &dir.path().join("none"), &redactor(), &mut OsEntropy).unwrap_err();
    assert!(matches!(missing, IngestError::Read { .. }), "{missing}");
    assert!(
        missing.to_string().starts_with("read the transcript "),
        "{missing}"
    );
    assert!(std::error::Error::source(&missing).is_some());

    let empty = ingest_path(&home, &transcript(&dir, ""), &redactor(), &mut OsEntropy).unwrap_err();
    assert_eq!(
        empty.to_string(),
        format!(
            "the transcript {} holds no line",
            dir.path().join("t.jsonl").display()
        )
    );

    std::fs::write(dir.path().join("t.jsonl"), [0xff, b'\n']).unwrap();
    let bytes = ingest_path(
        &home,
        &dir.path().join("t.jsonl"),
        &redactor(),
        &mut OsEntropy,
    )
    .unwrap_err();
    assert_eq!(
        bytes.to_string(),
        format!(
            "the transcript {} is not UTF-8",
            dir.path().join("t.jsonl").display()
        )
    );
    assert!(std::error::Error::source(&bytes).is_none());

    assert!(!home.store().exists());
}

#[test]
fn a_home_that_cannot_be_made_is_returned() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("f"), "").unwrap();
    let home = Home::locate(Some(OsStr::new(dir.path().join("f").as_os_str())), None).unwrap();
    let path = transcript(&dir, "x\n");

    let err = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(err, IngestError::Admin(_)), "{err}");
    assert!(
        err.to_string()
            .starts_with(&dir.path().join("f").display().to_string()),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn a_store_that_cannot_open_or_a_random_source_that_fails_is_returned() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    home.create().unwrap();
    let path = transcript(&dir, "x\n");
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let other = dir.path().join("other.jsonl");
    std::fs::write(&other, "y\n").unwrap();

    // A new run's seat key needs the random source first.
    let err = ingest_path(&home, &other, &redactor(), &mut Failing).unwrap_err();
    assert!(matches!(err, IngestError::Admin(_)), "{err}");
    std::fs::write(&path, "x\nmore\n").unwrap();
    let err = ingest_path(&home, &path, &redactor(), &mut Failing).unwrap_err();
    assert!(matches!(err, IngestError::Key(_)), "{err}");
    assert_eq!(err.to_string(), "draw random bytes: none");
    assert!(std::error::Error::source(&err).is_some());

    std::fs::write(
        home.store(),
        "not a database at all, but long enough to read a header",
    )
    .unwrap();
    let err = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();
    assert!(matches!(err, IngestError::Store(_)), "{err}");
    assert!(err.to_string().starts_with("the store: "), "{err}");
    assert!(std::error::Error::source(&err).is_some());
}

/// A random source that always fails.
struct Failing;

impl key::random::Entropy for Failing {
    fn fill(&mut self, _: &mut [u8]) -> Result<(), key::KeyError> {
        Err(key::KeyError::Entropy("none".into()))
    }
}

#[test]
fn a_store_that_fails_part_way_names_the_writer_and_what_landed() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    ingest_path(&home, &transcript(&dir, "x\n"), &redactor(), &mut OsEntropy).unwrap();
    let conn = rusqlite::Connection::open(home.store()).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER refuse BEFORE INSERT ON events WHEN NEW.seq = 150 \
         BEGIN SELECT RAISE(ABORT, 'refused'); END",
    )
    .unwrap();
    let text: String = (1..=250).map(|n| format!("line {n}\n")).collect();
    let long = dir.path().join("long.jsonl");
    std::fs::write(&long, text).unwrap();

    let err = ingest_path(&home, &long, &redactor(), &mut OsEntropy).unwrap_err();

    let IngestError::Partial { writer, landed, .. } = &err else {
        panic!("{err}");
    };
    assert_eq!(*landed, 100);
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.log(*writer).unwrap().len(), 100);
    assert!(
        err.to_string().starts_with(&format!(
            "writer {writer} recorded 100 of 250 lines before the store failed: the store: "
        )),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn the_redacted_path_is_redacted_like_the_lines() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let named = dir.path().join("user@example.com");
    std::fs::create_dir_all(&named).unwrap();
    let path = named.join("ACME-12345678.jsonl");
    let context =
        r#"{"attachment":{"type":"session_context","context":{"userEmail":"user@example.com"}}}"#;
    std::fs::write(&path, format!("{context}\n")).unwrap();
    let (redactor, _) = Redactor::new(&["ACME-[0-9]{8}".to_owned()]).unwrap();

    let ingested = ingest_path(&home, &path, &redactor, &mut OsEntropy).unwrap();

    let shown = &ingested.redacted_path;
    assert!(shown.contains("[REDACTED:account_email]"), "{shown}");
    assert!(shown.contains("[REDACTED:extra_pattern_1]"), "{shown}");
}

#[test]
fn every_line_is_redacted_before_anything_is_written() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let context = r#"{"attachment":{"type":"session_context","context":{"userEmail":"is user@example.com."}},"type":"attachment"}"#;
    let org = r#"{"attachment":{"type":"credential_org","organizationUuid":"0f0e0d0c-0000-0000-0000-000000000001"}}"#;
    let tool = r#"{"message":{"content":"AKIAIOSFODNN7EXAMPLE ACME-12345678"},"toolUseResult":"AKIAIOSFODNN7EXAMPLE","from":"user@example.com","org":"0f0e0d0c-0000-0000-0000-000000000001"}"#;
    let path = transcript(&dir, &format!("{tool}\n{context}\n{org}\nclean\n"));
    let (redactor, _) = Redactor::new(&["ACME-[0-9]{8}".to_owned()]).unwrap();

    let ingested = ingest_path(&home, &path, &redactor, &mut OsEntropy).unwrap();

    assert_eq!(
        ingested.redactions,
        [
            "aws_access_key",
            "extra_pattern_1",
            "aws_access_key",
            "account_email",
            "organization_id",
            "account_email",
            "organization_id",
        ]
    );
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let contents: Vec<serde_json::Value> = log.iter().map(|e| e.content.clone().unwrap()).collect();
    let all = serde_json::to_string(&contents).unwrap();
    for secret in [
        "AKIAIOSFODNN7EXAMPLE",
        "ACME-12345678",
        "user@example.com",
        "0f0e0d0c-0000",
    ] {
        assert!(!all.contains(secret), "{secret} reached the store");
    }
    assert_eq!(contents[0]["redactions"], 5);
    assert_eq!(contents[3], serde_json::json!({"line": 4, "text": "clean"}));
    let bytes = std::fs::read(home.store()).unwrap();
    assert!(!bytes.windows(20).any(|w| w == b"AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn an_identifier_too_long_to_redact_records_nothing() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let email = format!("{}@example.com", "a".repeat(1 << 20));
    let line = serde_json::json!({"attachment": {"type": "session_context", "context": {"userEmail": email}}});
    let path = transcript(&dir, &format!("{line}\n"));

    let err = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(err, IngestError::Redaction(_)));
    assert!(err.to_string().starts_with("redact the transcript: "));
    assert!(std::error::Error::source(&err).is_some());
    assert!(!home.store().exists());
}

/// A recorded Claude Code 2.1.296 transcript: 57 lines, a Bash call and
/// its result among them.
const RECORDED: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl";

#[test]
fn each_line_is_recorded_with_the_class_its_structure_gives() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text = std::fs::read_to_string(testkit::repo_root().join(RECORDED)).unwrap();
    let path = transcript(&dir, &text);

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((ingested.events, ingested.unparsed), (57, 0));
    assert_eq!(ingested.drifts, Vec::<String>::new());
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let class = |line: usize| log[line - 1].link.header.provenance.name();
    assert!(
        log.iter()
            .all(|e| e.link.header.kind == EventKind::TranscriptLine)
    );
    assert_eq!(class(1), "harness_meta");
    assert_eq!(class(3), "harness_text");
    assert_eq!(class(6), "harness_text");
    // A prompt is no `user` event until a hook observation records the
    // harness session's process ancestry (PRV-08).
    assert_eq!(class(7), "harness_text");
    assert_eq!(class(20), "assistant");
    assert_eq!(class(21), "tool_call");
    assert_eq!(class(22), "tool_result:Bash");
    assert_eq!(class(57), "harness_meta");
    assert_eq!(
        log[21].content.as_ref().unwrap()["text"],
        text.lines().nth(21).unwrap()
    );
    let device =
        key::signing::DeviceKey::fetch(home.key_store().as_ref(), &key::key_store::Entry::device())
            .unwrap();
    assert!(administration::verify::verify(&store, &device.public()).is_ok());
}

#[test]
fn an_unparsed_or_drifting_line_is_kept_redacted_counted_and_described() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let key = "sk-ant-api03-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345";
    let lines = [
        format!("{{not json {key}"),
        format!(r#"{{"type":"future_kind","secret":"{key}"}}"#),
        format!(r#"{{"type":"assistant","version":"2.1.296","note":"{key}"}}"#),
    ];
    let path = transcript(&dir, &format!("{}\n", lines.join("\n")));

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((ingested.events, ingested.unparsed), (3, 2));
    assert_eq!(ingested.drifts.len(), 2);
    assert_eq!(
        ingested.drifts[0],
        "claude-code:unversioned:future_kind:unrecorded-type"
    );
    assert!(
        ingested.drifts[1].starts_with("claude-code:2.1.296:assistant:missing:"),
        "{:?}",
        ingested.drifts
    );
    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let headers: Vec<(EventKind, Provenance)> = log
        .iter()
        .map(|e| (e.link.header.kind, e.link.header.provenance.clone()))
        .collect();
    assert_eq!(
        headers,
        [
            (EventKind::UnparsedLine, Provenance::Unparsed),
            (EventKind::UnparsedLine, Provenance::Unparsed),
            (EventKind::TranscriptLine, Provenance::HarnessText),
        ]
    );
    for stored in &log {
        let text = stored.content.as_ref().unwrap()["text"].as_str().unwrap();
        assert!(!text.contains(key), "{text}");
        assert!(text.contains("[REDACTED:anthropic_api_key]"), "{text}");
    }
}

#[test]
fn a_drifts_description_passes_the_redactor() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let line = r#"{"type":"assistant","version":"AKIAIOSFODNN7EXAMPLE","message":{}}"#;
    let path = transcript(&dir, &format!("{line}\n"));

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(
        ingested.drifts,
        ["claude-code:_REDACTED_aws_access_key_:assistant:unsupported-version"]
    );
}

#[test]
fn the_transcripts_under_a_root_are_its_jsonl_files_in_path_order() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("projects");
    std::fs::create_dir_all(root.join("-w-a/s1/subagents")).unwrap();
    std::fs::create_dir_all(root.join("-w-b")).unwrap();
    for file in [
        "-w-b/s2.jsonl",
        "-w-a/s1.jsonl",
        "-w-a/s1/subagents/agent-1.jsonl",
        "-w-a/s1/subagents/agent-1.meta.json",
        "-w-a/notes.txt",
    ] {
        std::fs::write(root.join(file), "x\n").unwrap();
    }

    let found = transcripts(&root).unwrap();

    assert_eq!(
        found,
        // Path order compares component by component: `s1` before
        // `s1.jsonl`.
        [
            root.join("-w-a/s1/subagents/agent-1.jsonl"),
            root.join("-w-a/s1.jsonl"),
            root.join("-w-b/s2.jsonl"),
        ]
    );
    assert_eq!(
        transcripts(&dir.path().join("none")).unwrap(),
        Vec::<std::path::PathBuf>::new()
    );
    let file = root.join("-w-b/s2.jsonl");
    let err = transcripts(&file).unwrap_err();
    assert!(
        err.to_string()
            .starts_with(&format!("list the transcripts in {}: ", file.display())),
        "{err}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[cfg(unix)]
#[test]
fn the_transcripts_under_a_root_list_a_linked_transcript_but_descend_no_linked_directory() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("projects");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(dir.path().join("elsewhere")).unwrap();
    std::fs::write(dir.path().join("elsewhere/s.jsonl"), "x\n").unwrap();
    std::os::unix::fs::symlink(dir.path().join("elsewhere"), root.join("dir")).unwrap();
    std::os::unix::fs::symlink(dir.path().join("elsewhere/s.jsonl"), root.join("s.jsonl")).unwrap();

    // The linked transcript is listed as the link; ingest resolves it
    // within the allowed roots (SEC-18). The linked directory, which
    // could loop, is not descended.
    assert_eq!(transcripts(&root).unwrap(), [root.join("s.jsonl")]);
}

/// A writer's segments as (first, last, closed).
fn ranges(home: &Home, writer: WriterId) -> Vec<(u64, u64, bool)> {
    Store::open(&home.store())
        .unwrap()
        .segments(writer)
        .unwrap()
        .iter()
        .map(|s| (s.first.get(), s.last.get(), s.closed))
        .collect()
}

#[test]
fn a_segment_closes_at_each_turn_end_and_at_the_transcript_s_end() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let end = r#"{"type":"assistant","message":{"stop_reason":"end_turn"}}"#;
    let call = r#"{"type":"assistant","message":{"stop_reason":"tool_use"}}"#;
    let partial = r#"{"type":"assistant","message":{"stop_reason":null}}"#;
    let text = format!("a\n{partial}\n{call}\n{end}\nb\n{end}\nc\n");

    let ingested =
        ingest_path(&home, &transcript(&dir, &text), &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(
        ranges(&home, ingested.writer),
        [(1, 4, true), (5, 6, true), (7, 7, true)]
    );
}

#[test]
fn a_transcript_ending_at_a_turn_end_opens_no_empty_segment() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let end = r#"{"type":"assistant","message":{"stop_reason":"end_turn"}}"#;

    let ingested = ingest_path(
        &home,
        &transcript(&dir, &format!("a\n{end}\n")),
        &redactor(),
        &mut OsEntropy,
    )
    .unwrap();

    assert_eq!(ranges(&home, ingested.writer), [(1, 2, true)]);
}

#[test]
fn a_turn_end_is_the_model_reply_that_ends_a_turn() {
    for (line, ends) in [
        (
            r#"{"type":"assistant","message":{"stop_reason":"end_turn"}}"#,
            true,
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":"stop_sequence"}}"#,
            true,
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":"max_tokens"}}"#,
            true,
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":"tool_use"}}"#,
            false,
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":"pause_turn"}}"#,
            false,
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":null}}"#,
            false,
        ),
        (
            r#"{"type":"user","message":{"stop_reason":"end_turn"}}"#,
            false,
        ),
        ("not json", false),
    ] {
        let value = serde_json::from_str(line).unwrap_or(Value::Null);
        assert_eq!(turn_end(&value), ends, "{line}");
    }
}

#[test]
fn a_segment_closes_at_each_subagent_end_its_parent_records() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = testkit::repo_root().join(
        "testdata/claude-code/2.1.296/transcripts/projects/-w-sub/a7cb6eb6-32e6-4f88-9192-12e8e99b99fd.jsonl",
    );

    let ingested = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    let store = Store::open(&home.store()).unwrap();
    let log = store.log(ingested.writer).unwrap();
    let ends: Vec<u64> = log
        .iter()
        .filter(|s| s.link.header.provenance == Provenance::SubagentResult)
        .map(|s| s.link.header.address.seq.get())
        .collect();
    assert!(!ends.is_empty(), "the recording holds a subagent end");
    let closed: Vec<u64> = ranges(&home, ingested.writer)
        .iter()
        .map(|(_, last, closed)| {
            assert!(closed);
            *last
        })
        .collect();
    for end in ends {
        assert!(
            closed.contains(&end),
            "a segment closes at {end}: {closed:?}"
        );
    }
    assert_eq!(closed.last(), Some(&(log.len() as u64)));
}

#[test]
fn a_line_s_transcript_timestamp_is_kept_only_when_well_formed() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text = [
        r#"{"timestamp":"2026-10-17T12:00:01.123Z"}"#,
        r#"{"timestamp":"2026-10-17T11:59:59+02:00"}"#,
        r#"{"no":"timestamp"}"#,
        r#"{"timestamp":"AKIAIOSFODNN7EXAMPLE"}"#,
        r#"{"timestamp":12}"#,
        r#"{"message":{"timestamp":"2026-10-17T12:00:00Z"}}"#,
        "not json\n",
    ]
    .join("\n");

    let ingested =
        ingest_path(&home, &transcript(&dir, &text), &redactor(), &mut OsEntropy).unwrap();

    let stamps: Vec<Option<String>> = Store::open(&home.store())
        .unwrap()
        .log(ingested.writer)
        .unwrap()
        .into_iter()
        .map(|s| s.transcript_timestamp)
        .collect();
    assert_eq!(
        stamps,
        [
            Some("2026-10-17T12:00:01.123Z".to_owned()),
            Some("2026-10-17T11:59:59+02:00".to_owned()),
            None,
            None,
            None,
            None,
            None
        ]
    );
}

#[test]
fn a_well_formed_transcript_timestamp_is_rfc_3339() {
    for (text, ok) in [
        ("2026-10-17T12:00:01Z", true),
        ("2026-10-17T12:00:01.1Z", true),
        ("2026-10-17T12:00:01.123456789Z", true),
        ("2026-10-17T12:00:01-07:30", true),
        ("2026-10-17T12:00:01.1234567890Z", false),
        ("2026-10-17T12:00:01.Z", false),
        ("2026-10-17 12:00:01Z", false),
        ("2026-10-17T12:00:01", false),
        ("2026-1a-17T12:00:01Z", false),
        ("2026-10-17T12:00:01+0200", false),
        ("2026-10-17T12:00:01Zjunk", false),
        ("", false),
    ] {
        assert_eq!(well_formed(text), ok, "{text}");
    }
}

#[test]
fn a_line_above_the_payload_threshold_goes_to_the_payload_store() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text = format!("{}\nshort\n", "x".repeat(300));

    let ingested = ingest_with(
        &home,
        &transcript(&dir, &text),
        &redactor(),
        &Inputs {
            payload: PayloadSettings {
                payload_threshold: 200,
            },
            ..Inputs::default()
        },
        &mut OsEntropy,
    )
    .unwrap();

    let log = Store::open(&home.store())
        .unwrap()
        .log(ingested.writer)
        .unwrap();
    assert!(log[0].payload.is_some());
    assert_eq!(log[0].content.as_ref().unwrap()["text"], "x".repeat(300));
    assert!(log[1].payload.is_none());
    assert_eq!(
        PayloadSettings::default().payload_threshold,
        store::DEFAULT_PAYLOAD_THRESHOLD
    );
}

#[test]
fn every_event_s_commitment_key_is_a_fresh_256_bit_secret_kept_beside_its_content() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text: String = (1..=64).map(|n| format!("line {n}\n")).collect();

    let ingested =
        ingest_path(&home, &transcript(&dir, &text), &redactor(), &mut OsEntropy).unwrap();

    let log = Store::open(&home.store())
        .unwrap()
        .log(ingested.writer)
        .unwrap();
    let keys: std::collections::BTreeSet<[u8; 32]> = log
        .iter()
        .map(|s| *s.commitment_key.as_ref().unwrap().as_bytes())
        .collect();
    assert_eq!(keys.len(), 64, "no two events share a commitment key");
    for stored in &log {
        let key = stored.commitment_key.as_ref().unwrap();
        assert_eq!(key.as_bytes().len() * 8, 256);
        assert!(!stored.link.header.canonical().contains(&key.to_hex()));
    }
}

/// `n` numbered lines, each a JSON object of the harness session
/// `session` with its own `uuid`.
fn numbered(session: &str, from: u64, to: u64) -> String {
    (from..=to)
        .map(|n| {
            format!(
                "{}\n",
                serde_json::json!({"type": "x", "uuid": format!("u{n}"), "sessionId": session, "n": n})
            )
        })
        .collect()
}

/// The device key `home` keeps, which verifies its record.
fn verifies(home: &Home) {
    let store = Store::open(&home.store()).unwrap();
    let device =
        key::signing::DeviceKey::fetch(home.key_store().as_ref(), &key::key_store::Entry::device())
            .unwrap();
    administration::verify::verify(&store, &device.public()).unwrap();
}

#[test]
fn a_grown_transcript_source_is_ingested_only_past_its_position() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = transcript(&dir, &numbered("s", 1, 40));
    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(&numbered("s", 41, 50));
    std::fs::write(&path, &text).unwrap();

    let grown = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((first.events, grown.events, again.events), (40, 10, 0));
    assert_eq!(grown.writer, first.writer);
    assert!(!grown.new_generation && grown.generation == 1);
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.log(first.writer).unwrap().len(), 50);
    let source = store.source(&path.to_string_lossy()).unwrap().unwrap();
    assert_eq!(source.position.offset, text.len() as u64);
    assert_eq!(
        source.position.prefix,
        record::digest::Digest::of(&[text.as_bytes()])
    );
    assert_eq!(source.position.lines, 50);
    let placed = store.placed(source.id).unwrap();
    assert_eq!(placed[49].place.line, 50);
    assert_eq!(placed[49].place.uuid.as_deref(), Some("u50"));
    verifies(&home);
}

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
