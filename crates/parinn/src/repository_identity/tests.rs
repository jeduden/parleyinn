use std::fs;
use std::path::{Path, PathBuf};

use crate::tests::{DEV, Process};
use crate::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

/// A zlib stream of `data` in stored blocks.
fn zlib(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01, 1];
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(&(!(data.len() as u16)).to_le_bytes());
    out.extend_from_slice(data);
    let (mut a, mut b) = (1_u32, 0_u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

/// The body of a root commit of the empty tree with `message`.
fn root_body(message: &str) -> String {
    format!("tree {}\n\n{message}\n", "0".repeat(40))
}

/// The id the root commit with `message` has.
fn root_id(message: &str) -> String {
    ::repository::commit_object_id(root_body(message).as_bytes())
        .as_str()
        .to_owned()
}

/// A repository at `dir` with one root commit with `message`, on `main`,
/// stored under its real id.
fn repository(dir: &Path, message: Option<&str>) {
    let git = dir.join(".git");
    fs::create_dir_all(git.join("refs/heads")).unwrap();
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    if let Some(message) = message {
        let id = root_id(message);
        let body = root_body(message);
        let object = format!("commit {}\0{body}", body.len());
        let path = git.join("objects").join(&id[..2]).join(&id[2..]);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, zlib(object.as_bytes())).unwrap();
        fs::write(git.join("refs/heads/main"), format!("{id}\n")).unwrap();
    }
}

fn clone(p: &Process, name: &str, message: Option<&str>) -> PathBuf {
    let dir = p.dir.path().join(name);
    fs::create_dir_all(&dir).unwrap();
    repository(&dir, message);
    dir
}

fn run(p: &Process, args: &[&str]) -> (u8, String, String) {
    p.invoke(args, &DEV)
}

fn shown(p: &Process, dir: &Path) -> serde_json::Value {
    let (code, out, err) = run(
        p,
        &["repository", "show", &dir.display().to_string(), "--json"],
    );
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    serde_json::from_str(&out).unwrap()
}

#[test]
fn show_says_a_directory_is_unbound_with_the_identity_its_clone_names() {
    let p = Process::new();
    let a = root_id("a");
    let app = clone(&p, "app", Some("a"));

    let v = shown(&p, &app);

    assert_eq!(v["status"], "unbound");
    assert_eq!(v["repository"], a);
    let (code, text, _) = run(&p, &["repository", "show", &app.display().to_string()]);
    assert_eq!(code, EXIT_OK);
    assert_eq!(text.trim(), format!("unbound: {a}"));
}

#[test]
fn show_defaults_to_the_working_directory_and_names_a_provisional_identity() {
    let mut p = Process::new();
    let work = clone(&p, "work", None);
    p.working_dir = Some(work);

    let (code, out, _) = run(&p, &["repository", "show"]);

    assert_eq!(code, EXIT_OK);
    assert_eq!(out.trim(), "unbound: provisional");
}

#[test]
fn a_repository_in_the_home_directory_is_the_repository_of_no_directory_below_it() {
    let mut p = Process::new();
    // A dotfiles repository at the home directory, which is `HOME`.
    repository(p.dir.path(), Some("dotfiles"));
    let notes = p.dir.path().join("notes");
    fs::create_dir_all(&notes).unwrap();
    p.working_dir = Some(notes.clone());

    assert_eq!(shown(&p, &notes)["status"], "no repository");
    let (code, out, _) = run(&p, &["repository", "show"]);
    assert_eq!((code, out.trim()), (EXIT_OK, "no repository"));
    let (code, _, err) = run(&p, &["repository", "bind", &notes.display().to_string()]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("lies in no repository"), "{err}");
}

