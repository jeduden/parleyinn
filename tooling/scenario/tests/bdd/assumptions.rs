//! Steps of `features/assumptions.feature`, SRS §2.3's contract tests
//! (ENG-17): each replays what the M0 spikes recorded from Claude Code
//! under `testdata/claude-code/<version>/`, for every recorded version,
//! through the `harness-contract` checks. A version's `manifest.json`
//! names its recordings by role.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use cucumber::{given, then, when};
use engineering::outcome;
use harness_contract::measured::{self, Measurement};
use harness_contract::transcript_tree::{self, Tree};
use harness_contract::{hook_input, recording, snapshot, text, transcript};
use serde_json::Value;

use crate::World;

/// What the §2.3 scenarios track beside the shared world, one entry per
/// recorded Claude Code version.
#[derive(Debug, Default)]
pub struct Assumptions {
    pub(crate) versions: Vec<Version>,
    /// The hook flow a step named.
    pub(crate) flow: String,
    /// The working directories a step named.
    pub(crate) cwds: Vec<String>,
    /// The compaction a step named: its key under `subagent_compaction`.
    pub(crate) compaction: String,
    /// The blocked compaction a step named: its key under
    /// `compaction_blocked` (S6).
    pub(crate) blocked: String,
}

/// One recorded Claude Code version and what a step read from it.
#[derive(Debug, Default)]
pub(crate) struct Version {
    pub(crate) dir: PathBuf,
    pub(crate) manifest: Value,
    /// Transcripts a step chose, by path; their lines once read.
    pub(crate) transcripts: Vec<PathBuf>,
    pub(crate) lines: Vec<Value>,
    /// The hook inputs a step selected.
    pub(crate) inputs: Vec<Value>,
    pub(crate) tree: Tree,
    /// A recording before and after a flow, by bytes and by lines.
    pub(crate) before: (Vec<u8>, Vec<Value>),
    pub(crate) after: (Vec<u8>, Vec<Value>),
    pub(crate) measurements: Vec<Measurement>,
    pub(crate) timeouts: Value,
    pub(crate) cleanup: (Vec<measured::Aged>, Vec<measured::Aged>),
    pub(crate) cleanup_doc: Value,
    /// The hook events the stub ParleyInn plugin registers, its name and its
    /// MCP server's (S4).
    pub(crate) registered: BTreeSet<String>,
    pub(crate) plugin: (String, String),
    /// The init message of one plugin target's harness session.
    pub(crate) init: Value,
    /// One plugin recording's directory, and the environment each of its
    /// hook handlers and its MCP server saw (ASM-22).
    pub(crate) target: PathBuf,
    pub(crate) hook_envs: Vec<BTreeMap<String, String>>,
    pub(crate) server_env: BTreeMap<String, String>,
}

