//! The parinn tests, part two.
//! The rest of the module's tests, moved from `tests.rs` to keep each file in its limit.

use super::part3::{log_with, logged_lines};
use super::*;
use configuration::source::OsUser;
use std::sync::mpsc;

#[test]
fn ingest_all_goes_on_past_a_transcript_it_cannot_ingest_and_fails() {
    let p = Process::new();
    transcript_in(&p, "-w-a", "bad.jsonl", &[0xff, b'\n']);
    transcript_in(&p, "-w-b", "good.jsonl", b"a\n");

    let (code, out, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(out.starts_with("ingested 1 lines into writer "), "{out}");
    assert!(err.contains("is not UTF-8"), "{err}");
}

#[cfg(unix)]
#[test]
fn ingest_all_refuses_a_linked_transcript_outside_the_roots_and_goes_on() {
    let p = Process::new();
    let outside = p.dir.path().join("outside.jsonl");
    std::fs::write(&outside, "a\n").unwrap();
    let dir = p.dir.path().join(".claude").join("projects").join("-w-a");
    std::fs::create_dir_all(&dir).unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("linked.jsonl")).unwrap();

    let (code, out, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(err.contains("parinn ingest: "), "{err}");
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert_eq!(counters, [Some("path_outside_allowed_roots".to_owned())]);
}

#[test]
fn ingest_all_reports_a_transcript_root_it_cannot_list() {
    let p = Process::new();
    p.configure(&format!(
        "[transcript]\nroots = [{:?}]\n",
        p.dir.path().join("file").display().to_string()
    ));
    std::fs::write(p.dir.path().join("file"), "").unwrap();

    let (code, _, err) = p.invoke(&["ingest", "--all"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn ingest: list the transcripts in "),
        "{err}"
    );
}

#[test]
fn verify_checks_the_record_and_counts_it() {
    let p = Process::new();
    let path = p.transcript(&CLEAN_LINE.repeat(2));
    p.invoke(&["ingest", "--path", &path], &DEV);

    let text = p.invoke(&["verify"], &DEV);
    let json = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!(
        text,
        (
            EXIT_OK,
            "checked 2 writers and 6 events: every chain, commitment and seat certificate holds; \
             integrity unsigned, as no log is signed yet, so a log cut at its tail or rewritten \
             whole goes unseen until seals; completeness covers only the transcript sources the \
             record names; the audit log's chain holds over 0 entries\n"
                .into(),
            String::new()
        )
    );
    assert_eq!(json.0, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&json.1).unwrap();
    assert_eq!(
        (&v["integrity"], &v["writers"], &v["events"], &v["broken"]),
        (
            &serde_json::json!("unsigned"),
            &serde_json::json!(2),
            &serde_json::json!(6),
            &serde_json::Value::Null
        )
    );
    assert_eq!(v["audit"], serde_json::json!({"files": 0, "entries": 0}));
    let order: Vec<&str> = v["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert_eq!(order.len(), 6);
    assert!(
        order.iter().all(|a| a.starts_with("parinn:room/")),
        "{order:?}"
    );
}

#[test]
fn verify_exits_3_naming_where_the_chain_breaks() {
    let p = Process::new();
    let path = p.transcript("a\nb\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE events SET provenance = 'user' WHERE kind = 'unparsed_line' AND seq = 2",
        [],
    )
    .unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    let (json_code, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_INTEGRITY, ""));
    assert!(
        err.starts_with("parinn verify: hash chain: chain broken at parinn:room/"),
        "{err}"
    );
    assert!(
        err.ends_with("/2: the stored hash does not match the event's header\n"),
        "{err}"
    );
    assert_eq!(json_code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        format!(
            "parinn verify: {}: {}\n",
            v["broken"]["check"].as_str().unwrap(),
            v["broken"]["message"].as_str().unwrap()
        ),
        err
    );
    assert_eq!(v["broken"]["check"], "hash chain");
    assert_eq!(
        (&v["integrity"], &v["broken"]["class"], &v["order"]),
        (
            &serde_json::json!("broken"),
            &serde_json::json!("chain"),
            &serde_json::Value::Null
        )
    );
    assert!(
        err.contains(v["broken"]["address"].as_str().unwrap()),
        "{v}"
    );
}

/// Brings a second history of the writer of `p`'s transcript into its
/// home: a copy of that writer's log that disagrees from seq 2 on.
/// Returns the writer and what the refusal said.
fn refuse_a_disagreeing_copy(p: &Process) -> (String, crate::forks::Refused) {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();
    let writer = store
        .writers()
        .unwrap()
        .into_iter()
        .max_by_key(|w| store.log(*w).unwrap().len())
        .unwrap();
    let mut copy: Vec<record::chain::Link> = store
        .log(writer)
        .unwrap()
        .into_iter()
        .map(|s| s.link)
        .collect();
    for link in copy.iter_mut().skip(1) {
        link.hash = record::digest::Digest::of(&[b"another history"]);
    }
    drop(store);
    let audit = |o: &audit::entry::Occurrence| ctx_audit(p, o);
    let refused = crate::forks::refuse(&home, &copy, "restore", &mut { audit }).unwrap();

    (writer.to_string(), refused)
}

fn ctx_audit(p: &Process, o: &audit::entry::Occurrence) -> bool {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    Log::new(home.audit()).append(o, p.now).is_ok()
}

#[test]
fn verify_exits_3_reporting_a_refused_fork_by_writer_range_and_digests() {
    let p = Process::new();
    let path = p.transcript("a\nb\nc\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let before = p.invoke(&["verify"], &DEV);
    assert_eq!(before.0, EXIT_OK);
    let (writer, refused) = refuse_a_disagreeing_copy(&p);
    let fork = refused.forks[0];

    let (code, out, err) = p.invoke(&["verify"], &DEV);
    let (json_code, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_INTEGRITY, ""));
    assert_eq!(
        err,
        format!(
            "parinn verify: refused: writer {writer} holds two events at seq 2 to {}: {} held, {} offered\n",
            fork.last, fork.held, fork.offered
        )
    );
    assert!(!err.contains("fork choose"), "{err}");
    assert_eq!(json_code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["integrity"], "refused");
    assert_eq!(v["broken"], serde_json::Value::Null);
    assert_eq!(
        v["refused"],
        serde_json::json!([{
            "writer": writer,
            "first": 2,
            "last": fork.last.get(),
            "held": fork.held.to_hex(),
            "offered": fork.offered.to_hex(),
        }])
    );
}

#[test]
fn a_refused_fork_is_audited_counted_and_leaves_the_record_as_it_was() {
    let p = Process::new();
    let path = p.transcript("a\nb\nc\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    let digest = |p: &Process| {
        let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
        let store = Store::open(&home.store()).unwrap();
        store
            .writers()
            .unwrap()
            .into_iter()
            .map(|w| store.log(w).unwrap())
            .collect::<Vec<_>>()
    };
    let held = digest(&p);

    refuse_a_disagreeing_copy(&p);

    assert_eq!(digest(&p), held);
    let audit = audit_text(&p);
    assert_eq!(
        audit.matches("\"counter\":\"fork_conflict\"").count(),
        1,
        "{audit}"
    );
    let (_, status, _) = p.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(v["counters"]["fork_conflict"]["total"], 1);
}

#[test]
fn verify_names_a_broken_chain_before_a_refused_fork() {
    let p = Process::new();
    let path = p.transcript("a\nb\nc\n");
    p.invoke(&["ingest", "--path", &path], &DEV);
    refuse_a_disagreeing_copy(&p);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute(
        "UPDATE events SET provenance = 'user' WHERE kind = 'unparsed_line' AND seq = 2",
        [],
    )
    .unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_INTEGRITY);
    assert!(
        err.starts_with("parinn verify: hash chain: chain broken at"),
        "{err}"
    );
}

#[test]
fn verify_without_a_store_or_with_a_failing_one_fails() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        err,
        format!(
            "parinn verify: no store at {}\n",
            p.parinn_home().join("store.db").display()
        )
    );

    owner_only_dirs(&p.parinn_home());
    junk_store(&p);
    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn verify: the store: "), "{err}");
}

