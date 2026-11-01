//! A review resumed from an earlier run: every agent the earlier run
//! took an answer from, and that its gap list does not name, keeps that
//! answer, its report and its logs, copied from the earlier run, when
//! its own answer key (`key`) is the one the earlier run recorded for
//! it, whatever the head; every other agent runs again, and why is
//! noted. So a resume on a new commit reuses every specialist whose
//! definition, diff, prompt, model, `claude` and salted files are the
//! same. A specialist named only by parts of its split diff runs just
//! those parts again, the parts that answered keeping their logs. A
//! generalist runs again whenever one of its specialists does, so it
//! always weighs the reports the review decides on. The verdict is then
//! decided as for a full run: a reused answer is judged exactly as a
//! fresh one. An earlier run that recorded no key, or a review whose
//! keys are unknown, reuses nothing of that agent.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use review_eval::log::{self, RunLog};

use crate::gaps::{self, Gap};
use crate::plan::{Plan, report_file};
use crate::run_id::{self, RUN_ID, RunId};
use crate::stall::{self, STALLS, Stall};
use crate::{ANSWERS, CONSULTED, LOGS, REVIEW};

/// What the earlier run left, read.
#[derive(Debug, Clone, PartialEq)]
pub struct Earlier {
    /// Its review directory, the one holding its outcome.
    pub dir: PathBuf,
    pub id: RunId,
    /// The gaps it is resumed for.
    pub gaps: Vec<Gap>,
    /// The answers it took, by agent.
    pub answers: BTreeMap<String, String>,
    pub consulted: BTreeSet<String>,
    /// For each specialist the gaps name by part, each part's log, by
    /// its place from one.
    pub parts: BTreeMap<String, BTreeMap<usize, RunLog>>,
    /// Its gaps counted across the runs before it; none when it wrote
    /// none.
    pub stalls: Vec<Stall>,
}

/// What one agent keeps from the earlier run.
#[derive(Debug, Clone, PartialEq)]
pub enum Reuse {
    /// Its whole answer, and for a specialist its report.
    Whole(String),
    /// The parts of its split diff that answered, by place, each with
    /// its log; the others run again.
    Parts(BTreeMap<usize, RunLog>),
}

/// The resumed review's plan: what each agent keeps, and a note for
/// each that runs again.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Resume {
    /// The earlier run's directory, as given.
    pub from: String,
    pub reuse: BTreeMap<String, Reuse>,
    pub notes: Vec<String>,
    /// Each agent not run again because it stalled, with why.
    pub stalled: BTreeMap<String, String>,
    /// The earlier run's stalls, which this run's gaps count on from.
    pub stalls: Vec<Stall>,
}

/// `rel`, a path under the work directory's `review/`, under the
/// earlier run's review directory `dir`.
fn earlier_file(dir: &Path, rel: &str) -> PathBuf {
    dir.join(
        rel.strip_prefix(REVIEW)
            .map_or(rel, |r| r.trim_start_matches('/')),
    )
}

/// Reads the earlier run in its review directory `from`, resumed for
/// the gap list at `only`.
///
/// # Errors
///
/// Fails naming the file that does not read, or does not read as its
/// shape.
pub fn read(only: &Path, from: &Path) -> Result<Earlier, String> {
    let load = |path: PathBuf| {
        fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))
            .map(|text| (path, text))
    };
    let named = |path: &Path, e: String| format!("{}: {e}", path.display());
    let (path, text) = load(only.to_path_buf())?;
    let gaps = gaps::parse(&text).map_err(|e| named(&path, e))?;
    let (path, text) = load(earlier_file(from, RUN_ID))?;
    let id = run_id::parse(&text).map_err(|e| named(&path, e))?;
    let (path, text) = load(earlier_file(from, ANSWERS))?;
    let answers = review_gate::parse_answers(&text).map_err(|e| named(&path, e.to_string()))?;
    let (path, text) = load(earlier_file(from, CONSULTED))?;
    let consulted = review_gate::parse_consulted(&text).map_err(|e| named(&path, e.to_string()))?;
    let path = earlier_file(from, STALLS);
    let stalls = match fs::read_to_string(&path) {
        Ok(text) => stall::parse(&text).map_err(|e| named(&path, e))?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let mut parts = BTreeMap::new();
    for gap in gaps.iter().filter(|g| g.part.is_some()) {
        if !parts.contains_key(&gap.name) {
            parts.insert(gap.name.clone(), part_logs(from, &gap.name)?);
        }
    }

    Ok(Earlier {
        dir: from.to_path_buf(),
        id,
        gaps,
        answers,
        consulted,
        parts,
        stalls,
    })
}