impl Version {
    pub(crate) fn name(&self) -> String {
        self.dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// The path the manifest gives at `key`, inside this version.
    pub(crate) fn path(&self, key: &str) -> Result<PathBuf, String> {
        let rel =
            recording::named(&self.manifest, key).map_err(|e| format!("{}: {e}", self.name()))?;
        Ok(self.dir.join(rel))
    }

    pub(crate) fn read_transcripts(&mut self) -> Result<(), String> {
        self.lines.clear();
        for path in &self.transcripts {
            self.lines.extend(recording::lines(path)?);
        }
        Ok(())
    }
}

pub(crate) fn asm(w: &mut World) -> &mut Assumptions {
    w.section::<Assumptions>()
}

/// Runs `check` on every recorded version, naming the version in each
/// problem.
pub(crate) fn each(
    w: &mut World,
    mut check: impl FnMut(&mut Version) -> Result<Vec<String>, String>,
) -> Result<(), String> {
    let mut problems = Vec::new();
    for v in &mut asm(w).versions {
        let name = v.name();
        problems.extend(check(v)?.into_iter().map(|p| format!("{name}: {p}")));
    }
    outcome(problems)
}

pub(crate) fn open_versions(w: &mut World, version: &str) -> Result<(), String> {
    if version != "supported" {
        return Err(format!(
            "only the \"supported\" versions are recorded, not {version:?}"
        ));
    }
    let root = testkit::repo_root().join("testdata/claude-code");
    asm(w).versions = recording::versions(&root)?
        .into_iter()
        .map(|dir| {
            let manifest = recording::json(&dir.join("manifest.json"))?;
            Ok(Version {
                dir,
                manifest,
                ..Version::default()
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(())
}

pub(crate) fn none_unless(found: bool, what: impl FnOnce() -> String) -> Vec<String> {
    if found { Vec::new() } else { vec![what()] }
}

// ASM-01: hook inputs.

#[given(regex = r#"^the hook inputs recorded from Claude Code "([^"]+)" in the "([^"]+)" flow$"#)]
fn hook_inputs_of_a_flow(w: &mut World, version: String, flow: String) -> Result<(), String> {
    open_versions(w, &version)?;
    asm(w).flow = flow;
    Ok(())
}

#[when(regex = r#"^the "([^"]+)" hook inputs are read$"#)]
fn hook_inputs_are_read(w: &mut World, event: String) -> Result<(), String> {
    let flow = asm(w).flow.clone();
    each(w, |v| {
        let all = recording::hook_inputs(&v.dir, &flow)?;
        v.inputs = hook_input::of_event(&all, &event)
            .into_iter()
            .cloned()
            .collect();
        Ok(none_unless(!v.inputs.is_empty(), || {
            format!("the {flow} flow recorded no {event} hook input")
        }))
    })
}

#[then(regex = r"^each carries (.+)$")]
fn each_carries(w: &mut World, fields: String) -> Result<(), String> {
    let fields = hook_input::names(&fields)?;
    let fields: Vec<&str> = fields.iter().map(String::as_str).collect();
    each(w, |v| {
        Ok(v.inputs
            .iter()
            .flat_map(|i| hook_input::missing(i, &fields))
            .collect())
    })
}

#[then(regex = r"^the last carries (.+)$")]
fn the_last_carries(w: &mut World, extra: String) -> Result<(), String> {
    let expects = hook_input::parse(&extra)?;
    each(w, |v| {
        Ok(v.inputs
            .last()
            .map(|i| hook_input::unmet(i, &expects))
            .unwrap_or_default())
    })
}

#[then(regex = r#"^the last one's "session_id" is (the same as|other than) the "([^"]+)" flow's$"#)]
fn the_last_ones_harness_session(
    w: &mut World,
    relation: String,
    flow: String,
) -> Result<(), String> {
    each(w, |v| {
        let first = recording::hook_inputs(&v.dir, &flow)?;
        let theirs = first
            .first()
            .map(|i| text(i, "session_id"))
            .unwrap_or_default();
        let ours = v
            .inputs
            .last()
            .map(|i| text(i, "session_id"))
            .unwrap_or_default();
        let same = ours == theirs;
        Ok(none_unless(same == (relation == "the same as"), || {
            format!("session_id {ours} is not {relation} the {flow} flow's {theirs}")
        }))
    })
}

// ASM-02: additional context.

#[given(
    regex = r#"^the transcripts recorded from Claude Code "([^"]+)" with every hook answering "additionalContext"$"#
)]
fn transcripts_answering_context(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        let list = v.manifest.get("hook_context").and_then(Value::as_array);
        v.transcripts = list
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|rel| v.dir.join(rel))
            .collect();
        Ok(none_unless(!v.transcripts.is_empty(), || {
            "the manifest names no hook_context".into()
        }))
    })
}

#[when(regex = r"^each line of the recordings? (?:is|are) read$")]
fn each_line_is_read(w: &mut World) -> Result<(), String> {
    each(w, |v| v.read_transcripts().map(|()| Vec::new()))
}

#[then(
    regex = r#"^the "([^"]+)" hook's additional context (shows|never shows) as a hook context attachment$"#
)]
fn context_shows(w: &mut World, event: String, shows: String) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::hook_context_events(&v.lines).contains(&event);
        Ok(none_unless(found == (shows == "shows"), || {
            let seen = if found { "shows" } else { "never shows" };
            format!("the {event} hook's additional context {seen} as an attachment")
        }))
    })
}

