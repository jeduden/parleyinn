//! The `coverage` command line: measure, print the table, and fail on a
//! crate below its floor.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Host;
use crate::lcov;
use crate::measure::{Shard, measure, measure_layer, merge_layers, shard_file_name};
use crate::plan::{Plan, TestLayer};
use crate::report::{shortfalls, table, tally};
use crate::shape;

/// Every crate meets its floors.
pub const EXIT_OK: u8 = 0;
/// A crate is below a floor, or the measurement failed.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

const USAGE: &str = "usage: coverage [--out DIR] [--layer NAME [--shard K/N] | --merge]

Runs cargo-llvm-cov once per test layer (unit, integration, e2e),
prints line coverage per crate and layer, writes each layer's lcov to
DIR (target/coverage by default), and fails when a crate is below a
floor in [workspace.metadata.coverage].

--layer NAME runs one layer alone and writes DIR/NAME.lcov, with no
table and no floors; --merge reads the layers written that way under
DIR and prints the table and the floors without running a test. CI
measures each layer in a job of its own, then merges them.

--shard K/N, for --layer e2e, runs shard K of N of the scenarios and
writes DIR/e2e-K-of-N.lcov; --merge adds the N files up as e2e.lcov
and fails when one is missing.";

/// What one run does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Every layer, then the table and the floors.
    Measure,
    /// One layer's lcov, nothing else; of the end-to-end layer, one
    /// shard's when the shard is set.
    Layer(TestLayer, Option<Shard>),
    /// The table and the floors from the lcov files already written.
    Merge,
}

/// Runs the command on `args` and returns its exit code. It is `main`
/// without the process: `host` runs cargo and writes files, and
/// `summary`, when set, is a Markdown file the table is appended to,
/// as GitHub's `$GITHUB_STEP_SUMMARY` is. `nested` says the process
/// runs inside a measurement, where it refuses to start another.
pub fn run(
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    host: &dyn Host,
    summary: Option<&Path>,
    nested: bool,
) -> u8 {
    let (out, mode) = match parse_args(args) {
        Ok(Some(parsed)) => parsed,
        Ok(None) => {
            let _ = writeln!(stdout, "{USAGE}");
            return EXIT_OK;
        }
        Err(msg) => {
            let _ = writeln!(stderr, "coverage: {msg}\n{USAGE}");
            return EXIT_USAGE;
        }
    };

    if nested {
        let _ = writeln!(
            stderr,
            "coverage: already inside a measurement ({}); refusing to recurse",
            crate::MEASURING
        );
        return EXIT_USAGE;
    }

    match report(host, &out, mode, summary) {
        Ok((table, shortfalls)) => {
            let _ = write!(stdout, "{table}");
            for s in &shortfalls {
                let _ = writeln!(stderr, "{s}");
            }
            if shortfalls.is_empty() {
                EXIT_OK
            } else {
                EXIT_FAILURE
            }
        }
        Err(msg) => {
            let _ = writeln!(stderr, "coverage: {msg}");
            EXIT_FAILURE
        }
    }
}

/// Reads `--out DIR` and the mode; `None` asks for the usage.
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<(PathBuf, Mode)>, String> {
    let mut out = PathBuf::from("target/coverage");
    let mut mode = Mode::Measure;
    let mut shard = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let (flag, value) = match arg.split_once('=') {
            Some((flag, value)) => (flag, Some(value.to_owned())),
            None => (arg.as_str(), None),
        };
        match flag {
            "-h" | "--help" => return Ok(None),
            "--out" => {
                out = value
                    .or_else(|| args.next())
                    .map(PathBuf::from)
                    .ok_or("--out needs a directory")?;
            }
            "--layer" => {
                let name = value
                    .or_else(|| args.next())
                    .ok_or("--layer needs a name: unit, integration or e2e")?;
                let layer = TestLayer::from_name(&name)
                    .ok_or_else(|| format!("no test layer {name:?}: unit, integration or e2e"))?;
                mode = set_mode(mode, Mode::Layer(layer, None))?;
            }
            "--shard" => {
                let text = value
                    .or_else(|| args.next())
                    .ok_or("--shard needs K/N, such as 1/4")?;
                shard = Some(text.parse::<Shard>()?);
            }
            "--merge" => mode = set_mode(mode, Mode::Merge)?,
            _ => return Err(format!("unexpected argument {arg:?}")),
        }
    }

    let mode = match (mode, shard) {
        (mode, None) => mode,
        (Mode::Layer(TestLayer::EndToEnd, _), shard) => Mode::Layer(TestLayer::EndToEnd, shard),
        _ => return Err("--shard goes with --layer e2e".to_owned()),
    };

    Ok(Some((out, mode)))
}

/// `wanted` in place of `Measure`; a second mode is refused.
fn set_mode(current: Mode, wanted: Mode) -> Result<Mode, String> {
    match current {
        Mode::Measure => Ok(wanted),
        _ => Err("--layer and --merge are one or the other, once".to_owned()),
    }
}

/// Measures, or merges, and returns the table and the shortfalls; for
/// one layer, what it wrote and no shortfall.
fn report(
    host: &dyn Host,
    out: &Path,
    mode: Mode,
    summary: Option<&Path>,
) -> Result<(String, Vec<String>), String> {
    let metadata = host.cargo_output(
        &["metadata", "--format-version", "1", "--no-deps", "--locked"].map(str::to_owned),
    )?;
    let plan = Plan::from_metadata(&metadata)?;
    let layers = match mode {
        Mode::Measure => measure(host, &plan, out)?,
        Mode::Merge => merge_layers(host, out)?,
        Mode::Layer(layer, shard) => {
            let name = shard.map_or_else(|| format!("{layer}.lcov"), |s| shard_file_name(layer, s));
            let line = match measure_layer(host, &plan, layer, shard, out)? {
                Some(_) => format!("wrote {}\n", out.join(name).display()),
                None => format!("no {layer} tests to measure\n"),
            };
            return Ok((line, Vec::new()));
        }
    };
    let all = lcov::merge(layers.iter().map(|(_, lines)| lines));
    let rows = tally(&plan, &layers, &all);
    let mut sources = Vec::new();
    for krate in &plan.crates {
        sources.extend(host.sources(&krate.dir)?);
    }
    let shape = shape::count(
        &plan,
        &sources,
        host.bound_scenarios(&plan.root.join("features"))?,
    );
    let table = format!("{}\n{}\n", table(&rows), shape.line());
    if let Some(summary) = summary {
        host.append(
            summary,
            &format!("## Line coverage per test layer\n\n{table}\n"),
        )?;
    }
    let mut problems = shortfalls(&plan, &rows);
    problems.extend(shape.inverted().filter(|_| plan.pyramid));

    Ok((table, problems))
}

#[cfg(test)]
mod tests;
