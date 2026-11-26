//! The boundary's tests of sockets opened by a program's descendants and its preflight.
//! The rest of the module's tests, moved from `tests.rs` to keep each file in its limit.

use super::*;

#[test]
fn a_socket_a_grandchild_opens_fails_the_start() {
    let c = must_confine();
    let home = TempDir::new().unwrap();
    // Each shell runs one more command after its child, so each forks:
    // the probe runs two processes below the one the trace started.
    let line = format!("sh -c \"{}; true\"; true", probe_line(PROBE_TEST));

    let calls = socket_calls(c.run(Path::new("sh"), &["-c", &line], home.path()));

    assert!(denied(&calls, "socket("), "{calls:?}");
}

#[test]
fn a_socket_a_thread_of_the_program_opens_fails_the_start() {
    let c = must_confine();
    let home = TempDir::new().unwrap();
    let args = [
        "--exact",
        "boundary::tests::probe_in_a_thread_opens_a_socket",
        "--ignored",
    ];

    let calls = socket_calls(c.run(&this_test_binary(), &args, home.path()));

    assert!(denied(&calls, "socket("), "{calls:?}");
}

#[test]
fn a_socket_a_program_the_traced_one_execs_opens_fails_the_start() {
    let c = must_confine();
    let home = TempDir::new().unwrap();
    let line = format!("exec {}", probe_line(PROBE_TEST));

    let calls = socket_calls(c.run(Path::new("sh"), &["-c", &line], home.path()));

    assert!(denied(&calls, "socket("), "{calls:?}");
}

#[test]
fn a_unix_socket_and_a_socket_pair_are_denied_under_the_trace() {
    let c = must_confine();
    let home = TempDir::new().unwrap();
    let run = |test: &str| {
        let args = ["--exact", test, "--ignored"];
        socket_calls(c.run(&this_test_binary(), &args, home.path()))
    };

    let unix = run("boundary::tests::probe_opens_a_unix_socket");
    let pair = run("boundary::tests::probe_opens_a_socket_pair");

    assert!(denied(&unix, "socket(AF_UNIX"), "{unix:?}");
    assert!(denied(&pair, "socketpair("), "{pair:?}");
}

#[test]
fn preflight_fails_unless_the_trace_failed_a_socket() {
    let injected =
        "socket(AF_INET, SOCK_STREAM, IPPROTO_TCP) = -1 EACCES (Permission denied) (INJECTED)";
    let let_through = "socket(AF_INET, SOCK_STREAM, IPPROTO_TCP) = 3";

    assert_eq!(judge_preflight(&format!("123 {injected}\n")), Ok(()));
    for trace in ["", "\n", let_through] {
        let err = judge_preflight(trace).unwrap_err();
        assert!(
            err.starts_with("the trace did not fail the preflight's socket"),
            "{err}"
        );
    }
}

#[test]
fn preflight_fails_a_trace_that_follows_no_child() {
    let _ = must_confine();
    let unfollowed: Vec<&str> = TRACE.iter().copied().filter(|a| *a != "-f").collect();

    let err = preflight_traced(UNSHARE, &unfollowed).unwrap_err();

    assert!(
        err.starts_with("the trace did not fail the preflight's socket"),
        "{err}"
    );
}

#[test]
fn a_tool_that_outlives_its_deadline_is_killed_and_named() {
    let mut sleeper = Command::new("sleep");
    sleeper.arg("30");
    let started = std::time::Instant::now();

    let err = output_within(&mut sleeper, std::time::Duration::from_millis(200)).unwrap_err();

    assert!(started.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(err, "sleep did not finish within 0.2 s");
    let mut echo = Command::new("sh");
    echo.args(["-c", "echo out; echo err >&2"]);
    let out = output_within(&mut echo, std::time::Duration::from_secs(10)).unwrap();
    assert_eq!(
        (&out.stdout[..], &out.stderr[..]),
        (&b"out\n"[..], &b"err\n"[..])
    );
    let missing = output_within(
        &mut Command::new("parinn-no-such-tool"),
        std::time::Duration::from_secs(1),
    );
    assert!(
        missing
            .unwrap_err()
            .starts_with("start parinn-no-such-tool: ")
    );
}

#[test]
fn a_preflight_that_hangs_fails_with_what_hung() {
    let _ = must_confine();
    // The "trace" sleeps instead of starting the probe.
    let hung = ["sh", "-c", "sleep 30", "-o"];

    let err = preflight_within(UNSHARE, &hung, std::time::Duration::from_millis(300)).unwrap_err();

    assert_eq!(err, "unshare did not finish within 0.3 s");
}

#[test]
fn preflight_refuses_a_strace_older_than_its_minimum() {
    assert_eq!(
        strace_new_enough("strace -- version 6.8\nCopyright"),
        Ok(())
    );
    assert_eq!(strace_new_enough("strace -- version 5.3"), Ok(()));
    assert_eq!(strace_new_enough("strace -- version 10.0"), Ok(()));
    for old in [
        "strace -- version 5.2",
        "strace -- version 4.26",
        "no version",
    ] {
        let err = strace_new_enough(old).unwrap_err();
        assert!(err.contains("needs strace 5.3 or later"), "{err}");
    }
}

#[test]
fn a_spawned_program_that_exited_by_itself_with_no_trace_fails() {
    let dir = TempDir::new().unwrap();
    let mut started = unconfined()
        .spawn(Path::new("/bin/true"), &[], dir.path(), &[])
        .unwrap();
    started.confinement.mode = Mode::Confined;
    started.trace = dir.path().join("never-written");

    let err = started.checked(true).unwrap_err();

    assert!(matches!(err, Error::Trace { .. }), "{err}");
}
