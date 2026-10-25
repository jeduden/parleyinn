//! The `license-check` command line: ENG-30's and ENG-31's checks on a
//! checkout, run by CI on every pull request. It prints each problem on
//! its own line and exits non-zero when there is any.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::third_party::Options;

/// Every crate a release links has a license text that fits, and each
/// target's committed file equals its regeneration.
pub const EXIT_OK: u8 = 0;
/// A crate lacks a fitting license text, or the check could not run.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

/// ENG-30's and ENG-31's checks on a checkout root, as the command runs
/// them.
pub type Check<'a> = &'a dyn Fn(&Path, &Options) -> Result<Vec<String>, String>;

const USAGE: &str = "usage: license-check [--root DIR] [--write] [--out DIR]";

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `check` runs ENG-30's and ENG-31's checks on a checkout root.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    check: Check,
) -> u8 {
    let (root, options) = match flags(args) {
        Ok(parsed) => parsed,
        Err(msg) => {
            let _ = writeln!(stderr, "license-check: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    match check(Path::new(&root), &options) {
        Err(msg) => {
            let _ = writeln!(stderr, "license-check: {msg}");
            EXIT_FAILURE
        }
        Ok(problems) if problems.is_empty() => {
            let _ = writeln!(
                stdout,
                "every crate a release links has a license text and each target's license file is current"
            );
            EXIT_OK
        }
        Ok(problems) => {
            for p in problems {
                let _ = writeln!(stderr, "license-check: {p}");
            }
            EXIT_FAILURE
        }
    }
}

/// The checkout root, from `--root DIR`, the current directory when
/// absent, and the options `--write` and `--out DIR` set.
fn flags(args: impl IntoIterator<Item = String>) -> Result<(String, Options), String> {
    let mut root = ".".to_owned();
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--write" => options.write = true,
            "--root" | "--out" => {
                let value = args
                    .next()
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| format!("{arg} needs a value"))?;
                if arg == "--root" {
                    root = value;
                } else {
                    options.out = Some(PathBuf::from(value));
                }
            }
            _ => return Err(format!("unexpected argument {arg:?}")),
        }
    }

    Ok((root, options))
}

#[cfg(test)]
mod tests;