#[test]
fn verify_reports_a_store_error_met_while_checking() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("UPDATE seats SET room = 'x'", []).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    let (_, json, _) = p.invoke(&["verify", "--json"], &DEV);

    assert_eq!(code, EXIT_INTEGRITY);
    assert!(
        err.starts_with("parinn verify: store: the store is corrupt"),
        "{err}"
    );
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        (&v["broken"]["class"], &v["broken"]["address"]),
        (&serde_json::json!("corrupt"), &serde_json::Value::Null)
    );
}

#[test]
fn verify_reports_a_store_that_fails_while_checking() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("DROP TABLE event_heads", []).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn verify: the store: no such table"),
        "{err}"
    );
}

#[test]
fn verify_fails_naming_the_store_when_its_refused_forks_cannot_be_read() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute("CREATE TABLE fork_refusals (unrelated TEXT)", [])
        .unwrap();

    let (code, out, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "");
    assert!(err.starts_with("parinn verify: the store: "), "{err}");
}

#[test]
fn verify_of_a_home_without_its_device_key_fails() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::remove_file(p.parinn_home().join("keys/device.key")).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("no key in the key store entry device"),
        "{err}"
    );
}

#[test]
fn verify_refuses_an_unknown_flag() {
    assert_eq!(
        invoke(&["verify", "--yaml"], &DEV),
        (
            EXIT_USAGE,
            String::new(),
            "parinn verify: unknown flag \"--yaml\"\n".into()
        )
    );
}

