//! The command line: `parinn-eval recall` runs the recall evaluation on
//! every seed and prints its report; with `--published FILE` it also
//! fails when FILE, the result the repository publishes, differs from
//! the report it just measured.

use std::path::Path;

use testkit::TempDir;

use crate::dataset::{Run, VERSION};
use crate::lines::Templates;
use crate::report::{self, Outcome};
use crate::{TARGET, evaluate, templates};

/// How to call it.
pub const USAGE: &str = "usage: parinn-eval recall [--published FILE]\n";

/// The result the repository publishes, relative to the checkout.
pub const PUBLISHED: &str = "tooling/eval/recall-v1.txt";

/// The dataset a seed makes from the fixture's line shapes.
pub type Make = fn(&Templates, u64) -> Vec<Run>;

/// Every seed's outcomes, each seed's dataset from `make` evaluated in
/// a fresh home, and how many lines of the datasets did not parse.
///
/// # Errors
///
/// Fails when the fixture cannot be read or a seed's evaluation fails.
pub fn measure(root: &Path, seeds: &[u64], make: Make) -> Result<(Vec<Outcome>, usize), String> {
    let templates = templates(root)?;
    let mut outcomes: Vec<Outcome> = Vec::new();
    let mut unparsed = 0;
    for &seed in seeds {
        let dir = TempDir::new().map_err(|e| format!("make a home: {e}"))?;
        let runs = make(&templates, seed);
        let evaluated = evaluate(&runs, seed, dir.path())?;
        unparsed += evaluated.unparsed;
        outcomes.extend(evaluated.outcomes);
    }

    Ok((outcomes, unparsed))
}

/// The report of `outcomes` over `seeds`, and whether every depth
/// reached [`TARGET`] with every line parsed.
#[must_use]
pub fn conclude(seeds: &[u64], outcomes: &[Outcome], unparsed: usize) -> (String, bool) {
    let mut out = report::render(VERSION, seeds, outcomes);
    let below = report::below(outcomes, TARGET);
    for depth in &below {
        out.push_str(&format!("depth {depth} is below the target of {TARGET}\n"));
    }
    if unparsed > 0 {
        out.push_str(&format!("{unparsed} lines of the dataset did not parse\n"));
    }

    (out, below.is_empty() && unparsed == 0)
}

/// Whether `published`'s text is `report`; what differs when not.
///
/// # Errors
///
/// Fails when the file cannot be read or holds another report.
pub fn matches(published: &Path, report: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(published)
        .map_err(|e| format!("read {}: {e}", published.display()))?;
    if text == report {
        Ok(())
    } else {
        Err(format!(
            "{} is not the result measured now; publish this report in its place",
            published.display()
        ))
    }
}

/// Runs `args` against the checkout at `root` over `seeds`, each
/// seed's dataset from `make`: the exit code and what to print.
#[must_use]
pub fn run(args: &[String], root: &Path, seeds: &[u64], make: Make) -> (u8, String) {
    let published = match args {
        [verb] if verb == "recall" => None,
        [verb, flag, file] if verb == "recall" && flag == "--published" => Some(root.join(file)),
        _ => return (2, USAGE.to_owned()),
    };
    match measure(root, seeds, make) {
        Ok((outcomes, unparsed)) => {
            let (out, met) = conclude(seeds, &outcomes, unparsed);
            let differs = published.and_then(|p| matches(&p, &out).err());
            match (met, differs) {
                (true, None) => (0, out),
                (_, Some(e)) => (1, format!("{out}parinn-eval: {e}\n")),
                (false, None) => (1, out),
            }
        }
        Err(e) => (1, format!("parinn-eval: {e}\n")),
    }
}

#[cfg(test)]
mod tests;
