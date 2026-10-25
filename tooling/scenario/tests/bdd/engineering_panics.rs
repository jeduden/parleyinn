//! The steps of ENG-04: a panic in a hook handler stops at the hook's
//! entry point, which fails open, writes nothing to stderr, the default
//! panic message included, and audits it. The development build of the
//! core panics in its hook handler when `PARINN_TEST_FAULT` is
//! `hook-panic`; the scenarios start that build.

use audit::log::Log;
use cucumber::{given, then};

use crate::World;
use crate::record::{home, last_run, set_env};

#[given(regex = r"^a hook handler that panics$")]
fn hook_handler_panics(w: &mut World) {
    set_env(w, "PARINN_TEST_FAULT", "hook-panic");
}

#[then(regex = r"^the hook output is empty$")]
fn hook_output_empty(w: &mut World) -> Result<(), String> {
    let out = last_run(w)?;
    if out.stdout.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the hook wrote {:?}",
        String::from_utf8_lossy(&out.stdout)
    ))
}

#[then(regex = r"^the hook handler wrote nothing to stderr, the default panic message included$")]
fn hook_stderr_empty(w: &mut World) -> Result<(), String> {
    let out = last_run(w)?;
    if out.stderr.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the hook wrote to stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    ))
}

#[then(regex = r#"^an audit entry records "([^"]+)"$"#)]
fn audit_entry_records(w: &mut World, what: String) -> Result<(), String> {
    let entries = Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    if entries.iter().any(|e| e.detail == what || e.kind == what) {
        return Ok(());
    }
    Err(format!(
        "no audit entry records {what:?} among {} entries",
        entries.len()
    ))
}