#[then(
    regex = r#"^the "([^"]+)" hook's output (shows|never shows) in a compact command's output$"#
)]
fn output_echoed(w: &mut World, event: String, shows: String) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::echoed_hooks(&v.lines).contains(&event);
        Ok(none_unless(found == (shows == "shows"), || {
            let seen = if found { "shows" } else { "never shows" };
            format!("the {event} hook's output {seen} in a compact command's output")
        }))
    })
}

#[then(
    regex = r#"^on exit status (\d) the "([^"]+)" hook's stderr shows (in a compact command's output|only in its own hook attachment)$"#
)]
fn stderr_shows(w: &mut World, status: String, event: String, place: String) -> Result<(), String> {
    each(w, |v| {
        let (lines, nonces) = stderr_probe(v, &status, &event)?;
        Ok(nonces
            .iter()
            .flat_map(|n| stderr_problems(&transcript::where_shown(&lines, n), &place))
            .map(|p| format!("on exit status {status} the {event} hook's stderr {p}"))
            .collect())
    })
}

#[then(regex = r"^on exit status (\d) every hook's stderr shows only in its own hook attachment$")]
fn every_stderr_attached(w: &mut World, status: String) -> Result<(), String> {
    each(w, |v| {
        let events: Vec<String> = v
            .manifest
            .pointer(&format!("/stderr/{status}/nonces"))
            .and_then(Value::as_object)
            .ok_or_else(|| format!("the manifest names no stderr nonces for exit status {status}"))?
            .keys()
            .cloned()
            .collect();
        let mut problems = Vec::new();
        for event in events {
            let (lines, nonces) = stderr_probe(v, &status, &event)?;
            problems.extend(
                nonces
                    .iter()
                    .flat_map(|n| {
                        stderr_problems(
                            &transcript::where_shown(&lines, n),
                            "only in its own hook attachment",
                        )
                    })
                    .map(|p| format!("on exit status {status} the {event} hook's stderr {p}")),
            );
        }
        Ok(problems)
    })
}

/// The transcript of the stderr probe for exit status `status`, and
/// the nonces `event`'s handler wrote to stderr in it.
fn stderr_probe(
    v: &Version,
    status: &str,
    event: &str,
) -> Result<(Vec<Value>, Vec<String>), String> {
    let lines = recording::lines(&v.path(&format!("stderr.{status}.transcript"))?)?;
    let nonces: Vec<String> = v
        .manifest
        .pointer(&format!("/stderr/{status}/nonces/{event}"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!("the manifest names no {event} stderr nonce for exit status {status}")
        })?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    Ok((lines, nonces))
}

fn stderr_problems(
    shown: &std::collections::BTreeSet<transcript::Shown>,
    place: &str,
) -> Vec<String> {
    if place == "in a compact command's output" {
        return none_unless(shown.contains(&transcript::Shown::CommandOutput), || {
            "never shows in a compact command's output".into()
        });
    }
    shown
        .iter()
        .filter(|s| **s != transcript::Shown::HookAttachment)
        .map(|s| format!("shows in {s:?}"))
        .collect()
}

#[then(
    regex = r#"^a harness resume drops a "SessionStart" additional context equal to one since the last compaction, and records it again after one$"#
)]
fn resume_drops_repeated_context(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let lines = recording::lines(&v.path("resume_context.transcript")?)?;
        let answer = recording::named(&v.manifest, "resume_context.answer")?;
        let answered_on: Vec<&str> = v
            .manifest
            .pointer("/resume_context/answered_on")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let sources: Vec<&str> = v
            .manifest
            .pointer("/resume_context/sources")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let answered = sources.iter().filter(|s| answered_on.contains(s)).count();
        let resumes = sources.iter().filter(|s| **s == "resume").count();
        let counts = transcript::context_per_compaction(&lines, "SessionStart", answer);
        let mut problems = Vec::new();
        if resumes < 2 || answered <= counts.iter().sum::<usize>() {
            problems.push(format!(
                "the SessionStart hook answered {answered} times over {resumes} resumes, and {} were recorded",
                counts.iter().sum::<usize>()
            ));
        }
        if counts.len() < 2 || counts.iter().any(|c| *c != 1) {
            problems.push(format!(
                "between compactions the answer was recorded {counts:?} times, not once each"
            ));
        }
        Ok(problems)
    })
}

