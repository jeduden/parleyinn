//! ENG-28: an agent's answer is reused by its answer key alone, and the
//! answer store keeps only accepted answers. `review-run`, given
//! `--answers`, takes every accepted answer stored under an unchanged
//! key in place of a run; on a new commit that changes one specialist's
//! diff it runs that specialist and the generalist weighing it, nothing
//! else; it never stores a refused report; and it runs again an agent
//! whose stored report fails the report check, never trusting it. A
//! resumed review whose agent failed the same way in two resumes in a
//! row does not run it again and names it stalled. The check runs
//! `review-run` on work directories holding the checkout's definitions,
//! with a stand-in for the `claude` command line that records each run
//! and refuses one specialist's report when told to.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use review_eval::host::{Command, Host, Output};

use crate::Checkout;
use crate::review_resume::{
    self, DIFFED, DIFFED_GENERALIST, GAP, HEAD, ITS_GENERALIST, ResumeHost, change_file,
    started_problems,
};
use crate::review_runs::{self, Event, Recorder, agent_of};

/// A report line in no form, which the report check refuses.
pub const NO_FORM: &str = "src/a.rs:2 — a line in no form";

/// A stand-in for the `claude` command line that answers as
/// [`Recorder`] does, but states [`NO_FORM`] as the agent `refuse` names.
#[derive(Debug, Default)]
pub struct StoreHost {
    pub refuse: Option<&'static str>,
    pub recorder: Recorder,
}

impl StoreHost {
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

impl Host for StoreHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        let out = self.recorder.run(command)?;
        if self.refuse.is_some_and(|r| r == agent_of(command))
            && let Some(log) = &command.stdout_to
        {
            let text =
                fs::read_to_string(log).map_err(|e| format!("read {}: {e}", log.display()))?;
            let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null,
                "message": {"id": "m-bad", "model": "claude-haiku-5-5", "usage": {},
                "content": [{"type": "text", "text": NO_FORM}]}});
            fs::write(log, format!("{call}\n{text}"))
                .map_err(|e| format!("write {}: {e}", log.display()))?;
        }
        Ok(out)
    }
}

/// Runs `review-run` on `workdir`, prepared from `checkout` and changed
/// by `change`, with `host` and the store at `store`, `extra` after;
/// returns the agents it started.
fn stored(
    checkout: &Checkout,
    workdir: &Path,
    host: StoreHost,
    store: &Path,
    change: &dyn Fn(&Path) -> Result<(), String>,
    extra: &[&str],
) -> Result<BTreeSet<String>, String> {
    review_runs::prepare(checkout, workdir)?;
    change(workdir)?;
    let mut args = vec![
        "--workdir".to_owned(),
        workdir.display().to_string(),
        "--head-sha".to_owned(),
        HEAD.to_owned(),
        "--breaker-fraction=1".to_owned(),
        format!("--answers={}", store.display()),
    ];
    args.extend(extra.iter().map(|a| (*a).to_owned()));
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = review_run::cli::run(&host, args, &mut out, &mut err);
    if code != review_run::cli::EXIT_OK {
        let err = String::from_utf8_lossy(&err);
        return Err(format!(
            "review-run exited {code}: {}",
            err.lines().last().unwrap_or_default()
        ));
    }
    Ok(host.started())
}

/// Agent `name`'s answer key in the run id `review-run` wrote in
/// `workdir`.
///
/// # Errors
///
/// Fails when the run id does not read or holds no key for it.
pub fn key_in(workdir: &Path, name: &str) -> Result<String, String> {
    let path = workdir.join(review_run::run_id::RUN_ID);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let id = review_run::run_id::parse(&text)?;
    id.agents
        .get(name)
        .and_then(|a| a.key.clone())
        .ok_or_else(|| format!("review-run recorded no answer key for {name}"))
}