#[test]
fn show_names_a_directory_in_no_repository_and_one_it_cannot_read() {
    let p = Process::new();
    let plain = p.dir.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let bad = clone(&p, "bad", Some("a"));
    fs::write(
        bad.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();

    assert_eq!(shown(&p, &plain)["status"], "no repository");
    assert_eq!(shown(&p, &bad)["status"], "unreadable");
}

#[test]
fn a_widening_bind_needs_a_terminal() {
    let mut p = Process::new();
    p.terminal = false;
    let app = clone(&p, "app", Some("a"));

    let (code, _, err) = run(&p, &["repository", "bind", &app.display().to_string()]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("terminal"), "{err}");
}

#[test]
fn a_widening_bind_records_the_act_and_binds_the_directory() {
    let p = Process::new();
    let a = root_id("a");
    let app = clone(&p, "app", Some("a"));

    let (code, out, err) = run(
        &p,
        &["repository", "bind", &app.display().to_string(), "--json"],
    );

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["repository"], a);
    assert_eq!(shown(&p, &app)["status"], "bound");
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    let acts: i64 = conn
        .query_row(
            "SELECT count(*) FROM events WHERE kind = 'principal_act' AND content LIKE '%repository_bind%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(acts, 1);
}

#[test]
fn a_widening_bind_rebinds_after_a_change_and_may_name_a_commit_the_clone_lacks() {
    let p = Process::new();
    let a = root_id("a");
    let app = clone(&p, "app", Some("a"));
    let at = app.display().to_string();
    assert_eq!(run(&p, &["repository", "bind", &at]).0, EXIT_OK);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some("b"));
    assert_eq!(shown(&p, &app)["status"], "changed");

    assert_eq!(run(&p, &["repository", "bind", &at]).0, EXIT_OK);
    assert_eq!(shown(&p, &app)["status"], "bound");
    let lacking = clone(&p, "lacking", Some("b"));
    let (code, out, _) = run(
        &p,
        &[
            "repository",
            "bind",
            &lacking.display().to_string(),
            "--commit",
            &a,
            "--json",
        ],
    );
    assert_eq!(code, EXIT_OK);
    assert!(out.contains(&a), "{out}");
}

#[test]
fn a_bind_of_a_directory_in_no_repository_or_with_a_bad_commit_fails() {
    let p = Process::new();
    let plain = p.dir.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let app = clone(&p, "app", Some("a"));

    let (code, _, err) = run(&p, &["repository", "bind", &plain.display().to_string()]);
    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("lies in no repository"), "{err}");
    let (code, _, err) = run(
        &p,
        &[
            "repository",
            "bind",
            &app.display().to_string(),
            "--commit",
            "nope",
        ],
    );
    assert_eq!(code, EXIT_USAGE);
    assert!(err.contains("40 lowercase hexadecimal"), "{err}");
}

#[test]
fn the_repository_verbs_refuse_what_they_do_not_know() {
    let p = Process::new();
    let a = root_id("a");

    for args in [
        &["repository"][..],
        &["repository", "forget"][..],
        &["repository", "show", "--now"][..],
        &["repository", "show", "a", "b"][..],
        &["repository", "show", "--commit", a.as_str()][..],
        &["repository", "bind", "--commit"][..],
    ] {
        assert_eq!(run(&p, args).0, EXIT_USAGE, "{args:?}");
    }
}

/// Runs the `SessionStart` hook in `cwd`, and an MCP server takes the
/// session binding it records.
fn start_in(p: &mut Process, session: &str, cwd: &Path) {
    start_untaken(p, session, cwd);
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute("UPDATE hook_observations SET taken = 1", [])
        .unwrap();
}

/// Runs the `SessionStart` hook in `cwd`; no MCP server takes its
/// session binding.
fn start_untaken(p: &mut Process, session: &str, cwd: &Path) {
    let projects = p.dir.path().join(".claude/projects/-w-app");
    fs::create_dir_all(&projects).unwrap();
    let path = projects.join(format!("{session}.jsonl"));
    fs::write(
        &path,
        "{\"type\":\"atis-latch\",\"atis\":\"\",\"sessionId\":\"s\"}\n",
    )
    .unwrap();
    p.lines = vec![
        serde_json::json!({
            "session_id": session,
            "transcript_path": path,
            "cwd": cwd,
            "hook_event_name": "SessionStart",
            "source": "startup",
        })
        .to_string(),
    ];
    assert_eq!(p.invoke(&["hook", "SessionStart"], &DEV).0, EXIT_OK);
    p.lines = Vec::new();
}

/// The repository bind events the record holds.
fn binds(p: &Process) -> Vec<serde_json::Value> {
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    let mut statement = conn
        .prepare("SELECT content FROM events WHERE kind = 'repository_bind' ORDER BY seq")
        .unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(|c| serde_json::from_str(&c.unwrap()).unwrap())
        .collect()
}

