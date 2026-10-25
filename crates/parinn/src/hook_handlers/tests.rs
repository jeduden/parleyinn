use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};

use super::*;
use crate::tests::{DEV, NOW, Process, audited};

/// The path of harness session `session`'s transcript under the default
/// transcript root, its directory made.
fn transcript_path(p: &Process, session: &str) -> PathBuf {
    let dir = p.dir.path().join(".claude").join("projects").join("-w-app");
    fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{session}.jsonl"))
}

/// A `SessionStart` hook input of harness session `session` in Claude
/// Code 2.1.296's recorded shape.
pub(crate) fn session_start(p: &Process, session: &str, source: &str) -> String {
    json!({
        "session_id": session,
        "transcript_path": transcript_path(p, session),
        "cwd": "/w/app",
        "hook_event_name": "SessionStart",
        "source": source,
    })
    .to_string()
}

/// A `UserPromptSubmit` hook input of `session` for prompt `prompt_id`.
fn user_prompt_submit(p: &Process, session: &str, prompt_id: &str, prompt: &str) -> String {
    json!({
        "session_id": session,
        "transcript_path": transcript_path(p, session),
        "cwd": "/w/app",
        "prompt_id": prompt_id,
        "permission_mode": "default",
        "hook_event_name": "UserPromptSubmit",
        "prompt": prompt,
    })
    .to_string()
}

fn store(p: &Process) -> Store {
    Store::open(&p.parinn_home().join("store.db")).unwrap()
}

/// What the store keeps on disk: its file, and the log a hook handler
/// leaves to the next opener.
fn stored(p: &Process) -> String {
    let file = |name: &str| fs::read(p.parinn_home().join(name)).unwrap_or_default();
    let text = [file("store.db"), file("store.db-wal")].concat();
    String::from_utf8_lossy(&text).into_owned()
}

fn held(p: &Process) -> Vec<store::Held> {
    store(p).hook_observations().unwrap()
}

/// Runs the hook `hook` on `input`.
fn hook(
    p: &mut Process,
    hook: &str,
    input: impl FnOnce(&Process) -> String,
) -> (u8, String, String) {
    p.lines = vec![input(p)];
    p.invoke(&["hook", hook], &DEV)
}

/// Each audit entry's counter and detail.
fn entries(p: &Process) -> Vec<(Option<String>, String)> {
    audited(p)
        .into_iter()
        .map(|e| (e.counter, e.detail))
        .collect()
}

