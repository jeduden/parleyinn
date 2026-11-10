//! ENG-28: an agent's answer is reused by its answer key alone, and the
//! answer store keeps only accepted answers. `review-run`, given
//! `--answers`, takes every accepted answer stored under an unchanged
//! key in place of a run; on a new commit that changes one specialist's
//! diff it runs that specialist and the generalist weighing it, nothing
//! else; it never stores a report holding a line that does not read;
//! and it runs again an agent
//! whose stored report fails the report check, never trusting it. A
//! resumed review whose agent failed the same way in two resumes in a
//! row does not run it again and names it stalled. The check runs
//! `review-run` on work directories holding the checkout's definitions,
//! with a stand-in for the `claude` command line that records each run
//! and has one specialist state a line in no form when told to.

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

/// A report line in no form, which the review names apart.
pub const NO_FORM: &str = "src/a.rs:2 — a line in no form";

/// A stand-in for the `claude` command line that answers as
/// [`Recorder`] does, but states [`NO_FORM`] as the agent `refuse` names.
#[derive(Debug, Default)]
pub struct StoreHost {
    pub refuse: Option<&'static str>,
    /// Whether each generalist keeps its finding as blocking.
    pub blocking: bool,
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
        if self.blocking
            && review_runs::GENERALISTS
                .iter()
                .any(|(g, _, _)| *g == agent_of(command))
            && let Some(log) = &command.stdout_to
        {
            let text =
                fs::read_to_string(log).map_err(|e| format!("read {}: {e}", log.display()))?;
            fs::write(log, text.replace(" — nit — ", " — blocking — "))
                .map_err(|e| format!("write {}: {e}", log.display()))?;
        }
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

/// Runs `review-run` on the prepared `workdir` with `host` and `args`;
/// returns the agents it started.
fn reviewed(host: StoreHost, args: Vec<String>) -> Result<BTreeSet<String>, String> {
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
    reviewed(host, args)
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
        "a review whose specialist states a line in no form",
    );
    problems.extend(refused_stored(&work.join("refused"), &fresh));
    problems.extend(poisoned(checkout, work, &store));

    problems
}

/// What is wrong with `review-run answers drop` on a store a first
/// review filled under `work`: it must remove [`GAP`]'s answer by its
/// key and no other, so a review reading the store alone runs [`GAP`]
/// again and nothing else.
#[must_use]
pub fn review_drops(checkout: &Checkout, work: &Path) -> Vec<String> {
    let store = work.join("dropped-answers");
    let unchanged = |_: &Path| Ok(());
    let first = work.join("drop-first");
    let key = stored(
        checkout,
        &first,
        StoreHost::default(),
        &store,
        &unchanged,
        &[],
    )
    .and_then(|_| key_in(&first, GAP));
    let key = match key {
        Ok(key) => key,
        Err(e) => return vec![e],
    };
    let args = [
        "answers",
        "drop",
        &key,
        "--answers",
        &store.display().to_string(),
    ];
    let code = review_run::cli::run(
        &StoreHost::default(),
        args.map(str::to_owned),
        &mut Vec::new(),
        &mut Vec::new(),
    );
    let mut problems = drop_left(code, &store, &key);
    let after = stored(
        checkout,
        &work.join("drop-after"),
        StoreHost::default(),
        &store,
        &unchanged,
        &["--answers-read-only"],
    );
    match after {
        Ok(started) => problems.extend(started_problems(
            &started,
            &[GAP],
            "a review after its answer was dropped",
        )),
        Err(e) => problems.push(e),
    }

    problems
}

/// What is wrong with `answers drop` of `key` from `store`, which
/// exited `code`: the answer is still stored.
fn drop_left(code: u8, store: &Path, key: &str) -> Vec<String> {
    if code == review_run::cli::EXIT_OK && !store.join(key).exists() {
        return Vec::new();
    }

    vec![format!("answers drop left {GAP}'s answer stored as {key}")]
}

