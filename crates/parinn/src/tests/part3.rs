//! The parinn tests, part three.
//! The rest of the module's tests, moved from `tests.rs` to keep each file in its limit.

use super::part2::SESSION_START;
use super::part4::{fork_detection_json, identity_json, identity_lines};
use super::*;
use std::io;

#[test]
fn a_hook_parinn_does_not_install_is_rejected_and_exits_0() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];
    for args in [&["hook", "Stop"][..], &["hook"]] {
        assert_eq!(
            p.invoke(args, &DEV),
            (EXIT_OK, String::new(), String::new()),
            "{args:?}"
        );
    }
    let entries = audited(&p);
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|e| e.operation == "hook" && e.detail == "no such hook"),
        "{entries:?}"
    );
}

#[test]
fn a_hook_handler_that_panics_fails_open_and_is_audited() {
    let mut p = Process::new();
    p.lines = vec![SESSION_START.to_owned()];
    p.hook_panics = true;

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (
            entries[0].kind.as_str(),
            entries[0].counter.as_deref(),
            entries[0].detail.as_str()
        ),
        ("failed", Some("hook_panicked"), "panic recovered")
    );
}

#[cfg(unix)]
#[test]
fn a_hook_in_a_home_it_must_refuse_writes_nothing_and_exits_0() {
    use std::os::unix::fs::PermissionsExt;
    let mut p = Process::new();
    std::fs::create_dir(p.parinn_home()).unwrap();
    std::fs::set_permissions(p.parinn_home(), std::fs::Permissions::from_mode(0o755)).unwrap();
    p.lines = vec!["not json".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert!(!p.parinn_home().join("audit").exists());
    p.home_id = Some("other".into());
    std::fs::set_permissions(p.parinn_home(), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(p.parinn_home().join("home.json"), r#"{"id":"mine"}"#).unwrap();
    std::fs::set_permissions(
        p.parinn_home().join("home.json"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert!(!p.parinn_home().join("audit").exists());
}

#[test]
fn a_hook_whose_audit_log_cannot_take_an_entry_still_exits_0() {
    let mut p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));
    p.lines = vec!["not json".to_owned()];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
}

#[test]
fn counter_ack_records_a_neutral_principal_act_through_the_last_audit_entry() {
    let mut p = Process::new();

    assert_eq!(
        p.invoke(&["counter", "ack"], &DEV),
        (
            EXIT_OK,
            "counters acknowledged; the audit log held no entry\n".into(),
            String::new()
        )
    );
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let (code, out, _) = p.invoke(&["counter", "ack", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["acknowledged"], 2);
    let acts = recorded(&p);
    assert_eq!(acts.len(), 2);
    assert_eq!(acts[0]["audit_entry"], serde_json::Value::Null);
    assert_eq!(acts[1]["act"], "counter_ack");
    assert_eq!(acts[1]["class"], "neutral");
    assert_eq!(acts[1]["principal_surface"], "cli");
    assert_eq!(acts[1]["audit_entry"], 2);
    let entries = audited(&p);
    assert_eq!(acts[1]["audit_hash"], entries[1].hash.to_hex());
    let kinds: Vec<String> = entries.into_iter().map(|e| e.kind).collect();
    assert_eq!(kinds, ["principal_act", "rejected", "principal_act"]);
}

#[test]
fn counter_ack_keeps_its_recorded_act_when_the_audit_log_takes_no_note() {
    let mut p = Process::new();
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let lock = p.parinn_home().join("audit/lock");
    std::fs::remove_file(&lock).unwrap();
    owner_only_dirs(&lock);

    let (code, out, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.starts_with("parinn counter ack: the acknowledgement is recorded, but the audit log took no note of it: "),
        "{err}"
    );
    assert_eq!(recorded(&p)[0]["audit_entry"], 1);
    assert_eq!(audited(&p).len(), 1);
    std::fs::remove_dir(&lock).unwrap();
    assert_eq!(p.invoke(&["doctor"], &DEV).0, EXIT_OK);
}

#[test]
fn an_acknowledgement_hides_no_failure_once_the_audit_log_began_again() {
    let mut p = Process::new();
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    assert_eq!(p.invoke(&["counter", "ack"], &DEV).0, EXIT_OK);
    std::fs::remove_dir_all(p.parinn_home().join("audit")).unwrap();
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    let (code, out, _) = p.invoke(&["doctor"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(
        out,
        "counter hook_rejected rose by 1 since the counters were last acknowledged\n"
    );
}

/// The content of every principal act the home's store holds.
pub(crate) fn recorded(p: &Process) -> Vec<serde_json::Value> {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();
    let node = Node::find(&home, &store).unwrap();
    act_record::acts(&store, &node)
        .unwrap()
        .into_iter()
        .map(|a| a.content)
        .collect()
}

#[test]
fn counter_ack_without_a_terminal_is_refused_audited_and_acknowledges_nothing() {
    let mut p = Process::new();
    p.terminal = false;

    assert_eq!(
        p.invoke(&["counter", "ack"], &DEV),
        (
            EXIT_FAILURE,
            String::new(),
            "parinn counter ack: refused: a principal act needs a terminal on standard input and \
             output (OWN-12)\n"
                .into()
        )
    );
    assert!(!p.parinn_home().join("store.db").exists());
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].counter.as_deref(), Some("principal_act_refused"));
    assert_eq!(entries[0].operation, "counter ack");
}

#[test]
fn counter_needs_ack_and_refuses_other_flags() {
    for (args, why) in [
        (&["counter"][..], "parinn counter: want ack\n"),
        (&["counter", "reset"], "parinn counter: want ack\n"),
        (
            &["counter", "ack", "--all"],
            "parinn counter ack: unknown flag \"--all\"\n",
        ),
    ] {
        assert_eq!(
            invoke(args, &DEV),
            (EXIT_USAGE, String::new(), why.into()),
            "{args:?}"
        );
    }
}

#[test]
fn counter_ack_reports_an_audit_log_it_cannot_append_to() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));

    let (code, _, err) = p.invoke(&["counter", "ack"], &DEV);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.starts_with("parinn counter ack: ") && err.contains("audit-000001.jsonl"),
        "{err}"
    );
}