// ASM-03 and ASM-16: the tree of transcripts.

#[given(regex = r#"^a recorded "~/.claude/projects" tree for Claude Code "([^"]+)"$"#)]
fn a_recorded_transcript_tree(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)
}

#[given(
    regex = r#"^a recorded "~/.claude/projects" tree for Claude Code "([^"]+)" after one harness session in each of (.+)$"#
)]
fn a_tree_after_harness_sessions(
    w: &mut World,
    version: String,
    cwds: String,
) -> Result<(), String> {
    open_versions(w, &version)?;
    asm(w).cwds = hook_input::names(&cwds)?;
    Ok(())
}

#[when(regex = r"^the recorded tree is walked$")]
fn the_tree_is_walked(w: &mut World) -> Result<(), String> {
    let by_cwd = !asm(w).cwds.is_empty();
    each(w, |v| {
        if by_cwd {
            let flow = recording::named(&v.manifest, "directories")?.to_owned();
            v.inputs = recording::hook_inputs(&v.dir, &flow)?;
        }
        v.tree = transcript_tree::walk(&v.path("projects")?)?;
        Ok(Vec::new())
    })
}

#[then(regex = r#"^each harness session yields one JSONL transcript under "[^"]+"$"#)]
fn one_transcript_per_harness_session(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(!v.tree.harness_sessions.is_empty(), || {
            "no harness session".into()
        });
        for s in &v.tree.harness_sessions {
            problems.extend(transcript_tree::harness_session_problems(s)?);
        }
        problems.extend(transcript_tree::duplicates(&v.tree));
        problems.extend(
            v.tree
                .strays
                .iter()
                .map(|p| format!("{} lies outside the layout", p.display())),
        );
        Ok(problems)
    })
}

#[then(regex = r#"^each subagent transcript lies under "[^"]+"$"#)]
fn subagents_nested(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(!v.tree.subagents.is_empty(), || {
            "no subagent transcript".into()
        });
        for s in &v.tree.subagents {
            problems.extend(transcript_tree::subagent_problems(s, &v.tree)?);
        }
        Ok(problems)
    })
}

#[then(
    regex = r#"^each subagent transcript has a "\.meta\.json" beside it naming its agent type$"#
)]
fn subagents_have_meta(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(v.tree
            .subagents
            .iter()
            .flat_map(transcript_tree::meta_problems)
            .collect())
    })
}

#[then(regex = r#"^the three harness sessions share the transcript directory "([^"]+)"$"#)]
fn harness_sessions_share_a_directory(w: &mut World, dir: String) -> Result<(), String> {
    let cwds = asm(w).cwds.clone();
    each(w, |v| {
        Ok(transcript_tree::shared_directory(
            &v.inputs, &cwds, &dir, &v.tree,
        ))
    })
}

// ASM-04: appends.

#[given(
    regex = r#"^a recorded transcript before and after a "([^"]+)" flow for Claude Code "([^"]+)"$"#
)]
fn before_and_after(w: &mut World, flow: String, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    asm(w).flow = flow;
    Ok(())
}

#[when(regex = r"^the two recordings are compared$")]
fn recordings_compared(w: &mut World) -> Result<(), String> {
    let flow = asm(w).flow.clone();
    each(w, |v| {
        let flow = v
            .manifest
            .get("appends")
            .and_then(|a| a.get(&flow))
            .cloned()
            .ok_or_else(|| format!("the manifest names no {flow} recording"))?;
        v.before = read_both(&v.dir.join(text(&flow, "before")))?;
        v.after = read_both(&v.dir.join(text(&flow, "after")))?;
        Ok(Vec::new())
    })
}

