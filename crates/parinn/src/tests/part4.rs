//! The parinn tests, part four.
//! The rest of the module's tests, moved from `tests.rs` to keep each file in its limit.

use super::*;
use configuration::source::OsUser;
use std::sync::mpsc;

#[test]
fn ingest_reads_within_the_configured_transcript_roots_only() {
    let p = crate::ingest_path::tests::off_terminal();
    let elsewhere = p.dir.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(elsewhere.join("t.jsonl"), "a\n").unwrap();
    p.configure(&format!(
        "[transcript]\nroots = [{:?}]\n",
        elsewhere.display().to_string()
    ));
    let (code, _, err) = p.invoke(
        &[
            "ingest",
            "--path",
            &elsewhere.join("t.jsonl").display().to_string(),
        ],
        &DEV,
    );
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));

    // Nothing in the home is a transcript: not its keys, not its store.
    let device_key = p.parinn_home().join("keys").join("device.key");
    let store = p.parinn_home().join("store.db");
    for path in [device_key, store] {
        let (code, _, err) = p.invoke(&["ingest", "--path", &path.display().to_string()], &DEV);
        assert_eq!(
            (code, err.as_str()),
            (EXIT_FAILURE, "parinn ingest: path outside allowed roots\n")
        );
    }
    // The default root no longer applies once the roots are configured.
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_FAILURE);
}

#[test]
fn ingest_says_when_it_cannot_audit_its_refusal() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.configure("recal.max_k = 10\n");
    let audit_dir = p.parinn_home().join("audit");
    let _ = std::fs::remove_dir_all(&audit_dir);
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("parinn ingest: audit a refused ingest: "),
        "{err}"
    );
}

#[test]
fn ingest_says_when_it_cannot_audit_a_path_outside_the_roots() {
    let p = crate::ingest_path::tests::off_terminal();
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK);
    let audit_dir = p.parinn_home().join("audit");
    let _ = std::fs::remove_dir_all(&audit_dir);
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, _, err) = p.invoke(&["ingest", "--path", "/etc/passwd"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("parinn ingest: audit a path outside allowed roots: "),
        "{err}"
    );
    assert!(
        err.ends_with("parinn ingest: path outside allowed roots\n"),
        "{err}"
    );
}

/// Every audit entry the home at `p` holds, as text.
pub(crate) fn audit_text(p: &Process) -> String {
    let dir = p.parinn_home().join("audit");
    let mut text = String::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return text;
    };
    for entry in entries {
        text.push_str(&std::fs::read_to_string(entry.unwrap().path()).unwrap());
    }
    text
}

#[test]
fn ingest_audits_and_counts_a_node_identity_change_before_it_appends() {
    let mut p = Process::new();
    let id = p.dir.path().join("machine-id");
    std::fs::write(&id, "first").unwrap();
    p.node_identity = Some(id.clone());
    assert_eq!(
        p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV)
            .0,
        EXIT_OK
    );
    std::fs::write(&id, "second").unwrap();

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let (_, out, _) = p.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["counters"]["node_identity_changed"]["total"], 1);
    assert!(
        audit_text(&p)
            .contains("node identity changed: a new device key and device seat replace the old")
    );
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let kinds: Vec<&str> = v["seats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["seat_kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds.iter().filter(|k| **k == "device").count(), 2);
    assert_eq!(kinds.iter().filter(|k| **k == "run").count(), 2);
}

#[test]
fn ingest_refuses_a_missing_or_unreadable_device_key_audited_and_counted() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let key = p.parinn_home().join("keys/device.key");
    std::fs::remove_file(&key).unwrap();

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("no second genesis starts"), "{err}");
    assert!(audit_text(&p).contains("device key missing from the key store"));

    std::fs::create_dir(&key).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("c\n")], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(
        audit_text(&p).contains("key store locked or unreadable"),
        "{err}"
    );
}

#[test]
fn a_principal_act_audits_a_node_identity_change_and_counts_no_act_from_before_it() {
    let mut p = Process::new();
    let id = p.dir.path().join("machine-id");
    std::fs::write(&id, "first").unwrap();
    p.node_identity = Some(id.clone());
    p.invoke(&["counter", "ack"], &DEV);
    std::fs::write(&id, "second").unwrap();

    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(audit_text(&p).contains("node identity changed"));
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![id]));
    let store = Store::open(&home.store()).unwrap();
    let node = Node::find(&home, &store).unwrap();
    assert_eq!(act_record::acts(&store, &node).unwrap().len(), 1);
}