#[test]
fn doctor_fails_while_a_counter_rose_since_the_last_acknowledgement() {
    let mut p = Process::new();
    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_OK,
            format!(
                "no home at {} yet: nothing to diagnose\n",
                p.parinn_home().display()
            ),
            String::new()
        )
    );
    p.invoke(&["counter", "ack"], &DEV);
    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_OK,
            "no counter rose since the counters were last acknowledged\n".into(),
            String::new()
        )
    );
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    assert_eq!(
        p.invoke(&["doctor"], &DEV),
        (
            EXIT_FAILURE,
            "counter hook_rejected rose by 2 since the counters were last acknowledged\n".into(),
            String::new()
        )
    );
    let (code, out, _) = p.invoke(&["doctor", "--json"], &DEV);
    assert_eq!(code, EXIT_FAILURE);
    assert_eq!(out, "{\"risen\":{\"hook_rejected\":2}}\n");

    p.invoke(&["counter", "ack"], &DEV);
    let (code, out, _) = p.invoke(&["doctor", "--json"], &DEV);
    assert_eq!((code, out.as_str()), (EXIT_OK, "{\"risen\":{}}\n"));
}

#[test]
fn doctor_and_status_exit_3_on_a_broken_audit_log() {
    let mut p = Process::new();
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let file = p.parinn_home().join("audit/audit-000001.jsonl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("rejected", "rejecteD")).unwrap();

    for command in ["doctor", "status"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_INTEGRITY, "{command}");
        assert_eq!(
            err,
            format!("parinn {command}: audit log audit-000001.jsonl line 1: hash does not match\n")
        );
    }
}

#[test]
fn doctor_and_status_refuse_unknown_flags() {
    for command in ["doctor", "status"] {
        assert_eq!(
            invoke(&[command, "--fix"], &DEV),
            (
                EXIT_USAGE,
                String::new(),
                format!("parinn {command}: unknown flag \"--fix\"\n")
            )
        );
    }
}