/// Runs the reviews with an answer store under `work`, prepared from
/// `checkout`, and returns what is wrong with what each started and
/// stored: a first that stores every answer, a second of the same
/// inputs, reading the store alone, that starts nothing, one on a new
/// commit changing [`DIFFED`]'s diff, one whose [`GAP`] states a line in
/// no form, which must leave its key unstored, and one after [`GAP`]'s
/// stored report was tampered with, which must run it again.
#[must_use]
pub fn review_answers(checkout: &Checkout, work: &Path) -> Vec<String> {
    let store = work.join("answers");
    let unchanged = |_: &Path| Ok(());
    let run =
        |dir: &str,
         host: StoreHost,
         store: &Path,
         change: &dyn Fn(&Path) -> Result<(), String>,
         extra: &[&str]| { stored(checkout, &work.join(dir), host, store, change, extra) };
    let mut problems = Vec::new();
    let mut check = |got: Result<BTreeSet<String>, String>, want: &[&str], what: &str| match got {
        Ok(started) => problems.extend(started_problems(&started, want, what)),
        Err(e) => problems.push(e),
    };
    let all = review_runs::SPECIALISTS
        .iter()
        .chain(review_runs::GENERALISTS.iter().map(|(g, _, _)| g))
        .copied()
        .collect::<Vec<_>>();
    check(
        run("first", StoreHost::default(), &store, &unchanged, &[]),
        &all,
        "a first review with an empty store",
    );
    check(
        run(
            "again",
            StoreHost::default(),
            &store,
            &unchanged,
            &["--answers-read-only"],
        ),
        &[],
        "a second review of the same inputs",
    );
    let diffed = format!("review/diffs/{DIFFED}.diff");
    check(
        run(
            "commit",
            StoreHost::default(),
            &store,
            &|d| change_file(d, &diffed),
            &[],
        ),
        &[DIFFED, DIFFED_GENERALIST],
        "a review of a new commit changing one specialist's diff",
    );
    let refusing = StoreHost {
        refuse: Some(GAP),
        ..StoreHost::default()
    };
    let fresh = work.join("refused-answers");
    check(
        run("refused", refusing, &fresh, &unchanged, &[]),
        &all,
        "a review whose specialist's report is refused",
    );
    match key_in(&work.join("refused"), GAP) {
        Ok(key) if fresh.join(&key).exists() => {
            problems.push(format!("{GAP}'s refused report was stored as {key}"));
        }
        Ok(_) => {}
        Err(e) => problems.push(e),
    }
    problems.extend(poisoned(checkout, work, &store));

    problems
}

/// What is wrong with a review after [`GAP`]'s stored report in `store`
/// was tampered with: it must run [`GAP`] again, its generalist's key
/// holding as its fresh report is the one first stored.
fn poisoned(checkout: &Checkout, work: &Path, store: &Path) -> Vec<String> {
    let key = match key_in(&work.join("first"), GAP) {
        Ok(key) => key,
        Err(e) => return vec![e],
    };
    let report = store.join(&key).join(review_run::store::REPORT);
    let tampered = fs::read_to_string(&report)
        .map(|t| t.replace(" — I9", " — ENG-999"))
        .and_then(|t| fs::write(&report, t))
        .map_err(|e| format!("tamper with {}: {e}", report.display()));
    if let Err(e) = tampered {
        return vec![e];
    }
    let unchanged = |_: &Path| Ok(());
    match stored(
        checkout,
        &work.join("poisoned"),
        StoreHost::default(),
        store,
        &unchanged,
        &[],
    ) {
        Ok(started) => started_problems(
            &started,
            &[GAP],
            "a review whose stored report was tampered with",
        ),
        Err(e) => vec![e],
    }
}

/// Runs a first review that cannot start [`GAP`] and three resumes in a
/// row that cannot either, under `work`, prepared from `checkout`, and
/// returns what is wrong: the third resume must not start [`GAP`] and
/// its outcome must name it stalled.
#[must_use]
pub fn review_stalls(checkout: &Checkout, work: &Path) -> Vec<String> {
    let failing = || ResumeHost {
        fail: Some(GAP),
        ..ResumeHost::default()
    };
    let mut from: Vec<String> = Vec::new();
    let mut last = BTreeSet::new();
    for at in 0..4 {
        let dir = work.join(format!("run{at}"));
        let mut extra = from.clone();
        extra.push("--breaker-fraction=1".to_owned());
        match review_runs::prepare(checkout, &dir)
            .and_then(|()| review_resume::run(&dir, failing(), HEAD, &extra))
        {
            Ok(started) => last = started,
            Err(e) => return vec![e],
        }
        from = review_resume::resumed_from(&dir);
    }
    let mut problems = started_problems(&last, &[ITS_GENERALIST], "the resume after a stall");
    let outcome =
        fs::read_to_string(work.join("run3").join(review_run::OUTCOME)).unwrap_or_default();
    let stalled = format!("stalled: {GAP} failed 3 times the same way; needs a person");
    if !outcome.contains(&stalled) {
        problems.push(format!("the review after a stall does not say {stalled:?}"));
    }

    problems
}

#[cfg(test)]
mod tests;