#[test]
fn ingest_fails_on_a_store_the_node_cannot_be_found_in_and_audits_no_key_failure() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("UPDATE seats SET certificate = '{'", [])
        .unwrap();
    let before = audit_text(&p);

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE, "{err}");
    let after = audit_text(&p);
    assert!(!after.contains("key store"), "{after}");
    assert!(!after.contains("device key missing"), "{after}");
    assert!(after.len() >= before.len());
}

/// Runs `statement` on the store in `p`'s home.
fn store_sql(p: &Process, statement: &str) {
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute_batch(statement)
        .unwrap();
}

/// Appends two events to a writer of the store in `p`'s home, leaving
/// its segment open, as a writer that stopped before closing it would;
/// its writer.
fn writer_left_open(p: &Process) -> store::WriterId {
    let writer = store::WriterId::derive(&[7; 32]);
    let new = |n: u8| store::NewEvent {
        kind: record::event::EventKind::UnparsedLine,
        provenance: record::event::Provenance::Unparsed,
        content: serde_json::json!({"line": n}),
        commitment_key: store::CommitmentKey::from_bytes([n; 32]),
        transcript_timestamp: None,
    };
    Store::open(&p.parinn_home().join("store.db"))
        .unwrap()
        .append_open(writer, &[new(1), new(2)])
        .unwrap();
    writer
}

#[test]
fn status_reports_each_writer_chain_ended_without_a_closed_segment() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    let writer = writer_left_open(&p);

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(
        out.contains(&format!(
            "\nwriter chains ended without a closed segment: {writer} (seq 1 to 2)\n"
        )),
        "{out}"
    );
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["segments"],
        serde_json::json!({
            "open": [{"writer": writer.to_string(), "first": 1, "last": 2}],
            "newer": [],
        })
    );
}

#[test]
fn verify_reports_each_writer_chain_ended_without_a_closed_segment_and_exits_0() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["open_segments"], serde_json::json!([]));
    let writer = writer_left_open(&p);
    // The writer has no seat certificate: verify names that first, so
    // give the open segment to the ingest seat's writer instead.
    store_sql(
        &p,
        &format!(
            "DELETE FROM segments WHERE writer = '{writer}'; DELETE FROM events WHERE writer = '{writer}'"
        ),
    );
    let ingest = Store::open(&p.parinn_home().join("store.db"))
        .unwrap()
        .open_segments()
        .unwrap();
    assert!(ingest.is_empty());
    store_sql(
        &p,
        "UPDATE segments SET closed = 0 WHERE (writer, first_seq) IN \
         (SELECT writer, max(first_seq) FROM segments WHERE writer IN \
          (SELECT writer FROM events WHERE kind = 'transcript_line') GROUP BY writer)",
    );

    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let text = p.invoke(&["verify"], &DEV).1;

    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let open = v["open_segments"].as_array().unwrap();
    assert_eq!(open.len(), 1, "{v}");
    assert_eq!(
        (&open[0]["first"], &open[0]["last"]),
        (&serde_json::json!(1), &serde_json::json!(1))
    );
    assert!(
        text.contains(&format!(
            "; writer chains ended without a closed segment: {} (seq 1 to 1)",
            open[0]["writer"].as_str().unwrap()
        )),
        "{text}"
    );
}

#[test]
fn status_and_verify_exit_3_naming_a_newer_store_and_leave_it_unchanged() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    store_sql(
        &p,
        "PRAGMA journal_mode = DELETE; UPDATE meta SET value = '2' WHERE key = 'schema_version'",
    );
    let path = p.parinn_home().join("store.db");
    let before = std::fs::read(&path).unwrap();

    for command in ["status", "verify"] {
        let (code, out, err) = p.invoke(&[command], &DEV);
        assert_eq!(
            (code, out.as_str(), err.as_str()),
            (
                EXIT_INTEGRITY,
                "",
                format!(
                    "parinn {command}: the store is schema version 2, newer than this parinn's 1\n"
                )
                .as_str()
            )
        );
    }
    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("the store is schema version 2"), "{err}");

    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn status_and_verify_exit_3_naming_a_newer_segment() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    store_sql(
        &p,
        "UPDATE segments SET format_version = 2 WHERE writer IN \
         (SELECT writer FROM events WHERE kind = 'transcript_line')",
    );
    let newer = Store::open(&p.parinn_home().join("store.db"))
        .unwrap()
        .newer_segments()
        .unwrap();
    assert_eq!(newer.len(), 1);
    let named = store::StoreError::NewerSegment(newer[0].clone()).to_string();

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_INTEGRITY, ""));
    assert!(out.contains(&format!("\nrefused: {named}\n")), "{out}");
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["segments"]["newer"][0]["format_version"], 2);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(
        (code, err),
        (
            EXIT_INTEGRITY,
            format!("parinn verify: hash chain: {named}\n")
        )
    );
}