fn counter(p: &Process, name: &str) -> u64 {
    let (_, out, _) = run(p, &["status", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    v["counters"][name]["total"].as_u64().unwrap_or(0)
}

#[test]
fn the_first_hook_event_in_a_clone_binds_its_identity_when_an_mcp_server_next_starts_once() {
    let mut p = Process::new();
    let a = root_id("a");
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);

    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);
    start_in(&mut p, "s2", &app);
    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);

    let binds = binds(&p);
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0]["repository"], a);
    assert_eq!(binds[0]["worktree_digest"].as_str().map(str::len), Some(64));
    assert_eq!(shown(&p, &app)["status"], "bound");
}

#[test]
fn an_mcp_server_binds_the_identity_of_the_clones_its_hooks_showed_too() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);

    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);

    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn a_change_of_identity_is_audited_and_counted_once_and_relinks_nothing() {
    let mut p = Process::new();
    let (a, b) = (root_id("a"), root_id("b"));
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    run(&p, &["mcp"]);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some("b"));
    start_in(&mut p, "s2", &app);

    run(&p, &["mcp"]);
    run(&p, &["mcp"]);

    assert_eq!(counter(&p, "repository_identity_changed"), 1);
    assert_eq!(binds(&p).len(), 1);
    let audit = crate::tests::audited(&p);
    assert!(
        audit
            .iter()
            .any(|e| e.detail.contains("repository identity changed")),
        "{audit:?}"
    );
    assert!(
        audit
            .iter()
            .all(|e| !e.detail.contains(&a) && !e.detail.contains(&b))
    );
}

#[test]
fn a_clone_that_cannot_be_read_is_counted_once() {
    let mut p = Process::new();
    let bad = clone(&p, "bad", Some("a"));
    fs::write(
        bad.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();
    start_in(&mut p, "s1", &bad);

    run(&p, &["mcp"]);
    run(&p, &["mcp"]);

    assert_eq!(counter(&p, "repository_read_failed"), 1);
    assert!(binds(&p).is_empty());
}

#[test]
fn only_hook_observations_not_yet_in_the_record_are_looked_at() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    run(&p, &["mcp"]);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some("b"));

    // No new hook event: the earlier observation is in the record, and
    // its clone is not walked again.
    run(&p, &["mcp"]);
    run(&p, &["mcp"]);

    assert_eq!(counter(&p, "repository_identity_changed"), 0);
}

#[test]
fn the_repository_verbs_report_a_home_they_cannot_open() {
    let p = Process::new();
    std::fs::create_dir_all(p.parinn_home()).unwrap();
    crate::tests::set_mode(&p.parinn_home(), 0o750);
    let app = p.dir.path().display().to_string();

    for verb in ["show", "bind"] {
        let (code, _, err) = run(&p, &["repository", verb, &app]);
        assert_eq!(code, EXIT_FAILURE, "{verb}");
        assert!(err.contains("lets other OS users in"), "{err}");
    }
}

#[test]
fn a_widening_bind_is_refused_while_a_run_leaves_a_risk_open() {
    let p = Process::new();
    crate::tests::transcript_in(&p, "-w-a", "s1.jsonl", b"{\"a\":1}\n");
    // A run only `parinn ingest` records leaves a residual risk open.
    assert_eq!(run(&p, &["ingest", "--all"]).0, EXIT_OK);
    let app = clone(&p, "app", Some("a"));

    let (code, _, err) = run(&p, &["repository", "bind", &app.display().to_string()]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("residual risk"), "{err}");
    assert_eq!(shown(&p, &app)["status"], "unbound");
}

#[test]
fn a_bind_the_audit_log_takes_no_note_of_is_recorded_and_reported() {
    let p = Process::new();
    let app = clone(&p, "app", Some("a"));
    fs::create_dir_all(p.parinn_home()).unwrap();
    crate::tests::set_mode(&p.parinn_home(), 0o700);
    let audit_dir = p.parinn_home().join("audit");
    let _ = fs::remove_dir_all(&audit_dir);
    fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);

    let (code, _, err) = run(&p, &["repository", "bind", &app.display().to_string()]);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("the bind is recorded, but the audit log took no note"),
        "{err}"
    );
}