fn read_both(path: &Path) -> Result<(Vec<u8>, Vec<Value>), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok((bytes, recording::lines(path)?))
}

#[then(regex = r"^the earlier recording is a byte prefix of the later one$")]
fn byte_prefix(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let grew = v.after.0.len() > v.before.0.len();
        Ok(none_unless(
            grew && v.after.0.starts_with(&v.before.0),
            || "the earlier recording is not a byte prefix of the later one".into(),
        ))
    })
}

#[then(regex = r"^the first line appended continues from a line before the earlier one's last$")]
fn appended_branches(w: &mut World) -> Result<(), String> {
    each(w, |v| Ok(transcript::branched(&v.before.1, &v.after.1)))
}

#[then(regex = r#"^a "([^"]+)" line is appended$"#)]
fn line_appended(w: &mut World, kind: String) -> Result<(), String> {
    each(w, |v| {
        let found = transcript::appended_types(&v.before.1, &v.after.1).contains(&kind);
        Ok(none_unless(found, || {
            format!("no {kind} line was appended")
        }))
    })
}

// ASM-05: cleanup.

#[given(
    regex = r#"^a recorded "~/.claude/projects" listing before and after harness starts with "cleanupPeriodDays" of (\d+) for Claude Code "([^"]+)"$"#
)]
fn tree_before_after_cleanup(w: &mut World, days: u32, version: String) -> Result<(), String> {
    open_versions(w, &version)?;
    each(w, |v| {
        v.cleanup_doc = recording::json(&v.path("cleanup")?)?;
        let period = v
            .cleanup_doc
            .get("cleanup_period_days")
            .and_then(Value::as_u64);
        Ok(none_unless(period == Some(u64::from(days)), || {
            format!("the cleanup was recorded with a period of {period:?} days, not {days}")
        }))
    })
}

#[when(regex = r"^the two trees are compared$")]
fn trees_compared(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        v.cleanup = measured::cleanup(&v.cleanup_doc)?;
        Ok(Vec::new())
    })
}

#[then(regex = r"^a transcript older than (\d+) days was (deleted|kept)$")]
fn old_transcript(w: &mut World, days: f64, fate: String) -> Result<(), String> {
    each(w, |v| {
        let list = if fate == "deleted" {
            &v.cleanup.0
        } else {
            &v.cleanup.1
        };
        let found = list.iter().any(|a| a.age_days > days);
        Ok(none_unless(found, || {
            format!("no transcript older than {days} days was {fate}")
        }))
    })
}

#[then(regex = r"^no transcript younger than (\d+) days was deleted$")]
fn young_kept(w: &mut World, days: f64) -> Result<(), String> {
    each(w, |v| {
        Ok(v.cleanup
            .0
            .iter()
            .filter(|a| a.age_days < days)
            .map(|a| format!("{} was deleted at {} days", a.path, a.age_days))
            .collect())
    })
}

// ASM-06: subagent compaction.

#[given(regex = r#"^a recorded hook input for Claude Code "([^"]+)"$"#)]
fn a_recorded_hook_input(w: &mut World, version: String) -> Result<(), String> {
    open_versions(w, &version)
}

#[given(
    regex = r"^the hook input was captured during (a foreground subagent's|a background subagent's|the main agent's, while a background subagent ran,) compaction$"
)]
fn during_compaction(w: &mut World, case: String) -> Result<(), String> {
    let key = match case.as_str() {
        "a foreground subagent's" => "foreground",
        "a background subagent's" => "background",
        _ => "main_during_background",
    };
    asm(w).compaction = format!("subagent_compaction.{key}");
    let base = asm(w).compaction.clone();
    each(w, |v| {
        let flow = recording::named(&v.manifest, &format!("{base}.hooks"))?.to_owned();
        v.inputs = recording::hook_inputs(&v.dir, &flow)?;
        Ok(none_unless(
            hook_input::ordered(&v.inputs, "SubagentStart", "PreCompact"),
            || format!("the {flow} flow holds no PreCompact after a SubagentStart"),
        ))
    })
}