#[test]
fn status_of_a_store_it_cannot_read_fails() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home());
    Home::locate(Some(p.parinn_home().as_os_str()), None)
        .unwrap()
        .create()
        .unwrap();
    junk_store(&p);

    let (code, _, err) = p.invoke(&["status"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn status: the store: "), "{err}");
}

#[test]
fn ingest_puts_a_line_above_the_configured_payload_threshold_in_the_payload_store() {
    let p = Process::new();
    p.configure("payload.threshold_bytes = 100\n");
    let long = format!(
        "{{\"type\":\"atis-latch\",\"atis\":\"{}\",\"sessionId\":\"s\"}}\n",
        "a".repeat(200)
    );

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript(&long)], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let held = std::fs::read_dir(p.parinn_home().join("payloads"))
        .unwrap()
        .any(|e| {
            std::fs::read_to_string(e.unwrap().path())
                .unwrap()
                .contains(&"a".repeat(200))
        });
    assert!(held, "the line's content is in the payload store");
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn verify_of_a_home_whose_first_start_was_cut_short_before_it_recorded_exits_0() {
    use key::key_store::Entry;
    use key::signing::{DeviceKey, RecoveryKey, SeatKey};
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    home.create().unwrap();
    drop(Store::open(&home.store()).unwrap());
    // A first start killed after it staged its keys, before it recorded.
    let keys = home.key_store();
    DeviceKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(keys.as_ref(), &Entry::staged(&Entry::device()))
        .unwrap();
    RecoveryKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(keys.as_ref(), &Entry::staged(&Entry::recovery()))
        .unwrap();
    SeatKey::mint(&mut OsEntropy)
        .unwrap()
        .keep(keys.as_ref(), &Entry::staged_seat())
        .unwrap();
    key::file::write_secret(&home.staged(), &format!("genesis {}", "0".repeat(32))).unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(out.starts_with("checked 0 writers and 0 events: "), "{out}");
    assert!(
        out.ends_with("; the node's first start was interrupted, and its next start finishes it\n"),
        "{out}"
    );
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        (&v["writers"], &v["staged"]),
        (&serde_json::json!(0), &serde_json::json!(true))
    );

    // Its next start finishes it, and the record verifies against the
    // key it placed.
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    assert_eq!(code, EXIT_OK);
    let (_, out, _) = p.invoke(&["verify", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["staged"], serde_json::json!(false));
}

#[test]
fn a_search_the_deadline_cancels_is_told_in_fixed_text_and_audited() {
    let mut p = Process::new();
    p.invoke(
        &[
            "ingest",
            "--path",
            &p.transcript(&"alpha pad\n".repeat(2100)),
        ],
        &DEV,
    );
    p.query_deadline = Some(Duration::ZERO);
    p.lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.into(),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"event_search","arguments":{"query":"alpha","scope":"room"}}}"#.into(),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let reply: serde_json::Value = serde_json::from_str(out.lines().nth(1).unwrap()).unwrap();
    assert_eq!(reply["result"]["isError"], true);
    let cancelled: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("query_cancelled"))
        .collect();
    assert_eq!(cancelled.len(), 1);
    assert_eq!(
        (
            cancelled[0].operation.as_str(),
            cancelled[0].detail.as_str()
        ),
        ("mcp", "query cancelled at its deadline")
    );
}

#[test]
fn a_cancelled_search_the_audit_log_cannot_take_is_said_on_stderr() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_query_cancelled(&home, &|| NOW, &mut err)(2);

    let err = String::from_utf8(err).unwrap();
    assert_eq!(
        err.matches("parinn mcp: audit a cancelled search: ")
            .count(),
        2,
        "{err}"
    );
}