#[test]
fn mcp_answers_each_line_of_input() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    p.lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.into(),
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.into(),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let lines: Vec<serde_json::Value> = out
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["result"]["serverInfo"]["version"], "dev");
    assert_eq!(lines[1]["id"], 2);
}

#[test]
fn mcp_refuses_flags_and_reports_a_store_it_cannot_open() {
    for flag in ["--x", "--json"] {
        assert_eq!(
            invoke(&["mcp", flag], &DEV),
            (
                EXIT_USAGE,
                String::new(),
                format!("parinn mcp: unknown flag {flag:?}\n")
            )
        );
    }
    let p = Process::new();
    owner_only_dirs(&p.parinn_home());
    junk_store(&p);
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.starts_with("parinn mcp: the store: "), "{err}");
}

#[test]
fn mcp_starts_the_node_it_records_in_and_refuses_one_whose_device_key_is_missing() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(p.parinn_home().join("store.db").exists());
    std::fs::remove_file(p.parinn_home().join("keys/device.key")).unwrap();
    let (code, _, err) = p.invoke(&["mcp"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("the device key is missing"), "{err}");
}

/// A `tools/call` of `event_get`, carrying the tool-use id `tool_use_id`
/// when given.
fn tool_call(id: u64, tool_use_id: Option<&str>) -> String {
    let mut params = serde_json::json!({"name": "event_get", "arguments": {"address": "x"}});
    if let Some(t) = tool_use_id {
        params["_meta"] = serde_json::json!({"claudecode/toolUseId": t});
    }
    serde_json::json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": params})
        .to_string()
}