#[test]
fn status_shows_the_home_the_store_the_audit_log_and_the_counters() {
    let mut p = Process::new();
    let home = p.parinn_home().display().to_string();
    let id = identity_lines();
    assert_eq!(
        p.invoke(&["status"], &DEV),
        (
            EXIT_OK,
            format!(
                "home: {home} (none yet)\ndeployment mode: automation\nkeys: files only this OS user can read; an unsandboxed agent of the same OS user could read them\n{id}"
            ),
            String::new()
        )
    );
    Home::locate(Some(p.parinn_home().as_os_str()), None)
        .unwrap()
        .create()
        .unwrap();
    assert_eq!(
        p.invoke(&["status"], &DEV).1,
        format!(
            "home: {home}\nstore: {home}/store.db (none yet)\naudit log: {home}/audit (0 entries)\n\
             counters: none\ndeployment mode: automation\nkeys: files only this OS user can read; an unsandboxed agent of the same OS user could read them\n{id}"
        )
    );
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);

    let (code, out, err) = p.invoke(&["status"], &DEV);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let bytes = std::fs::metadata(p.parinn_home().join("store.db"))
        .unwrap()
        .len();
    assert_eq!(
        out,
        format!(
            "home: {home}\nstore: {home}/store.db ({bytes} bytes, schema version 1)\naudit log: {home}/audit (1 entries)\n\
             counters: hook_rejected 1 (1 since acknowledged)\ndeployment mode: automation\nkeys: files only this OS user can read; an unsandboxed agent of the same OS user could read them\n{id}"
        )
    );
    let (code, out, _) = p.invoke(&["status", "--json"], &DEV);
    assert_eq!(code, EXIT_OK);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v,
        serde_json::json!({
            "home": home,
            "store": {"path": format!("{home}/store.db"), "bytes": bytes, "schema_version": 1},
            "audit": {"path": format!("{home}/audit"), "entries": 1},
            "counters": {"hook_rejected": {"total": 1, "since_ack": 1}},
            "deployment_mode": "automation",
            "awaiting_acceptance": [],
            "managed_policy": {"refused": null},
            "keys": {
                "store": "file",
                "readable_by_agents": true,
                "where": "files only this OS user can read; an unsandboxed agent of the same OS user could read them",
            },
            "node_identity": identity_json(),
            "fork_detection": fork_detection_json(),
            "segments": {"open": [], "newer": []},
            "transcript_sources": {"held": [], "lost": []},
        })
    );
}

#[test]
fn status_of_a_home_not_made_yet_shows_no_store() {
    let fresh = Process::new();
    let (_, out, _) = fresh.invoke(&["status", "--json"], &DEV);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v,
        serde_json::json!({
            "home": fresh.parinn_home().display().to_string(),
            "store": null,
            "audit": null,
            "counters": {},
            "deployment_mode": "automation",
            "awaiting_acceptance": [],
            "managed_policy": {"refused": null},
            "keys": {
                "store": "file",
                "readable_by_agents": true,
                "where": "files only this OS user can read; an unsandboxed agent of the same OS user could read them",
            },
            "node_identity": identity_json(),
            "fork_detection": fork_detection_json(),
            "segments": null,
            "transcript_sources": null,
        })
    );
}

#[cfg(unix)]
#[test]
fn every_command_refuses_a_home_others_can_reach_naming_it() {
    use std::os::unix::fs::PermissionsExt;
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::set_permissions(p.parinn_home(), std::fs::Permissions::from_mode(0o750)).unwrap();
    let path = p.transcript("b\n");
    for args in [
        &["ingest", "--path", path.as_str()][..],
        &["mcp"],
        &["verify"],
        &["status"],
        &["doctor"],
        &["counter", "ack"],
    ] {
        let (code, out, err) = p.invoke(args, &DEV);
        let name = if args[0] == "counter" {
            "counter ack"
        } else {
            args[0]
        };
        assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""), "{args:?}");
        assert_eq!(
            err,
            format!(
                "parinn {name}: {}: mode 0750 lets other OS users in; the home wants owner-only \
                 permissions\n",
                p.parinn_home().display()
            ),
            "{args:?}"
        );
    }
}