#[test]
fn no_cancelled_search_is_audited_when_none_was_cancelled() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_query_cancelled(&home, &|| NOW, &mut err)(0);

    assert!(err.is_empty());
    assert!(!p.parinn_home().join("audit").exists());
}

#[test]
fn two_closures_share_one_writer_through_shared() {
    let mut sink: Vec<u8> = Vec::new();
    {
        let cell = RefCell::new(&mut sink as &mut dyn Write);
        let (mut a, mut b) = (Shared(&cell), Shared(&cell));

        a.write_all(b"one ").unwrap();
        b.write_all(b"two").unwrap();
        a.flush().unwrap();
    }

    assert_eq!(sink, b"one two");
}

#[test]
fn ingest_audits_each_gap_of_the_search_index_once_and_status_counts_it() {
    let p = Process::new();
    let path = p.transcript("a\n");
    let (_, out, _) = p.invoke(&["ingest", &format!("--path={path}"), "--json"], &DEV);
    let first: serde_json::Value = serde_json::from_str(&out).unwrap();
    let small = record::id::WriterId::parse(first["writer"].as_str().unwrap()).unwrap();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let large = index_gaps::tests::leave_out_a_writer_sharing_the_slot_of(&home, small);

    for name in ["b.jsonl", "c.jsonl"] {
        let other = transcript_in(&p, "p", name, b"b\n");
        let (code, _, err) = p.invoke(&["ingest", "--path", &other.display().to_string()], &DEV);
        assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    }

    let gaps: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("search_index_writer_unindexed"))
        .collect();
    assert_eq!(gaps.len(), 1, "once, not per ingest");
    assert_eq!(gaps[0].kind, "dropped");
    assert_eq!(gaps[0].operation, "ingest");
    assert!(gaps[0].detail.ends_with(&large.to_string()));
    let (_, status, _) = p.invoke(&["status"], &DEV);
    assert!(
        status.contains("search_index_writer_unindexed 1 (1 since acknowledged)"),
        "{status}"
    );
}

#[test]
fn ingest_says_which_search_index_gap_it_cannot_audit_and_reports_it_at_the_next_ingest() {
    let p = Process::new();
    let path = p.transcript("a\n");
    let (_, out, _) = p.invoke(&["ingest", &format!("--path={path}"), "--json"], &DEV);
    let first: serde_json::Value = serde_json::from_str(&out).unwrap();
    let small = record::id::WriterId::parse(first["writer"].as_str().unwrap()).unwrap();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    index_gaps::tests::leave_out_a_writer_sharing_the_slot_of(&home, small);
    let audit_dir = p.parinn_home().join("audit");
    let kept = std::fs::read_dir(&audit_dir).unwrap().count();
    assert!(kept > 0);
    std::fs::rename(&audit_dir, p.parinn_home().join("audit-kept")).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let other = transcript_in(&p, "p", "b.jsonl", b"b\n");
    let (code, _, err) = p.invoke(&["ingest", "--path", &other.display().to_string()], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(
        err.contains(
            "parinn ingest: audit the search index gaps: the audit log refused 1 entries; \
             the next ingest tries again"
        ),
        "{err}"
    );
    std::fs::remove_file(&audit_dir).unwrap();
    std::fs::rename(p.parinn_home().join("audit-kept"), &audit_dir).unwrap();
    let again = transcript_in(&p, "p", "c.jsonl", b"c\n");
    p.invoke(&["ingest", "--path", &again.display().to_string()], &DEV);
    let gaps = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("search_index_writer_unindexed"))
        .count();
    assert_eq!(gaps, 1);
}

