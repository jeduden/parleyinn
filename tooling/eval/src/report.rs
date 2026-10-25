//! The report of a recall evaluation: per depth and per seed, how many
//! planted values the agent recalled exactly, the recall, and its 95%
//! Wilson score interval over every seed together (§11.1 asks for
//! confidence intervals). The agent reads one page of the tool's
//! default size; the report also says how many of the values a page
//! half that size would have held, so a ranking that only just holds
//! a value shows.

use std::fmt::Write as _;

/// The page the agent reads: `event_search`'s default number of hits,
/// which RCL-02 sets at 10.
pub const PAGE: usize = 10;

/// The smaller page the report also counts.
pub const SMALL_PAGE: usize = PAGE / 2;

/// What the agent found for one fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub seed: u64,
    /// Compactions between the value and the question.
    pub depth: usize,
    pub question: String,
    pub value: String,
    /// The rank, from 1, of the first hit holding the value; none when
    /// no hit the agent read holds it.
    pub rank: Option<usize>,
    /// Whether the run's context after its last compaction still holds
    /// the value: what a run without recall could answer from.
    pub in_context: bool,
}

impl Outcome {
    /// Whether a page of `page` hits held the value.
    #[must_use]
    pub fn within(&self, page: usize) -> bool {
        self.rank.is_some_and(|r| r <= page)
    }
}

/// Recalled out of asked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    pub found: u64,
    pub asked: u64,
}

impl Tally {
    /// The share recalled; none when nothing was asked.
    #[must_use]
    pub fn recall(self) -> Option<f64> {
        (self.asked > 0).then(|| self.found as f64 / self.asked as f64)
    }

    /// The 95% Wilson score interval of the recall; none when nothing
    /// was asked.
    #[must_use]
    pub fn interval(self) -> Option<(f64, f64)> {
        let p = self.recall()?;
        let (n, z) = (self.asked as f64, 1.96_f64);
        let centre = (p + z * z / (2.0 * n)) / (1.0 + z * z / n);
        let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / (1.0 + z * z / n);
        Some((centre - half, centre + half))
    }
}

/// The tally, within `page` hits, of the `outcomes` that `keep` keeps.
#[must_use]
pub fn tally(outcomes: &[Outcome], page: usize, keep: impl Fn(&Outcome) -> bool) -> Tally {
    outcomes
        .iter()
        .filter(|o| keep(o))
        .fold(Tally::default(), |t, o| Tally {
            found: t.found + u64::from(o.within(page)),
            asked: t.asked + 1,
        })
}

/// The depths `outcomes` hold, in order and once each.
fn depths(outcomes: &[Outcome]) -> Vec<usize> {
    let mut d: Vec<usize> = outcomes.iter().map(|o| o.depth).collect();
    d.sort_unstable();
    d.dedup();
    d
}

/// The depths whose recall within [`PAGE`] over every seed is below
/// `target`.
#[must_use]
pub fn below(outcomes: &[Outcome], target: f64) -> Vec<usize> {
    depths(outcomes)
        .into_iter()
        .filter(|&d| {
            tally(outcomes, PAGE, |o| o.depth == d)
                .recall()
                .unwrap_or(0.0)
                < target
        })
        .collect()
}

/// A share as a percentage with one decimal.
fn percent(share: f64) -> String {
    format!("{:.1}%", share * 100.0)
}

/// The report's text: a line per depth over every seed with its
/// interval, the same within [`SMALL_PAGE`], what the context holds
/// without recall per depth, a line per seed and depth, then every
/// value not recalled.
#[must_use]
pub fn render(version: u32, seeds: &[u64], outcomes: &[Outcome]) -> String {
    let mut out = format!(
        "recall after compaction, dataset v{version}, seeds {seeds:?}, {} questions, \
         {PAGE} hits read per question\n",
        outcomes.len()
    );
    for page in [PAGE, SMALL_PAGE] {
        for d in depths(outcomes) {
            let t = tally(outcomes, page, |o| o.depth == d);
            let (low, high) = t.interval().unwrap_or((0.0, 0.0));
            let _ = writeln!(
                out,
                "within {page} hits, depth {d}: {}/{} recalled, {} (95% interval {} to {})",
                t.found,
                t.asked,
                percent(t.recall().unwrap_or(0.0)),
                percent(low),
                percent(high)
            );
        }
    }
    for d in depths(outcomes) {
        let kept = outcomes
            .iter()
            .filter(|o| o.depth == d && o.in_context)
            .count();
        let asked = outcomes.iter().filter(|o| o.depth == d).count();
        let _ = writeln!(
            out,
            "without recall, in the context after the last compaction, depth {d}: {kept}/{asked}"
        );
    }
    for &seed in seeds {
        for d in depths(outcomes) {
            let t = tally(outcomes, PAGE, |o| o.seed == seed && o.depth == d);
            let _ = writeln!(out, "seed {seed}, depth {d}: {}/{}", t.found, t.asked);
        }
    }
    for o in outcomes.iter().filter(|o| !o.within(PAGE)) {
        let _ = writeln!(
            out,
            "not recalled: seed {}, depth {}: {:?} = {:?}",
            o.seed, o.depth, o.question, o.value
        );
    }

    out
}

#[cfg(test)]
mod tests;