#[test]
fn every_command_refuses_a_home_bound_to_another_home_id() {
    let mut p = Process::new();
    p.invoke(&["counter", "ack"], &DEV);
    let binding = p.parinn_home().join("home.json");
    std::fs::write(&binding, r#"{"id":"home-a"}"#).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binding, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    p.home_id = Some("home-b".into());
    for command in ["verify", "status", "doctor", "mcp"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_FAILURE, "{command}");
        assert_eq!(
            err,
            format!(
                "parinn {command}: the home id PARINN_HOME_ID supplies does not match the home's\n"
            )
        );
    }
    p.home_id = Some("home-a".into());
    assert_eq!(p.invoke(&["doctor"], &DEV).0, EXIT_OK);
}

#[test]
fn verify_checks_the_audit_chain_too() {
    let mut p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    p.lines = vec!["not json".to_owned()];
    p.invoke(&["hook", "UserPromptSubmit"], &DEV);
    let (code, out, _) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_OK);
    assert!(
        out.ends_with("the audit log's chain holds over 1 entries\n"),
        "{out}"
    );
    let file = p.parinn_home().join("audit/audit-000001.jsonl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("rejected", "rejecteD")).unwrap();

    let (code, _, err) = p.invoke(&["verify"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    assert_eq!(
        err,
        "parinn verify: hash chain: audit log audit-000001.jsonl line 1: hash does not match\n"
    );
    let (code, out, _) = p.invoke(&["verify", "--json"], &DEV);
    assert_eq!(code, EXIT_INTEGRITY);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["broken"],
        serde_json::json!({
            "class": "audit",
            "check": "hash chain",
            "address": null,
            "message": "audit log audit-000001.jsonl line 1: hash does not match",
        })
    );
}

#[test]
fn a_panic_in_the_mcp_server_is_audited() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();

    mcp_panicked(&home, NOW).unwrap();

    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (
            entries[0].counter.as_deref(),
            entries[0].operation.as_str(),
            entries[0].detail.as_str()
        ),
        (Some("mcp_panicked"), "mcp", "panic recovered")
    );
}

#[test]
fn a_caught_mcp_panic_the_audit_log_cannot_take_is_said_on_stderr() {
    let p = Process::new();
    owner_only_dirs(&p.parinn_home().join("audit/audit-000001.jsonl"));
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_mcp_panic(&home, &|| NOW, &mut err)();

    let err = String::from_utf8(err).unwrap();
    assert!(
        err.starts_with("parinn mcp: audit a caught panic: ") && err.contains("audit-000001.jsonl"),
        "{err}"
    );
}

#[test]
fn a_caught_mcp_panic_is_audited_quietly() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let mut err = Vec::new();

    on_mcp_panic(&home, &|| NOW, &mut err)();

    assert!(err.is_empty());
    assert_eq!(audited(&p).len(), 1);
}

#[test]
fn an_audit_log_that_cannot_be_read_fails_verify_doctor_and_status() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript(CLEAN_LINE)], &DEV);
    let audit = p.parinn_home().join("audit");
    std::fs::write(&audit, "not a directory").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&audit, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    for command in ["verify", "doctor", "status"] {
        let (code, _, err) = p.invoke(&[command], &DEV);
        assert_eq!(code, EXIT_FAILURE, "{command}");
        assert!(
            err.starts_with(&format!("parinn {command}: {}", audit.display())),
            "{command}: {err}"
        );
    }
}

#[test]
fn ingest_says_which_redaction_it_cannot_audit_and_still_records() {
    let p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"(\"]\n");
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK);
    // An audit log that takes no entry.
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, out, err) = p.invoke(
        &["ingest", "--path", &p.transcript("AKIAIOSFODNN7EXAMPLE\n")],
        &DEV,
    );

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("ingested 1 lines"), "{out}");
    assert!(
        err.contains("parinn ingest: audit a refused redaction pattern: "),
        "{err}"
    );
    assert!(err.contains("parinn ingest: audit a redaction: "), "{err}");
}

