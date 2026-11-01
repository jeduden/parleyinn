//! The `review-run answers` command: `answers drop KEY --answers DIR`
//! takes the answer stored under `KEY` out of the answer store `DIR`, so
//! an answer found wrong or injected is never reused again, while every
//! other answer stays; it says what it dropped, or why it could not (I5,
//! I6). It writes nothing else.

use std::io::Write;

use crate::cli::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};
use crate::store::Store;

/// How to call it.
pub const USAGE: &str = "usage: review-run answers drop KEY --answers DIR";

/// The arguments after `answers` when `args`, past the job summary's
/// flag the executable adds first, name the `answers` command; none
/// otherwise.
#[must_use]
pub fn subcommand(args: &[String]) -> Option<Vec<String>> {
    let mut rest = args.iter().skip_while(|a| a.starts_with("--summary="));
    (rest.next().map(String::as_str) == Some("answers")).then(|| rest.cloned().collect())
}

/// Runs `answers` with `args`, the arguments after it, and returns its
/// exit code.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let (key, dir) = match parse(args) {
        Ok(parsed) => parsed,
        Err(msg) => {
            let _ = writeln!(stderr, "review-run answers: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    let store = Store {
        dir: dir.clone().into(),
        read_only: false,
    };
    match store.drop(&key) {
        Ok(()) => match writeln!(stdout, "dropped {key} from {dir}") {
            Ok(()) => EXIT_OK,
            Err(e) => {
                let _ = writeln!(stderr, "review-run answers: dropped {key}, but say so: {e}");
                EXIT_FAILURE
            }
        },
        Err(e) => {
            let _ = writeln!(stderr, "review-run answers: {e}");
            EXIT_FAILURE
        }
    }
}

/// Reads `drop KEY --answers DIR`, the flag before or after the key, as
/// `--answers DIR` or `--answers=DIR`.
fn parse(args: &[String]) -> Result<(String, String), String> {
    let mut args = args.iter();
    if args.next().map(String::as_str) != Some("drop") {
        return Err("the one answers command is drop".to_owned());
    }
    let (mut key, mut dir) = (None, None);
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--answers=") {
            dir = Some(value.to_owned());
        } else if arg == "--answers" {
            dir = args.next().cloned();
        } else if arg.starts_with("--") {
            return Err(format!("unknown flag {arg:?}"));
        } else if key.is_none() {
            key = Some(arg.clone());
        } else {
            return Err(format!("unexpected argument {arg:?}"));
        }
    }
    let dir = dir
        .filter(|d| !d.is_empty())
        .ok_or("--answers needs a value")?;
    let key = key.ok_or("drop needs the KEY of the answer to drop")?;

    Ok((key, dir))
}

#[cfg(test)]
mod tests;