fn markers(p: &Process) -> Vec<String> {
    fs::read_dir(p.parinn_home().join(MARKERS))
        .map(|l| {
            l.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_session_start_holds_its_session_binding_at_the_transcripts_ingest_marker() {
    let mut p = Process::new();
    fs::write(transcript_path(&p, "s1"), "0123456789").unwrap();

    let ran = hook(&mut p, "SessionStart", |p| session_start(p, "s1", "fork"));

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(entries(&p), Vec::new());
    let held = held(&p);
    assert_eq!(held.len(), 1);
    let o = &held[0].observation;
    assert_eq!(
        (
            o.hook.as_str(),
            o.harness_session.as_str(),
            o.cwd.as_str(),
            o.offset,
            o.source.as_deref(),
            o.at,
            held[0].appended
        ),
        (
            "SessionStart",
            "s1",
            "/w/app",
            Some(10),
            Some("fork"),
            NOW,
            false
        )
    );
    assert_eq!(o.ancestry, Some((1, false)));
    assert_eq!(
        o.harness,
        Some(crate::tests::HARNESS),
        "its harness process, whose MCP server takes the session binding"
    );
    assert_eq!(
        markers(&p),
        Vec::<String>::new(),
        "its start marker cleared"
    );
}

#[test]
fn a_user_prompt_submit_keeps_only_the_prompt_id_and_a_prompt_commitment() {
    let mut p = Process::new();
    p.attended = true;

    hook(&mut p, "UserPromptSubmit", |p| {
        user_prompt_submit(p, "s1", "p1", "deploy with AKIAIOSFODNN7EXAMPLE")
    });

    let held = held(&p);
    let o = &held[0].observation;
    assert_eq!(
        (
            o.prompt_id.as_deref(),
            o.attended,
            o.offset,
            o.ancestry,
            o.harness
        ),
        (Some("p1"), true, Some(0), None, None)
    );
    let commitment = store(&p).prompt_commitment("s1", "p1").unwrap().unwrap();
    let redacted = Redactor::new(&[])
        .unwrap()
        .0
        .redact_text("deploy with AKIAIOSFODNN7EXAMPLE")
        .text;
    assert_ne!(redacted, "deploy with AKIAIOSFODNN7EXAMPLE");
    assert_eq!(
        commit(&commitment.key, &json!(redacted)).unwrap(),
        commitment.commitment
    );
    assert!(stored(&p).contains("p1"), "the prompt id is kept");
    assert!(
        !stored(&p).contains("deploy with"),
        "no prompt text is kept"
    );
}

#[test]
fn while_a_settings_layer_fails_the_prompt_commitment_is_dropped_and_counted() {
    let mut p = Process::new();
    p.configure("[node\n");

    hook(&mut p, "UserPromptSubmit", |p| {
        user_prompt_submit(p, "s1", "p1", "hi")
    });

    assert_eq!(held(&p).len(), 1, "the observation is kept");
    assert_eq!(store(&p).prompt_commitment("s1", "p1").unwrap(), None);
    assert_eq!(
        entries(&p),
        [(
            Some("prompt_commitment_dropped".into()),
            "prompt commitment dropped while a settings layer fails".into()
        )]
    );
}

#[test]
fn a_hook_input_failing_the_schema_is_rejected_audited_and_holds_nothing() {
    let mut p = Process::new();
    let mut input: Value = serde_json::from_str(&session_start(&p, "s1", "startup")).unwrap();
    input["session_id"] = json!(7);

    let ran = hook(&mut p, "SessionStart", |_| input.to_string());

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(
        entries(&p),
        [(
            Some("hook_input_rejected".into()),
            "hook input failed schema validation".into()
        )]
    );
    assert!(!p.parinn_home().join("store.db").exists());
}

#[test]
fn a_drifting_hook_input_is_audited_and_what_it_carries_still_held() {
    let mut p = Process::new();
    let mut input: Value = serde_json::from_str(&session_start(&p, "s1", "startup")).unwrap();
    input.as_object_mut().unwrap().remove("source");

    let ran = hook(&mut p, "SessionStart", |_| input.to_string());

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(
        entries(&p),
        [(
            Some("format_drift".into()),
            "format drift claude-code:2.1.296:SessionStart:missing:source".into()
        )]
    );
    assert_eq!(held(&p)[0].observation.source, None);
}

#[test]
fn an_unknown_source_is_recorded_as_other() {
    let mut p = Process::new();

    hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "teleport")
    });

    assert_eq!(held(&p)[0].observation.source.as_deref(), Some("other"));
}

#[test]
fn a_transcript_path_outside_the_roots_is_refused_and_never_read() {
    let mut p = Process::new();
    let mut input: Value = serde_json::from_str(&session_start(&p, "s1", "startup")).unwrap();
    input["transcript_path"] = json!(p.parinn_home().join("store.db"));

    hook(&mut p, "SessionStart", |_| input.to_string());

    assert_eq!(
        entries(&p),
        [(
            Some("path_outside_allowed_roots".into()),
            "path outside allowed roots".into()
        )]
    );
    assert_eq!(held(&p)[0].observation.offset, None);
}

#[cfg(unix)]
#[test]
fn a_transcript_that_cannot_be_read_holds_no_offset_and_is_counted() {
    let mut p = Process::new();
    let path = transcript_path(&p, "s1");
    let dir = path.parent().unwrap().to_path_buf();
    fs::write(&path, "x").unwrap();
    crate::tests::set_mode(&dir, 0o300);

    hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });
    crate::tests::set_mode(&dir, 0o700);

    // As root every file can be read, so nothing fails there.
    let failed = entries(&p).contains(&(
        Some("hook_failed".into()),
        "transcript path could not be read".into(),
    ));
    assert_eq!(failed, held(&p)[0].observation.offset.is_none());
}

#[test]
fn a_permission_request_is_decoded_and_holds_nothing() {
    let mut p = Process::new();

    let ran = hook(&mut p, "PermissionRequest", |_| {
        json!({"session_id": "s1", "transcript_path": "/x", "tool_name": "Bash"}).to_string()
    });

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(held(&p), Vec::new());
}