#[test]
fn a_bind_the_record_refuses_is_counted_apart_from_a_clone_that_cannot_be_read() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    // The record takes no event from here on, so the bind fails to append.
    run(&p, &["status"]);
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER refuse BEFORE INSERT ON events WHEN new.kind = 'repository_bind'
         BEGIN SELECT RAISE(ABORT, 'no'); END",
    )
    .unwrap();
    drop(conn);

    run(&p, &["mcp"]);

    assert_eq!(counter(&p, "repository_bind_failed"), 1);
    assert_eq!(counter(&p, "repository_read_failed"), 0);
    assert!(binds(&p).is_empty());

    // Tried again at each start, counted once for the observation.
    run(&p, &["mcp"]);
    run(&p, &["mcp"]);
    assert_eq!(counter(&p, "repository_bind_failed"), 1);

    // Once the record takes the event, the next start binds it.
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute_batch("DROP TRIGGER refuse")
        .unwrap();
    run(&p, &["mcp"]);
    assert_eq!(binds(&p).len(), 1);
    assert_eq!(counter(&p, "repository_bind_failed"), 1);
}

#[test]
fn a_failed_bind_mark_that_cannot_be_made_still_counts_the_failure() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let (ctx, mut store, node) = node_and_held(&p);
    let held = store.hook_observations().unwrap();
    let conn = rusqlite::Connection::open(p.parinn_home().join("store.db")).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER refuse BEFORE INSERT ON events WHEN new.kind = 'repository_bind'
         BEGIN SELECT RAISE(ABORT, 'no'); END;
         CREATE TABLE repository_worktree_marks (mark TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TRIGGER no_marks BEFORE INSERT ON repository_worktree_marks
         BEGIN SELECT RAISE(ABORT, 'no marks'); END;",
    )
    .unwrap();

    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| false);

    assert_eq!(counter(&p, "repository_bind_failed"), 1);
}

#[test]
fn a_look_that_runs_out_of_the_shared_time_is_counted_as_that_and_not_as_a_broken_clone() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let ctx = crate::settings::tests::ctx_of(&p);
    let home = ctx.home().unwrap();
    home.create().unwrap();
    let mut store = store::Store::open(&home.store()).unwrap();
    let node =
        administration::node::Node::open(&home, &mut store, &mut key::random::OsEntropy).unwrap();
    let held = store.hook_observations().unwrap();

    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| true);

    assert_eq!(counter(&p, "repository_read_timed_out"), 1);
    assert_eq!(counter(&p, "repository_read_failed"), 0);
    assert!(binds(&p).is_empty());
}

#[test]
fn a_look_the_audit_log_takes_no_note_of_goes_on_and_leaves_the_record_as_it_was() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let ctx = crate::settings::tests::ctx_of(&p);
    let home = ctx.home().unwrap();
    let mut store = store::Store::open(&home.store()).unwrap();
    let node =
        administration::node::Node::open(&home, &mut store, &mut key::random::OsEntropy).unwrap();
    let held = store.hook_observations().unwrap();
    let audit_dir = p.parinn_home().join("audit");
    let _ = fs::remove_dir_all(&audit_dir);
    fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);

    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| true);

    assert!(binds(&p).is_empty());
}

/// What reports an occurrence to `ctx`'s audit log.
fn say<'a>(ctx: &'a crate::Context<'a>) -> impl Fn(&audit::entry::Occurrence) + 'a {
    move |occurrence| {
        let _ = ctx.audit(occurrence);
    }
}

/// A store with `p`'s node, and the observations it holds.
fn node_and_held(
    p: &Process,
) -> (
    crate::Context<'static>,
    store::Store,
    administration::node::Node,
) {
    let ctx = crate::settings::tests::ctx_of(p);
    let home = ctx.home().unwrap();
    home.create().unwrap();
    let mut store = store::Store::open(&home.store()).unwrap();
    let node =
        administration::node::Node::open(&home, &mut store, &mut key::random::OsEntropy).unwrap();
    (ctx, store, node)
}