#[when(regex = r#"^the hook "([^"]+)" runs with the recorded hook input$"#)]
fn the_hook_runs(w: &mut World, event: String) -> Result<(), String> {
    each(w, |v| {
        let all = std::mem::take(&mut v.inputs);
        let parent = all
            .iter()
            .find(|i| text(i, "hook_event_name") == "SessionStart")
            .cloned()
            .ok_or("the flow holds no SessionStart of the main agent")?;
        let start = all
            .iter()
            .position(|i| text(i, "hook_event_name") == "SubagentStart")
            .ok_or("the flow holds no SubagentStart")?;
        v.inputs = std::iter::once(parent)
            .chain(
                hook_input::of_event(&all[start..], &event)
                    .into_iter()
                    .cloned(),
            )
            .collect();
        Ok(none_unless(v.inputs.len() > 1, || {
            format!("the flow holds no {event} hook input")
        }))
    })
}

#[then(
    regex = r#"^its "session_id" and "transcript_path" are the main agent's harness session's$"#
)]
fn the_main_agents_identity(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let Some((main, rest)) = v.inputs.split_first() else {
            return Ok(vec!["no hook input".into()]);
        };
        let rest: Vec<&Value> = rest.iter().collect();
        Ok(hook_input::identity_mismatches(
            main,
            &rest,
            &["session_id", "transcript_path"],
        ))
    })
}

#[then(regex = r#"^it carries (".+)$"#)]
fn it_carries(w: &mut World, extra: String) -> Result<(), String> {
    let expects = hook_input::parse(&extra)?;
    each(w, |v| {
        Ok(v.inputs
            .iter()
            .skip(1)
            .flat_map(|i| hook_input::unmet(i, &expects))
            .collect())
    })
}

#[then(regex = r"^it carries no subagent-specific field$")]
fn no_subagent_field(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(v.inputs
            .iter()
            .skip(1)
            .flat_map(hook_input::subagent_fields)
            .collect())
    })
}

/// The transcripts of the compaction a step named: the compacting
/// agent's and the other agent's.
fn compaction_transcripts(v: &Version, base: &str) -> Result<(Vec<Value>, Vec<Value>), String> {
    Ok((
        recording::lines(&v.path(&format!("{base}.compacting_transcript"))?)?,
        recording::lines(&v.path(&format!("{base}.other_transcript"))?)?,
    ))
}

#[then(
    regex = r#"^the additional context of the "SessionStart" hook that follows it lands in the compacting agent's transcript, where the model reads it, and in no other$"#
)]
fn session_start_context_lands_in_compacting(w: &mut World) -> Result<(), String> {
    let base = asm(w).compaction.clone();
    each(w, |v| {
        let (compacting, other) = compaction_transcripts(v, &base)?;
        let mut problems = Vec::new();
        if !transcript::context_after_compaction(&compacting, "SessionStart") {
            problems.push(
                "no SessionStart context follows the compaction in the compacting agent's transcript"
                    .to_owned(),
            );
        }
        if transcript::context_after_compaction(&other, "SessionStart") {
            problems
                .push("SessionStart context follows a compaction in the other transcript".into());
        }
        Ok(problems)
    })
}

#[then(regex = r"^the compaction's boundary lies in the compacting agent's transcript$")]
fn boundary_in_compacting(w: &mut World) -> Result<(), String> {
    let base = asm(w).compaction.clone();
    each(w, |v| {
        let (compacting, other) = compaction_transcripts(v, &base)?;
        let mut problems = none_unless(transcript::has_compaction(&compacting), || {
            "the compacting agent's transcript holds no compaction".into()
        });
        if transcript::has_compaction(&other) {
            problems.push("the other transcript holds a compaction".into());
        }
        Ok(problems)
    })
}