/// Each part log of specialist `name` in the earlier run's directory
/// `from`, `<name>.<i>.jsonl`, read, by its place.
fn part_logs(from: &Path, name: &str) -> Result<BTreeMap<usize, RunLog>, String> {
    let logs = earlier_file(from, LOGS);
    let entries = fs::read_dir(&logs).map_err(|e| format!("read {}: {e}", logs.display()))?;
    let mut out = BTreeMap::new();
    for entry in listed(entries, &logs)? {
        let file = entry.file_name();
        let Some(index) = file
            .to_str()
            .and_then(|f| {
                f.strip_prefix(name)?
                    .strip_prefix('.')?
                    .strip_suffix(".jsonl")
            })
            .and_then(|i| i.parse::<usize>().ok())
        else {
            continue;
        };
        let path = entry.path();
        let text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        out.insert(index, log::parse(&text));
    }

    Ok(out)
}

/// A note on what a head that moved from `earlier`'s to `now`'s keeps;
/// none when it did not move or one is unknown.
fn moved(earlier: &RunId, now: &RunId) -> Option<String> {
    match (&earlier.head, &now.head) {
        (Some(was), Some(is)) if was != is => Some(format!(
            "the head changed from {was} to {is}: only agents whose answer key is unchanged \
             keep their answer"
        )),
        _ => None,
    }
}

/// Why agent `name` cannot keep its earlier answer: its answer key is
/// not the one the earlier run recorded, with the change named when its
/// definition or diff is it, or it was not in the earlier run; none when
/// it can.
fn changed(name: &str, earlier: &Earlier, now: &RunId) -> Option<String> {
    let (Some(was), Some(is)) = (earlier.id.agents.get(name), now.agents.get(name)) else {
        return Some("it was not in the earlier run".to_owned());
    };
    let same = matches!((&was.key, &is.key), (Some(a), Some(b)) if a == b);
    if same {
        return None;
    }
    Some(
        if was.key.is_none() {
            "the earlier run recorded no answer key for it"
        } else if is.key.is_none() {
            "its answer key is unknown"
        } else if was.definition != is.definition {
            "its definition changed"
        } else if was.diff != is.diff {
            "its diff changed"
        } else {
            "its answer key changed: its model, effort, turns or prompt, the claude version, \
             the split limit, the pull request's agent instructions or a file every key reads"
        }
        .to_owned(),
    )
}

/// What each of `plan`'s agents keeps from `earlier`, for a review on
/// `now`, the earlier run's directory given as `from`; an agent whose
/// gap recurred in more than `max_resumes` resumes in a row is not run
/// again.
#[must_use]
pub fn plan(plan: &Plan, earlier: &Earlier, now: &RunId, from: &str, max_resumes: u32) -> Resume {
    let mut resume = Resume {
        from: from.to_owned(),
        stalls: earlier.stalls.clone(),
        ..Resume::default()
    };
    for (name, why) in stall::stuck(&earlier.stalls, max_resumes) {
        if plan.agents().contains(&name) {
            resume.notes.push(format!("{name}: not run again: {why}"));
            resume.stalled.insert(name, why);
        }
    }
    if let Some(note) = moved(&earlier.id, now) {
        resume.notes.push(note);
    }
    let gapped =
        |name: &str| -> Vec<&Gap> { earlier.gaps.iter().filter(|g| g.name == name).collect() };
    for name in plan.specialists() {
        if resume.stalled.contains_key(&name) {
            continue;
        }
        match keep(&name, &gapped(&name), earlier, now) {
            Ok(reuse) => {
                if let Reuse::Parts(kept) = &reuse {
                    resume.notes.push(format!(
                        "{name}: runs again in part, keeping {}",
                        places(kept.keys())
                    ));
                }
                resume.reuse.insert(name, reuse);
            }
            Err(why) => resume.notes.push(format!("{name}: runs again: {why}")),
        }
    }
    for name in plan.generalists() {
        if resume.stalled.contains_key(&name) {
            continue;
        }
        let rerun = plan
            .dispatch
            .get(&name)
            .into_iter()
            .flatten()
            .find(|s| !matches!(resume.reuse.get(*s), Some(Reuse::Whole(_))));
        let why = match (gapped(&name).first(), rerun) {
            (Some(gap), _) => Err(gap.reason.clone()),
            (None, Some(s)) => Err(format!("its specialist {s} runs again")),
            (None, None) => whole(&name, earlier, now),
        };
        match why {
            Ok(reuse) => {
                resume.reuse.insert(name, reuse);
            }
            Err(why) => resume.notes.push(format!("{name}: runs again: {why}")),
        }
    }

    resume
}

/// The places of `parts`, said.
fn places<'a>(parts: impl Iterator<Item = &'a usize>) -> String {
    let said: Vec<String> = parts.map(|p| format!("part {p}")).collect();
    said.join(", ")
}

