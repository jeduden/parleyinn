//! A candidate model against the published scorecard, per component:
//! whether it passes more cases, the same at a lower cost, the same,
//! or fewer, so the stakeholder can move a component to the model that
//! serves its instructions better.

use std::fmt::Write as _;

use crate::published::Line;

/// A cost per run at or below this share of the published one makes a
/// candidate better; at or above the inverse, worse.
const CHEAPER: f64 = 0.8;
/// A run pass rate this far below the published one makes a candidate
/// worse even with the same cases passed.
const RATE_DROP: f64 = 0.1;

/// Where a candidate stands against the published model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    Better,
    Equivalent,
    Worse,
    /// The published scorecard has no line for the component.
    New,
}

impl Standing {
    /// The word for it.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Better => "better",
            Self::Equivalent => "equivalent",
            Self::Worse => "worse",
            Self::New => "new",
        }
    }
}

/// One component's comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub component: String,
    pub standing: Standing,
    pub why: String,
    pub published: Option<Line>,
    pub candidate: Line,
}

/// Compares each of `candidate`'s lines with `published`'s line for
/// the same component.
#[must_use]
pub fn compare(published: &[Line], candidate: &[Line]) -> Vec<Comparison> {
    let mut out: Vec<Comparison> = candidate
        .iter()
        .map(|c| {
            let p = published.iter().find(|p| p.component == c.component);
            let (standing, why) = p.map_or_else(
                || (Standing::New, "no published line".to_owned()),
                |p| stand(p, c),
            );
            Comparison {
                component: c.component.clone(),
                standing,
                why,
                published: p.cloned(),
                candidate: c.clone(),
            }
        })
        .collect();
    out.sort_by(|a, b| a.component.cmp(&b.component));

    out
}

/// Where `candidate` stands against `published`, and why.
fn stand(published: &Line, candidate: &Line) -> (Standing, String) {
    if candidate.cases_passed > published.cases_passed {
        return (
            Standing::Better,
            format!(
                "passes {} of {} cases, the published model {}",
                candidate.cases_passed, candidate.cases, published.cases_passed
            ),
        );
    }
    if candidate.cases_passed < published.cases_passed {
        return (
            Standing::Worse,
            format!(
                "passes {} of {} cases, the published model {}",
                candidate.cases_passed, candidate.cases, published.cases_passed
            ),
        );
    }
    let (pr, cr) = (published.rate(), candidate.rate());
    if cr < pr - RATE_DROP {
        return (
            Standing::Worse,
            format!(
                "the same cases, but the runs' pass rate fell from {:.0}% to {:.0}%",
                pr * 100.0,
                cr * 100.0
            ),
        );
    }
    let (pc, cc) = (published.cost_per_run(), candidate.cost_per_run());
    if pc > 0.0 {
        let share = cc / pc;
        if share <= CHEAPER {
            return (
                Standing::Better,
                format!(
                    "the same cases at {:.0}% of the cost per run",
                    share * 100.0
                ),
            );
        }
        if share >= 1.0 / CHEAPER {
            return (
                Standing::Worse,
                format!(
                    "the same cases at {:.0}% of the cost per run",
                    share * 100.0
                ),
            );
        }
    }

    (
        Standing::Equivalent,
        "the same cases at about the same cost".to_owned(),
    )
}

/// How many of `comparisons` stand each way: better, equivalent,
/// worse, new.
#[must_use]
pub fn counts(comparisons: &[Comparison]) -> [usize; 4] {
    let mut out = [0; 4];
    for c in comparisons {
        out[c.standing as usize] += 1;
    }

    out
}

/// The comparisons as Markdown.
#[must_use]
pub fn render(comparisons: &[Comparison]) -> String {
    let [better, equivalent, worse, new] = counts(comparisons);
    let mut out = format!(
        "## Model comparison\n\n{better} better, {equivalent} equivalent, {worse} worse, {new} new.\n\n\
         | component | published | candidate | standing | why |\n|---|---|---|---|---|\n"
    );
    for c in comparisons {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            c.component,
            c.published.as_ref().map_or("none".to_owned(), cell),
            cell(&c.candidate),
            c.standing.word(),
            c.why
        );
    }

    out
}

/// One line's figures in a cell.
fn cell(l: &Line) -> String {
    format!(
        "{} ({}): {}/{} cases, {}/{} runs, ${:.2} per run",
        l.model,
        l.resolved,
        l.cases_passed,
        l.cases,
        l.runs_passed,
        l.runs,
        l.cost_per_run()
    )
}

#[cfg(test)]
mod tests;