#[then(
    regex = r"^no transcript of the harness session held the compaction's boundary when its hooks ran$"
)]
fn no_boundary_at_hook_time(w: &mut World) -> Result<(), String> {
    let base = asm(w).compaction.clone();
    each(w, |v| {
        let flow = recording::named(&v.manifest, &format!("{base}.hooks"))?.to_owned();
        let mut hooks = Vec::new();
        for path in recording::files(&v.dir.join("hooks").join(&flow), ".json")? {
            let input = recording::json(&path)?;
            let snap = std::fs::read_to_string(path.with_extension("snap"))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            let source = match text(&input, "source") {
                "" => text(&input, "trigger"),
                s => s,
            };
            hooks.push((
                text(&input, "hook_event_name").to_owned(),
                source.to_owned(),
                snapshot::parse(&snap)?,
            ));
        }
        let compactions = hooks
            .iter()
            .filter(|(e, s, _)| e == "SessionStart" && s == "compact")
            .count();
        let mut problems = none_unless(compactions > 0, || {
            format!("the {flow} flow holds no compaction")
        });
        problems.extend(snapshot::early_boundaries(&hooks));
        Ok(problems)
    })
}

// ASM-09: a PreCompact hook exiting 2 (S6).

#[given(
    regex = r#"^a PreCompact hook whose handler exits 2, recorded in the "(manual|full window)" recording$"#
)]
fn a_blocking_pre_compact(w: &mut World, recording: String) -> Result<(), String> {
    asm(w).blocked = format!("compaction_blocked.{}", recording.replace(' ', "_"));
    let base = asm(w).blocked.clone();
    each(w, |v| {
        let flow = recording::named(&v.manifest, &format!("{base}.hooks"))?.to_owned();
        v.inputs = recording::hook_inputs(&v.dir, &flow)?;
        v.transcripts = vec![v.path(&format!("{base}.transcript"))?];
        v.read_transcripts()?;
        Ok(Vec::new())
    })
}

#[when(
    regex = r#"^compaction is triggered (manually, by "/compact"|automatically, on to the full context window)$"#
)]
fn compaction_is_triggered(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(none_unless(
            !hook_input::of_event(&v.inputs, "PreCompact").is_empty(),
            || "the recording holds no PreCompact hook input".into(),
        ))
    })
}

#[then(regex = r#"^every "PreCompact" hook input of the recording carries (".+)$"#)]
fn every_pre_compact_carries(w: &mut World, extra: String) -> Result<(), String> {
    let expects = hook_input::parse(&extra)?;
    each(w, |v| {
        Ok(hook_input::of_event(&v.inputs, "PreCompact")
            .into_iter()
            .flat_map(|i| hook_input::unmet(i, &expects))
            .collect())
    })
}

#[then(regex = r#"^no transcript of the recording holds a "compact_boundary" line$"#)]
fn no_boundary(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        Ok(none_unless(!transcript::has_compaction(&v.lines), || {
            "the transcript holds a compact_boundary line".into()
        }))
    })
}

#[then(
    regex = r#"^no "PostCompact" hook fired, nor a "SessionStart" hook with "source" set to "compact"$"#
)]
fn no_compaction_hooks(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let mut problems = none_unless(
            hook_input::of_event(&v.inputs, "PostCompact").is_empty(),
            || "a PostCompact hook fired".into(),
        );
        if hook_input::of_event(&v.inputs, "SessionStart")
            .iter()
            .any(|i| text(i, "source") == "compact")
        {
            problems.push("a SessionStart hook fired with source compact".into());
        }
        Ok(problems)
    })
}