#[test]
fn an_observation_already_in_the_record_is_still_looked_at_if_no_look_marked_it() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let (ctx, mut store, node) = node_and_held(&p);
    // A server's live ingest recorded the observation before any look.
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute("UPDATE hook_observations SET appended = 1", [])
        .unwrap();
    let held = store.hook_observations().unwrap();
    assert!(held.iter().all(|h| h.appended));

    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| false);

    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn a_look_that_ran_out_of_time_is_not_made_or_counted_again_for_the_same_observation() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let (ctx, mut store, node) = node_and_held(&p);

    for _ in 0..3 {
        let held = store.hook_observations().unwrap();
        super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| true);
    }

    assert_eq!(counter(&p, "repository_read_timed_out"), 1);
    // A new hook event for the directory looks again, with time.
    start_in(&mut p, "s2", &app);
    let held = store.hook_observations().unwrap();
    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| false);
    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn a_server_start_audits_a_change_it_finds_and_says_a_failed_audit_in_the_log() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let (ctx, mut store, _) = node_and_held(&p);
    let home = ctx.home().unwrap();
    super::bind_observed(&ctx, &home, &mut store);
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some("b"));
    start_in(&mut p, "s2", &app);

    super::bind_observed(&ctx, &home, &mut store);

    assert_eq!(counter(&p, "repository_identity_changed"), 1);
    // The audit log cannot take the next note; the start goes on.
    fs::remove_dir_all(app.join(".git")).unwrap();
    repository(&app, Some("c"));
    start_in(&mut p, "s3", &app);
    let audit_dir = p.parinn_home().join("audit");
    let _ = fs::remove_dir_all(&audit_dir);
    fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);
    super::bind_observed(&ctx, &home, &mut store);
    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn a_server_start_in_a_home_without_a_node_looks_at_nothing() {
    let p = Process::new();
    let ctx = crate::settings::tests::ctx_of(&p);
    let home = ctx.home().unwrap();
    home.create().unwrap();
    let mut store = store::Store::open(&home.store()).unwrap();

    super::bind_observed(&ctx, &home, &mut store);

    assert!(binds(&p).is_empty());
}

#[test]
fn a_look_the_store_cannot_mark_goes_on() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    let (ctx, mut store, node) = node_and_held(&p);
    let held = store.hook_observations().unwrap();
    super::bind_held(&say(&ctx), None, &mut store, &node, held.clone(), &|| false);
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute_batch(
            "DELETE FROM repository_worktree_marks WHERE mark LIKE 'looked:%';
             CREATE TRIGGER no_marks BEFORE INSERT ON repository_worktree_marks
             WHEN NEW.mark LIKE 'looked:%' BEGIN SELECT RAISE(ABORT, 'no marks'); END;",
        )
        .unwrap();

    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| false);

    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn a_hook_input_no_mcp_server_took_steers_no_read_of_the_directory_it_names() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));
    start_untaken(&mut p, "forged", &app);
    let (ctx, mut store, node) = node_and_held(&p);

    let held = store.hook_observations().unwrap();
    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| false);

    assert!(binds(&p).is_empty());
    assert_eq!(shown(&p, &app)["status"], "unbound");
    // Once a server takes it, the next start binds.
    rusqlite::Connection::open(p.parinn_home().join("store.db"))
        .unwrap()
        .execute("UPDATE hook_observations SET taken = 1", [])
        .unwrap();
    let held = store.hook_observations().unwrap();
    super::bind_held(&say(&ctx), None, &mut store, &node, held, &|| false);
    assert_eq!(binds(&p).len(), 1);
}

#[test]
fn show_writes_nothing_and_starts_no_node() {
    let mut p = Process::new();
    let app = clone(&p, "app", Some("a"));

    // A home that never started stays absent.
    assert_eq!(shown(&p, &app)["status"], "unbound");
    assert!(!p.parinn_home().exists());

    // A started node's store is left byte for byte as it was.
    start_in(&mut p, "s1", &app);
    run(&p, &["mcp"]);
    let before = fs::read(p.parinn_home().join("store.db")).unwrap();
    assert_eq!(shown(&p, &app)["status"], "bound");
    let after = fs::read(p.parinn_home().join("store.db")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn a_worktree_bound_again_after_a_node_identity_change_is_audited_and_counted_once() {
    let mut p = Process::new();
    let id = p.dir.path().join("machine-id");
    fs::write(&id, "first").unwrap();
    p.node_identity = Some(id.clone());
    let app = clone(&p, "app", Some("a"));
    start_in(&mut p, "s1", &app);
    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);
    fs::write(&id, "second").unwrap();

    start_in(&mut p, "s2", &app);
    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);
    assert_eq!(run(&p, &["mcp"]).0, EXIT_OK);

    assert_eq!(counter(&p, "repository_identity_rebound"), 1);
    assert_eq!(counter(&p, "repository_identity_changed"), 0);
    assert!(crate::tests::audited(&p).iter().any(|e| {
        e.detail
            .contains("worktree bound again after node key change")
    }));
}