#[test]
fn a_permission_request_in_a_home_with_no_store_creates_none() {
    let mut p = Process::new();

    let ran = hook(&mut p, "PermissionRequest", |_| {
        json!({"session_id": "s1", "transcript_path": "/x", "tool_name": "Bash"}).to_string()
    });

    assert_eq!(ran.0, EXIT_OK);
    assert!(!p.parinn_home().join("store.db").exists());
}

#[test]
fn a_hook_input_without_a_session_holds_nothing() {
    let mut p = Process::new();

    hook(&mut p, "SessionStart", |_| {
        json!({"hook_event_name": "SessionStart"}).to_string()
    });

    assert_eq!(held(&p), Vec::new());
}

#[test]
fn a_store_that_cannot_open_is_counted_and_the_hook_answers_empty() {
    let mut p = Process::new();
    crate::tests::owner_only_dirs(&p.parinn_home().join("store.db"));

    let ran = hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(
        entries(&p),
        [(
            Some("hook_failed".into()),
            "store could not be opened".into()
        )]
    );
}

#[test]
fn a_hook_handler_past_its_budget_answers_empty_audited_and_clears_its_marker() {
    let mut p = Process::new();
    p.hook_budget = Some(Duration::from_nanos(1));

    let ran = hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    let budget = entries(&p).into_iter().find(|(c, _)| c.is_some());
    assert_eq!(
        budget,
        Some((
            Some("hook_budget_exceeded".into()),
            "hook handler ran past its hook budget and answered empty".into()
        ))
    );
    assert_eq!(markers(&p), Vec::<String>::new());
}

/// Leaves a start marker named `name` holding `text` in `p`'s home.
fn leave_marker(p: &Process, name: &str, text: &str) {
    let dir = p.parinn_home().join(MARKERS);
    crate::tests::owner_only_dirs(&dir);
    fs::write(dir.join(name), text).unwrap();
    crate::tests::set_mode(&dir.join(name), 0o600);
}

#[test]
fn a_start_marker_left_past_its_harness_timeout_is_counted_once() {
    let mut p = Process::new();
    leave_marker(&p, "SessionStart.aa", &(NOW - 6).to_string());
    leave_marker(&p, "UserPromptSubmit.bb", &(NOW - 4).to_string());
    leave_marker(&p, "junk", "x");
    leave_marker(&p, ".SessionStart.cc", &(NOW - 60).to_string());

    hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });
    hook(&mut p, "SessionStart", |p| {
        session_start(p, "s2", "startup")
    });

    let unfinished = entries(&p)
        .into_iter()
        .filter(|(c, _)| c.as_deref() == Some("hook_unfinished"))
        .count();
    assert_eq!(unfinished, 2, "the stale marker and the one without a time");
    let mut left = markers(&p);
    left.sort();
    assert_eq!(left, [".SessionStart.cc", "UserPromptSubmit.bb"]);
}

#[test]
fn parinn_status_and_the_mcp_server_as_it_starts_count_a_marker_left_behind() {
    let p = Process::new();
    leave_marker(&p, "PermissionRequest.aa", &(NOW - 31).to_string());
    assert_eq!(p.invoke(&["status"], &DEV).0, EXIT_OK);
    leave_marker(&p, "SessionStart.bb", &(NOW - 6).to_string());
    p.invoke(&["mcp"], &DEV);

    let unfinished = entries(&p)
        .into_iter()
        .filter(|(c, _)| c.as_deref() == Some("hook_unfinished"))
        .count();
    assert_eq!(unfinished, 2);
    assert_eq!(markers(&p), Vec::<String>::new());
}

/// Holds a `SessionStart` hook observation of `session` made at `at`.
fn hold_start(p: &Process, session: &str, at: u64) {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    home.create().unwrap();
    let observation = Observation {
        hook: "SessionStart".into(),
        harness_session: session.into(),
        transcript_path: "/t".into(),
        cwd: "/w".into(),
        offset: None,
        source: Some("startup".into()),
        prompt_id: None,
        attended: false,
        ancestry: None,
        harness: None,
        at,
    };
    store(p).hold_observation(&observation, None).unwrap();
}

fn untaken(p: &Process) -> usize {
    entries(p)
        .into_iter()
        .filter(|(c, _)| c.as_deref() == Some("session_binding_not_taken"))
        .count()
}