/// The words the recording's PreCompact handler wrote to stderr.
fn stderr_words(v: &Version, base: &str) -> Result<Vec<String>, String> {
    let words: Vec<String> = harness_contract::at(&v.manifest, &format!("{base}.nonces"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    if words.is_empty() {
        return Err(format!("{}: the manifest names no {base}.nonces", v.name()));
    }
    Ok(words)
}

#[then(
    regex = r"^the words the PreCompact handler wrote to stderr reached the model in the command's output, and the next model reply repeated them$"
)]
fn stderr_reached_the_model(w: &mut World) -> Result<(), String> {
    let base = asm(w).blocked.clone();
    each(w, |v| {
        let mut problems = Vec::new();
        for word in stderr_words(v, &base)? {
            let shown = BTreeSet::from([
                transcript::Shown::CommandOutput,
                transcript::Shown::Other("assistant".into()),
            ]);
            if transcript::where_shown(&v.lines, &word) != shown {
                problems.push(format!(
                    "{word} shows elsewhere than in command output and a model reply"
                ));
            }
            if transcript::model_replies_after(&v.lines, &word).first() != Some(&word.as_str()) {
                problems.push(format!("the next model reply does not repeat {word}"));
            }
        }
        Ok(problems)
    })
}

#[then(regex = r"^the words the PreCompact handler wrote to stderr reached no transcript$")]
fn stderr_reached_no_transcript(w: &mut World) -> Result<(), String> {
    let base = asm(w).blocked.clone();
    each(w, |v| {
        Ok(stderr_words(v, &base)?
            .into_iter()
            .filter(|word| !transcript::where_shown(&v.lines, word).is_empty())
            .map(|word| format!("{word} reached the transcript"))
            .collect())
    })
}

#[then(regex = r"^the model replied to the next prompt without a compaction$")]
fn replied_without_compaction(w: &mut World) -> Result<(), String> {
    each(w, |v| {
        let resumed = hook_input::ordered(&v.inputs, "PreCompact", "UserPromptSubmit");
        let mut problems = none_unless(resumed, || {
            "no prompt followed the blocked compaction".into()
        });
        if !transcript::refusals(&v.lines).is_empty() {
            problems.push("the harness refused a prompt".into());
        }
        if transcript::model_replies_after(&v.lines, "Compaction blocked by PreCompact hook")
            .is_empty()
        {
            problems.push("no model reply followed the blocked compaction".into());
        }
        Ok(problems)
    })
}

#[then(
    regex = r#"^each prompt at the full context window was refused with "([^"]+)", and no "Stop" hook fired$"#
)]
fn refused_at_full_context_window(w: &mut World, refusal: String) -> Result<(), String> {
    each(w, |v| {
        let refusals = transcript::refusals(&v.lines);
        let prompts = hook_input::of_event(&v.inputs, "UserPromptSubmit").len();
        let mut problems = none_unless(refusals.len() == prompts && prompts > 1, || {
            format!("{} refusals for {prompts} prompts", refusals.len())
        });
        problems.extend(
            refusals
                .iter()
                .filter(|r| **r != refusal)
                .map(|r| format!("a refusal reads {r:?}")),
        );
        if transcript::model_replied_after_refusal(&v.lines) {
            problems.push("a model reply followed the first refusal".into());
        }
        if !hook_input::of_event(&v.inputs, "Stop").is_empty() {
            problems.push("a Stop hook fired".into());
        }
        Ok(problems)
    })
}

/// What ASM-09's recordings showed, as OQ-01 must say it, and the
/// finding each phrase records.
const S6_FINDINGS: [(&str, &str); 2] = [
    (
        "blocks a manual compaction and every automatic one (ASM-09)",
        "exit status 2 blocks every compaction (ASM-09)",
    ),
    (
        "refuses every later prompt",
        "the full context window refuses every prompt",
    ),
];

#[then(regex = r"^OQ-01 in SRS §13 records what ASM-09's recordings showed$")]
fn oq_01_records_s6(_w: &mut World) -> Result<(), String> {
    let path = testkit::repo_root().join("docs/srs/13-open-questions-and-risks.md");
    let body = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    // OQ-01's list item, its lines joined by single spaces as Markdown reads them.
    let words = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let item = words.split("- **OQ-").find(|i| i.starts_with("01**: "));
    let item = item.ok_or("SRS §13 holds no OQ-01")?;
    outcome(
        S6_FINDINGS
            .iter()
            .filter(|(said, _)| !item.contains(said))
            .map(|(_, finding)| format!("OQ-01 does not record that {finding}"))
            .collect(),
    )
}

// ASM-10: harness timeouts.
