//! ENG-28: the review reads every specialist's run log, whatever result
//! the run ended with, and decides from the log alone what it covered.
//! The check runs `review-run` once per [`Mode`] on a work directory
//! whose one specialist has a diff of [`DIFF_LINES`] lines, with a
//! stand-in for the `claude` command line whose specialist reads its
//! diff with `Read` calls as the real command line logs them, and whose
//! generalist keeps the specialist's finding as a nit; then the gate on
//! what `review-run` wrote. A specialist cut off at its turn budget after
//! its reads covered its whole diff must be consulted, its finding its
//! report, and the review decided with no gap; one cut off before it
//! stated any line or an outcome must be a `partial:` gap saying so, its
//! report no clean pass, so the review requests changes; one that read
//! part of its diff must be a `partial:` gap naming the lines it did not
//! read, so the review requests changes and the gate posts that request;
//! a line that
//! says it is no finding must be set aside and listed in
//! `review/ignored.json`, the finding beside it kept; a finding-shaped
//! line that does not read must be its own blocking finding quoting it,
//! the finding beside it kept; and a specialist whose log holds no model
//! call did not run, so it is run once more, as it cost nothing, and the
//! gate posts nothing.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use review_eval::host::{Command, Host, Output};

use crate::Checkout;
use crate::review_refusals::{GENERALIST, SPECIALIST, gate};
use crate::review_runs::{FINDING, agent_of, diff_of, read_lines, version};

/// The lines of the specialist's diff.
pub const DIFF_LINES: usize = 30;
/// The lines each of the stand-in's reads takes.
pub const READ: usize = 20;
/// The lines the specialist reads when it reads only part of its diff.
pub const PART: usize = 10;
/// A line saying all is well at a place, which is no finding.
pub const ALL_WELL: &str = "src/b.rs:2 — no finding";
/// A finding-shaped line in no form.
pub const MALFORMED: &str = "src/c.rs:3 — a line in no form";

/// How the stand-in specialist runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// It reads its whole diff, states its finding and is cut off at its
    /// turn budget before its answer.
    Cut,
    /// It reads its whole diff and is cut off at its turn budget before
    /// it states any line or an outcome.
    CutSilent,
    /// It reads only its diff's first [`PART`] lines, states its finding
    /// and answers.
    Partial,
    /// It reads its whole diff, states its finding and [`ALL_WELL`], and
    /// answers.
    AllWell,
    /// It reads its whole diff, states its finding and [`MALFORMED`],
    /// and answers.
    Malformed,
    /// It ends before any model call.
    Silent,
}

impl Mode {
    /// Every mode, in the order the check runs them.
    pub const ALL: [Self; 6] = [
        Self::Cut,
        Self::CutSilent,
        Self::Partial,
        Self::AllWell,
        Self::Malformed,
        Self::Silent,
    ];

    /// Its name, for its work directory and its problems.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Cut => "cut",
            Self::CutSilent => "cut-silent",
            Self::Partial => "partial",
            Self::AllWell => "all-well",
            Self::Malformed => "malformed",
            Self::Silent => "silent",
        }
    }
}

/// A stand-in for the `claude` command line: the specialist runs as
/// `mode` says, each of its runs counted; the generalist keeps its
/// finding as a nit and closes with a sentence.
#[derive(Debug)]
pub struct LogHost {
    pub mode: Mode,
    runs: Mutex<usize>,
}

impl LogHost {
    /// A stand-in whose specialist runs as `mode` says.
    #[must_use]
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            runs: Mutex::new(0),
        }
    }

    /// How many times the specialist ran.
    #[must_use]
    pub fn runs(&self) -> usize {
        *self.runs.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Host for LogHost {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if let Some(out) = version(command) {
            return Ok(out);
        }
        let text = if agent_of(command) == GENERALIST {
            let answer = format!("src/a.rs:1 — nit — a long line — {SPECIALIST}\nSound.");
            let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null,
                "message": {"id": "g", "model": "claude-opus-5-5", "usage": {},
                "content": [{"type": "text", "text": answer}]}});
            let result = serde_json::json!({"type": "result", "subtype": "success",
                "is_error": false, "result": answer, "total_cost_usd": 0.05, "num_turns": 2});
            format!("{call}\n{result}\n")
        } else {
            *self.runs.lock().unwrap_or_else(PoisonError::into_inner) += 1;
            specialist_log(command, self.mode)
        };
        let log = command
            .stdout_to
            .as_deref()
            .ok_or("the run logs to no file")?;
        fs::write(log, text).map_err(|e| format!("write {}: {e}", log.display()))?;

        Ok(Output {
            code: Some(0),
            ..Output::default()
        })
    }
}

