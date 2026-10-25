//! `parinn-eval`, the evaluation module of SRS §11.1: versioned
//! datasets and fixed seeds run through ParleyInn's own components, so
//! every result replays. Its first evaluation is recall after
//! compaction for M1 (§12.2): long scripted runs with exact values
//! planted before one to three compactions are ingested into a fresh,
//! isolated home, and a scripted agent asks the MCP server's recall
//! tools for each value. The result is measured, never written in.
//! Beside it the report counts what a run without recall still holds:
//! the values left in its context after its last compaction.
//!
//! It is repository tooling and never links into a shipped executable.

pub mod agent;
pub mod cli;
pub mod dataset;
pub mod lines;
pub mod report;

use std::path::Path;

use administration::home::Home;
use administration::node::Node;
use key::random::OsEntropy;
use mcp::Server;
use redaction::Redactor;
use store::Store;

use crate::agent::Tools;
use crate::dataset::{Fact, Run};
use crate::lines::Templates;
use crate::report::Outcome;

/// The recall every depth must reach (§12.2, M1).
pub const TARGET: f64 = 0.95;

/// The line shapes of the committed fixture under `root`, the
/// repository checkout.
///
/// # Errors
///
/// Fails when the fixture cannot be read or lacks a shape.
pub fn templates(root: &Path) -> Result<Templates, String> {
    let path = root.join(lines::FIXTURE);
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Templates::from_transcript(&text)
}

/// What one seed's evaluation found, and how many of its lines the
/// transcript parser could not read, which a dataset must not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluated {
    pub outcomes: Vec<Outcome>,
    pub unparsed: usize,
}

/// Each run's transcript written under `dir` and ingested into `home`:
/// the writer of each run's own events, and how many lines did not
/// parse.
fn ingest_runs(home: &Home, runs: &[Run], dir: &Path) -> Result<(Vec<String>, usize), String> {
    let (redactor, _) = Redactor::new(&[]).map_err(|e| e.to_string())?;
    let mut writers = Vec::new();
    let mut unparsed = 0;
    for run in runs {
        let path = dir.join(format!("{}.jsonl", run.session));
        std::fs::write(&path, run.lines.join("\n") + "\n")
            .map_err(|e| format!("write {}: {e}", path.display()))?;
        let ingested = ingest::ingest_path(home, &path, &redactor, &mut OsEntropy)
            .map_err(|e| e.to_string())?;
        unparsed += ingested.unparsed;
        writers.push(ingested.writer.to_string());
    }

    Ok((writers, unparsed))
}

/// The outcome of asking for `fact` in the run whose writer is
/// `writer`, and whether `context`, what the run holds after its last
/// compaction, holds the value without recall.
///
/// # Errors
///
/// Fails when a recall tool call fails.
pub fn ask(
    tools: &mut dyn Tools,
    seed: u64,
    writer: &str,
    fact: &Fact,
    context: &[String],
) -> Result<Outcome, String> {
    let question = fact.question();
    let read = agent::recall(tools, writer, &question)?;

    Ok(Outcome {
        seed,
        depth: fact.depth,
        rank: read
            .iter()
            .position(|c| agent::holds(c, &fact.value))
            .map(|i| i + 1),
        in_context: context.iter().any(|l| agent::holds(l, &fact.value)),
        question,
        value: fact.value.clone(),
    })
}

/// Evaluates `runs`, `seed`'s dataset, in a home under `dir`: each
/// run's transcript written and ingested, then every fact asked for in
/// its own run through the recall tools.
///
/// # Errors
///
/// Fails when a transcript cannot be written or ingested, the store or
/// node cannot be opened, or a recall tool call fails.
pub fn evaluate(runs: &[Run], seed: u64, dir: &Path) -> Result<Evaluated, String> {
    let home =
        Home::locate(Some(dir.join(".parinn").as_os_str()), None).map_err(|e| e.to_string())?;
    let (writers, unparsed) = ingest_runs(&home, runs, dir)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = Node::find(&home, &store).map_err(|e| e.to_string())?;
    let mut server = Server::new(
        store,
        "parinn-eval",
        node.device().public(),
        node.personal_room(),
    );
    let mut outcomes = Vec::new();
    for (run, writer) in runs.iter().zip(&writers) {
        let context = run.after_last_compaction();
        for fact in &run.facts {
            outcomes.push(ask(&mut server, seed, writer, fact, context)?);
        }
    }

    Ok(Evaluated { outcomes, unparsed })
}

#[cfg(test)]
mod tests;
