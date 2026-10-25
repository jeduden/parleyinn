//! The recall-after-compaction dataset of §11.1, version 1: long
//! scripted runs with exact values planted in them (ids, numbers, paths
//! and error strings), each run compacted three times. A value planted
//! before the first compaction is asked for after three, one planted
//! between the last two after one. A seed fixes every choice, so a
//! dataset and its result replay exactly.
//!
//! What makes a value hard to find is what makes it hard in a real
//! run: every turn talks about the same few components, so a search
//! for a value's subject matches many lines; later turns name the
//! subject again without the value; and each compaction's summary names
//! the subjects it dropped the values of.

use crate::lines::{Session, Templates};

/// The dataset's version. A change to what [`generate`] makes is a new
/// version, so a published result names the dataset it measured.
pub const VERSION: u32 = 1;

/// The seeds every published result runs: three repetitions (§11.1).
pub const SEEDS: [u64; 3] = [1, 2, 3];

/// Compactions per run.
pub const COMPACTIONS: usize = 3;

/// Runs per seed.
pub const RUNS: usize = 4;

/// Turns between two compactions.
pub const TURNS: usize = 16;

/// The components a run's turns talk about.
const COMPONENTS: [&str; 12] = [
    "ledger",
    "billing",
    "gateway",
    "scheduler",
    "indexer",
    "mailer",
    "checkout",
    "inventory",
    "notifier",
    "reporting",
    "payments",
    "session",
];

/// The kinds of exact value a fact holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Ticket,
    Latency,
    CrashLog,
    MigrationError,
    DebugPort,
    GoodCommit,
}

/// Every kind, one fact of each per component.
pub const KINDS: [Kind; 6] = [
    Kind::Ticket,
    Kind::Latency,
    Kind::CrashLog,
    Kind::MigrationError,
    Kind::DebugPort,
    Kind::GoodCommit,
];

impl Kind {
    /// The words a question about a fact of this kind names its subject
    /// by, after the component's name.
    #[must_use]
    pub fn subject(self) -> &'static str {
        match self {
            Self::Ticket => "incident ticket",
            Self::Latency => "p95 latency",
            Self::CrashLog => "crash log",
            Self::MigrationError => "migration error",
            Self::DebugPort => "debug port",
            Self::GoodCommit => "last good commit",
        }
    }
}

/// One planted value and the question that asks for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub kind: Kind,
    pub component: String,
    /// The exact value an answer must hold.
    pub value: String,
    /// How many compactions lie between the value and the question.
    pub depth: usize,
}

impl Fact {
    /// The question's words, as an agent searching for the fact types
    /// them: the component and the kind's subject.
    #[must_use]
    pub fn question(&self) -> String {
        format!("{} {}", self.component, self.kind.subject())
    }
}

/// One scripted run: its harness session id, its transcript lines and
/// the facts planted in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub session: String,
    pub lines: Vec<String>,
    /// Where the last compaction's boundary line is in `lines`.
    pub boundary: usize,
    pub facts: Vec<Fact>,
}

impl Run {
    /// What the model's context holds of the run after its last
    /// compaction: the boundary, the summary and every later line.
    #[must_use]
    pub fn after_last_compaction(&self) -> &[String] {
        &self.lines[self.boundary..]
    }
}

/// A small seeded generator (xorshift64*): no clock and no OS
/// randomness, so a seed replays its dataset exactly.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn draw(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number from `low` to below `high`.
    pub fn within(&mut self, low: u64, high: u64) -> u64 {
        low + self.draw() % (high - low)
    }

    /// One of `items`.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.within(0, items.len() as u64) as usize]
    }

    /// `n` lowercase hexadecimal digits.
    pub fn hex(&mut self, n: usize) -> String {
        (0..n)
            .map(|_| char::from(b"0123456789abcdef"[self.within(0, 16) as usize]))
            .collect()
    }

    /// An id in the form of a harness session id.
    pub fn uuid(&mut self) -> String {
        format!(
            "{}-{}-4{}-a{}-{}",
            self.hex(8),
            self.hex(4),
            self.hex(3),
            self.hex(3),
            self.hex(12)
        )
    }
}

/// The values planted so far, so no two facts share one.
#[derive(Debug, Default)]
struct Planted(Vec<String>);