/// The log of the specialist's run of `command` as `mode` says: its
/// reads of its diff, the lines it stated, and its result.
#[must_use]
pub fn specialist_log(command: &Command, mode: Mode) -> String {
    if mode == Mode::Silent {
        return "{\"type\":\"result\",\"subtype\":\"error_during_execution\",\"is_error\":true,\
                \"result\":\"\",\"total_cost_usd\":0,\"num_turns\":1}\n"
            .to_owned();
    }
    let path = command
        .cwd
        .join(diff_of(command).unwrap_or_default())
        .display()
        .to_string();
    let reads = if mode == Mode::Partial {
        read_lines("m", &path, (1, PART))
    } else {
        read_lines("m", &path, (1, READ)) + &read_lines("m", &path, (READ + 1, READ))
    };
    let stated = match mode {
        Mode::AllWell => format!("{FINDING}\n{ALL_WELL}"),
        Mode::Malformed => format!("{FINDING}\n{MALFORMED}"),
        Mode::CutSilent => "I read the diff; now the context.".to_owned(),
        _ => FINDING.to_owned(),
    };
    let call = serde_json::json!({"type": "assistant", "parent_tool_use_id": null, "message": {
        "id": "m", "model": "claude-haiku-5-5",
        "usage": {"input_tokens": 10, "output_tokens": 5},
        "content": [{"type": "text", "text": stated}]}});
    let result = if matches!(mode, Mode::Cut | Mode::CutSilent) {
        serde_json::json!({"type": "result", "subtype": "error_max_turns", "is_error": true,
            "result": "", "total_cost_usd": 0.05, "num_turns": 20})
    } else {
        serde_json::json!({"type": "result", "subtype": "success", "is_error": false,
            "result": "outcome: findings — one finding.", "total_cost_usd": 0.01,
            "num_turns": 4})
    };

    format!("{reads}{call}\n{result}\n")
}

/// Lays out in `workdir` what the review job prepares for `review-run`,
/// as [`crate::review_refusals::prepare`] does, and the specialist's
/// diff of [`DIFF_LINES`] lines.
///
/// # Errors
///
/// Fails naming the file that cannot be read or written.
pub fn prepare(checkout: &Checkout, workdir: &Path) -> Result<(), String> {
    crate::review_refusals::prepare(checkout, workdir)?;
    let diffs = workdir.join("review/diffs");
    let diff = "+let x = 1;\n".repeat(DIFF_LINES);
    fs::create_dir_all(&diffs)
        .and_then(|()| fs::write(diffs.join(format!("{SPECIALIST}.diff")), diff))
        .map_err(|e| format!("write {}: {e}", diffs.display()))
}

/// What `review-run` wrote in a work directory, read.
#[derive(Debug)]
pub struct Wrote {
    /// The specialist's report.
    pub report: String,
    /// The review outcome.
    pub outcome: Option<review_gate::ReviewOutcome>,
    /// The agents consulted.
    pub consulted: String,
    /// What was not reviewed whole.
    pub gaps: Vec<review_run::gaps::Gap>,
    /// The lines set aside as no finding.
    pub ignored: String,
    /// What the gate posts, or why it posts nothing.
    pub gate: Result<String, String>,
}

impl Default for Wrote {
    /// Nothing written, and a gate that posts nothing.
    fn default() -> Self {
        Self {
            report: String::new(),
            outcome: None,
            consulted: String::new(),
            gaps: Vec::new(),
            ignored: String::new(),
            gate: Err(String::new()),
        }
    }
}