#[test]
fn ingest_counts_and_says_a_lookup_of_the_search_index_gaps_that_failed() {
    let p = Process::new();
    let path = p.transcript("a\n");
    p.invoke(&["ingest", &format!("--path={path}")], &DEV);
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut ingested = ingest::ingest_path(
        &home,
        std::path::Path::new(&path),
        &Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap();
    ingested.index_gaps_unread = Some("the store has no search index".to_owned());
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
        home_id: None,
        hook_panics: false,
        hook_hangs: false,
        terminal: true,
        working_dir: None,
        managed_policy: std::path::PathBuf::from("/nonexistent/managed_policy.toml"),
        node_identity: None,
        os_user: OsUser::Known(u32::MAX),
        input: &input,
        clock: &|| NOW,
        attended: false,
        hook_budget: Some(LENIENT),
        ancestry: own_harness,
        live_limit: None,
        query_deadline: None,
    };
    let mut err = Vec::new();

    audit_index_gaps(&ctx, &ingested, &mut err);

    assert_eq!(
        String::from_utf8(err).unwrap(),
        "parinn ingest: look up the search index gaps: the store has no search index\n"
    );
    let counted: Vec<_> = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("search_index_gap_lookup_failed"))
        .collect();
    assert_eq!(counted.len(), 1);
    assert_eq!(
        (counted[0].kind.as_str(), counted[0].operation.as_str()),
        ("failed", "ingest")
    );
}

#[test]
fn ingest_says_a_search_index_gap_it_cannot_audit_when_the_home_cannot_be_found() {
    let p = Process::new();
    let path = p.transcript("a\n");
    p.invoke(&["ingest", &format!("--path={path}")], &DEV);
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut ingested = ingest::ingest_path(
        &home,
        std::path::Path::new(&path),
        &Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap();
    ingested.index_gaps = vec![store::IndexGap {
        writer: record::id::WriterId::derive(&[7; 32]),
        class: store::GapClass::WriterUnindexed,
    }];
    let input = || mpsc::channel().1;
    let ctx = Context {
        build: &DEV,
        parinn_home: None,
        home: None,
        home_id: None,
        hook_panics: false,
        hook_hangs: false,
        terminal: true,
        working_dir: None,
        managed_policy: std::path::PathBuf::from("/nonexistent/managed_policy.toml"),
        node_identity: None,
        os_user: OsUser::Known(u32::MAX),
        input: &input,
        clock: &|| NOW,
        attended: false,
        hook_budget: Some(LENIENT),
        ancestry: own_harness,
        live_limit: None,
        query_deadline: None,
    };
    let mut err = Vec::new();

    audit_index_gaps(&ctx, &ingested, &mut err);

    let said = String::from_utf8(err).unwrap();
    assert!(
        said.starts_with("parinn ingest: audit the search index gaps: "),
        "{said}"
    );
}

/// The two lines `parinn status` ends its facts with on this OS, each
/// followed by a newline.
pub(super) fn identity_lines() -> String {
    let against = match identity_against() {
        Some(against) => format!("checked ({against})"),
        None => "not checked on this OS".to_owned(),
    };

    format!(
        "node identity: {against}\nfork detection: a second event at one (writer, seq) is refused for the segments the node holds, found by a local scan with parinn verify; the equivocated mark waits for seals; the check between peers arrives with the peer network\n"
    )
}

/// What the node identity is checked against on this OS.
fn identity_against() -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        Some("Linux machine id")
    } else if cfg!(target_os = "macos") {
        Some("macOS hardware UUID")
    } else if cfg!(windows) {
        Some("Windows MachineGuid")
    } else {
        None
    }
}

/// `parinn status --json`'s node identity on this OS.
pub(super) fn identity_json() -> serde_json::Value {
    let against = identity_against();

    serde_json::json!({"checked": against.is_some(), "against": against})
}

/// `parinn status --json`'s fork detection.
pub(super) fn fork_detection_json() -> serde_json::Value {
    serde_json::json!({
        "local": "parinn verify",
        "refused": "a second event at one (writer, seq), for the segments the node holds",
        "equivocated": "the mark waits for seals",
        "peer": "the check between peers arrives with the peer network",
    })
}

