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
/// Fails when `read` fails or a step passes its script arguments.
pub fn inline_scripts(
    body: &str,
    read: impl Fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let mut out = String::new();
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let step = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some(name) = step.strip_prefix(RUN) else {
            out.push_str(line);
            continue;
        };
        let name = name.trim_end();
        if name.contains(char::is_whitespace) {
            return Err(format!("{RUN}{name} passes the script arguments"));
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

#[cfg(test)]
mod tests;