impl Wrote {
    /// What `review-run` wrote in `workdir`, and what the gate posts
    /// from it.
    #[must_use]
    pub fn read(workdir: &Path) -> Self {
        let read = |rel: &str| fs::read_to_string(workdir.join(rel)).unwrap_or_default();
        Self {
            report: read(&review_run::plan::report_file(SPECIALIST)),
            outcome: review_gate::parse_outcome(&read(review_run::OUTCOME)).ok(),
            consulted: read(review_run::CONSULTED),
            gaps: review_run::gaps::parse(&read(review_run::gaps::GAPS)).unwrap_or_default(),
            ignored: read(review_run::reading::IGNORED),
            gate: gate(workdir).map(|review| review.event),
        }
    }

    /// Whether the outcome holds a blocking finding at `path` whose body
    /// holds `words`.
    fn blocks(&self, path: &str, words: &str) -> bool {
        self.outcome.as_ref().is_some_and(|o| {
            o.findings.iter().any(|f| {
                f.severity == review_gate::BLOCKING && f.path == path && f.body.contains(words)
            })
        })
    }

    /// The outcome's verdict, or nothing when it does not read.
    fn verdict(&self) -> &str {
        self.outcome.as_ref().map_or("", |o| o.verdict.as_str())
    }

    /// The gap named for the specialist, when there is one.
    fn gap(&self) -> Option<&str> {
        self.gaps
            .iter()
            .find(|g| g.name == SPECIALIST)
            .map(|g| g.reason.as_str())
    }
}

/// The report the specialist's finding gives, ended with the outcome
/// `review-run` derives for it or takes from its answer.
fn report_of(outcome: &str) -> String {
    format!("{FINDING}\n{outcome}\n")
}

/// The outcome line the specialist answers.
const ANSWERED: &str = "outcome: findings — one finding.";

/// What is wrong with what `review-run` and the gate made of a run of
/// the specialist as `mode` says, `wrote` what they wrote and `runs`
/// how many times it ran.
#[must_use]
pub fn mode_problems(mode: Mode, wrote: &Wrote, runs: usize) -> Vec<String> {
    let consulted = wrote.consulted.contains(&format!("\"{SPECIALIST}\""));
    let wants = match mode {
        Mode::Cut => cut_wants(wrote, consulted, runs),
        Mode::CutSilent => cut_silent_wants(wrote, consulted, runs),
        Mode::Partial => partial_wants(wrote, consulted),
        Mode::AllWell => vec![
            (
                wrote.report == report_of(ANSWERED),
                "the finding beside a line saying all is well is not kept",
            ),
            (
                wrote.ignored.contains(ALL_WELL),
                "a line saying all is well is not listed in review/ignored.json",
            ),
            (
                wrote.verdict() == review_gate::APPROVE_VERDICT,
                "a line set aside as no finding blocks the review",
            ),
        ],
        Mode::Malformed => vec![
            (
                wrote.report == report_of(ANSWERED),
                "the finding beside a line in no form is not kept",
            ),
            (
                wrote.blocks(
                    &review_run::plan::report_file(SPECIALIST),
                    &format!("`{MALFORMED}`"),
                ),
                "a line in no form is not its own blocking finding quoting it",
            ),
            (
                wrote.gate.as_deref() == Ok(review_gate::REQUEST_CHANGES),
                "the gate does not post the request for changes a line in no form makes",
            ),
        ],
        Mode::Silent => silent_wants(wrote, consulted, runs),
    };

    wants
        .into_iter()
        .filter(|(holds, _)| !holds)
        .map(|(_, what)| format!("{}: {what}", mode.name()))
        .collect()
}

/// What a specialist cut off after reading its whole diff must give,
/// each with what is wrong when it does not hold.
fn cut_wants(wrote: &Wrote, consulted: bool, runs: usize) -> Vec<(bool, &'static str)> {
    vec![
        (
            consulted,
            "a specialist cut off after reading its whole diff is not consulted",
        ),
        (
            wrote.gap().is_none(),
            "a specialist that read its whole diff is named not reviewed",
        ),
        (
            wrote.report == report_of("outcome: findings — 1 finding stated in the run log"),
            "the finding the specialist stated before it was cut off is not its report",
        ),
        (
            wrote.verdict() == review_gate::APPROVE_VERDICT
                && wrote.gate.as_deref() == Ok(review_gate::APPROVE),
            "a review whose cut specialist read its whole diff is not decided as any other",
        ),
        (
            runs == 1,
            "a specialist cut off after it was charged is run again",
        ),
    ]
}

