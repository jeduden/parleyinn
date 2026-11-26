//! Steps of `features/assumptions.feature`, SRS §2.3's contract tests
//! (ENG-17), the second half: the hook timeouts, the harness context and
//! the transcript lines' fields, moved from `assumptions.rs` to keep each
//! file in its limit.

use std::fs;

use cucumber::{given, then, when};
use harness_contract::measured::{self, Measurement};
use harness_contract::{plugin, recording, text, transcript};
use serde_json::Value;

use crate::World;
use crate::assumptions::*;

#[given(regex = r#"^the hook timeouts measured on Claude Code "([^"]+)"$"#)]
fn timeouts_measured(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.timeouts = recording::json(&v.path("timeouts")?)?;
        v.measurements = measured::measurements(&v.timeouts)?;
        Ok(Vec::new())
    })
}

#[when(
    regex = r#"^the "([^"]+)" handler sleeps past its harness timeout in (print|interactive) mode with (?:a (\d+) s timeout|no timeout)$"#
)]
fn handler_sleeps(
    w: &mut World,
    event: String,
    mode: String,
    timeout: String,
) -> Result<(), String> {
    let timeout = if timeout.is_empty() {
        None
    } else {
        Some(timeout.parse::<f64>().map_err(|e| e.to_string())?)
    };
    asm(w).flow = format!("{event}/{mode}");
    each(w, |v| {
        let picked: Vec<Measurement> = measured::matching(&v.measurements, &event, &mode, timeout)
            .into_iter()
            .cloned()
            .collect();
        let found = !picked.is_empty();
        v.measurements = picked;
        Ok(none_unless(found, || {
            format!("no {event} measurement in {mode} mode with {timeout:?} s")
        }))
    })
}

#[then(regex = r"^the harness terminated the handler after about ([\d.]+) s$")]
fn terminated_after(w: &mut World, limit: f64) -> Result<(), String> {
    each(w, |v| {
        Ok(v.measurements
            .iter()
            .filter(|m| m.finished || !measured::about(m.ran_ms, limit))
            .map(|m| {
                format!(
                    "run {}: {} ran {} ms, not about {limit} s",
                    m.trial, m.event, m.ran_ms
                )
            })
            .collect())
    })
}

#[then(
    regex = r#"^no "SessionEnd" handler started in a print-mode harness session whose "Stop" handler reached its harness timeout$"#
)]
fn session_end_skipped(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let (checked, mut problems) = measured::session_end_after_stop_timeout(&v.timeouts)?;
        if checked == 0 {
            problems.push("no print-mode trial timed out its Stop handler".into());
        }
        Ok(problems)
    })
}

// ASM-11 to ASM-15: transcript lines.

#[given(
    regex = r#"^a recorded transcript for Claude Code "([^"]+)" (?:from a worktree with a CLAUDE\.md file, an MCP server and a skill|with an assistant reply carrying two tool calls|with a Bash tool call and its tool result)$"#
)]
fn the_harness_context_transcript(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.transcripts = vec![v.path("harness_context.transcript")?];
        Ok(Vec::new())
    })
}

#[given(
    regex = r#"^a recorded transcript for Claude Code "([^"]+)"(?: with extended thinking enabled)?$"#
)]
fn every_transcript(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.transcripts = recording::files(&v.dir.join("transcripts"), ".jsonl")?;
        Ok(none_unless(!v.transcripts.is_empty(), || {
            "no transcript".into()
        }))
    })
}

#[then(
    regex = r#"^the CLAUDE\.md file, the MCP server instructions and the skill listing appear as "attachment" lines$"#
)]
fn harness_context_attached(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let markers = harness_context_markers(&v.manifest)?;
        Ok(markers
            .iter()
            .filter(|(_, m)| transcript::attachment_with(&v.lines, m).is_none())
            .map(|(what, _)| format!("the {what} appears in no attachment line"))
            .collect())
    })
}