#[test]
fn ingest_says_which_limited_redaction_it_cannot_audit_and_still_records() {
    let p = Process::new();
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK);
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);
    let chain = format!("AIza{}", "a".repeat(35)).repeat(5_000);

    let (code, out, err) = p.invoke(
        &[
            "ingest",
            "--path",
            &p.transcript(&format!("{{\"t\":\"{chain}\"}}\n")),
        ],
        &DEV,
    );

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.starts_with("ingested 1 lines"), "{out}");
    assert!(
        err.contains("parinn ingest: audit a limited redaction: "),
        "{err}"
    );
}

#[test]
fn a_hook_input_past_1_mib_is_audited_and_injects_nothing() {
    let mut p = Process::new();
    p.lines = vec![
        "{\"p\":\"".to_owned(),
        format!("{}\"}}", "x".repeat(1 << 20)),
    ];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    let entries = audited(&p);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].detail, "hook input exceeds 1 MiB");
    assert_eq!(entries[0].counter.as_deref(), Some("hook_input_too_large"));
}

#[test]
fn a_line_the_reader_cut_at_its_limit_is_a_hook_input_past_1_mib() {
    let mut p = Process::new();
    p.lines = vec!["{".to_owned()];
    p.input_too_large = true;

    assert_eq!(
        p.invoke(&["hook", "SessionStart"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p)[0].detail, "hook input exceeds 1 MiB");
}

#[test]
fn a_hook_input_nested_past_depth_64_is_audited_and_injects_nothing() {
    let mut p = Process::new();
    p.lines = vec![format!("{}{}", "[".repeat(65), "]".repeat(65))];

    assert_eq!(
        p.invoke(&["hook", "UserPromptSubmit"], &DEV),
        (EXIT_OK, String::new(), String::new())
    );
    assert_eq!(audited(&p)[0].detail, "hook input nesting exceeds depth 64");
}

#[test]
fn bounded_lines_refuse_a_line_past_the_limit_without_holding_it_and_go_on() {
    let input = "ab\ncdefgh\nij\n".as_bytes();
    let lines: Vec<io::Result<String>> = bounded_lines(input, 4).collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].as_ref().unwrap(), "ab");
    assert_eq!(
        lines[1].as_ref().unwrap_err().kind(),
        io::ErrorKind::FileTooLarge
    );
    assert_eq!(lines[2].as_ref().unwrap(), "ij");
    let whole: Vec<String> = bounded_lines("a\r\nabcd\r\nb".as_bytes(), 4)
        .map(Result::unwrap)
        .collect();
    assert_eq!(whole, ["a", "abcd", "b"]);
    // One byte past the limit, its line ending read with it.
    let just_over: Vec<io::Result<String>> = bounded_lines("abcde\nx".as_bytes(), 4).collect();
    assert!(just_over[0].is_err());
    assert_eq!(just_over[1].as_ref().unwrap(), "x");
    let at_end: Vec<io::Result<String>> = bounded_lines("abcdef".as_bytes(), 4).collect();
    assert_eq!(at_end.len(), 1);
    assert!(at_end[0].is_err());
    struct Failing;
    impl io::Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("gone"))
        }
    }
    let failed: Vec<io::Result<String>> = bounded_lines(io::BufReader::new(Failing), 4).collect();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].is_err());
    let bad: Vec<io::Result<String>> = bounded_lines(&[0xff, b'\n', b'a'][..], 4).collect();
    assert_eq!(
        bad[0].as_ref().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(bad[1].as_ref().unwrap(), "a");
}

/// Logs one event carrying an AWS access key through the structured
/// log of `p`'s home, with `home_id` supplied.
fn log_one(p: &Process, home_id: Option<&str>) -> std::sync::Arc<structured_log::StructuredLog> {
    log_with(p, home_id, || {
        tracing::info!(key = "AKIAIOSFODNN7EXAMPLE", "logged");
    })
}

