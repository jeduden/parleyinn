//! The benchmark suite: each benchmark registered against the NFR id it
//! measures toward, and each NFR id ENG-15 names that no benchmark of
//! real code covers yet, with what it waits on.
//!
//! A benchmark measures code that exists. The MCP tools and the store
//! at scale that NFR-01..05 set targets for arrive with M1's other
//! phases, so they are declared [`AWAITING`] here rather than stood in
//! for by a benchmark of something else. [`SUITE`]'s core start-up is a
//! [`Coverage::Floor`]: a cost every hook handler pays, not a hook
//! handler. Its ingestion benchmark times `parinn ingest` over a
//! synthetic transcript in a fresh home (NFR-04). Its hook handler
//! benchmarks time each handler on a hook input over a fresh home:
//! `SessionStart` against its hook budget (NFR-02), `UserPromptSubmit`
//! a floor of NFR-01's target, which a store of 1M events under fleet
//! load sets. The ENG-15
//! scenario stays pending until every NFR id it names has a
//! [`Coverage::Target`] benchmark.
//!
//! The CI job runs the base commit's suite and the head's. A benchmark
//! renamed or removed on the head goes into [`RETIRED`] in the same
//! change, so the gate reports it rather than failing on it as missing.

use crate::samples::valid_name;

/// How much of its requirement's target a benchmark measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// What the requirement's target names: the requirement is covered.
    Target,
    /// A cost the target's subject pays, a lower bound on it; the
    /// requirement is not covered.
    Floor,
}

/// What a benchmark's core executable runs over, made before each
/// sample and outside its time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixture {
    /// Nothing: the core starts in the benchmark's own environment.
    None,
    /// A fresh home, `HOME` and `PARINN_HOME` both inside it, holding one
    /// transcript of this many lines under its transcript roots, taken
    /// from a recorded Claude Code transcript.
    Transcript { lines: usize },
    /// A fresh home holding one transcript of [`HOOK_LINES`] lines, as
    /// [`Fixture::Transcript`] does, and on the core's stdin the hook
    /// input of the hook this names, in Claude Code 2.1.296's recorded
    /// shape, naming that transcript.
    HookInput { hook: &'static str },
    /// A fresh home as for [`Fixture::Transcript`], prepared by running
    /// the core with `prepare`, outside the time. The core under test
    /// then reads the recall requests of [`crate::fixture::SEARCH_REQUESTS`]
    /// on its standard input.
    Search {
        lines: usize,
        prepare: &'static [&'static str],
    },
}

/// The lines the transcript a hook handler's benchmark names holds.
pub const HOOK_LINES: usize = 200;

/// One benchmark of the suite. Every benchmark today starts the core
/// executable with `args` and times it until it exits; a benchmark that
/// calls into a crate in-process joins with the crate it measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Benchmark {
    /// Its name in the samples file and the gate's report.
    pub name: &'static str,
    /// The NFR id it measures toward.
    pub requirement: &'static str,
    pub coverage: Coverage,
    /// What it measures, in words a reader of the report checks it by.
    pub measures: &'static str,
    /// The core executable's arguments.
    pub args: &'static [&'static str],
    /// What the core runs over.
    pub fixture: Fixture,
}

/// An NFR id ENG-15 names that no benchmark of real code covers yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Awaiting {
    pub requirement: &'static str,
    /// What the benchmark waits on.
    pub on: &'static str,
}

/// The lines the ingestion benchmark's transcript holds.
pub const INGEST_LINES: usize = 2_000;

/// The events the search benchmark's store holds.
pub const SEARCH_LINES: usize = 2_000;

/// The suite.
pub const SUITE: [Benchmark; 5] = [
    Benchmark {
        name: "core_start_up",
        requirement: "NFR-01",
        coverage: Coverage::Floor,
        measures: "the core executable started with `version` until it exits: \
                   the process start every hook handler pays before its own work, \
                   not a hook handler",
        args: &["version"],
        fixture: Fixture::None,
    },
    Benchmark {
        name: "ingest_transcript",
        requirement: "NFR-04",
        coverage: Coverage::Target,
        measures: "`parinn ingest --all` over one transcript of 2,000 recorded Claude Code \
                   lines in a fresh home, until it exits: ingestion's events per second",
        args: &["ingest", "--all"],
        fixture: Fixture::Transcript {
            lines: INGEST_LINES,
        },
    },
    Benchmark {
        name: "hook_session_start",
        requirement: "NFR-02",
        coverage: Coverage::Target,
        measures: "`parinn hook SessionStart` on a recorded-shape hook input in a fresh home, \
                   until it exits: the handler holding its session binding at the \
                   transcript's ingest marker, against its 150 ms hook budget",
        args: &["hook", "SessionStart"],
        fixture: Fixture::HookInput {
            hook: "SessionStart",
        },
    },
    Benchmark {
        name: "hook_user_prompt_submit",
        requirement: "NFR-01",
        coverage: Coverage::Floor,
        measures: "`parinn hook UserPromptSubmit` on a recorded-shape hook input in a fresh \
                   home, until it exits: the handler keeping its prompt commitment, on a \
                   fresh home rather than a store of 1M events under fleet load",
        args: &["hook", "UserPromptSubmit"],
        fixture: Fixture::HookInput {
            hook: "UserPromptSubmit",
        },
    },
    Benchmark {
        name: "search_events",
        requirement: "NFR-03",
        coverage: Coverage::Target,
        measures: "`parinn mcp` answering one `event_search` for a common word over a store \
                   of 2,000 ingested Claude Code lines, until its input closes: a search \
                   ranking the newest 2,000 matches, with the process start it pays",
        args: &["mcp"],
        fixture: Fixture::Search {
            lines: SEARCH_LINES,
            prepare: &["ingest", "--all"],
        },
    },
];

/// The NFR ids ENG-15 names that have no benchmark of real code yet.
pub const AWAITING: [Awaiting; 2] = [
    Awaiting {
        requirement: "NFR-01",
        on: "the hook handlers timed on a store of 1M events with the room view open and \
             ten harnesses writing (plans 2609292012 and M7's)",
    },
    Awaiting {
        requirement: "NFR-05",
        on: "the store at 10M events and 50 concurrent runs writing (plan 2609292012)",
    },
];

/// The benchmarks the suite no longer runs, by the name the base's
/// suite gave them.
pub const RETIRED: [&str; 0] = [];

/// The problems with `suite`: a name the samples file cannot carry, or
/// one used twice, or one also retired.
#[must_use]
pub fn problems(suite: &[Benchmark], retired: &[&str]) -> Vec<String> {
    let mut problems = Vec::new();
    for (i, b) in suite.iter().enumerate() {
        if !valid_name(b.name) {
            problems.push(format!(
                "benchmark {:?} is not lowercase letters, digits and underscores",
                b.name
            ));
        }
        if suite[..i].iter().any(|earlier| earlier.name == b.name) {
            problems.push(format!("benchmark {} is registered twice", b.name));
        }
        if retired.contains(&b.name) {
            problems.push(format!("benchmark {} is registered and retired", b.name));
        }
    }

    problems
}

#[cfg(test)]
mod tests;