/// How many events the store holds.
fn event_count(p: &Process) -> i64 {
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.query_row("SELECT count(*) FROM events", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn rebuild_derives_the_origins_again_and_appends_no_event() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", CLEAN_LINE.as_bytes());
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    let before = event_count(&p);

    let (code, out, err) = p.invoke(&["rebuild", "--json"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["origins"], 2);
    assert_eq!(v["worktrees"], 0);
    assert_eq!(v["indexed"], before);
    assert_eq!(event_count(&p), before);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
    let (_, text, _) = p.invoke(&["rebuild"], &DEV);
    assert!(
        text.starts_with("rebuilt: 2 origins, 0 worktrees"),
        "{text}"
    );
}

#[test]
fn a_kept_worktree_identity_the_record_lacks_fails_verify_until_rebuild_derives_it_again() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", CLEAN_LINE.as_bytes());
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "INSERT INTO repository_worktrees (worktree, identity) VALUES ('stray', 'x')",
        [],
    )
    .unwrap();
    drop(conn);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert!(
        err.contains("the identity kept for worktree stray"),
        "{err}"
    );

    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn a_kept_origin_changed_in_place_fails_verify_until_rebuild_derives_it_again() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "s1.jsonl", CLEAN_LINE.as_bytes());
    assert_eq!(p.invoke(&["ingest", "--all"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("UPDATE origins SET origin = 'peer'", [])
        .unwrap();
    drop(conn);

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert!(err.contains("derived artifact consistency"), "{err}");
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert!(out.contains("derived_artifact_consistency"), "{out}");

    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["verify"], &DEV).0, EXIT_OK);
}

#[test]
fn rebuild_without_a_store_fails_and_an_unknown_flag_is_usage() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["rebuild"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("no store at "), "{err}");
    let (code, _, err) = p.invoke(&["rebuild", "--now"], &DEV);
    assert_eq!(code, EXIT_USAGE);
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn rebuild_refuses_a_home_it_cannot_open_a_newer_store_and_a_store_with_no_node() {
    let mut p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let db = p.parinn_home().join("store.db");

    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch("DROP TABLE seats").unwrap();
    drop(conn);
    let (code, _, err) = p.invoke(&["rebuild"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn rebuild: "), "{err}");

    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute(
        "UPDATE meta SET value = '99' WHERE key = 'schema_version'",
        [],
    )
    .unwrap();
    drop(conn);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_INTEGRITY);

    p.home_id = Some("home-b".into());
    let binding = p.parinn_home().join("home.json");
    std::fs::write(&binding, r#"{"id":"home-a"}"#).unwrap();
    set_mode(&binding, 0o600);
    assert_eq!(p.invoke(&["rebuild"], &DEV).0, EXIT_FAILURE);
}

#[test]
fn status_says_whether_an_mcp_server_took_the_session_bindings() {
    assert_eq!(
        binding_line(0, 2),
        "MCP server: none has taken a session binding on this node (2 untaken)"
    );
    assert_eq!(binding_line(3, 0), "MCP server: took 3 session bindings");
    assert_eq!(
        binding_line(3, 1),
        "MCP server: took 3 session bindings, 1 untaken"
    );
}

#[test]
fn status_reports_the_session_bindings_in_text_and_json_once_any_was_observed() {
    let mut p = Process::new();
    let (_, none, _) = p.invoke(&["status", "--json"], &DEV);
    let none: serde_json::Value = serde_json::from_str(&none).unwrap();
    assert!(none.get("mcp_server").is_none());

    let app = p.dir.path().join("app");
    std::fs::create_dir_all(&app).unwrap();
    p.lines = vec![
        serde_json::json!({
            "session_id": "s1",
            "transcript_path": p.transcript("a\n"),
            "cwd": app,
            "hook_event_name": "SessionStart",
            "source": "startup",
        })
        .to_string(),
    ];
    assert_eq!(p.invoke(&["hook", "SessionStart"], &DEV).0, EXIT_OK);
    p.lines = Vec::new();

    let (_, json, _) = p.invoke(&["status", "--json"], &DEV);
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    let (_, text, _) = p.invoke(&["status"], &DEV);

    assert_eq!(json["mcp_server"]["bindings_untaken"], 1);
    assert_eq!(json["mcp_server"]["bindings_taken"], 0);
    assert!(
        text.contains("MCP server: none has taken a session binding on this node (1 untaken)"),
        "{text}"
    );
}

#[test]
fn limited_log_fields_are_audited_one_entry_each() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let before = audited(&p).len();
    let ctx = crate::settings::tests::ctx_of(&p);

    audit_limited_log_fields(&ctx, 2);

    let entries = audited(&p);
    assert_eq!(entries.len(), before + 2);
    assert_eq!(entries[before].kind, "dropped");
    assert_eq!(
        entries[before].counter.as_deref(),
        Some("redaction_scan_limited")
    );
    assert_eq!(entries[before].detail, crate::SCAN_LIMITED_DETAIL);
}
