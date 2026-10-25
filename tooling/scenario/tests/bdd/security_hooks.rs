//! The steps of SEC-16 that phase 9 of plan 2609292012 carries: a hook
//! input whose field fails the schema is rejected fail-open, injecting
//! nothing, audited and counted. `@SEC-16` is bound.

use cucumber::{then, when};
use serde_json::json;

use crate::World;
use crate::record::run_core;
use crate::record_hooks::{SESSION, answered_empty, session_start};
use crate::record_transcript::remember_counters;

#[when(regex = r#"^the hook "([A-Za-z]+)" runs with a hook input whose "([a-z_]+)" is a number$"#)]
fn hook_input_with_a_number(w: &mut World, hook: String, field: String) -> Result<(), String> {
    remember_counters(w)?;
    let mut input = session_start(w, SESSION, "startup")?;
    input[field.as_str()] = json!(42);
    input["hook_event_name"] = json!(hook);
    run_core(w, &["hook", &hook], Some(input.to_string().as_bytes()))
}

#[then(regex = r"^the hook handler exits 0 with empty output and no injection$")]
fn exits_0_empty(w: &mut World) -> Result<(), String> {
    answered_empty(w)
}
