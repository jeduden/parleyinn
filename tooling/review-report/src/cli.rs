//! The `review-report` command line:
//!
//! - `review-report trace --workdir DIR` writes the trace and the
//!   report page of the review in `DIR`, as review-run does after every
//!   run, so they can be written again from a kept work directory;
//! - `review-report stability --out FILE DIR DIR...` compares the
//!   reviews in two or more work directories of one pull-request state
//!   and writes their stability to `FILE`, printing its headline,
//!   `key=value` per line, for the workflow's step outputs;
//! - `review-report corpus FILE` reads the stability probe's corpus and
//!   prints its job matrices, for the same ([`crate::corpus`]).

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::stability::{self, Stability};

/// It did what was asked.
pub const EXIT_OK: u8 = 0;
/// A file could not be written.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

/// How to call it.
pub const USAGE: &str = "usage: review-report trace --workdir DIR | \
review-report stability --out FILE DIR DIR... | review-report corpus FILE";

/// Runs the command on `args` and returns its exit code.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let args: Vec<String> = args.into_iter().collect();
    let result = match args.split_first() {
        Some((command, rest)) if command == "trace" => trace(rest, stderr),
        Some((command, rest)) if command == "stability" => stable(rest, stdout, stderr),
        Some((command, [file])) if command == "corpus" => corpus(file, stdout, stderr),
        _ => Err(EXIT_USAGE),
    };
    match result {
        Ok(()) => EXIT_OK,
        Err(code) => {
            if code == EXIT_USAGE {
                let _ = writeln!(stderr, "{USAGE}");
            }
            code
        }
    }
}

/// `trace --workdir DIR`.
fn trace(args: &[String], stderr: &mut dyn Write) -> Result<(), u8> {
    let [flag, workdir] = args else {
        return Err(EXIT_USAGE);
    };
    if flag != "--workdir" {
        return Err(EXIT_USAGE);
    }
    let problems = crate::write(Path::new(workdir), None);
    for problem in &problems {
        let _ = writeln!(stderr, "review-report: {problem}");
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(EXIT_FAILURE)
    }
}

/// `stability --out FILE DIR DIR...`.
fn stable(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<(), u8> {
    let [flag, out, dirs @ ..] = args else {
        return Err(EXIT_USAGE);
    };
    if flag != "--out" || dirs.len() < 2 {
        return Err(EXIT_USAGE);
    }
    let mut notes = Vec::new();
    let runs: Vec<_> = dirs
        .iter()
        .map(|d| stability::read(Path::new(d), &mut notes))
        .collect();
    for note in &notes {
        let _ = writeln!(stderr, "review-report: {note}");
    }
    let measured = stability::compare(&runs);
    let written = serde_json::to_string_pretty(&measured)
        .map_err(|e| e.to_string())
        .and_then(|text| fs::write(out, format!("{text}\n")).map_err(|e| e.to_string()));
    if let Err(e) = written {
        let _ = writeln!(stderr, "review-report: {out} not written: {e}");
        return Err(EXIT_FAILURE);
    }
    if !measured.same_state {
        let _ = writeln!(
            stderr,
            "review-report: the runs reviewed different heads or inputs; their stability means nothing"
        );
    }
    let _ = write!(stdout, "{}", headline(&measured));

    Ok(())
}

/// `corpus FILE`.
fn corpus(file: &str, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<(), u8> {
    let states = fs::read_to_string(file)
        .map_err(|e| format!("read {file}: {e}"))
        .and_then(|text| crate::corpus::parse(&text));
    match states {
        Ok(states) => {
            let _ = write!(stdout, "{}", crate::corpus::matrices(&states));
            Ok(())
        }
        Err(e) => {
            let _ = writeln!(stderr, "review-report: {e}");
            Err(EXIT_FAILURE)
        }
    }
}

/// The headline of `s`, `key=value` per line.
#[must_use]
pub fn headline(s: &Stability) -> String {
    format!(
        "runs={}\nflip_rate={}\njaccard={}\nstable={}\nsame_state={}\nwithin_limits={}\n",
        s.runs, s.flip_rate, s.jaccard, s.stable, s.same_state, s.within_limits
    )
}

#[cfg(test)]
mod tests;