/// The structured log of `p`'s home, with `home_id` supplied, its
/// settings layers where `p`'s commands find them, as it stands after
/// `logging` ran under it.
pub(super) fn log_with(
    p: &Process,
    home_id: Option<&str>,
    logging: impl FnOnce(),
) -> std::sync::Arc<structured_log::StructuredLog> {
    let ctx = crate::settings::tests::ctx_of(p);
    let (managed, os_user) = (ctx.managed_policy.clone(), ctx.os_user);
    let log = std::sync::Arc::new(structured_log(
        None,
        Some(p.dir.path().as_os_str().to_owned()),
        home_id.map(Into::into),
        LogSettings { managed, os_user },
        || NOW,
    ));
    let dispatch = tracing::Dispatch::from(log.clone());
    tracing::dispatcher::with_default(&dispatch, logging);
    log
}

pub(super) fn logged_lines(p: &Process) -> Vec<Value> {
    std::fs::read_to_string(log_file(p))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The log redacts each field with the principal's redaction patterns
/// too, and writes no line while the principal's configuration fails,
/// so its patterns are unknown (SEC-08).
#[test]
fn the_structured_log_redacts_with_the_principal_patterns_or_not_at_all() {
    let p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"ACME-[0-9]{8}\"]\n");
    let log = log_with(&p, None, || {
        tracing::info!(key = "id ACME-12345678", "logged")
    });
    assert_eq!(log.dropped(), 0);
    assert_eq!(
        logged_lines(&p)[0]["fields"]["key"],
        "id [REDACTED:extra_pattern_1]"
    );

    p.configure("recal.max_k = 10\n");
    let log = log_with(&p, None, || {
        tracing::info!(key = "id ACME-12345678", "logged")
    });
    assert_eq!(log.dropped(), 1);
    assert_eq!(logged_lines(&p).len(), 1);
}

/// A repository's redaction patterns are never applied, not even to the
/// log, and a failing repository configuration leaves the log written
/// (SEC-08, SEC-11).
#[test]
fn the_structured_log_ignores_a_repository_configuration() {
    let p = Process::new();
    p.configure("recall.max_k = 10\n");
    p.repository("[redaction]\nextra_patterns = [\"keep-me\"]\n");
    log_with(&p, None, || tracing::info!(key = "keep-me", "logged"));
    assert_eq!(logged_lines(&p)[0]["fields"]["key"], "keep-me");

    p.repository("recall.max_k = 'ten'\n");
    let log = log_with(&p, None, || tracing::info!(key = "x", "logged"));
    assert_eq!(log.dropped(), 0);
    assert_eq!(logged_lines(&p).len(), 2);
}

/// The transcript path ingest logs is redacted with the harness account
/// identifiers the transcript taught it (SEC-08).
#[test]
fn the_logged_transcript_path_is_redacted_with_the_harness_account_identifiers() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let dir = p.dir.path().join(".claude/projects/someone@example.com");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("t.jsonl");
    let context = r#"{"attachment":{"type":"session_context","context":{"userEmail":"someone@example.com"}}}"#;
    std::fs::write(&path, format!("{context}\n")).unwrap();
    let path = path.display().to_string();

    log_with(&p, None, || {
        assert_eq!(p.invoke(&["ingest", "--path", &path], &DEV).0, 0);
    });

    let logged = logged_lines(&p);
    let transcript = logged[0]["fields"]["transcript"].as_str().unwrap();
    assert!(!transcript.contains("someone@example.com"), "{transcript}");
}

fn log_file(p: &Process) -> std::path::PathBuf {
    p.parinn_home().join("logs").join(structured_log::LOG_FILE)
}

