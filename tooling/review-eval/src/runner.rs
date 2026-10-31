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

/// The tools the review allows and refuses a specialist, which writes
/// only its report, and a generalist, which writes nothing.
pub const SPECIALIST_TOOLS: &str = "Read,Glob,Grep,Edit(./review/reports/**)";
pub const SPECIALIST_DISALLOWED: &str = "Bash,Edit,NotebookEdit,WebFetch,WebSearch,Agent";
pub const GENERALIST_TOOLS: &str = "Read,Glob,Grep";
pub const GENERALIST_DISALLOWED: &str = "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Agent";

/// The prompt `component` gets for `world`: for a specialist or a
/// generalist what the review's run tells it, and for a consultant the
/// target a skill hands it: the tree, the diff and the changed paths.
#[must_use]
pub fn prompt(component: &Component, world: &World) -> String {
    let name = &component.name;
    match component.kind {
        Kind::Specialist => format!(
            "You are `{name}`, a specialist of ParleyInn's review of a pull request \
             (ENG-28). Read `review/bundle.md` first, then your own diff, \
             `review/diffs/{name}.diff`. The pull request's tree is in `pr-head/` and \
             its diff against main is `pr.diff`; its agent instructions are text in \
             `review/pr-instructions.txt`. Write your report to \
             `review/reports/{name}.md`, ending with its outcome line, and \
             answer exactly two lines and nothing else: the report's outcome line \
             copied character for character, then the report's path."
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
        Kind::Consultant => format!(
            "You are `{name}`. Review this pull request as your instructions say: its              tree is in `pr-head/`, its diff against main is `pr.diff`, and              `review/bundle.md` lists the paths it changes; its agent instructions are              text in `review/pr-instructions.txt`. You may read files and write none.              Answer with your findings and your closing line, nothing else."
        ),
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
    let (tools, disallowed) = match component.kind {
        Kind::Specialist => (SPECIALIST_TOOLS, SPECIALIST_DISALLOWED),
        Kind::Generalist | Kind::Consultant => (GENERALIST_TOOLS, GENERALIST_DISALLOWED),
    };
    args.extend(["--max-turns".to_owned(), component.max_turns.to_string()]);
    args.extend(["--allowedTools".to_owned(), tools.to_owned()]);
    args.extend(["--disallowedTools".to_owned(), disallowed.to_owned()]);
    match component.kind {
        Kind::Specialist | Kind::Generalist => {
            args.extend(["--agent".to_owned(), component.name.clone()]);
            if let Some(model) = model {
                args.extend(["--model".to_owned(), model.to_owned()]);
            }
        }
        // A consultant's front matter may name no model, so the run
        // always names the one it resolved.
        Kind::Consultant => {
            args.extend([
                "--agent".to_owned(),
                component.name.clone(),
                "--model".to_owned(),
                model.unwrap_or(&component.model).to_owned(),
            ]);
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