impl Planted {
    /// A value `make` draws that no earlier fact holds.
    fn fresh(&mut self, rng: &mut Rng, make: impl Fn(&mut Rng) -> String) -> String {
        loop {
            let value = make(rng);
            if !self.0.contains(&value) {
                self.0.push(value.clone());
                return value;
            }
        }
    }
}

/// The error names a migration fails with.
const ERRORS: [&str; 5] = [
    "ECONNRESET",
    "ETIMEDOUT",
    "deadlock detected",
    "unique violation",
    "checksum mismatch",
];

/// A value of `kind` for `component`.
fn value_of(kind: Kind, component: &str, rng: &mut Rng) -> String {
    match kind {
        Kind::Ticket => {
            let prefix: String = component.chars().take(3).collect();
            format!("{}-{}", prefix.to_uppercase(), rng.within(10_000, 100_000))
        }
        Kind::Latency => format!("{} ms", rng.within(1_000, 10_000)),
        Kind::CrashLog => format!(
            "/srv/{component}/releases/r{}/crash-{}.log",
            rng.within(100, 1_000),
            rng.hex(6)
        ),
        Kind::MigrationError => format!(
            "{} on {component} shard {}",
            rng.pick(&ERRORS),
            rng.within(10, 100)
        ),
        Kind::DebugPort => rng.within(20_000, 60_000).to_string(),
        Kind::GoodCommit => rng.hex(10),
    }
}

/// Plants `fact` in one turn, the way a run would show it: some in a
/// command's output, some in the assistant's reply.
fn plant(session: &mut Session<'_>, fact: &Fact, rng: &mut Rng) {
    let (c, v) = (&fact.component, &fact.value);
    match fact.kind {
        Kind::Ticket => {
            session.prompt(&format!("The {c} outage needs tracking. Open a ticket."));
            session.reply(&format!(
                "I opened incident ticket {v} for the {c} outage and linked the logs to it."
            ));
        }
        Kind::Latency => {
            session.prompt(&format!("How slow is {c} under load?"));
            let (p50, p99) = (rng.within(100, 1_000), rng.within(10_000, 20_000));
            session.command(
                &format!("./bench/latency.sh {c}"),
                &format!("{c} latency report\np50 {p50} ms\np95 latency {v}\np99 {p99} ms"),
            );
            session.reply(&format!("The {c} p95 latency is {v}."));
        }
        Kind::CrashLog => {
            session.prompt(&format!("Find where {c} wrote its crash log."));
            session.command(
                &format!("find /srv/{c} -name 'crash-*.log'"),
                &format!("{c} crash log: {v}"),
            );
            session.reply(&format!("The {c} crash log is at {v}."));
        }
        Kind::MigrationError => {
            session.prompt(&format!("Run the {c} schema migration."));
            session.command(
                &format!("./migrate.sh {c}"),
                &format!(
                    "applying {c} migration {}\n{c} migration error: {v}\naborted",
                    rng.within(1, 99)
                ),
            );
            session.reply(&format!("The {c} migration error was: {v}."));
        }
        Kind::DebugPort => {
            session.prompt(&format!("Attach a debugger to {c}."));
            session.reply(&format!(
                "The {c} service listens for the debugger on debug port {v}; attach there."
            ));
        }
        Kind::GoodCommit => {
            session.prompt(&format!("Bisect the {c} regression."));
            session.command(
                &format!("git bisect run ./test.sh {c}"),
                &format!("bisect done: {c} last good commit is {v}"),
            );
            session.reply(&format!("The {c} last good commit is {v}."));
        }
    }
}

