//! The `review-gate` command line: nine required flags in; a review,
//! nothing or an error out.

use std::io::{self, Write};

use crate::{
    Input, decide, parse_answers, parse_check_runs, parse_checked, parse_consulted, parse_dispatch,
    parse_outcome, parse_specialists,
};

/// The review was printed, or there is none to post.
pub const EXIT_OK: u8 = 0;
/// The gate could not decide; nothing may be posted.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: review-gate --outcome FILE --check-runs FILE --reviewed SHA --head SHA --checked FILE --dispatch FILE --consulted FILE --reports DIR --answers FILE";

/// The flags, in the order the usage names them. Every one is
/// required: a gate run without the deterministic findings could
/// approve what a blocking one withholds, one without the dispatch a
/// review that skipped a specialist, and one without the reports and
/// the answers a report another agent wrote.
const FLAGS: [&str; 9] = [
    "outcome",
    "check-runs",
    "reviewed",
    "head",
    "checked",
    "dispatch",
    "consulted",
    "reports",
    "answers",
];

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `read` maps a path to the file's text, and
/// every test drives the command through here.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    read: &dyn Fn(&str) -> io::Result<String>,
) -> u8 {
    let [
        outcome,
        check_runs,
        reviewed,
        head,
        checked,
        dispatch,
        consulted,
        reports,
        answers,
    ] = match parse_flags(args) {
        Ok(flags) => flags,
        Err(msg) => {
            let _ = writeln!(stderr, "review-gate: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    let files = Files {
        outcome: &outcome,
        check_runs: &check_runs,
        checked: &checked,
        dispatch: &dispatch,
        consulted: &consulted,
        reports: &reports,
        answers: &answers,
    };
    let input = read_input(&files, read).map(|i| Input {
        reviewed,
        head,
        ..i
    });
    let decided = input.and_then(|input| {
        decide(&input)
            .map_err(|e| e.to_string())
            .map(|r| (r, input.reviewed))
    });

    match decided {
        Err(msg) => {
            let _ = writeln!(stderr, "review-gate: {msg}");
            EXIT_FAILURE
        }
        Ok((None, reviewed)) => {
            let _ = writeln!(
                stderr,
                "review-gate: the pull request moved past {reviewed}; its newer run reviews it"
            );
            EXIT_OK
        }
        Ok((Some(review), _)) => {
            match serde_json::to_writer(&mut *stdout, &review)
                .and_then(|()| writeln!(stdout).map_err(serde_json::Error::io))
            {
                Ok(()) => EXIT_OK,
                Err(err) => {
                    let _ = writeln!(stderr, "review-gate: {err}");
                    EXIT_FAILURE
                }
            }
        }
    }
}

/// Reads `--name value` and `--name=value` pairs into the values of
/// [`FLAGS`], in order; each is required.
fn parse_flags(args: impl IntoIterator<Item = String>) -> Result<[String; 9], String> {
    let mut values: [Option<String>; 9] = Default::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        let slot = FLAGS
            .iter()
            .position(|f| *f == name)
            .ok_or_else(|| format!("unknown flag {arg:?}"))?;
        // An empty value is no value: an empty head would match an
        // empty reviewed head and post a review on no commit.
        let value = inline
            .or_else(|| args.next())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("--{name} needs a value"))?;
        values[slot] = Some(value);
    }

    let mut required: [String; 9] = Default::default();
    for (i, value) in values.into_iter().enumerate() {
        required[i] = value.ok_or_else(|| format!("--{} is required", FLAGS[i]))?;
    }

    Ok(required)
}

/// The paths of the files the gate reads.
struct Files<'a> {
    outcome: &'a str,
    check_runs: &'a str,
    checked: &'a str,
    dispatch: &'a str,
    consulted: &'a str,
    /// The directory of the specialists' reports, one `<name>.md` each.
    reports: &'a str,
    answers: &'a str,
}

/// Loads the review outcome, the check runs, the deterministic checks'
/// findings, the dispatch and the agents the review consulted.
fn read_input(files: &Files, read: &dyn Fn(&str) -> io::Result<String>) -> Result<Input, String> {
    let load = |path: &str| read(path).map_err(|e| format!("read {path}: {e}"));
    let outcome = parse_outcome(&load(files.outcome)?).map_err(|e| e.to_string())?;
    let check_runs = parse_check_runs(&load(files.check_runs)?).map_err(|e| e.to_string())?;
    let checked = parse_checked(&load(files.checked)?).map_err(|e| e.to_string())?;
    let dispatch = load(files.dispatch)?;
    let dispatched = parse_dispatch(&dispatch).map_err(|e| e.to_string())?;
    let specialists = parse_specialists(&dispatch).map_err(|e| e.to_string())?;
    let reports = specialists
        .iter()
        .filter_map(|name| {
            read(&format!("{}/{name}.md", files.reports))
                .ok()
                .map(|text| (name.clone(), text))
        })
        .collect();
    let answers = parse_answers(&load(files.answers)?).map_err(|e| e.to_string())?;
    let consulted = parse_consulted(&load(files.consulted)?).map_err(|e| e.to_string())?;

    Ok(Input {
        outcome,
        check_runs,
        checked,
        dispatched,
        consulted,
        specialists,
        reports,
        answers,
        ..Input::default()
    })
}

#[cfg(test)]
mod tests;
