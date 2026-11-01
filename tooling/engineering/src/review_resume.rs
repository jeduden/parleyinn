//! ENG-28: what the review did not review is never read as clean, and a
//! review resumed from it is judged as a full run. `review-run` writes
//! beside the review outcome each dispatched agent left without an
//! accepted report or answer, and names it not reviewed; run again
//! with that list and the earlier run, it runs that agent again with
//! the generalist that weighs it, keeps every other answer, and decides
//! the review as a full run would; across a changed head or input it
//! keeps nothing, and across a changed definition or diff nothing of
//! the agent whose it is. The check runs
//! `review-run` six times on work directories holding the checkout's
//! definitions, with a stand-in for the `claude` command line that
//! fails one specialist on the first run alone, and requires each run
//! to start exactly the agents it must.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use review_eval::host::{Command, Host, Output};

use crate::Checkout;
use crate::review_runs::{self, Event, Recorder, agent_of};

/// The specialist the first run cannot start.
pub const GAP: &str = "lang-rust";
/// The generalist that weighs it.
pub const ITS_GENERALIST: &str = "review-code";
/// The specialist whose definition the last run changes.
pub const CHANGED: &str = "test-engineer-unit";
/// The specialist whose diff the last run changes.
pub const DIFFED: &str = "lang-markdown";
/// The generalist that weighs it.
pub const DIFFED_GENERALIST: &str = "review-spec";
/// The head the runs review.
pub const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
/// Another head.
pub const MOVED: &str = "89abcdef0123456789abcdef0123456789abcdef";

/// A stand-in for the `claude` command line: the agent `fail` names
/// cannot start; every other answers as [`Recorder`] does, which keeps
/// when each run starts.
#[derive(Debug, Default)]
pub struct ResumeHost {
    pub fail: Option<&'static str>,
    pub recorder: Recorder,
}

impl ResumeHost {
    /// The agents whose run started.
    #[must_use]
    pub fn started(self) -> BTreeSet<String> {
        self.recorder
            .events
            .into_inner()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| match e {
                Event::Started(name) => Some(name),
                Event::Ended(_) => None,
            })
            .collect()
    }
}

impl Host for ResumeHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if self.fail == Some(agent_of(command).as_str()) {
            return Err("claude: not found".to_owned());
        }
        self.recorder.run(command)
    }
}

/// Runs `review-run` on the prepared `workdir`, reviewing `head`, with
/// `host` and the arguments `extra`, and returns the agents it started.
///
/// # Errors
///
/// Fails when `review-run` exits with an error, naming why.
pub fn run(
    workdir: &Path,
    host: ResumeHost,
    head: &str,
    extra: &[String],
) -> Result<BTreeSet<String>, String> {
    let mut args = vec![
        "--workdir".to_owned(),
        workdir.display().to_string(),
        "--head-sha".to_owned(),
        head.to_owned(),
    ];
    args.extend_from_slice(extra);
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = review_run::cli::run(&host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        let why = err.lines().last().unwrap_or_default();
        return Err(format!("review-run exited {code}: {why}"));
    }

    Ok(host.started())
}

/// The arguments that resume a review from the earlier run in
/// `earlier`, its work directory.
#[must_use]
pub fn resumed_from(earlier: &Path) -> Vec<String> {
    let review = earlier.join("review");
    vec![
        "--only".to_owned(),
        review.join("gaps.json").display().to_string(),
        "--from".to_owned(),
        review.display().to_string(),
    ]
}

/// What is wrong with what the run in `workdir`, which could not start
/// [`GAP`], wrote: a gap list that does not name it alone, an outcome
/// that approves or does not name it not reviewed.
#[must_use]
pub fn gap_problems(workdir: &Path) -> Vec<String> {
    let text = fs::read_to_string(workdir.join(review_run::gaps::GAPS)).unwrap_or_default();
    let mut problems = Vec::new();
    match review_run::gaps::parse(&text) {
        Ok(gaps) if gaps.len() == 1 && gaps.iter().all(|g| g.name == GAP) => {}
        Ok(gaps) => problems.push(format!(
            "the gap list names {:?}, not {GAP} alone",
            gaps.iter().map(|g| g.label()).collect::<Vec<_>>()
        )),
        Err(e) => problems.push(format!("review-run wrote no gap list that reads: {e}")),
    }
    let text = fs::read_to_string(workdir.join(review_run::OUTCOME)).unwrap_or_default();
    match review_gate::parse_outcome(&text) {
        Ok(o) if o.verdict == review_gate::APPROVE_VERDICT => {
            problems.push(format!("the review approves without {GAP}"));
        }
        Ok(o) if !o.summary.contains(&format!("Not reviewed: {GAP}.")) => {
            problems.push(format!(
                "the review outcome does not name {GAP} not reviewed"
            ));
        }
        Ok(_) => {}
        Err(_) => problems.push("review-run wrote no review outcome that reads".to_owned()),
    }

    problems
}

/// What is wrong with `started`, the agents a run started, against
/// `want`, the ones it must start and no other.
#[must_use]
pub fn started_problems(started: &BTreeSet<String>, want: &[&str], run: &str) -> Vec<String> {
    let want: BTreeSet<String> = want.iter().map(|w| (*w).to_owned()).collect();
    if *started == want {
        return Vec::new();
    }
    vec![format!(
        "{run} started {:?}, not {:?}",
        started.iter().collect::<Vec<_>>(),
        want.iter().collect::<Vec<_>>()
    )]
}