#[test]
fn a_session_binding_no_mcp_server_took_within_30_s_is_counted_once() {
    let mut p = Process::new();
    hold_start(&p, "old", NOW - 31);
    hold_start(&p, "recent", NOW - 29);

    hook(&mut p, "UserPromptSubmit", |p| {
        user_prompt_submit(p, "old", "p1", "hi")
    });
    assert_eq!(untaken(&p), 1);
    hook(&mut p, "UserPromptSubmit", |p| {
        user_prompt_submit(p, "old", "p2", "hi")
    });
    assert_eq!(untaken(&p), 1);

    p.now = NOW + 2;
    assert_eq!(p.invoke(&["status"], &DEV).0, EXIT_OK);
    assert_eq!(untaken(&p), 2, "parinn status counts the recent one later");
}

#[test]
fn parinn_ingest_counts_an_untaken_session_binding() {
    let p = Process::new();
    hold_start(&p, "old", NOW - 40);
    let path = p.transcript("{\"a\":1}\n");

    assert_eq!(p.invoke(&["ingest", "--path", &path], &DEV).0, EXIT_OK);

    assert_eq!(untaken(&p), 1);
}

/// A transcript of harness session `s1` in deployment mode
/// `interactive`, its prompt `p1` held unattended by the hook handlers,
/// whose two lines both carry `p1`: one matches, one conflicts.
fn prompt_matched_twice(p: &mut Process) -> PathBuf {
    p.configure("[node]\ndeployment_mode = \"interactive\"\n");
    let (accepted, _, err) = p.invoke(&["configuration", "accept"], &DEV);
    assert_eq!(accepted, EXIT_OK, "{err}");
    hook(p, "SessionStart", |p| session_start(p, "s1", "startup"));
    hook(p, "UserPromptSubmit", |p| {
        user_prompt_submit(p, "s1", "p1", "hello")
    });
    let recorded = fs::read_to_string(
        testkit::repo_root()
            .join("testdata/claude-code/2.1.296/transcripts/rewind/1-before-rewind.jsonl"),
    )
    .unwrap();
    let typed: Value = recorded
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|l| l["promptSource"] == "typed")
        .unwrap();
    let line = |text: &str| {
        let mut line = typed.clone();
        line["message"]["content"] = json!(text);
        line["promptId"] = json!("p1");
        line["uuid"] = json!(text);
        line.to_string()
    };
    let path = transcript_path(p, "s1");
    fs::write(&path, format!("{}\n{}\n", line("hello"), line("again"))).unwrap();

    path
}

