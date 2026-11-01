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
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    let (mut out, mut job, mut checked) = (String::new(), "", false);
    for (at, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some(name) = job_header(line) {
            (job, checked) = (name, false);
        }
        if trimmed.starts_with("- ") {
            checked |= checks_out_main(&lines[at..]);
        }
        let step = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some(name) = step.strip_prefix(RUN) else {
            out.push_str(line);
            continue;
        };
        let name = name.trim_end();
        if name.contains(char::is_whitespace) {
            return Err(format!("{RUN}{name} passes the script arguments"));
        }
        if !checked {
            return Err(format!(
                "{RUN}{name} in job {job} runs before a checkout of main's tree"
            ));
        }
        let script = read(&format!("scripts/{name}"))?;
        let head = &line[..line.len() - step.len()];
        let indent = " ".repeat(head.len());
        out.push_str(&format!("{head}run: |\n"));
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

/// Whether the step that `lines` open, ending where a line is no deeper
/// than its first, is a checkout of main's tree: that action, with no
/// condition and no other path or ref.
fn checks_out_main(lines: &[&str]) -> bool {
    let Some((first, rest)) = lines.split_first() else {
        return false;
    };
    let indent = first.len() - first.trim_start().len();
    let deeper =
        |line: &&&str| line.trim().is_empty() || line.len() - line.trim_start().len() > indent;
    let step = std::iter::once(first.trim_start().trim_start_matches("- "))
        .chain(rest.iter().take_while(deeper).map(|line| line.trim_start()));
    let (mut action, mut other) = (false, false);
    for line in step {
        action |= line.starts_with("uses: actions/checkout@");
        other |= ["if:", "path:", "ref:"]
            .iter()
            .any(|key| line.starts_with(key));
    }

    action && !other
}

#[cfg(test)]
mod tests;