/// What is wrong with the resumed review in `workdir`: an outcome that
/// does not approve, or a dispatched agent left out of those consulted.
#[must_use]
pub fn resumed_problems(workdir: &Path) -> Vec<String> {
    let text = fs::read_to_string(workdir.join(review_run::OUTCOME)).unwrap_or_default();
    let mut problems = match review_gate::parse_outcome(&text) {
        Ok(o) if o.verdict == review_gate::APPROVE_VERDICT => Vec::new(),
        Ok(o) => vec![format!(
            "the resumed review does not approve: {}",
            o.summary
        )],
        Err(_) => vec!["the resumed review wrote no outcome that reads".to_owned()],
    };
    let text = fs::read_to_string(workdir.join(review_run::CONSULTED)).unwrap_or_default();
    let consulted = review_gate::parse_consulted(&text).unwrap_or_default();
    for name in all() {
        if !consulted.contains(name) {
            problems.push(format!("the resumed review does not consult {name}"));
        }
    }

    problems
}

/// Every agent [`review_runs::DISPATCH`] names.
fn all() -> Vec<&'static str> {
    let mut out: Vec<&str> = review_runs::SPECIALISTS.to_vec();
    out.extend(review_runs::GENERALISTS.iter().map(|(g, _, _)| *g));
    out
}

/// Changes [`CHANGED`]'s definition in the work directory `dir`: a line
/// more at its end.
///
/// # Errors
///
/// Fails when it cannot be read or written.
pub fn change_definition(dir: &Path) -> Result<(), String> {
    let path = dir.join(format!(".claude/agents/{CHANGED}.md"));
    fs::read_to_string(&path)
        .and_then(|text| fs::write(&path, format!("{text}\n")))
        .map_err(|e| format!("change {}: {e}", path.display()))
}

/// Writes a changed line as the file `rel` under the work directory
/// `dir`, where the earlier run had none.
///
/// # Errors
///
/// Fails when it cannot be written.
pub fn change_file(dir: &Path, rel: &str) -> Result<(), String> {
    let path = dir.join(rel);
    path.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(&path, "+a changed line\n"))
        .map_err(|e| format!("change {}: {e}", path.display()))
}

/// Runs the six reviews under `work`, prepared from `checkout`, and
/// returns what is wrong with what each started and wrote: the first,
/// which cannot start [`GAP`]; the one resumed from it on the same head;
/// one resumed on [`MOVED`]; one resumed with the pull request's diff,
/// an input every agent reads, changed; one resumed with [`CHANGED`]'s
/// definition changed; and one resumed with [`DIFFED`]'s diff changed.
#[must_use]
pub fn review_resumes(checkout: &Checkout, work: &Path) -> Vec<String> {
    let first = work.join("first");
    let failing = ResumeHost {
        fail: Some(GAP),
        ..ResumeHost::default()
    };
    if let Err(e) =
        review_runs::prepare(checkout, &first).and_then(|()| run(&first, failing, HEAD, &[]))
    {
        return vec![e];
    }
    let mut problems = gap_problems(&first);
    let from = resumed_from(&first);
    let again = |dir: &str, head: &str, change: &dyn Fn(&Path) -> Result<(), String>| {
        let dir = work.join(dir);
        review_runs::prepare(checkout, &dir)?;
        change(&dir)?;
        run(&dir, ResumeHost::default(), head, &from).map(|s| (dir, s))
    };
    let unchanged = |_: &Path| Ok(());
    match again("resumed", HEAD, &unchanged) {
        Ok((dir, started)) => {
            problems.extend(started_problems(
                &started,
                &[GAP, ITS_GENERALIST],
                "the resumed review",
            ));
            problems.extend(resumed_problems(&dir));
        }
        Err(e) => problems.push(e),
    }
    match again("moved", MOVED, &unchanged) {
        Ok((_, started)) => {
            problems.extend(started_problems(
                &started,
                &all(),
                "a review resumed on another head",
            ));
        }
        Err(e) => problems.push(e),
    }
    match again("inputs", HEAD, &|d| change_file(d, "pr.diff")) {
        Ok((_, started)) => problems.extend(started_problems(
            &started,
            &all(),
            "a review resumed with a changed input",
        )),
        Err(e) => problems.push(e),
    }
    match again("changed", HEAD, &change_definition) {
        Ok((_, started)) => problems.extend(started_problems(
            &started,
            &[GAP, CHANGED, ITS_GENERALIST],
            "a review resumed with a changed definition",
        )),
        Err(e) => problems.push(e),
    }
    match again("diffed", HEAD, &|d| {
        change_file(d, &format!("review/diffs/{DIFFED}.diff"))
    }) {
        Ok((_, started)) => problems.extend(started_problems(
            &started,
            &[GAP, DIFFED, ITS_GENERALIST, DIFFED_GENERALIST],
            "a review resumed with a changed diff",
        )),
        Err(e) => problems.push(e),
    }

    problems
}

#[cfg(test)]
mod tests;