/// What a specialist cut off before it stated any line or an outcome
/// must give: no clean pass.
fn cut_silent_wants(wrote: &Wrote, consulted: bool, runs: usize) -> Vec<(bool, &'static str)> {
    let why = "partial: it ended without a successful result (its run ended with \
               error_max_turns) before it stated any line or an outcome";
    vec![
        (
            consulted,
            "a specialist cut off before it stated anything is not consulted",
        ),
        (
            wrote.gap() == Some(why),
            "a specialist cut off before it stated anything is not named partial",
        ),
        (
            wrote.blocks("review/", "before it stated any line or an outcome"),
            "a specialist cut off before it stated anything is not named in a blocking finding",
        ),
        (
            wrote.report == format!("{}\n", review_gate::report::CUT_BEFORE_STATING),
            "the report of a specialist cut off before it stated anything reads as a clean pass",
        ),
        (
            wrote.gate.as_deref() == Ok(review_gate::REQUEST_CHANGES),
            "the gate does not post the request for changes a specialist cut off before it \
             stated anything makes",
        ),
        (
            runs == 1,
            "a specialist cut off after it was charged is run again",
        ),
    ]
}

/// What a specialist that read part of its diff must give.
fn partial_wants(wrote: &Wrote, consulted: bool) -> Vec<(bool, &'static str)> {
    let why = format!(
        "partial: its log shows no Read of lines {}-{DIFF_LINES} of \
         review/diffs/{SPECIALIST}.diff, of {DIFF_LINES} lines",
        PART + 1
    );
    vec![
        (
            consulted,
            "a specialist that read part of its diff is not consulted",
        ),
        (
            wrote.gap() == Some(why.as_str()),
            "the lines a specialist did not read are not its gap",
        ),
        (
            wrote.blocks("review/", "did not read its whole diff"),
            "the lines a specialist did not read are not named in a blocking finding",
        ),
        (
            wrote.gate.as_deref() == Ok(review_gate::REQUEST_CHANGES),
            "the gate does not post the request for changes a partial specialist makes",
        ),
    ]
}

/// What a specialist whose log holds no model call must give.
fn silent_wants(wrote: &Wrote, consulted: bool, runs: usize) -> Vec<(bool, &'static str)> {
    let unconsulted = format!("dispatched but not consulted: {SPECIALIST}");
    vec![
        (
            !consulted,
            "a specialist whose log holds no model call is consulted",
        ),
        (
            runs == 2,
            "a specialist that did not run and cost nothing is not run once more",
        ),
        (
            wrote.blocks("review/", &format!("{SPECIALIST} did not run")),
            "a specialist that did not run is not named in a blocking finding",
        ),
        (
            wrote.gate.as_ref().is_err_and(|e| e.contains(&unconsulted)),
            "the gate posts a review a dispatched specialist did not run in",
        ),
    ]
}

/// Runs `review-run` on a work directory under `workdir`, prepared
/// from `checkout`, once per [`Mode`], and returns what is wrong with
/// what it and the gate made of each.
#[must_use]
pub fn review_run_reads_every_log(checkout: &Checkout, workdir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for mode in Mode::ALL {
        let dir = workdir.join(mode.name());
        if let Err(e) = prepare(checkout, &dir) {
            problems.push(e);
            continue;
        }
        let host = LogHost::new(mode);
        let args = [
            "--workdir".to_owned(),
            dir.display().to_string(),
            "--breaker-fraction=1".to_owned(),
        ];
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = review_run::cli::run(&host, args, &mut out, &mut err);
        if code != review_run::cli::EXIT_OK {
            let err = String::from_utf8_lossy(&err);
            problems.push(format!(
                "{}: review-run exited {code}: {}",
                mode.name(),
                err.lines().last().unwrap_or_default()
            ));
            continue;
        }
        problems.extend(mode_problems(mode, &Wrote::read(&dir), host.runs()));
    }

    problems
}

#[cfg(test)]
mod tests;
