//! The steps of ADM-11: `parinn status` shows where the store lies and
//! how big it is, its schema version, the deployment mode, the failure
//! counters, whether the node identity is checked on this OS, and what
//! fork detection covers until the peer network arrives. The counter
//! the scenario names is raised by the core itself: a hook handler held
//! to a budget no handler keeps.

use cucumber::{given, then};
use serde_json::Value;

use crate::World;
use crate::record::{HOOK_BUDGET, home, last_run, run_core_with};
use crate::record_hooks::session_start;

/// Raises `hook_budget_exceeded` to `count` by running the core's
/// `SessionStart` handler with a hook budget of nothing, `count` times.
#[given(regex = r#"^the counter "hook_budget_exceeded" is (\d+)$"#)]
fn counter_is(w: &mut World, count: u64) -> Result<(), String> {
    // The agent's run is recorded first, so the node holds a store.
    crate::recall::record_runs(w)?;
    for n in 0..count {
        let input = session_start(w, &format!("status-run-{n}"), "startup")?;
        run_core_with(
            w,
            &["hook", "SessionStart"],
            Some(input.to_string().as_bytes()),
            &[(HOOK_BUDGET, "0")],
        )?;
    }
    Ok(())
}

/// What `parinn status --json` printed.
fn shown(w: &mut World) -> Result<Value, String> {
    let out = last_run(w)?;
    serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "parinn status --json printed no JSON: {e}: {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

#[then(
    regex = r#"^the output shows the store location, its size, the schema version and the deployment mode "([^"]+)"$"#
)]
fn shows_the_store(w: &mut World, mode: String) -> Result<(), String> {
    let status = shown(w)?;
    let store = home(w)?.store();
    let size = std::fs::metadata(&store).map_err(|e| e.to_string())?.len();
    let want = serde_json::json!({
        "path": store.display().to_string(),
        "bytes": size,
        "schema_version": ::store::SCHEMA_VERSION,
    });
    if status["store"] != want {
        return Err(format!(
            "status shows store {}, not {want}",
            status["store"]
        ));
    }
    if status["deployment_mode"] != mode.as_str() {
        return Err(format!(
            "status shows deployment mode {}, not {mode}",
            status["deployment_mode"]
        ));
    }
    Ok(())
}

#[then(regex = r#"^the output shows the counter "([a-z_]+)" with value (\d+)$"#)]
fn shows_the_counter(w: &mut World, counter: String, value: u64) -> Result<(), String> {
    let status = shown(w)?;
    if status["counters"][&counter]["total"] == value {
        return Ok(());
    }
    Err(format!(
        "status shows {counter} as {}, not {value}",
        status["counters"][&counter]
    ))
}

#[then(regex = r"^the output says, for this OS, whether node identity is checked$")]
fn says_node_identity(w: &mut World) -> Result<(), String> {
    let status = shown(w)?;
    let identity = &status["node_identity"];
    let supported = cfg!(any(target_os = "linux", target_os = "macos", windows));
    match (identity["checked"].as_bool(), identity["against"].as_str()) {
        (Some(checked), Some(_)) if checked && supported => Ok(()),
        (Some(false), None) if !supported => Ok(()),
        _ => Err(format!(
            "status says of the node identity {identity}, not whether this OS checks it"
        )),
    }
}

#[then(
    regex = r#"^the output says that a second event at one \(writer, seq\) is refused for the segments the node holds, that the "equivocated" mark waits for seals and that the check between peers arrives with the peer network$"#
)]
fn says_fork_detection(w: &mut World) -> Result<(), String> {
    let status = shown(w)?;
    let forks = &status["fork_detection"];
    let facts = [
        (
            "refused",
            "a second event at one (writer, seq), for the segments the node holds",
        ),
        ("equivocated", "the mark waits for seals"),
        (
            "peer",
            "the check between peers arrives with the peer network",
        ),
    ];
    for (key, want) in facts {
        if forks[key] != want {
            return Err(format!("status says {key}: {}, not {want:?}", forks[key]));
        }
    }
    Ok(())
}
