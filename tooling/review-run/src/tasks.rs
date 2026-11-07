//! The review's tasks and what each waits for: every specialist's run,
//! whole or one part of its split diff, waits for nothing; a split
//! specialist's merge waits for its parts; and each generalist waits
//! for its own specialists alone, the dispatch's, and for the breaker
//! to be decided, so it starts while unrelated specialists still run,
//! once too few of them run to trip the breaker. The schedule runs them
//! within the review's bound, specialists first; the runs come back in
//! plan order, not in the order they ended, so the outcome decided from
//! them is the one the two waves of old gave. A specialist that ends in
//! a gap is counted by the breaker, and once it trips no generalist
//! starts, which, as it is decided before any starts, no order of
//! ending changes; a run waits while its tier's cap does not fit in the
//! pull request's cost cap beside the runs in flight, and a run the
//! cost cap refuses is not started, and ends in a gap the breaker counts
//! as any other. An agent
//! kept whole from an earlier run is taken as it answered there, and a
//! part kept is held to the report check again.

use std::collections::BTreeMap;

use review_eval::{component, runner};
use review_gate::report::assemble;

use crate::controls::Controls;
use crate::memo;
use crate::parts;
use crate::pipeline::pipeline;
use crate::plan::Plan;
use crate::resume::Reuse;
use crate::retry::{refusal, run_one};
use crate::run::{AgentRun, Review, Unit, describe, not_started, reused, specialist_units};

/// A task's result: its run, and a specialist's report.
pub(crate) type Done = (AgentRun, Option<String>);

/// One task of the review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Task {
    /// A specialist's run, whole or of one part.
    Unit(Unit),
    /// A split specialist's merge of its parts' runs, by its name.
    Merge(String),
    /// A generalist's run.
    Generalist(Unit),
}

/// The tasks, what each waits for, and each agent's last task, the one
/// whose run is the agent's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Graph {
    pub(crate) tasks: Vec<Task>,
    pub(crate) waits: Vec<Vec<usize>>,
    pub(crate) last: BTreeMap<String, usize>,
}

/// The graph of `units`, the specialists' runs, then a merge for each of
/// `plan`'s specialists run in parts, then `generalists`, each waiting
/// for the last task of each of its specialists that has one.
pub(crate) fn graph(plan: &Plan, units: Vec<Unit>, generalists: Vec<Unit>) -> Graph {
    let mut g = Graph {
        waits: vec![Vec::new(); units.len()],
        tasks: Vec::new(),
        last: BTreeMap::new(),
    };
    let mut merges = Vec::new();
    for name in plan.specialists() {
        let own: Vec<usize> = (0..units.len())
            .filter(|&at| units[at].name == name)
            .collect();
        match own.as_slice() {
            [at] if units[*at].part.is_none() => {
                g.last.insert(name, *at);
            }
            [] => {}
            _ => {
                g.last.insert(name.clone(), g.waits.len());
                g.waits.push(own);
                merges.push(Task::Merge(name));
            }
        }
    }
    g.tasks = units.into_iter().map(Task::Unit).chain(merges).collect();
    for unit in generalists {
        let waits = plan
            .dispatch
            .get(&unit.name)
            .into_iter()
            .flatten()
            .filter_map(|s| g.last.get(s).copied())
            .collect();
        g.last.insert(unit.name.clone(), g.tasks.len());
        g.waits.push(waits);
        g.tasks.push(Task::Generalist(unit));
    }
    g
}

/// Runs every agent the review's plan dispatches under `c` and returns
/// their runs: the specialists kept whole, then every other specialist
/// in plan order, then the generalists in plan order.
pub(crate) fn run_all(r: &Review, c: &Controls) -> Vec<AgentRun> {
    let mut runs: Vec<AgentRun> = r
        .plan
        .specialists()
        .iter()
        .filter_map(|n| whole(r, c, n))
        .collect();
    runs.iter().for_each(|_| c.breaker.end(false));
    for name in r.plan.specialists() {
        if let Some(run) = stalled(r, c, &name) {
            c.breaker.end(true);
            runs.push(run);
        }
    }
    let (units, mut unsplit) = specialist_units(r.workdir, r.plan, r.options.split, &|n| {
        matches!(reuse(r, n), Some(Reuse::Whole(_))) || stalled_why(r, n).is_some()
    });
    unsplit.iter().for_each(|_| settle(c, true));
    let mut kept = BTreeMap::new();
    let mut generalists = Vec::new();
    for name in r.plan.generalists() {
        match stalled(r, c, &name).or_else(|| whole(r, c, &name)) {
            Some(run) => {
                kept.insert(name, run);
            }
            None => generalists.push(Unit::whole(name)),
        }
    }
    let g = graph(r.plan, units, generalists);
    // A run starts only once its tier's cap fits in the pull request's
    // cost cap beside the runs in flight, and a generalist only once the
    // breaker is decided. A task the gate holds is asked again as each
    // task ends, and with nothing running the lowest ready one starts
    // all the same, to be refused there if it still does not fit.
    let gate = |at: usize| {
        let (unit, generalist) = match &g.tasks[at] {
            Task::Unit(unit) => (Some(unit), false),
            Task::Generalist(unit) => (Some(unit), true),
            Task::Merge(_) => (None, false),
        };
        let fits = unit.is_none_or(|u| c.budget.fits(runner::budget_of(u.kind)));
        fits && (!generalist || c.breaker.decided())
    };
    let done = pipeline(&g.waits, r.options.parallel, &gate, &|at, inputs| {
        task(r, c, &g.tasks[at], inputs)
    });
    let mut done: Vec<Option<Done>> = done.into_iter().map(Some).collect();
    let mut take = |name: &str| {
        g.last
            .get(name)
            .and_then(|&at| done.get_mut(at))
            .and_then(Option::take)
            .map(|(run, _)| run)
    };
    for name in r.plan.specialists() {
        match unsplit.iter().position(|run| run.name == name) {
            Some(at) => runs.push(unsplit.remove(at)),
            None => runs.extend(take(&name)),
        }
    }
    for name in r.plan.generalists() {
        runs.extend(kept.remove(&name).or_else(|| take(&name)));
    }

    runs
}

