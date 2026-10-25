//! What spike S4 recorded of the stub ParleyInn plugin on each target
//! (ASM-07): the hooks its `hooks/hooks.json` registers, the MCP
//! servers its `.mcp.json` bundles, and the hook inputs and the init
//! message of each harness session it loaded into, and the environment
//! its hook handlers and its MCP server saw (ASM-22).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The hook events a plugin's `hooks/hooks.json` registers, in name
/// order.
///
/// # Errors
///
/// Fails when the document registers no hook.
pub fn registered(hooks: &Value) -> Result<BTreeSet<String>, String> {
    let events: BTreeSet<String> = hooks
        .get("hooks")
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    if events.is_empty() {
        return Err("the plugin registers no hooks".into());
    }
    Ok(events)
}

/// The MCP servers a plugin's `.mcp.json` bundles, in name order.
///
/// # Errors
///
/// Fails when the document bundles no MCP server.
pub fn servers(mcp: &Value) -> Result<Vec<String>, String> {
    let names: Vec<String> = mcp
        .get("mcpServers")
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default();
    if names.is_empty() {
        return Err("the plugin bundles no MCP server".into());
    }
    Ok(names)
}

/// The hook events the recorded `inputs` fired, each once.
#[must_use]
pub fn fired(inputs: &[Value]) -> BTreeSet<String> {
    inputs
        .iter()
        .map(|i| crate::text(i, "hook_event_name"))
        .filter(|e| !e.is_empty())
        .map(String::from)
        .collect()
}

/// One problem per registered event that did not fire.
#[must_use]
pub fn unfired(registered: &BTreeSet<String>, fired: &BTreeSet<String>) -> Vec<String> {
    registered
        .difference(fired)
        .map(|e| format!("{e} registered but did not fire"))
        .collect()
}

/// The tools an init message lists for `server` of `plugin`.
#[must_use]
pub fn tools(init: &Value, plugin: &str, server: &str) -> Vec<String> {
    let prefix = format!("mcp__plugin_{plugin}_{server}__");
    init.get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|t| t.starts_with(&prefix))
        .map(String::from)
        .collect()
}

/// Whether an init message shows `server` of `plugin` connected.
#[must_use]
pub fn connected(init: &Value, plugin: &str, server: &str) -> bool {
    let name = format!("plugin:{plugin}:{server}");
    init.get("mcp_servers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|s| crate::text(s, "name") == name && crate::text(s, "status") == "connected")
}

/// What a stub hook handler's `.env` record says it saw: one
/// `NAME=VALUE` per line; a line without `=` is skipped.
#[must_use]
pub fn environment(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect()
}

/// What the stub MCP server saw at its start, from its log.
///
/// # Errors
///
/// Fails when the log holds no start entry with an environment of
/// strings.
pub fn server_environment(lines: &[Value]) -> Result<BTreeMap<String, String>, String> {
    lines
        .iter()
        .filter(|line| crate::text(line, "kind") == "start")
        .find_map(|line| {
            line.get("env")?
                .as_object()?
                .iter()
                .map(|(name, value)| Some((name.clone(), value.as_str()?.to_owned())))
                .collect()
        })
        .ok_or_else(|| "the MCP server's log holds no start with its environment".into())
}

/// The value `name` had, or `None` when it was unset.
#[must_use]
pub fn value<'a>(env: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    env.get(name).map(String::as_str).filter(|v| *v != UNSET)
}

/// How the stub records a name that was not set.
const UNSET: &str = "<unset>";

#[cfg(test)]
mod tests;
