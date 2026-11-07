//! An agent's answer taken from the answer store, or kept in it: before
//! a run, the unit's answer key is looked up, and an answer stored under
//! it that is accepted again is taken in place of the run, its log
//! copied beside the review's logs and a whole specialist's report
//! written as a fresh one's would be; after a run, an accepted answer is
//! stored under its key. A stored answer that does not read or is no
//! longer accepted is told and the unit runs; an answer that cannot be
//! stored is told and counted (I6). No key, no store: nothing is reused
//! or stored.

use std::fs;

use review_eval::component::Kind;
use review_eval::log::RunLog;
use review_eval::report;
use review_gate::report::findings_check;

use crate::LOGS;
use crate::outcome;
use crate::plan::report_file;
use crate::run::{Review, Unit, reused, write_report};
use crate::store::{self, Put, Store};
use crate::tasks::Done;
use crate::usage;

/// `unit`'s answer key in the review `r`: a specialist's or its part's
/// own, a generalist's beside the reports its specialists left in the
/// work directory; none without the review's keys.
pub(crate) fn key_of(r: &Review, unit: &Unit) -> Option<String> {
    let keys = r.keys?;
    match (unit.kind, &unit.part) {
        (_, Some(part)) => keys.part(&unit.name, part.index, part.count),
        (Kind::Specialist, None) => keys.own(&unit.name).map(str::to_owned),
        _ => keys.generalist_in(r.workdir, &unit.name),
    }
}

/// `unit`'s answer from the store under `key`, when one is there and is
/// accepted again; told to progress when one is there but not taken.
pub(crate) fn recall(r: &Review, store: &Store, unit: &Unit, key: &str) -> Option<Done> {
    let label = || format!("{} ({})", unit.label(), unit.kind.word());
    let stored = match store.get(key) {
        Ok(Some(stored)) => stored,
        Ok(None) => return None,
        Err(e) => {
            (r.progress)(&format!(
                "{}: its stored answer {key} does not read, so it runs: {e}",
                label()
            ));
            retire(r, store, key, &label());
            return None;
        }
    };
    let taken = store::accepted(unit.kind, &stored.log, stored.report.as_deref(), r.known)
        .and_then(|()| partial(r, unit, &stored.log));
    if let Err(why) = taken {
        (r.progress)(&format!(
            "{}: its stored answer {key} is not taken, so it runs: {why}",
            label()
        ));
        retire(r, store, key, &label());
        return None;
    }
    let mut run = reused(
        unit.label(),
        unit.kind,
        &stored.log.answer,
        stored.log.texts.clone(),
    );
    run.from_store = true;
    run.answer_key = Some(key.to_owned());
    let log = r.workdir.join(LOGS).join(format!("{}.jsonl", unit.stem()));
    if let Err(e) = fs::write(&log, &stored.log_text) {
        (r.progress)(&format!(
            "{}: its stored log was not copied to {}: {e}",
            label(),
            log.display()
        ));
    }
    if let (Kind::Specialist, None, Some(text)) = (unit.kind, &unit.part, &stored.report)
        && let Err(e) = write_report(r.workdir, &unit.name, text)
    {
        run.reused = false;
        run.from_store = false;
        run.failure = Some(e);
    }

    Some((run, stored.report))
}

/// Takes the answer under `key`, which is not taken, out of `store`, so
/// the answer the agent gives instead is stored in its place; told to
/// progress, whether it went or not. A store read alone keeps it.
fn retire(r: &Review, store: &Store, key: &str, label: &str) {
    if store.read_only {
        return;
    }
    let said = store.drop(key).map_or_else(
        |e| format!("was not dropped: {e}, so its new answer is not stored"),
        |()| "was dropped, so its new answer is stored in its place".to_owned(),
    );
    (r.progress)(&format!("{label}: its stored answer {key} {said}"));
}

/// Stores `done`, `unit`'s run under `key`, when it is accepted: its
/// final attempt's log, as written, and a specialist's report. Told to
/// progress when it is not accepted or cannot be stored; returns
/// whether storing failed.
pub(crate) fn keep(r: &Review, store: &Store, unit: &Unit, key: &str, done: &Done) -> bool {
    let (run, report) = done;
    if !run.answered() {
        return false;
    }
    let label = format!("{} ({})", unit.label(), unit.kind.word());
    let path = r.workdir.join(LOGS).join(format!("{}.jsonl", unit.stem()));
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            (r.progress)(&format!(
                "{label}: its answer was not stored: read {}: {e}",
                path.display()
            ));
            return true;
        }
    };
    if leaks(r, &text, report.as_deref()) {
        (r.progress)(&format!(
            "{label}: its answer was not stored: it held the Claude token"
        ));
        return true;
    }
    let log = review_eval::log::parse(&text);
    let taken = store::accepted(unit.kind, &log, report.as_deref(), r.known)
        .and_then(|()| partial(r, unit, &log));
    if let Err(why) = taken {
        (r.progress)(&format!("{label}: its answer was not stored: {why}"));
        return false;
    }
    let entry = usage::usage(std::slice::from_ref(run), None).remove(0);
    match store.put(key, &text, report.as_deref(), &entry) {
        Ok(Put::Stored) => {
            (r.progress)(&format!("{label}: its answer was stored as {key}"));
            false
        }
        Ok(Put::Present) => {
            (r.progress)(&format!("{label}: its answer was already stored as {key}"));
            false
        }
        Ok(Put::ReadOnly) => {
            (r.progress)(&format!(
                "{label}: its answer was not stored: the answer store is read alone"
            ));
            false
        }
        Err(e) => {
            (r.progress)(&format!("{label}: its answer was not stored: {e}"));
            true
        }
    }
}

/// Whether `unit`'s answer, its log `log`, is whole: a generalist's
/// weighing must weigh every place its specialists' reports in the work
/// directory name, those the gate takes, as the outcome holds it to, so
/// a partial weighing is neither stored nor taken and its gap runs.
///
/// # Errors
///
/// Fails naming the places it weighed no finding at.
fn partial(r: &Review, unit: &Unit, log: &RunLog) -> Result<(), String> {
    if unit.kind != Kind::Generalist {
        return Ok(());
    }
    let reports: Vec<String> = r
        .plan
        .dispatch
        .get(&unit.name)
        .into_iter()
        .flatten()
        .filter_map(|s| fs::read_to_string(r.workdir.join(report_file(s))).ok())
        .filter(|text| findings_check(text, Some(r.known)).is_none())
        .collect();
    let weighing = report::weighing(&log.texts, &log.answer);
    let missed = outcome::missed(reports.iter().map(String::as_str), &weighing);
    if missed.is_empty() {
        return Ok(());
    }

    Err(format!(
        "its weighing is partial: it weighed no finding at {}",
        missed.join(", ")
    ))
}

/// Whether the log `text` or the `report` holds a form of the Claude
/// token the review scrubs: such an answer is never stored (ENG-28, I6).
fn leaks(r: &Review, text: &str, report: Option<&str>) -> bool {
    r.scrubber
        .is_some_and(|s| s.scrub(text).1 + report.map_or(0, |t| s.scrub(t).1) > 0)
}

#[cfg(test)]
mod tests;
