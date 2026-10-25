//! The `parinn` executable. Everything it does is [`parinn::run`]; this
//! entry point only hands it the process's arguments, environment and
//! streams.

use std::io::{self, IsTerminal};
use std::process::ExitCode;
use std::sync::mpsc::{self, Receiver};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    if args.first().is_some_and(|a| a == "hook") {
        // A hook handler writes nothing to stderr, the default panic
        // message included (§9.1, ENG-04); its entry point catches the
        // panic and audits it.
        std::panic::set_hook(Box::new(|_| {}));
    }
    // A development build's seam for the tests: the clock stands still
    // at the time it names; a release build reads the wall clock.
    let fixed: Option<u64> = cfg!(debug_assertions)
        .then(|| std::env::var("PARINN_TEST_CLOCK").ok())
        .flatten()
        .and_then(|t| t.parse().ok());
    let clock = move || fixed.unwrap_or_else(now);
    let ctx = parinn::Context {
        build: &parinn::BUILD,
        parinn_home: std::env::var_os("PARINN_HOME"),
        home: std::env::var_os("HOME"),
        home_id: std::env::var_os("PARINN_HOME_ID"),
        // The development build's seam for the `@ENG-04` scenario; a
        // release build never reads it.
        // The development build's seam for the `@NFR-02` scenario.
        hook_hangs: cfg!(debug_assertions)
            && std::env::var_os("PARINN_TEST_FAULT").is_some_and(|v| v == "hook-hang"),
        hook_panics: cfg!(debug_assertions)
            && std::env::var_os("PARINN_TEST_FAULT").is_some_and(|v| v == "hook-panic"),
        terminal: terminal(),
        working_dir: std::env::current_dir().ok(),
        // A development build's seam for the tests; a release build
        // reads only the documented path.
        managed_policy: parinn::managed_policy(
            cfg!(debug_assertions)
                .then(|| std::env::var_os("PARINN_TEST_MANAGED_POLICY"))
                .flatten(),
        ),
        // A development build's seam for the tests; a release build
        // reads only this OS's node identity.
        node_identity: cfg!(debug_assertions)
            .then(|| std::env::var_os("PARINN_TEST_NODE_IDENTITY"))
            .flatten()
            .map(std::path::PathBuf::from),
        os_user: os_user(),
        input: &stdin_lines,
        clock: &clock,
        // The harness's attended mark (ASM-22), which only a hook
        // handler reads (PRV-04).
        attended: std::env::var_os("CLAUDE_CODE_SESSION_ATTENDED").is_some_and(|v| v == "1"),
        // A development build's seam for the tests; a release build
        // keeps each hook's own budget (NFR-02).
        ancestry: hook::ancestry::own,
        // A development build's seam for the tests; a release build's
        // live ingest reads every line.
        live_limit: cfg!(debug_assertions)
            .then(|| std::env::var("PARINN_TEST_INGEST_LIMIT").ok())
            .flatten()
            .and_then(|n| n.parse().ok()),
        hook_budget: parinn::budget_override(
            cfg!(debug_assertions)
                .then(|| std::env::var("PARINN_TEST_HOOK_BUDGET_MS").ok())
                .flatten()
                .as_deref(),
        ),
        // A development build's seam for the `@SEC-04` scenario, which
        // needs a search past its deadline; a release build keeps 2 s.
        query_deadline: cfg!(debug_assertions)
            .then(|| std::env::var("PARINN_TEST_QUERY_DEADLINE_MS").ok())
            .flatten()
            .and_then(|ms| ms.parse().ok())
            .map(std::time::Duration::from_millis),
    };

    let log = parinn::structured_log(
        std::env::var_os("PARINN_HOME"),
        std::env::var_os("HOME"),
        std::env::var_os("PARINN_HOME_ID"),
        parinn::LogSettings {
            managed: ctx.managed_policy.clone(),
            os_user: ctx.os_user,
        },
        now,
    );
    let log = std::sync::Arc::new(log);
    let dispatch = tracing::Dispatch::from(log.clone());
    let code = tracing::dispatcher::with_default(&dispatch, || {
        parinn::run(args, &ctx, &mut io::stdout(), &mut io::stderr())
    });
    parinn::audit_dropped_log_lines(&ctx, log.dropped());

    ExitCode::from(code)
}

/// The wall clock, in seconds since the Unix epoch, for audit entries.
#[expect(
    clippy::disallowed_methods,
    reason = "an audit entry records when it was made; no derived artifact reads it (I10)"
)]
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The running OS user, on an OS whose file modes ParleyInn checks; unknown
/// there when it cannot be learnt, which refuses managed policy.
fn os_user() -> configuration::source::OsUser {
    use configuration::source::OsUser;
    #[cfg(unix)]
    return administration::home::running_uid().map_or(OsUser::Unknown, OsUser::Known);
    #[cfg(not(unix))]
    OsUser::Unchecked
}

/// Whether the process's standard input and output are both a terminal
/// (OWN-12).
fn terminal() -> bool {
    #[expect(
        clippy::disallowed_methods,
        reason = "asking whether stdin is a terminal reads nothing, so it waits for nothing"
    )]
    let stdin = io::stdin();

    stdin.is_terminal() && io::stdout().is_terminal()
}

/// The process's input, one line at a time, read on a thread of its
/// own so its reader waits for each line with a deadline (ENG-03), and
/// no line held past its limit (SEC-05).
fn stdin_lines() -> Receiver<io::Result<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        #[expect(
            clippy::disallowed_methods,
            reason = "the reading thread blocks on stdin; its reader waits with recv_timeout"
        )]
        let stdin = io::stdin();
        // Reading stops once the reader is gone.
        parinn::bounded_lines(stdin.lock(), parinn::INPUT_LINE_LIMIT)
            .map_while(|line| tx.send(line).ok())
            .for_each(drop);
    });

    rx
}