#[test]
fn the_structured_log_waits_for_the_home_and_redacts_each_field() {
    let p = Process::new();
    log_one(&p, None);
    assert!(!p.parinn_home().exists(), "a log makes no home");

    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let log = log_one(&p, None);

    assert_eq!(log.dropped(), 0);
    let text = std::fs::read_to_string(log_file(&p)).unwrap();
    let line: Value = serde_json::from_str(text.trim_end()).unwrap();
    assert_eq!(line["fields"]["key"], "[REDACTED:aws_access_key]");
    assert_eq!(line["time"], NOW);
    assert!(
        Home::locate(Some(p.parinn_home().as_os_str()), None)
            .unwrap()
            .check()
            .is_ok()
    );
}

#[test]
fn a_home_the_core_refuses_gets_no_log() {
    let mut p = Process::new();
    p.home_id = Some("h1".to_owned());
    std::fs::create_dir_all(p.parinn_home()).unwrap();
    set_mode(&p.parinn_home(), 0o700);
    std::fs::write(p.parinn_home().join("home.json"), r#"{"id":"h1"}"#).unwrap();
    set_mode(&p.parinn_home().join("home.json"), 0o600);

    log_one(&p, Some("h2"));
    assert!(!log_file(&p).exists());
    set_mode(&p.parinn_home(), 0o755);
    log_one(&p, Some("h1"));
    assert!(!log_file(&p).exists());
    let none = structured_log(None, None, None, LogSettings::default(), || NOW);
    let dispatch = tracing::Dispatch::new(none);
    tracing::dispatcher::with_default(&dispatch, || tracing::info!("nowhere"));
}

#[test]
fn dropped_log_lines_are_audited_one_entry_each() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    let before = audited(&p).len();
    let ctx = crate::settings::tests::ctx_of(&p);

    audit_dropped_log_lines(&ctx, 2);

    let entries = audited(&p);
    assert_eq!(entries.len(), before + 2);
    assert_eq!(entries[before].counter.as_deref(), Some("log_line_dropped"));
    assert_eq!(entries[before].detail, "log line dropped");
}

#[test]
fn ingest_refuses_a_path_outside_the_allowed_roots_and_reads_nothing() {
    let p = crate::ingest_path::tests::off_terminal();
    let outside = p.dir.path().join("t.jsonl");
    std::fs::write(&outside, "AKIAIOSFODNN7EXAMPLE\n").unwrap();
    let climbing = format!(
        "{}/../../t.jsonl",
        p.transcript("a\n").trim_end_matches("/t.jsonl")
    );

    for path in [outside.display().to_string(), climbing] {
        let (code, out, err) = p.invoke(&["ingest", "--path", &path], &DEV);

        assert_eq!(
            (code, out.as_str(), err.as_str()),
            (
                EXIT_FAILURE,
                "",
                "parinn ingest: path outside allowed roots\n"
            )
        );
    }
    let entries = audited(&p);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].detail, "path outside allowed roots");
    assert_eq!(
        entries[0].counter.as_deref(),
        Some("path_outside_allowed_roots")
    );
    assert!(!p.parinn_home().join("store.db").exists());
}

/// While the principal's configuration fails, ingest cannot know the
/// redaction patterns the principal asked for, so it records nothing:
/// the refusal is audited and counted, and the transcript stays for an
/// ingest once the configuration is fixed (SEC-08, I1, I6).
#[test]
fn ingest_refuses_while_a_settings_layer_fails() {
    let p = Process::new();
    p.configure("recal.max_k = 10\n");
    let transcript = p.transcript("{\"a\":1}\n");

    let (code, out, err) = p.invoke(&["ingest", "--path", &transcript], &DEV);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(
        err.contains("parinn ingest: a settings layer failed"),
        "{err}"
    );
    let counters: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    assert!(
        counters.contains(&Some("settings_layer_invalid".to_owned())),
        "{counters:?}"
    );
    assert!(
        counters.contains(&Some("ingest_refused_settings_invalid".to_owned())),
        "{counters:?}"
    );
    assert!(!p.parinn_home().join("store.db").exists());

    p.configure("recall.max_k = 10\n");
    assert_eq!(p.invoke(&["ingest", "--path", &transcript], &DEV).0, 0);
}
