//! The scripts a review workflow step runs, read as the step's own
//! text. A step may hold a long `run:` body in a script under
//! `scripts/`; the ENG-28 checks (the agents run by code, the key held
//! apart, the token staged and scrubbed) must see that body as they
//! would inline, so moving it out of the workflow weakens none of them.

use crate::Checkout;

use super::{Workflow, parse};

/// What a step names to run a script of the repository.
const RUN: &str = "run: scripts/";

/// Reads and parses the workflow `file` of `checkout`, with each
/// `run: scripts/<name>.sh` step replaced by the script's text.
///
/// # Errors
///
/// Fails when the workflow or a script it runs cannot be read, or a
/// step passes a script arguments, which the text could not follow.
pub fn load(checkout: &Checkout, file: &str) -> Result<Workflow, String> {
    let body = checkout.read(file)?;

    Ok(parse(&inline_scripts(&body, |path| checkout.read(path))?))
}

/// `body` with each step that runs a script of `scripts/` as a literal
/// block holding the script, read by `read`.
///
/// # Errors
///
/// Fails when `read` fails, a step passes its script arguments, or a
/// step runs a script before its job checks out main's tree, where
/// GitHub would find no script to run.
pub fn inline_scripts(
    body: &str,
    read: impl Fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    inline(body, read, false)
}

/// Like [`inline_scripts`], for a workflow the review does not run: a
/// step that passes its script arguments, or runs it before a checkout,
/// still holds the script, headed by a no-op command with the step's own
/// words, so a hidden agent run or a token handed in the arguments is
/// seen by the ENG-28 sweep and not skipped.
///
/// # Errors
///
/// Fails when `read` fails.
pub fn inline_scripts_lenient(
    body: &str,
    read: impl Fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    inline(body, read, true)
}

fn inline(
    body: &str,
    read: impl Fn(&str) -> Result<String, String>,
    lenient: bool,
) -> Result<String, String> {
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    let (mut out, mut job, mut checked) = (String::new(), "", false);
    for (at, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some(name) = job_header(line) {
            (job, checked) = (name, false);
        }
        if trimmed.starts_with("- ") {
            checked |= checks_out_main(line.len() - trimmed.len(), trimmed, &lines[at + 1..]);
        }
        let step = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some(name) = step.strip_prefix(RUN) else {
            out.push_str(line);
            continue;
        };
        let name = name.trim_end();
        if !lenient && name.contains(char::is_whitespace) {
            return Err(format!("{RUN}{name} passes the script arguments"));
        }
        if !lenient && !checked {
            return Err(format!(
                "{RUN}{name} in job {job} runs before a checkout of main's tree"
            ));
        }
        let file = name.split_whitespace().next().unwrap_or(name);
        let script = read(&format!("scripts/{file}"))?;
        let head = &line[..line.len() - step.len()];
        let indent = " ".repeat(head.len());
        out.push_str(&format!("{head}run: |\n"));
        if name != file {
            out.push_str(&format!("{indent}  : {RUN}{name}\n"));
        }
        for text in script.lines() {
            out.push_str(&format!("{indent}  {text}\n"));
        }
    }

    Ok(out)
}

/// The job a line opens: a key two spaces in, with no value.
fn job_header(line: &str) -> Option<&str> {
    let key = line.strip_prefix("  ")?.trim_end().strip_suffix(':')?;

    (!key.is_empty() && !key.starts_with([' ', '#', '-']) && !key.contains(' ')).then_some(key)
}

/// Whether the step that opens with `first`, trimmed from `indent`
/// spaces in, and goes on
/// through the lines of `rest` deeper than it, is a checkout of main's
/// tree: that action, with no condition, other path, ref or repository,
/// in block or flow form.
fn checks_out_main(indent: usize, first: &str, rest: &[&str]) -> bool {
    let deeper =
        |line: &&&str| line.trim().is_empty() || line.len() - line.trim_start().len() > indent;
    let step = std::iter::once(first.trim_start_matches("- "))
        .chain(rest.iter().take_while(deeper).map(|line| line.trim_start()));
    let (mut action, mut other) = (false, false);
    for line in step {
        action |= line.starts_with("uses: actions/checkout@");
        other |= line.starts_with("if:")
            || ["path:", "ref:", "repository:"]
                .iter()
                .any(|key| line.contains(key));
    }

    action && !other
}

#[cfg(test)]
mod tests;