/// One turn about `component` that plants nothing: work around it, as
/// most of a run is.
fn noise(session: &mut Session<'_>, component: &str, rng: &mut Rng) {
    let n = rng.within(2, 900);
    match rng.within(0, 5) {
        0 => {
            session.prompt(&format!("Run the {component} tests again."));
            session.command(
                &format!("cargo test -p {component}"),
                &format!("test result: ok. {n} passed; 0 failed"),
            );
            session.reply(&format!("All {n} {component} tests pass."));
        }
        1 => {
            session.prompt(&format!("Check the {component} logs for anything new."));
            session.command(
                &format!("tail -n 50 /var/log/{component}.log"),
                &format!("{component}: {n} requests served, no errors"),
            );
            session.reply(&format!("The {component} log shows no errors."));
        }
        2 => {
            session.prompt(&format!("Is the {component} service healthy?"));
            session.reply(&format!(
                "{component} reports healthy with {n} open connections."
            ));
        }
        3 => {
            session.prompt(&format!("Refactor the {component} retry loop."));
            session.command(
                &format!("git diff --stat src/{component}"),
                &format!("src/{component}/retry.rs | {n} ++++----"),
            );
            session.reply(&format!("The {component} retry loop is refactored."));
        }
        _ => {
            session.prompt(&format!("Summarize where we are on {component}."));
            session.reply(&format!(
                "On {component}: tests pass, the latency is being watched, and the \
                 crash log and the last migration were looked at."
            ));
        }
    }
}

/// A turn that names `fact`'s subject again without its value, as a
/// later turn in a real run does.
fn echo(session: &mut Session<'_>, fact: &Fact) {
    session.prompt(&format!(
        "Is the {} still the one we found earlier?",
        fact.question()
    ));
    session.reply(&format!(
        "Yes, the {} has not changed since we found it.",
        fact.question()
    ));
}

/// The summary a compaction leaves of `facts`: their subjects, never
/// their values, as a lossy summary does.
fn summary(facts: &[Fact]) -> String {
    let subjects: Vec<String> = facts.iter().map(Fact::question).collect();
    format!(
        "1. Primary Request and Intent:\n   - Investigated {}.\n\n\
         2. Errors and fixes:\n   - The values found were reported in the replies above.",
        subjects.join(", ")
    )
}

/// One run: [`COMPACTIONS`] + 1 stretches of [`TURNS`] turns, each of
/// the first [`COMPACTIONS`] planting one fact of every kind about a
/// component of its own.
fn run(templates: &Templates, rng: &mut Rng, planted: &mut Planted) -> Run {
    let id = rng.uuid();
    let mut session = Session::new(templates, &id);
    let mut components: Vec<&str> = Vec::new();
    while components.len() < COMPACTIONS {
        let c = *rng.pick(&COMPONENTS);
        if !components.contains(&c) {
            components.push(c);
        }
    }
    let mut facts: Vec<Fact> = Vec::new();
    for stretch in 0..=COMPACTIONS {
        let mut here: Vec<Fact> = components.get(stretch).map_or_else(Vec::new, |c| {
            KINDS
                .iter()
                .map(|&kind| Fact {
                    kind,
                    component: (*c).to_owned(),
                    value: planted.fresh(rng, |r| value_of(kind, c, r)),
                    depth: COMPACTIONS - stretch,
                })
                .collect()
        });
        let mut slots: Vec<usize> = (0..TURNS).collect();
        let mut at: Vec<(usize, Fact)> = here
            .drain(..)
            .map(|f| (slots.remove(rng.within(0, slots.len() as u64) as usize), f))
            .collect();
        at.sort_by_key(|(slot, _)| *slot);
        let mut planting = at.iter().peekable();
        for turn in 0..TURNS {
            match planting.next_if(|(slot, _)| *slot == turn) {
                Some((_, fact)) => plant(&mut session, fact, rng),
                None if !facts.is_empty() && rng.within(0, 4) == 0 => {
                    let earlier = rng.pick(&facts).clone();
                    echo(&mut session, &earlier);
                }
                None => {
                    let c = *rng.pick(&components);
                    noise(&mut session, c, rng);
                }
            }
        }
        let new: Vec<Fact> = at.into_iter().map(|(_, f)| f).collect();
        if stretch < COMPACTIONS {
            session.compact(&summary(&new));
        }
        facts.extend(new);
    }

    let session_id = session.id().to_owned();
    let (lines, boundary) = session.into_lines();
    Run {
        session: session_id,
        lines,
        boundary,
        facts,
    }
}

/// The dataset `seed` makes: [`RUNS`] runs.
#[must_use]
pub fn generate(templates: &Templates, seed: u64) -> Vec<Run> {
    let mut rng = Rng::new(seed);
    let mut planted = Planted::default();
    (0..RUNS)
        .map(|_| run(templates, &mut rng, &mut planted))
        .collect()
}

#[cfg(test)]
mod tests;