#[then(regex = r#"^each of those lines carries "attachment\.type" and "renderedRole"$"#)]
fn attachments_carry_type_and_role(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let markers = harness_context_markers(&v.manifest)?;
        Ok(markers
            .iter()
            .filter_map(|(what, m)| transcript::attachment_with(&v.lines, m).map(|l| (what, l)))
            .filter(|(_, l)| {
                text(l, "attachment.type").is_empty() || text(l, "renderedRole").is_empty()
            })
            .map(|(what, _)| format!("the {what} line lacks attachment.type or renderedRole"))
            .collect())
    })
}

fn harness_context_markers(manifest: &Value) -> Result<Vec<(String, String)>, String> {
    recording::pairs(manifest, "harness_context.markers")
}

#[then(regex = r"^each assistant line carries exactly one content block$")]
fn one_block_per_line(w: &mut World) -> Result<(), String> {
    each(w, |v| Ok(transcript::multi_block(&v.lines)))
}

#[then(regex = r#"^the reply's lines share one "message\.id" and repeat its "usage"$"#)]
fn lines_share_message(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(transcript::spans_tool_calls(&v.lines, 2), || {
            "no message spans two tool-call lines".into()
        });
        problems.extend(transcript::usage_spread(&v.lines));
        Ok(problems)
    })
}

#[then(
    regex = r#"^the tool-result line carries the output in "message\.content" and again in "toolUseResult"$"#
)]
fn result_repeats_output(w: &mut World) -> Result<(), String> {
    tool_result_problems(w, false)
}

#[then(regex = r#"^its "sourceToolAssistantUUID" names the line carrying the Bash tool call$"#)]
fn result_names_its_call(w: &mut World) -> Result<(), String> {
    tool_result_problems(w, true)
}

fn tool_result_problems(w: &mut World, source: bool) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::tool_results(&v.lines, "Bash");
        let mut problems = if source { found.source } else { found.output };
        if found.checked == 0 {
            problems.push("no Bash tool result".into());
        }
        Ok(problems)
    })
}

#[then(
    regex = r#"^every line of type "([^"]+)" carries (neither "uuid" nor "timestamp"|no "uuid")$"#
)]
fn line_type_lacks_keys(w: &mut World, kind: String, missing: String) -> Result<(), String> {
    let both = missing.starts_with("neither");
    each(w, |v| {
        let (n, uuid, timestamp) = transcript::keys_by_type(&v.lines, &kind);
        let mut problems = none_unless(n > 0, || format!("no line of type {kind}"));
        if uuid > 0 {
            problems.push(format!("{uuid} of {n} {kind} lines carry a uuid"));
        }
        if both && timestamp > 0 {
            problems.push(format!("{timestamp} of {n} {kind} lines carry a timestamp"));
        }
        Ok(problems)
    })
}

#[then(regex = r"^every thinking block carries a signature and empty thinking text$")]
fn thinking_is_signed_and_empty(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let (n, mut problems) = transcript::thinking(&v.lines);
        if n == 0 {
            problems.push("no thinking block".into());
        }
        Ok(problems)
    })
}

// ASM-07: the plugin on each target (S4).

/// Reads the plugin target the manifest names at `key` into `v`: its
/// hook inputs and init message.
fn read_target(v: &mut Version, key: &str) -> Result<(), String> {
    let target = v.path(key)?;
    v.inputs = recording::files(&target.join("hooks"), ".json")?
        .iter()
        .map(|p| recording::json(p))
        .collect::<Result<_, _>>()?;
    let init = target.join("init.json");
    v.init = if init.exists() {
        recording::json(&init)?
    } else {
        Value::Null
    };
    Ok(())
}

#[given(
    regex = r#"^the stub ParleyInn plugin recorded for Claude Code "([^"]+)", bundling ParleyInn's hooks and a stub MCP server$"#
)]
fn the_stub_plugin(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        let kit = v.path("plugin.kit")?;
        v.registered = plugin::registered(&recording::json(&kit.join("hooks/hooks.json"))?)?;
        let name = text(
            &recording::json(&kit.join(".claude-plugin/plugin.json"))?,
            "name",
        )
        .to_owned();
        let servers = plugin::servers(&recording::json(&kit.join(".mcp.json"))?)?;
        let problems = none_unless(servers.len() == 1, || {
            format!("the plugin bundles {} MCP servers, not one", servers.len())
        });
        v.plugin = (name, servers.concat());
        Ok(problems)
    })
}

