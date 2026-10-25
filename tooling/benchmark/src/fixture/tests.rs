use super::*;
use testkit::TempDir;

#[test]
fn a_transcript_repeats_the_recorded_lines_each_copy_with_its_own_uuids() {
    let recorded = RECORDED.lines().count();

    let text = transcript(recorded + 2);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), recorded + 2);
    let first_uuid = RECORDED
        .lines()
        .find(|l| l.contains("\"uuid\":\""))
        .unwrap();
    let at = RECORDED.lines().position(|l| l == first_uuid).unwrap();
    assert!(lines[at].contains("\"uuid\":\"c0-"));
    assert!(text.ends_with('\n'));
    assert!(transcript(0).is_empty());
}

#[test]
fn a_hook_input_names_the_homes_transcript_in_its_hooks_recorded_shape() {
    let base = TempDir::new().unwrap();
    let home = Home::with_transcript(base.path(), "x\n").unwrap();
    let transcript = home.env()[0]
        .1
        .join(".claude/projects/-w-bench/bench.jsonl");

    let start = home.hook_input("SessionStart");
    let prompt = home.hook_input("UserPromptSubmit");
    let other = home.hook_input("PermissionRequest");

    assert!(start.contains(&json_string(&transcript.to_string_lossy())));
    assert!(start.ends_with(r#""hook_event_name":"SessionStart","source":"startup"}"#));
    assert!(prompt.contains(r#""prompt":"Reply with just the word A1.""#));
    assert!(other.ends_with(r#""hook_event_name":"PermissionRequest"}"#));
    assert_eq!(json_string("a\"b\\c\u{1}"), r#""a\"b\\c\u0001""#);
}

#[test]
fn a_home_holds_its_transcript_and_is_removed_when_dropped() {
    let base = TempDir::new().unwrap();

    let home = Home::with_transcript(base.path(), "x\n").unwrap();
    let [(home_var, root), (parinn_var, parinn)] = home.env();

    assert_eq!((home_var, parinn_var), ("HOME", "PARINN_HOME"));
    assert_eq!(parinn, root.join(".parinn"));
    let placed = root.join(".claude/projects/-w-bench/bench.jsonl");
    assert_eq!(fs::read_to_string(placed).unwrap(), "x\n");
    let other = Home::with_transcript(base.path(), "y\n").unwrap();
    assert_ne!(other.env()[0].1, root);
    drop(home);
    assert!(!root.exists());
    assert!(Home::with_transcript(&base.path().join("f\0"), "").is_err());
}

#[test]
fn a_home_reports_whether_its_audit_log_counted_a_counter() {
    let base = TempDir::new().unwrap();
    let home = Home::with_transcript(base.path(), "x\n").unwrap();
    assert!(!home.counted("hook_budget_exceeded").unwrap());

    let audit = home.env()[1].1.join("audit");
    fs::create_dir_all(&audit).unwrap();
    fs::write(
        audit.join("audit-000001.jsonl"),
        "{\"counter\":\"other\"}\n",
    )
    .unwrap();
    assert!(!home.counted("hook_budget_exceeded").unwrap());

    fs::write(
        audit.join("audit-000002.jsonl"),
        "{\"counter\":\"hook_budget_exceeded\"}\n",
    )
    .unwrap();
    assert!(home.counted("hook_budget_exceeded").unwrap());
}

#[test]
fn the_search_requests_initialize_then_ask_event_search_in_the_room() {
    let lines: Vec<&str> = SEARCH_REQUESTS.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains(r#""method":"initialize""#));
    assert!(lines[1].contains(r#""name":"event_search""#));
    assert!(lines[1].contains(r#""scope":"room""#));
    assert!(SEARCH_REQUESTS.ends_with('\n'));
}