/// Runs `task`, given `inputs`, the results of the tasks it waits for.
fn task(r: &Review, c: &Controls, task: &Task, inputs: Vec<Done>) -> Done {
    match task {
        Task::Unit(unit) => {
            let done = one(r, c, unit);
            if unit.part.is_none() {
                settle(c, ended_in_gap(&done.0, done.1.as_deref(), r));
            }
            done
        }
        Task::Merge(name) => {
            let (mut run, report) = parts::merge(
                name,
                inputs
                    .into_iter()
                    .map(|(run, report)| (run, report.unwrap_or_default()))
                    .collect(),
            );
            if let Err(e) = crate::run::write_report(r.workdir, name, &report) {
                run.failure = Some(run.failure.map_or(e.clone(), |f| format!("{f}; {e}")));
            }
            run.answer_key = r.keys.and_then(|k| k.own(name)).map(str::to_owned);
            (r.progress)(&describe(&run));
            settle(c, ended_in_gap(&run, Some(&report), r));
            (run, Some(report))
        }
        Task::Generalist(unit) => {
            // A cost cap reached is named before a mass failure: both
            // keep it from starting, and the cap is what was set.
            let tripped = c
                .budget
                .refusal(runner::budget_of(unit.kind))
                .map_or_else(|| c.breaker.refusal(), |_| None);
            let done = match tripped {
                Some(why) => {
                    let done = not_started(r.workdir, unit, why);
                    (r.progress)(&describe(&done.0));
                    done
                }
                None => one(r, c, unit),
            };
            c.live.finish();
            done
        }
    }
}

/// Counts a specialist finished, and ended for the breaker, in a gap
/// when `failed`.
fn settle(c: &Controls, failed: bool) {
    c.live.finish();
    c.breaker.end(failed);
}

/// Whether specialist `run`, its report `report`, ended in a gap: a
/// part or the whole that did not answer, a cost cap's included, or a
/// report the gate refuses.
fn ended_in_gap(run: &AgentRun, report: Option<&str>, r: &Review) -> bool {
    run.failure.is_some() || refusal(report.unwrap_or_default(), &run.log.answer, r.known).is_some()
}

/// Why the resume plan does not run `name` again, when it stalled.
fn stalled_why<'a>(r: &'a Review, name: &str) -> Option<&'a str> {
    r.resume
        .and_then(|resume| resume.stalled.get(name))
        .map(String::as_str)
}

/// Agent `name`'s run, not started because it stalled, told to
/// progress; none when it did not stall.
fn stalled(r: &Review, c: &Controls, name: &str) -> Option<AgentRun> {
    let why = stalled_why(r, name)?;
    let (run, _) = not_started(r.workdir, &Unit::whole(name.to_owned()), why.to_owned());
    (r.progress)(&describe(&run));
    c.live.finish();
    Some(run)
}

/// What the resume plan keeps of `name`, when the review resumes.
fn reuse<'a>(r: &'a Review, name: &str) -> Option<&'a Reuse> {
    r.resume.and_then(|resume| resume.reuse.get(name))
}

/// Agent `name`'s run kept whole from the earlier run, told to
/// progress; none when it is not kept whole.
fn whole(r: &Review, c: &Controls, name: &str) -> Option<AgentRun> {
    let Some(Reuse::Whole(answer)) = reuse(r, name) else {
        return None;
    };
    let mut run = reused(
        name.to_owned(),
        component::kind_of(name),
        answer,
        Vec::new(),
    );
    run.answer_key = memo::key_of(r, &Unit::whole(name.to_owned()));
    (r.progress)(&describe(&run));
    c.live.finish();
    Some(run)
}

/// `unit`'s run: the part the earlier run kept, when its report still
/// passes the check, else the answer the store holds under its key, when
/// it is accepted again, else a run of its own, kept in the store once
/// accepted; told to progress.
fn one(r: &Review, c: &Controls, unit: &Unit) -> Done {
    let kept = match (reuse(r, &unit.name), &unit.part) {
        (Some(Reuse::Parts(kept)), Some(part)) => kept.get(&part.index),
        _ => None,
    };
    let key = memo::key_of(r, unit);
    if let Some(log) = kept {
        let report = assemble(&log.texts, &log.answer, None);
        // A kept part is held to the report check as a fresh one.
        match refusal(&report, &log.answer, r.known) {
            None => {
                let mut run = reused(unit.label(), unit.kind, &log.answer, log.texts.clone());
                run.answer_key = key;
                (r.progress)(&describe(&run));
                return (run, Some(report));
            }
            Some(why) => (r.progress)(&format!(
                "{} ({}): its kept report was refused, so it runs again: {why}",
                unit.label(),
                unit.kind.word()
            )),
        }
    }
    if let (Some(store), Some(key)) = (r.store, &key)
        && let Some(done) = memo::recall(r, store, unit, key)
    {
        (r.progress)(&describe(&done.0));
        return done;
    }
    let mut done = run_one(r, c, unit);
    if let (Some(store), Some(key)) = (r.store, &key) {
        done.0.store_failed = memo::keep(r, store, unit, key, &done);
    }
    done.0.answer_key = key;
    (r.progress)(&describe(&done.0));
    done
}

#[cfg(test)]
mod tests;