#[when(regex = r#"^it is installed on "([^"]+)"$"#)]
fn installed_on(w: &mut World, target: String) -> Result<(), String> {
    each(w, |v| {
        read_target(v, &format!("plugin.targets.{target}"))?;
        Ok(none_unless(!v.init.is_null(), || {
            format!("{target}: no init message recorded")
        }))
    })
}

#[then(regex = r"^the recorded harness session shows every hook the stub plugin registers firing$")]
fn every_registered_hook_fired(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(plugin::unfired(&v.registered, &plugin::fired(&v.inputs)))
    })
}

#[then(regex = r"^the recorded harness session lists the stub plugin's MCP tools$")]
fn lists_the_parinn_tools(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let (name, server) = (v.plugin.0.as_str(), v.plugin.1.as_str());
        let mut problems = none_unless(plugin::connected(&v.init, name, server), || {
            format!("the MCP server {server} of plugin {name} is not connected")
        });
        if plugin::tools(&v.init, name, server).is_empty() {
            problems.push(format!(
                "no tool of the MCP server {server} of plugin {name}"
            ));
        }
        Ok(problems)
    })
}

// ASM-22: the harness's attended mark (S4).

#[given(regex = r#"^the stub ParleyInn plugin's recording "([^"]+)" for Claude Code "([^"]+)"$"#)]
fn a_plugin_recording(w: &mut World, recording: String, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        let target = format!("plugin.targets.{recording}");
        let key = if recording::named(&v.manifest, &target).is_ok() {
            target
        } else {
            format!("plugin.{}", recording.replace('-', "_"))
        };
        v.target = v.path(&key)?;
        Ok(Vec::new())
    })
}

#[when(regex = r"^the environment its hook handlers and its MCP server saw is read$")]
fn environments_are_read(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        v.hook_envs = recording::files(&v.target.join("hooks"), ".env")?
            .iter()
            .map(|p| {
                fs::read_to_string(p)
                    .map(|t| plugin::environment(&t))
                    .map_err(|e| format!("read {}: {e}", p.display()))
            })
            .collect::<Result<_, _>>()?;
        v.server_env = plugin::server_environment(&recording::lines(&v.target.join("mcp.jsonl"))?)?;
        Ok(none_unless(!v.hook_envs.is_empty(), || {
            "no hook handler's environment recorded".into()
        }))
    })
}

#[then(
    regex = r#"^every hook handler saw "([^"]+)" set to "([^"]+)" and "([^"]+)" set to "([^"]+)"$"#
)]
fn every_hook_handler_saw(
    w: &mut World,
    first: String,
    first_value: String,
    second: String,
    second_value: String,
) -> Result<(), String> {
    each(w, |v| {
        let mut problems = Vec::new();
        for (n, env) in v.hook_envs.iter().enumerate() {
            for (name, want) in [(&first, &first_value), (&second, &second_value)] {
                let got = plugin::value(env, name);
                if got != Some(want.as_str()) {
                    problems.push(format!(
                        "hook handler {}: {name} is {got:?}, not {want:?}",
                        n + 1
                    ));
                }
            }
        }
        Ok(problems)
    })
}

#[then(regex = r#"^the MCP server saw "([^"]+)" (unset|set to "([^"]+)")$"#)]
fn the_mcp_server_saw(
    w: &mut World,
    name: String,
    how: String,
    want: String,
) -> Result<(), String> {
    let want = (how != "unset").then_some(want);
    each(w, |v| {
        let got = plugin::value(&v.server_env, &name);
        Ok(none_unless(got == want.as_deref(), || {
            format!("the MCP server's {name} is {got:?}, not {want:?}")
        }))
    })
}