#[test]
fn a_bound_mcp_server_serves_a_call_under_its_tool_use_ids_run_and_counts_each_refusal() {
    let mut p = Process::new();
    let session = "5e55a0de-0000-4000-8000-0000000000e1";
    let start = crate::hook_handlers::tests::session_start(&p, session, "startup");
    let path: String =
        serde_json::from_str::<serde_json::Value>(&start).unwrap()["transcript_path"]
            .as_str()
            .unwrap()
            .to_owned();
    let call = serde_json::json!({"type": "assistant", "message": {"content": [
        {"type": "tool_use", "id": "toolu_1", "name": "event_get"}
    ]}});
    std::fs::write(&path, format!("{call}\n")).unwrap();
    p.lines = vec![start];
    p.invoke(&["hook", "SessionStart"], &DEV);
    p.lines = vec![
        tool_call(1, Some("toolu_1")),
        tool_call(2, None),
        tool_call(3, Some("toolu_none")),
    ];

    let (code, out, err) = p.invoke(&["mcp"], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let texts: Vec<String> = out
        .lines()
        .map(|l| {
            let reply: serde_json::Value = serde_json::from_str(l).unwrap();
            reply["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert!(!texts[0].contains("tool-use id"), "served: {}", texts[0]);
    assert!(texts[1].contains("carries no tool-use id"), "{}", texts[1]);
    assert!(texts[2].contains("within 2 s"), "{}", texts[2]);
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    for counter in ["tool_call_without_tool_use_id", "tool_call_run_not_found"] {
        assert_eq!(
            counters
                .iter()
                .filter(|c| c.as_deref() == Some(counter))
                .count(),
            1,
            "{counter}: {counters:?}"
        );
    }
    assert!(
        !counters.contains(&Some("session_binding_not_taken".to_owned())),
        "taken"
    );
}

/// Leaves, in `p`'s home, the start marker an MCP server killed before
/// it stopped would, naming the transcript at `path`.
fn stale_marker(p: &Process, path: &str) {
    let dir = p.parinn_home().join("mcp");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("0011223344556677.marker"), format!("{path}\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::set_permissions(
            dir.join("0011223344556677.marker"),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
    }
}

#[test]
fn the_next_reader_audits_what_a_killed_mcp_server_left() {
    for command in [&["ingest", "--all"][..], &["mcp"]] {
        let p = Process::new();
        let path = p.transcript(CLEAN_LINE);
        p.invoke(&["counter", "ack"], &DEV);
        let resolved = std::fs::canonicalize(&path).unwrap();
        stale_marker(&p, &resolved.display().to_string());

        let (code, _, err) = p.invoke(command, &DEV);

        assert_eq!((code, err.as_str()), (EXIT_OK, ""), "{command:?}");
        let entries: Vec<(Option<String>, String)> = audited(&p)
            .into_iter()
            .map(|e| (e.counter, e.operation))
            .collect();
        let operation = if command[0] == "mcp" { "mcp" } else { "ingest" };
        assert!(
            entries.contains(&(
                Some("ingest_remainder_left".to_owned()),
                operation.to_owned()
            )),
            "{command:?}: {entries:?}"
        );
        assert!(
            !p.parinn_home()
                .join("mcp")
                .join("0011223344556677.marker")
                .exists()
        );
    }
}

#[test]
fn ambiguous_calls_are_counted_on_their_own() {
    use mcp::tool_call::{Cause, Found};
    let p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
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

    log_with(&p, None, || {
        record_found(
            &ctx,
            &[Found {
                tool_use_id: Some("toolu_2".to_owned()),
                run: Err(Cause::Ambiguous),
                waited: std::time::Duration::ZERO,
            }],
        );
    });

    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(counters.contains(&Some("tool_call_run_ambiguous".to_owned())));
    let refused = logged_lines(&p)
        .into_iter()
        .find(|l| l["fields"]["tool_use_id"] == "toolu_2")
        .expect("the refusal is logged");
    assert_eq!(refused["fields"]["cause"], Cause::Ambiguous.as_str());
}

#[test]
fn an_mcp_server_serves_without_live_ingest_while_a_settings_layer_fails() {
    let p = Process::new();
    p.configure("recal.max_k = 10\n");

    let (code, _, _) = p.invoke(&["mcp"], &DEV);

    assert_eq!(code, EXIT_OK);
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(
        counters.contains(&Some("ingest_refused_settings_invalid".to_owned())),
        "{counters:?}"
    );
}

#[test]
fn mcp_reports_a_reply_it_cannot_write() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let lines = vec![r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#.to_owned()];
    let input = move || {
        let (tx, rx) = mpsc::channel();
        for line in lines.clone() {
            tx.send(Ok(line)).unwrap();
        }
        rx
    };
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

    let code = run(["mcp".to_owned()], &ctx, &mut Closed, &mut err);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(String::from_utf8(err).unwrap(), "parinn mcp: closed\n");
}

#[test]
fn a_command_without_a_home_fails() {
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
    for command in ["ingest", "mcp", "verify", "status", "doctor", "counter"] {
        let mut args = vec![command.to_owned()];
        if command == "ingest" {
            args.extend(["--path".to_owned(), "t".to_owned()]);
        }
        if command == "counter" {
            args.push("ack".to_owned());
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());

        let code = run(args, &ctx, &mut out, &mut err);

        assert_eq!(code, EXIT_FAILURE, "{command}");
        let name = if command == "counter" {
            "counter ack"
        } else {
            command
        };
        assert_eq!(
            String::from_utf8(err).unwrap(),
            format!("parinn {name}: no home: set PARINN_HOME or HOME\n")
        );
    }
}

/// Makes `path` and the directories above it that do not exist yet,
/// each readable by its owner alone, as the home's check wants (SEC-02).
pub(crate) fn owner_only_dirs(path: &std::path::Path) {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path).unwrap();
}

/// Writes a store file that is no database, readable by its owner alone
/// so the store reads it rather than refuse its permissions (SEC-02).
pub(crate) fn junk_store(p: &Process) {
    let path = p.parinn_home().join("store.db");
    std::fs::write(&path, "not a database, but long enough to read a header").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

/// The audit log's entries in `p`'s home.
pub(crate) fn audited(p: &Process) -> Vec<audit::entry::Entry> {
    audit::log::Log::new(p.parinn_home().join("audit"))
        .entries()
        .unwrap()
}

pub(super) const SESSION_START: &str =
    r#"{"session_id":"s","hook_event_name":"SessionStart","source":"startup"}"#;

#[test]
fn a_hook_with_a_valid_hook_input_injects_nothing_and_audits_nothing() {
    let mut p = Process::new();
    p.lines = vec![crate::hook_handlers::tests::session_start(
        &p, "s1", "startup",
    )];

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p), Vec::new());
}

#[test]
fn a_hook_input_that_is_no_json_object_is_audited_and_counted() {
    let mut p = Process::new();
    p.lines = vec!["{\"session_id\": ".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(
        (
            e.kind.as_str(),
            e.counter.as_deref(),
            e.operation.as_str(),
            e.detail.as_str(),
            e.time
        ),
        (
            "rejected",
            Some("hook_rejected"),
            "hook UserPromptSubmit",
            "not a JSON object",
            NOW
        )
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode =
            |path: &std::path::Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&p.parinn_home()), 0o700);
        assert_eq!(
            mode(&p.parinn_home().join("audit/audit-000001.jsonl")),
            0o600
        );
    }
}

#[test]
fn a_hook_input_that_cannot_be_read_or_never_ends_is_rejected() {
    let mut p = Process::new();
    p.input_error = true;
    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    p.input_error = false;
    p.held_open = true;
    p.lines = vec![SESSION_START.to_owned()];
    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let details: Vec<String> = audited(&p).into_iter().map(|e| e.detail).collect();
    assert_eq!(details, ["not a JSON object", "not a JSON object"]);
}

#[test]
fn a_hook_input_that_trickles_past_its_wait_is_rejected() {
    let p = Process::new();
    // A line every 50 ms, each inside the wait, until the hook stops
    // listening: the whole input must still end within the wait.
    let input = || {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            while tx.send(Ok("{".to_owned())).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        rx
    };
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
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(
        ["hook".to_owned(), "SessionStart".to_owned()],
        &ctx,
        &mut out,
        &mut err,
    );

    assert_eq!(
        (code, out.is_empty(), err.is_empty()),
        (EXIT_OK, true, true)
    );
    let details: Vec<String> = audited(&p).into_iter().map(|e| e.detail).collect();
    assert_eq!(details, ["not a JSON object"]);
}

#[test]
fn a_panic_while_auditing_a_hook_failure_still_exits_0() {
    let p = Process::new();
    let input = || {
        let (tx, rx) = mpsc::channel();
        tx.send(Ok("not json".to_owned())).unwrap();
        rx
    };
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
        clock: &|| panic!("the clock fails"),
        attended: false,
        hook_budget: Some(LENIENT),
        ancestry: own_harness,
        live_limit: None,
        query_deadline: None,
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(
        ["hook".to_owned(), "UserPromptSubmit".to_owned()],
        &ctx,
        &mut out,
        &mut err,
    );

    assert_eq!(
        (code, out.is_empty(), err.is_empty()),
        (EXIT_OK, true, true)
    );
}
