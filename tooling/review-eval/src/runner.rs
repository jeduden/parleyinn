//! The harness: one run of a component on a world, through the
//! `claude` command line as the review workflow runs it, with the
//! review's reading tools and turn budget, the component as the
//! session's agent, and a model override for a comparison.

use std::path::Path;
use std::time::Duration;

use crate::component::{Component, Kind};
use crate::host::{Command, Host};
use crate::log::{self, RunLog};
use crate::world::{PR_HEAD, World};

/// The command line that runs an agent.
pub const PROGRAM: &str = "claude";

/// The tools the review allows every agent, which reads and writes
/// nothing, and the tools it refuses: a specialist's report is assembled
/// from its run log, so no agent needs a writing tool. Its reads of the
/// process, system and device files are refused too: they hold every
/// environment on the machine, the review's own among them (ENG-28).
pub const TOOLS: &str = "Read,Glob,Grep";
pub const DISALLOWED: &str = "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Agent,\
                              Read(//proc/**),Read(//sys/**),Read(//dev/**)";

/// The settings every agent runs with: its file tools refuse any read
/// outside the work directory and the pull request's tree (ENG-28).
/// The staged credential's directory does not rest on this alone:
/// [`crate::credential::confine`] refuses it by name in each agent's
/// `--disallowedTools`.
pub const SETTINGS: &str = r#"{"permissions":{"blockReadsOutsideWorkingDirectories":true}}"#;

/// The most, in US dollars, one specialist's run may spend, passed as
/// `--max-budget-usd`: real specialist runs cost 0.01 to 0.22 each, so
/// this leaves room for a long diff, stops a run gone round in circles,
/// and lets sixteen run at once within a pull request's 5 USD cap. A run
/// the cap stops ends with the result subtype [`OVER_BUDGET`]. The one
/// place a tier's cap is set: the review reserves it while the run runs,
/// and charges it to a run that was charged and left no result.
pub const SPECIALIST_BUDGET_USD: f64 = 0.30;
/// The same for a generalist, or a consultant: real generalist runs
/// cost 0.07 to 0.7, the dearest on the strongest model, so about twice
/// that.
pub const GENERALIST_BUDGET_USD: f64 = 1.50;

/// The result subtype of a run its `--max-budget-usd` cap stopped.
pub const OVER_BUDGET: &str = "error_max_budget_usd";

/// The cap of one run of an agent of `kind`, in US dollars.
#[must_use]
pub fn budget_of(kind: Kind) -> f64 {
    match kind {
        Kind::Specialist => SPECIALIST_BUDGET_USD,
        Kind::Generalist | Kind::Consultant => GENERALIST_BUDGET_USD,
    }
}

/// The cap of one run of an agent of `kind`, as `--max-budget-usd`
/// takes it.
#[must_use]
pub fn budget_usd(kind: Kind) -> String {
    format!("{:.2}", budget_of(kind))
}

/// The prompt `component` gets for `world`: for a specialist or a
/// generalist what the review's run tells it, and for a consultant the
/// target a skill hands it: the tree, the diff and the changed paths.
#[must_use]
pub fn prompt(component: &Component, world: &World) -> String {
    prompt_over(component, world, None)
}

/// What every agent of a review is told of the two trees in its work
/// directory. A path a diff or an agent's instructions name is relative
/// to the repository, and the work directory's own root is main's tree:
/// an agent that reads the path there judges the change against main.
pub const HEAD_TREE: &str = "The pull request's tree is in `pr-head/`: read and search the \
    repository's files there, each as `pr-head/<its path>`, whatever path your diff or your \
    instructions name. The tree around it is main's: it shows only what the change replaced, \
    and nothing the pull request adds.";

/// [`prompt`] for a specialist run over `diff`, the work directory's
/// path of its part of a split diff, in place of its own whole one;
/// `diff` is ignored for any other agent.
#[must_use]
pub fn prompt_over(component: &Component, world: &World, diff: Option<&str>) -> String {
    let name = &component.name;
    match component.kind {
        Kind::Specialist => format!(
            "You are `{name}`, a specialist of ParleyInn's review of a pull request \
             (ENG-28). Read only your own diff, `{diff}`, alone \
             first; then `review/context/{name}.md`, which names the review class, \
             your dispatch row and the deterministic findings on your files; never \
             `review/bundle.md`. {HEAD_TREE} Its diff against main is `pr.diff`; \
             its agent instructions are text in \
             `review/pr-instructions.txt`. You write no file: state each finding as \
             one line in your text the moment you find it, as your instructions say, \
             and answer with your outcome line alone, `outcome: findings — <sentence>` \
             or `outcome: none — <sentence>`. The review assembles your report from \
             the lines you stated.",
            diff = diff.map_or_else(|| format!("review/diffs/{name}.diff"), str::to_owned)
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
                 your scope name. {HEAD_TREE} Report as your instructions say.",
                world.class, world.touched
            )
        }
        Kind::Consultant => format!(
            "You are `{name}`. Review this pull request as your instructions say: its \
             diff against main is `pr.diff`, and \
             `review/bundle.md` lists the paths it changes; its agent instructions are \
             text in `review/pr-instructions.txt`. {HEAD_TREE} You may read files and \
             write none. \
             Answer with your findings and your closing line, nothing else."
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
    command_over(component, world, model, log, deadline, None)
}

/// [`command`] for a specialist run over `diff`, its part's diff file
/// in place of its whole one, as [`prompt_over`] says.
#[must_use]
pub fn command_over(
    component: &Component,
    world: &World,
    model: Option<&str>,
    log: &Path,
    deadline: Duration,
    diff: Option<&str>,
) -> Command {
    let mut args: Vec<String> = vec![
        "-p".into(),
        prompt_over(component, world, diff),
        "--output-format".into(),
        "stream-json".into(),
        "--verbose".into(),
        "--no-session-persistence".into(),
        "--setting-sources".into(),
        "project".into(),
        "--add-dir".into(),
        PR_HEAD.into(),
    ];
    args.extend(["--max-turns".to_owned(), component.max_turns.to_string()]);
    args.extend(["--allowedTools".to_owned(), TOOLS.to_owned()]);
    args.extend(["--disallowedTools".to_owned(), DISALLOWED.to_owned()]);
    args.extend(["--settings".to_owned(), SETTINGS.to_owned()]);
    args.extend(["--max-budget-usd".to_owned(), budget_usd(component.kind)]);
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