/// Agent `name`'s whole earlier answer, when it is unchanged and the
/// earlier run took one from it.
fn whole(name: &str, earlier: &Earlier, now: &RunId) -> Result<Reuse, String> {
    if let Some(why) = changed(name, earlier, now) {
        return Err(why);
    }
    match earlier.answers.get(name) {
        Some(answer) if earlier.consulted.contains(name) => Ok(Reuse::Whole(answer.clone())),
        _ => Err("the earlier run took no answer from it".to_owned()),
    }
}

/// What specialist `name`, named by `gaps`, keeps: its whole answer
/// when no gap names it; when every gap names a part, the parts that
/// answered; else nothing, with why.
fn keep(name: &str, gaps: &[&Gap], earlier: &Earlier, now: &RunId) -> Result<Reuse, String> {
    let Some(first) = gaps.first() else {
        return whole(name, earlier, now);
    };
    if let Some(why) = changed(name, earlier, now) {
        return Err(why);
    }
    let failed: BTreeSet<usize> = gaps.iter().filter_map(|g| g.part).collect();
    if failed.len() < gaps.len() {
        return Err(first.reason.clone());
    }
    let kept: BTreeMap<usize, RunLog> = earlier
        .parts
        .get(name)
        .into_iter()
        .flatten()
        .filter(|(at, log)| !failed.contains(at) && log.answered && !log.answer.trim().is_empty())
        .map(|(at, log)| (*at, log.clone()))
        .collect();
    if kept.is_empty() {
        return Err(first.reason.clone());
    }

    Ok(Reuse::Parts(kept))
}

/// Copies into the work directory `workdir`, from the earlier run's
/// review directory `from`, what `resume` keeps: each whole specialist's
/// report, and each kept agent's or part's logs; returns a note naming
/// each log of a kept agent left behind because its name is not UTF-8.
///
/// # Errors
///
/// Fails naming a file that is there but cannot be copied, or the logs'
/// directory when it does not list.
pub fn copy(
    workdir: &Path,
    from: &Path,
    plan: &Plan,
    resume: &Resume,
) -> Result<Vec<String>, String> {
    let mut notes = Vec::new();
    for dir in [LOGS, crate::REPORTS] {
        let path = workdir.join(dir);
        fs::create_dir_all(&path).map_err(|e| format!("make {}: {e}", path.display()))?;
    }
    let specialists = plan.specialists();
    for (name, reuse) in &resume.reuse {
        if matches!(reuse, Reuse::Whole(_)) && specialists.contains(name) {
            let rel = report_file(name);
            copy_file(&earlier_file(from, &rel), &workdir.join(&rel))?;
        }
        let logs = earlier_file(from, LOGS);
        let entries = match fs::read_dir(&logs) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("read {}: {e}", logs.display())),
        };
        for entry in listed(entries, &logs)? {
            let file = entry.file_name();
            let Some(file) = file.to_str() else {
                let lossy = file.to_string_lossy();
                if lossy.starts_with(&format!("{name}.")) {
                    notes.push(format!(
                        "{name}: log {lossy} not copied: its name is not UTF-8"
                    ));
                }
                continue;
            };
            if is_log_of(file, name, reuse) {
                copy_file(&entry.path(), &workdir.join(LOGS).join(file))?;
            }
        }
    }

    Ok(notes)
}

/// The entries of the directory `dir`, or why one of them did not list.
fn listed<T>(entries: impl Iterator<Item = io::Result<T>>, dir: &Path) -> Result<Vec<T>, String> {
    entries
        .collect::<io::Result<Vec<T>>>()
        .map_err(|e| format!("read {}: {e}", dir.display()))
}

/// Whether the log file named `file` is one of agent `name`'s that
/// `reuse` keeps: its own, its first attempt's, or a kept part's.
fn is_log_of(file: &str, name: &str, reuse: &Reuse) -> bool {
    let Some(rest) = file.strip_prefix(name).and_then(|r| r.strip_prefix('.')) else {
        return false;
    };
    let own = |rest: &str| ["jsonl", "stderr", "first.jsonl", "first.stderr"].contains(&rest);
    match reuse {
        Reuse::Whole(_) => {
            own(rest)
                || rest
                    .split_once('.')
                    .is_some_and(|(i, r)| i.parse::<usize>().is_ok() && own(r))
        }
        Reuse::Parts(kept) => rest
            .split_once('.')
            .is_some_and(|(i, r)| i.parse().is_ok_and(|i: usize| kept.contains_key(&i)) && own(r)),
    }
}

/// Copies `from` to `to`; a file that is not there is left out, so the
/// review decides on its absence.
fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    match fs::copy(from, to) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound && !from.exists() => Ok(()),
        Err(e) => Err(format!("copy {} to {}: {e}", from.display(), to.display())),
    }
}

#[cfg(test)]
mod tests;