#[test]
fn ingest_matches_a_prompt_and_counts_a_conflict_and_a_missing_attended_mark() {
    let mut p = Process::new();
    let path = prompt_matched_twice(&mut p);

    let (code, _, err) = p.invoke(&["ingest", "--path", &path.display().to_string()], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    let counted: Vec<Option<String>> = audited(&p).into_iter().map(|e| e.counter).collect();
    for counter in ["prompt_match_conflicts", "harness_session_not_attended"] {
        assert!(
            counted.contains(&Some(counter.to_owned())),
            "{counter}: {counted:?}"
        );
    }
}

#[test]
fn a_held_fork_is_read_again_once_its_owner_is_ingested() {
    let mut p = Process::new();
    hook(&mut p, "SessionStart", |p| session_start(p, "fork", "fork"));
    let numbered = |session: &str| -> String {
        (1..=3)
            .map(|n| {
                format!(
                    "{}\n",
                    json!({"type": "x", "uuid": format!("u{n}"), "sessionId": session})
                )
            })
            .collect()
    };
    let (parent, fork) = (transcript_path(&p, "parent"), transcript_path(&p, "fork"));
    fs::write(&parent, numbered("parent")).unwrap();
    fs::write(&fork, numbered("fork")).unwrap();
    let shown = |path: &PathBuf| path.display().to_string();

    p.invoke(&["ingest", "--path", &shown(&fork)], &DEV);
    let held_first = store(&p)
        .source(&fs::canonicalize(&fork).unwrap().to_string_lossy())
        .unwrap()
        .unwrap()
        .held_since;
    p.invoke(&["ingest", "--path", &shown(&parent)], &DEV);

    assert!(held_first.is_some());
    let after = store(&p)
        .source(&fs::canonicalize(&fork).unwrap().to_string_lossy())
        .unwrap()
        .unwrap();
    assert_eq!(after.held_since, None, "released by the re-read");
}

#[test]
fn a_hanging_handler_answers_empty_by_its_budget() {
    let mut p = Process::new();
    p.hook_hangs = true;
    p.hook_budget = Some(Duration::from_millis(200));

    let ran = hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(
        entries(&p),
        [(
            Some("hook_budget_exceeded".into()),
            "hook handler ran past its hook budget and answered empty".into()
        )]
    );
    assert_eq!(markers(&p), Vec::<String>::new());
}

#[test]
fn the_working_directory_and_transcript_path_are_redacted_before_they_are_held() {
    let mut p = Process::new();
    let mut input: Value = serde_json::from_str(&session_start(&p, "s1", "startup")).unwrap();
    input["cwd"] = json!("/w/AKIAIOSFODNN7EXAMPLE");

    hook(&mut p, "SessionStart", |_| input.to_string());

    let held = held(&p);
    assert_eq!(held[0].observation.cwd, "/w/[REDACTED:aws_access_key]");
    assert!(stored(&p).contains("/w/[REDACTED:aws_access_key]"));
    assert!(!stored(&p).contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn the_principals_redaction_patterns_redact_what_a_hook_holds() {
    let mut p = Process::new();
    p.configure("[redaction]\nextra_patterns = [\"ACME-[0-9]{8}\"]\n");
    let mut input: Value = serde_json::from_str(&session_start(&p, "s1", "startup")).unwrap();
    input["cwd"] = json!("/w/ACME-12345678");

    hook(&mut p, "SessionStart", |_| input.to_string());

    assert_eq!(held(&p)[0].observation.cwd, "/w/[REDACTED:extra_pattern_1]");
}

#[test]
fn a_hook_handler_leaves_the_stores_log_to_the_next_opener() {
    let mut p = Process::new();

    hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });

    let log = fs::metadata(p.parinn_home().join("store.db-wal")).unwrap();
    assert!(log.len() > 0, "its commit is in the log, not checkpointed");
    assert_eq!(held(&p).len(), 1, "the next opener reads what it held");
}

/// What the redaction rules `redactor` leave of `text`.
fn redacted_by(redactor: Option<Redactor>, text: &str) -> Option<String> {
    redactor.map(|r| r.redact_text(text).text)
}

#[test]
fn redaction_rules_compiled_for_the_patterns_in_force_are_kept_else_built_again() {
    let patterns = vec!["ACME-[0-9]{8}".to_owned()];
    let without = || Redactor::new(&[]).ok().map(|(r, _)| r);
    let text = "id ACME-12345678";

    let kept = redactor_for(&patterns, Some((patterns.clone(), without())));
    let other = redactor_for(&patterns, Some((Vec::new(), without())));
    let failed = redactor_for(&patterns, Some((patterns.clone(), None)));
    let unsent = redactor_for(&patterns, None);

    // Built for `patterns`, the compiled rules are kept as they are.
    assert_eq!(redacted_by(kept, text).as_deref(), Some(text));
    let built = Some("id [REDACTED:extra_pattern_1]");
    assert_eq!(redacted_by(other, text).as_deref(), built);
    assert_eq!(redacted_by(failed, text).as_deref(), built);
    assert_eq!(redacted_by(unsent, text).as_deref(), built);
}

#[test]
fn redaction_rules_compile_on_a_thread_that_sends_them_with_their_patterns() {
    let patterns = vec!["ACME-[0-9]{8}".to_owned()];

    let (built_for, redactor) = compile(patterns.clone())
        .recv_timeout(Duration::from_secs(30))
        .unwrap();

    assert_eq!(built_for, patterns);
    assert_eq!(
        redacted_by(redactor, "id ACME-12345678").as_deref(),
        Some("id [REDACTED:extra_pattern_1]")
    );
}

#[test]
fn a_store_a_stopped_hook_handler_made_no_schema_in_reads_as_holding_nothing() {
    let p = Process::new();
    crate::tests::owner_only_dirs(&p.parinn_home());
    let store = p.parinn_home().join("store.db");
    fs::write(&store, "").unwrap();
    crate::tests::set_mode(&store, 0o600);

    let (code, out, err) = p.invoke(&["status"], &DEV);

    assert_eq!(code, EXIT_OK, "{err}");
    assert!(out.contains("home: "), "{out}");
}

#[test]
fn a_source_names_only_a_known_start_and_a_budget_parses_as_milliseconds() {
    for source in SOURCES {
        assert_eq!(source_name(source), source);
    }
    assert_eq!(source_name("x"), "other");
    assert_eq!(budget_override(Some("75")), Some(Duration::from_millis(75)));
    assert_eq!(budget_override(Some("x")), None);
    assert_eq!(budget_override(None), None);
}

#[test]
fn a_transcript_size_that_cannot_be_read_is_none_and_counted() {
    let p = Process::new();
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    home.create().unwrap();
    let sources = Sources {
        configuration: p.dir.path().join("configuration.toml"),
        repository: None,
        managed: p.dir.path().join("managed"),
        running: configuration::source::OsUser::Known(u32::MAX),
    };
    let job = Job {
        hook: Hook::SessionStart,
        text: String::new(),
        home: None,
        os_home: None,
        now: NOW,
        attended: false,
        ancestry: || None,
        panics: false,
        hangs: false,
        deadline: now_instant(),
    };
    let recording = Recording::new(&job, &home, &sources);

    assert_eq!(recording.size(Ok(7)), Some(7));
    assert_eq!(recording.size(Err(io::ErrorKind::NotFound.into())), Some(0));
    assert_eq!(entries(&p), Vec::new());
    assert_eq!(
        recording.size(Err(io::ErrorKind::PermissionDenied.into())),
        None
    );
    assert_eq!(
        entries(&p),
        [(
            Some("hook_failed".into()),
            "transcript path could not be read".into()
        )]
    );
}

#[test]
fn a_hook_observation_the_store_cannot_hold_is_counted_and_the_hook_answers_empty() {
    let mut p = Process::new();
    hold_start(&p, "old", NOW - 31);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute_batch("DROP TABLE hook_observations").unwrap();
    drop(conn);

    let ran = hook(&mut p, "SessionStart", |p| {
        session_start(p, "s1", "startup")
    });

    assert_eq!(ran, (EXIT_OK, String::new(), String::new()));
    assert_eq!(
        entries(&p),
        [(
            Some("hook_failed".into()),
            "hook observation could not be held".into()
        )],
        "no untaken session binding is counted from a store that cannot list them"
    );
}

#[test]
fn session_bindings_a_store_cannot_list_are_left_uncounted() {
    let p = Process::new();
    hold_start(&p, "old", NOW - 31);
    let mut store = store(&p);
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute_batch("DROP TABLE hook_observations")
        .unwrap();
    let audits = std::cell::Cell::new(0);

    count_untaken(&mut store, NOW, &|_| audits.set(audits.get() + 1));

    assert_eq!(audits.get(), 0);
}

#[test]
fn a_prompt_input_without_its_prompt_has_no_prompt_commitment() {
    let input = HookInput {
        session_id: Some("s1".into()),
        prompt_id: Some("p1".into()),
        ..HookInput::default()
    };
    let redactor = Redactor::new(&[]).unwrap().0;

    assert_eq!(committed(&input, &redactor), None);
}

#[test]
fn ingest_says_which_prompt_match_it_cannot_audit() {
    let mut p = Process::new();
    let path = prompt_matched_twice(&mut p);
    // Nothing was audited yet, so a file takes the audit log's place.
    let audit_dir = p.parinn_home().join("audit");
    fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);

    let (_, _, err) = p.invoke(&["ingest", "--path", &path.display().to_string()], &DEV);

    assert!(
        err.contains("parinn ingest: audit a prompt match: "),
        "{err}"
    );
}

#[test]
fn an_untaken_session_binding_says_whether_any_mcp_server_ever_took_one() {
    let p = Process::new();
    hold_start(&p, "old", NOW - 31);
    let mut store = store(&p);
    let details = std::cell::RefCell::new(Vec::new());
    let say = |o: &Occurrence| {
        if let Occurrence::Degraded { detail, .. } = o {
            details.borrow_mut().push(*detail);
        }
    };

    count_untaken(&mut store, NOW, &say);
    // A server took a binding since; the next untaken one is not the first.
    hold_start(&p, "taken", NOW - 40);
    store.take_bindings(0).unwrap();
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute(
            "UPDATE hook_observations SET taken = 1 WHERE harness_session = 'taken'",
            [],
        )
        .unwrap();
    hold_start(&p, "later", NOW - 31);
    count_untaken(&mut store, NOW, &say);

    assert_eq!(
        *details.borrow(),
        [
            "no MCP server has taken a session binding on this node",
            "no MCP server took the session binding within 30 s"
        ]
    );
}