/// Runs three reviews in a row under `work`, each given the previous
/// one's kept findings, whose generalists keep a blocking finding on a
/// file that does not change, and returns what is wrong: the second
/// must name no stakeholder ruling and the third must name one in a
/// blocking finding of its own.
#[must_use]
pub fn review_rulings(checkout: &Checkout, work: &Path) -> Vec<String> {
    let mut previous: Option<String> = None;
    let mut bodies = Vec::new();
    for at in 0..3 {
        let dir = work.join(format!("run{at}"));
        let ran = ruling_run(checkout, &dir, previous.as_deref());
        match ran.and_then(|()| outcome_bodies(&dir)) {
            Ok(found) => bodies.push(found),
            Err(e) => return vec![e],
        }
        previous = Some(
            dir.join(review_run::findings::FINDINGS)
                .display()
                .to_string(),
        );
    }

    ruling_problems(&bodies)
}

/// One review under `dir`, whose generalists keep a blocking finding on
/// the files [`GENERALISTS`] names, given `previous` findings.
fn ruling_run(checkout: &Checkout, dir: &Path, previous: Option<&str>) -> Result<(), String> {
    review_runs::prepare(checkout, dir)?;
    for (_, path, _) in review_runs::GENERALISTS {
        let file = dir.join(review_eval::world::PR_HEAD).join(path);
        let parent = file.parent().unwrap_or(dir);
        fs::create_dir_all(parent).map_err(|e| format!("make {}: {e}", parent.display()))?;
        fs::write(&file, "a line\n").map_err(|e| format!("write {}: {e}", file.display()))?;
    }
    let mut args = vec![
        "--workdir".to_owned(),
        dir.display().to_string(),
        "--head-sha".to_owned(),
        HEAD.to_owned(),
        "--breaker-fraction=1".to_owned(),
    ];
    args.extend(previous.map(|p| format!("--previous={p}")));
    let host = StoreHost {
        blocking: true,
        ..StoreHost::default()
    };
    reviewed(host, args).map(drop)
}

/// The bodies of the findings of the outcome review-run wrote in `dir`.
fn outcome_bodies(dir: &Path) -> Result<Vec<String>, String> {
    let path = dir.join(review_run::OUTCOME);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let outcome: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let findings = outcome["findings"].as_array().cloned().unwrap_or_default();

    Ok(findings
        .iter()
        .filter_map(|f| f["body"].as_str().map(str::to_owned))
        .collect())
}

/// What is wrong with the finding bodies of three reviews in a row.
fn ruling_problems(bodies: &[Vec<String>]) -> Vec<String> {
    let ruled = |at: usize| {
        bodies[at]
            .iter()
            .any(|b| b.starts_with(review_run::findings::RULING))
    };
    let mut problems = Vec::new();
    if ruled(1) {
        problems.push("the second review of a blocking finding already needs a ruling".to_owned());
    }
    if !ruled(2) {
        problems.push(
            "the third review of a blocking finding on an unchanged file names no stakeholder ruling"
                .to_owned(),
        );
    }

    problems
}

/// What is wrong with the store `store` after the review in `workdir`,
/// whose [`GAP`] stated a line in no form: its answer stored.
#[must_use]
pub fn refused_stored(workdir: &Path, store: &Path) -> Vec<String> {
    match key_in(workdir, GAP) {
        Ok(key) if store.join(&key).exists() => {
            vec![format!(
                "{GAP}'s answer, whose report held a line that does not read, was stored as {key}"
            )]
        }
        Ok(_) => Vec::new(),
        Err(e) => vec![e],
    }
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
    let outcome =
        fs::read_to_string(work.join("run3").join(review_run::OUTCOME)).unwrap_or_default();
    stall_problems(&last, &outcome)
}

/// What is wrong with the resume after a stall, which started `started`
/// and wrote the outcome `outcome`: it started more than
/// [`ITS_GENERALIST`], or its outcome does not name [`GAP`] stalled.
#[must_use]
pub fn stall_problems(started: &BTreeSet<String>, outcome: &str) -> Vec<String> {
    let mut problems = started_problems(started, &[ITS_GENERALIST], "the resume after a stall");
    let stalled = format!("stalled: {GAP} failed 3 times the same way; needs a person");
    if !outcome.contains(&stalled) {
        problems.push(format!("the review after a stall does not say {stalled:?}"));
    }

    problems
}

#[cfg(test)]
mod tests;
