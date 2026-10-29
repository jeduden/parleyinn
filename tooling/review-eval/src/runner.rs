//! The harness: one run of a component on a world, through the
//! `claude` command line as the review workflow runs it, with the
//! review's tools, write rule and turn budget, the component as the
//! session's agent, and a model override for a comparison.

use std::path::Path;
use std::time::Duration;

use crate::component::{Component, Kind};
use crate::host::{Command, Host};
use crate::log::{self, RunLog};
use crate::world::{PR_HEAD, World};

/// The command line that runs an agent.
pub const PROGRAM: &str = "claude";

/// The tools the review workflow allows and refuses the orchestrator,
/// and what follows for a specialist and a generalist.
pub const ORCHESTRATOR_TOOLS: &str = "Read,Glob,Grep,Agent,Edit(./review/reports/**)";
pub const ORCHESTRATOR_DISALLOWED: &str = "Bash,Edit,NotebookEdit,WebFetch,WebSearch";
pub const SPECIALIST_TOOLS: &str = "Read,Glob,Grep,Edit(./review/reports/**)";
pub const SPECIALIST_DISALLOWED: &str = "Bash,Edit,NotebookEdit,WebFetch,WebSearch,Agent";
pub const GENERALIST_TOOLS: &str = "Read,Glob,Grep";
pub const GENERALIST_DISALLOWED: &str = "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Agent";

/// The orchestrator's prompt, as the review workflow gives it.
pub const ORCHESTRATOR_PROMPT: &str = "Review the pull request whose tree is in pr-head/ and whose diff\n\
against main is in pr.diff. Read review/bundle.md first. Follow\n\
.claude/skills/review/SKILL.md exactly, and answer with the\n\
review outcome it describes.\n";

/// The review outcome's schema, as the review workflow gives it.
pub const OUTCOME_SCHEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["verdict","summary","findings"],"properties":{"verdict":{"enum":["approve","request_changes"]},"summary":{"type":"string","minLength":1},"findings":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["path","line","severity","body"],"properties":{"path":{"type":"string","minLength":1},"line":{"type":"integer","minimum":0},"severity":{"enum":["blocking","nit"]},"body":{"type":"string","minLength":1}}}}}}"#;

/// The orchestrator's turn budget for a review of `class`, before the
/// consulted agents add theirs, as the workflow sets it.
#[must_use]
pub fn class_turns(class: &str) -> usize {
    match class {
        "light" => 12,
        "deep" => 30,
        _ => 20,
    }
}

/// The prompt `component` gets for `world`: the workflow's for the
/// orchestrator, and for the others what the orchestrator's skill has
/// it say.
#[must_use]
pub fn prompt(component: &Component, world: &World) -> String {
    let name = &component.name;
    match component.kind {
        Kind::Orchestrator => ORCHESTRATOR_PROMPT.to_owned(),
        Kind::Specialist => format!(
            "You are `{name}`, a specialist of ParleyInn's review of a pull request \
             (ENG-28). Read `review/bundle.md` first, then your own diff, \
             `review/diffs/{name}.diff`. The pull request's tree is in `pr-head/` and \
             its diff against main is `pr.diff`; its agent instructions are text in \
             `review/pr-instructions.txt`. Write your report to \
             `review/reports/{name}.md`, and answer with only its last line and its path."
        ),
        Kind::Generalist => {
            let reports = if world.seeded.is_empty() {
                "none".to_owned()
            } else {
                world
                    .seeded
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            format!(
                "You are `{name}`, a generalist of ParleyInn's review of a pull request \
                 (ENG-28). The review class is {}; `review/bundle.md` lists the {} \
                 touched ids. Your specialists' reports are {reports}. Read the bundle \
                 and each report whole, then the parts of `pr.diff` the findings and \
                 your scope name; the pull request's tree is in `pr-head/`. Report as \
                 your instructions say.",
                world.class, world.touched
            )
        }
    }
}

/// The command that runs `component` on `world`, on `model` when one is
/// named, logging to `log` and killed after `deadline`.
#[must_use]
pub fn command(
    component: &Component,
    world: &World,
    model: Option<&str>,
    log: &Path,
    deadline: Duration,
) -> Command {
    let mut args: Vec<String> = vec![
        "-p".into(),
        prompt(component, world),
        "--output-format".into(),
        "stream-json".into(),
        "--verbose".into(),
        "--no-session-persistence".into(),
        "--setting-sources".into(),
        "project".into(),
        "--add-dir".into(),
        PR_HEAD.into(),
    ];
    let (tools, disallowed, turns) = match component.kind {
        Kind::Orchestrator => (
            ORCHESTRATOR_TOOLS,
            ORCHESTRATOR_DISALLOWED,
            class_turns(&world.class) + world.consults,
        ),
        Kind::Specialist => (SPECIALIST_TOOLS, SPECIALIST_DISALLOWED, component.max_turns),
        Kind::Generalist => (GENERALIST_TOOLS, GENERALIST_DISALLOWED, component.max_turns),
    };
    args.extend(["--max-turns".to_owned(), turns.to_string()]);
    args.extend(["--allowedTools".to_owned(), tools.to_owned()]);
    args.extend(["--disallowedTools".to_owned(), disallowed.to_owned()]);
    match component.kind {
        Kind::Orchestrator => {
            args.extend([
                "--model".to_owned(),
                model.unwrap_or(&component.model).to_owned(),
                "--effort".to_owned(),
                component.effort.clone(),
                "--json-schema".to_owned(),
                OUTCOME_SCHEMA.to_owned(),
            ]);
        }
        Kind::Specialist | Kind::Generalist => {
            args.extend(["--agent".to_owned(), component.name.clone()]);
            if let Some(model) = model {
                args.extend(["--model".to_owned(), model.to_owned()]);
            }
        }
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();

    Command::new(PROGRAM, &refs, &world.main)
        .logging_to(log)
        .within(deadline)
}

/// Runs `command` and reads its log.
///
/// # Errors
///
/// Fails when the command cannot start, or logged nothing readable.
pub fn run(host: &dyn Host, command: &Command) -> Result<RunLog, String> {
    let out = host.run(command)?;
    let path = command
        .stdout_to
        .as_deref()
        .ok_or("the run logs to no file")?;
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut log = log::parse(&text);
    if out.timed_out {
        log.answered = false;
        log.subtype = "deadline".to_owned();
    }

    Ok(log)
}

#[cfg(test)]
mod tests;
