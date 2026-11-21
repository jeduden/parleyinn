use std::fs;

use super::*;
use crate::TempDir;

/// A stand-in for the core: `mcp` answers each request with its own id
/// and the server name in `STUB_NAME`, `hook <name>` saves its input
/// as `hooked-<name>` in its working directory, the harness's home.
const STUB: &str = r#"#!/bin/sh
case "$1" in
  mcp)
    [ -z "$STUB_SILENT" ] || { cat > /dev/null; exit 0; }
    while IFS= read -r line; do
      id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9]*\).*/\1/p')
      [ -n "$id" ] || continue
      printf '{"jsonrpc":"2.0","id":%s,"result":{"serverInfo":{"name":"%s"}}}\n' "$id" "${STUB_NAME:-parinn}"
    done
    [ -z "$STUB_HANG" ] || sleep 30 ;;
  hook) cat > "hooked-$2" ;;
esac
"#;

/// A home holding the stub core, and the stub's path.
fn world() -> (TempDir, PathBuf) {
    let home = TempDir::new().unwrap();
    let core = home.path().join("core");
    write_executable(&core, STUB).unwrap();

    (home, core)
}

#[test]
fn a_started_server_answers_initialize_and_stops_when_its_input_closes() {
    let (home, core) = world();

    let harness = Harness::start(home.path(), &core, true, &[]).unwrap();

    harness.stop().unwrap();
}

#[test]
fn a_request_returns_its_own_reply() {
    let (home, core) = world();
    let mut harness = Harness::start(home.path(), &core, true, &[]).unwrap();

    let reply = harness.request("tools/list", json!({})).unwrap();

    assert_eq!(reply["result"]["serverInfo"]["name"], "parinn");
    harness.stop().unwrap();
}

#[test]
fn a_reply_skips_the_replies_to_other_requests() {
    let (home, core) = world();
    let mut harness = Harness::start(home.path(), &core, true, &[]).unwrap();
    let _first = harness.send("a", json!({})).unwrap();
    let second = harness.send("b", json!({})).unwrap();

    let reply = harness.reply(second).unwrap();

    assert_eq!(reply["id"], second);
    harness.kill().unwrap();
}

#[test]
fn a_server_started_later_initializes_then() {
    let (home, core) = world();
    let mut harness = Harness::start(home.path(), &core, false, &[]).unwrap();

    harness.start_mcp().unwrap();

    harness.stop().unwrap();
}

#[test]
fn a_server_that_is_not_parinn_fails_the_start() {
    let (home, core) = world();

    let err = Harness::start(home.path(), &core, true, &[("STUB_NAME", "other")]).unwrap_err();

    assert!(err.contains("answered initialize"), "{err}");
}

#[test]
fn a_hook_runs_as_a_child_of_the_harness_on_its_input() {
    let (home, core) = world();
    let mut harness = Harness::start(home.path(), &core, false, &[]).unwrap();

    harness
        .hook("SessionStart", &json!({"session_id": "s"}))
        .unwrap();

    let saved = fs::read_to_string(home.path().join("hooked-SessionStart")).unwrap();
    assert_eq!(saved.trim(), r#"{"session_id":"s"}"#);
    harness.stop().unwrap();
}

#[test]
fn each_harness_in_a_home_has_a_directory_of_its_own() {
    let (home, core) = world();

    let first = Harness::start(home.path(), &core, false, &[]).unwrap();
    let second = Harness::start(home.path(), &core, false, &[]).unwrap();

    assert!(home.path().join("harnesses/h0/claude").is_file());
    assert!(home.path().join("harnesses/h1/claude").is_file());
    first.kill().unwrap();
    second.kill().unwrap();
}

#[test]
fn the_start_env_reaches_the_harness_children() {
    let (home, core) = world();
    let err = Harness::start(home.path(), &core, true, &[("STUB_NAME", "x")]).unwrap_err();

    assert!(err.contains(r#""name":"x""#), "{err}");
}

#[test]
fn a_request_the_server_never_answers_fails_after_the_patience() {
    let (home, core) = world();
    let mut harness =
        Harness::start_within(Duration::from_millis(300), home.path(), &core, true, &[]).unwrap();
    let id = harness.send("a", json!({})).unwrap();

    harness.reply(id).unwrap();
    let err = harness.reply(id).unwrap_err();

    assert_eq!(err, format!("the MCP server did not answer request {id}"));
    harness.kill().unwrap();
}

#[test]
fn a_hook_that_never_finishes_fails_after_the_patience() {
    let (home, core) = world();
    let mut harness =
        Harness::start_within(Duration::from_millis(300), home.path(), &core, false, &[]).unwrap();
    // The harness ends before it reads the request, so nothing renames it.
    fs::write(home.path().join("harnesses/h0/stop"), "").unwrap();
    std::thread::sleep(Duration::from_millis(300));

    let err = harness.hook("Stop", &json!({})).unwrap_err();

    assert_eq!(err, "waited 0 s for the hook Stop to finish");
    harness.kill().unwrap();
}

#[test]
fn a_server_that_outlives_its_input_fails_the_stop() {
    let (home, core) = world();
    let harness = Harness::start_within(
        Duration::from_millis(500),
        home.path(),
        &core,
        true,
        &[("STUB_HANG", "1")],
    )
    .unwrap();

    let err = harness.stop().unwrap_err();

    assert_eq!(err, "the MCP server did not stop when its input closed");
}

#[test]
fn a_ready_wait_returns_at_once() {
    wait_until("nothing", || Ok(true)).unwrap();
}

#[test]
fn a_wait_that_never_holds_names_what_it_waited_for() {
    let err = wait_up_to(Duration::from_millis(120), "the thing", || Ok(false)).unwrap_err();

    assert_eq!(err, "waited 0 s for the thing");
}

#[test]
fn a_failing_check_ends_the_wait() {
    let err = wait_up_to(Duration::from_secs(5), "x", || Err("broke".to_owned())).unwrap_err();

    assert_eq!(err, "broke");
}

#[test]
fn a_server_that_never_answers_initialize_fails_the_start() {
    let (home, core) = world();

    let err = Harness::start_within(
        Duration::from_millis(300),
        home.path(),
        &core,
        true,
        &[("STUB_SILENT", "1")],
    )
    .unwrap_err();

    assert_eq!(err, "the MCP server did not answer request 1");
}

#[test]
fn a_harness_names_its_directory_when_printed() {
    let (home, core) = world();
    let harness = Harness::start(home.path(), &core, false, &[]).unwrap();

    let shown = format!("{harness:?}");

    assert!(shown.starts_with("Harness { dir: "), "{shown}");
    assert!(shown.contains("harnesses/h0"), "{shown}");
    harness.kill().unwrap();
}

#[test]
fn the_json_lines_of_the_harness_output_are_forwarded_in_order() {
    let (tx, rx) = mpsc::channel();

    forward("{\"id\":1}\nnot json\n{\"id\":2}\n".as_bytes(), &tx);

    assert_eq!(
        rx.try_iter().collect::<Vec<_>>(),
        [json!({"id": 1}), json!({"id": 2})]
    );
}

#[test]
fn forwarding_ends_when_nothing_receives_any_more() {
    let (tx, rx) = mpsc::channel();
    drop(rx);

    forward("{\"id\":1}\n{\"id\":2}\n".as_bytes(), &tx);
}
