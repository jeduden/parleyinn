use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use review_eval::host::{Command, Output};
use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost, agent_of, prompt_of};
use crate::plan::{Plan, parse_dispatch};
use crate::run::Options;

/// The ids a report may cite: the invariants and ENG-28.
fn known() -> BTreeSet<String> {
    let mut known = review_gate::report::invariants();
    known.insert("ENG-28".to_owned());
    known
}

#[test]
fn a_report_the_gate_takes_is_no_refusal() {
    let report = "a.rs:1 — if not addressed: x — I2\noutcome: findings — one.\n";

    assert_eq!(refusal(report, "outcome: findings — one.", &known()), None);
}

#[test]
fn a_refused_line_is_quoted_beside_the_gate_s_problem() {
    let report = "a.rs:1 — if not addressed: x — I2\n\
                  revised: `a.rs:1 — old` → `a.rs:1 — new`\n\
                  outcome: findings — one.\n";

    assert_eq!(
        refusal(report, "outcome: findings — one.", &known()).as_deref(),
        Some(
            "has a report whose line 2 is neither a linked finding nor a requirement request: \
             `revised: `a.rs:1 — old` → `a.rs:1 — new``"
        )
    );
}

#[test]
fn an_unknown_id_and_a_template_place_quote_their_line_too() {
    let cited = "a.rs:1 — if not addressed: x — ENG-999\noutcome: findings — one.\n";
    let refused = refusal(cited, "outcome: findings — one.", &known()).unwrap();
    assert!(
        refused.ends_with(
            "cites ENG-999, which no requirement or invariant has: \
             `a.rs:1 — if not addressed: x — ENG-999`"
        ),
        "{refused}"
    );

    let template = "path:line — if not addressed: x — I2\noutcome: findings — one.\n";
    let refused = refusal(template, "outcome: findings — one.", &known()).unwrap();
    assert!(
        refused.ends_with(": `path:line — if not addressed: x — I2`"),
        "{refused}"
    );
}

#[test]
fn a_line_saying_all_is_well_quotes_its_line_and_says_to_leave_it_out() {
    let report = "a.rs:1 — if not addressed: the range matches — as checked.\n\
                  outcome: none — clean.\n";

    assert_eq!(
        refusal(report, "outcome: none — clean.", &known()).as_deref(),
        Some(
            "has a report whose line 1 cites no requirement or invariant id: a line saying all \
             is well is not a finding, so leave it out: \
             `a.rs:1 — if not addressed: the range matches — as checked.`"
        )
    );
}

#[test]
fn a_line_ending_with_no_finding_is_a_note_and_no_refusal() {
    let report = "note: all is well, not a finding: a.rs:1 — if not addressed: x — no finding.\n\
                  outcome: none — clean.\n";

    assert_eq!(refusal(report, "outcome: none — clean.", &known()), None);
}

#[test]
fn a_missing_dash_and_a_remark_after_the_ids_quote_their_line() {
    for (report, want) in [
        (
            "a.rs:1: if not addressed: x — I2\noutcome: findings — one.\n",
            "names a place with no spaced dash after it: write `path:line — if not addressed: …` \
             or `path:line — request: …`, the dash between spaces: \
             `a.rs:1: if not addressed: x — I2`",
        ),
        (
            "a.rs:1 — if not addressed: x — I2 (provisional)\noutcome: findings — one.\n",
            "correct it later with revised:: `a.rs:1 — if not addressed: x — I2 (provisional)`",
        ),
    ] {
        let refused = refusal(report, "outcome: findings — one.", &known()).unwrap();

        assert!(refused.ends_with(want), "{refused}");
    }
}

#[test]
fn a_problem_of_no_line_is_the_gate_s_problem_alone() {
    let cases = [
        ("outcome: findings — one.\n", "outcome: findings — one."),
        (
            "a.rs:1 — if not addressed: x — I2\noutcome: none — clean.\n",
            "outcome: none — clean.",
        ),
        ("", ""),
        (
            "a.rs:1 — if not addressed: x — I2\noutcome: findings — one.\n",
            "outcome: none — clean.",
        ),
    ];
    let want = [
        "has a report whose outcome is findings but which holds none",
        "has a report whose outcome is none but which holds findings",
        "has an empty report",
        "answered another outcome in the run log than its report ends with",
    ];
    for ((report, answer), want) in cases.into_iter().zip(want) {
        assert_eq!(refusal(report, answer, &known()).as_deref(), Some(want));
    }
}

#[test]
fn a_run_that_did_not_answer_is_told_why_and_to_state_as_it_goes() {
    let note = note("its run ended with error_max_turns");

    assert!(
        note.starts_with(
            "Your previous attempt did not answer: its run ended with error_max_turns. \
             This attempt is judged alone: state each finding line the moment you have checked \
             it, read each file once, search no whole tree, and send your outcome line \
             before your turns or time run out. "
        ),
        "{note}"
    );
    assert!(note.ends_with(FORMS), "{note}");
}

#[test]
fn the_forms_are_two_lines_each_with_a_real_place() {
    assert_eq!(FORMS.lines().count(), 2);
    assert!(FORMS.contains("`revised: <one full finding line>`"));
    assert!(FORMS.contains("`withdrawn: docs/x.md:12 — <reason>`"));
    assert!(FORMS.contains("A line is only for a problem: what is sound gets none"));
    assert!(FORMS.contains("`outcome: findings — <sentence>`"));
    assert!(!FORMS.contains(review_gate::report::TEMPLATE_PLACE));
}

/// A work directory defining lang-rust and review-code, lang-rust's
/// diff split in two parts at [`SPLIT`] when `split` holds.
fn workdir(split: bool) -> TempDir {
    let dir = TempDir::new().unwrap();
    fake::define(dir.path(), "lang-rust", 20);
    fake::define(dir.path(), "review-code", 40);
    if split {
        let diffs = dir.path().join("review/diffs");
        fs::create_dir_all(&diffs).unwrap();
        let section = |path: &str| format!("diff --git a/{path} b/{path}\n{}\n", "+".repeat(60));
        fs::write(
            diffs.join("lang-rust.diff"),
            section("a.rs") + &section("b.rs"),
        )
        .unwrap();
    }
    dir
}

const SPLIT: usize = 100;

/// Reviews lang-rust under review-code with `claude` answering as
/// `script`: the runs, the progress lines and the host.
fn review_of(
    dir: &Path,
    script: impl Fn(&Command) -> Result<Output, String> + Sync + 'static,
) -> (Vec<AgentRun>, Vec<String>, FakeHost) {
    let host = FakeHost::new(script);
    let lines = Mutex::new(Vec::new());
    let plan = Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(r#"{"review-code": ["lang-rust"]}"#).unwrap(),
    };
    let options = Options {
        split: SPLIT,
        ..Options::default()
    };
    let runs = crate::run::review(&host, dir, &plan, &known(), options, None, &|l| {
        lines.lock().unwrap().push(l.to_owned());
    })
    .unwrap();
    (runs, lines.into_inner().unwrap(), host)
}

/// A malformed revision, as a specialist wrote one.
const MALFORMED: &str = "revised: `a.rs:1 — old` → `a.rs:1 — new`";

/// A specialist that, before it is told of a refusal, states a finding
/// and [`MALFORMED`]; told, states nothing and answers none.
fn refused_once(c: &Command) -> Result<Output, String> {
    let prompt = prompt_of(c);
    if !prompt.contains("a specialist") {
        return fake::generalist(c);
    }
    if prompt.contains("previous attempt") {
        return fake::answer(c, &[], "outcome: none — clean.");
    }
    let outcome = "outcome: findings — one.";
    fake::answer(c, &[fake::FINDING, MALFORMED, outcome], outcome)
}

/// The prompts lang-rust's runs had, in the order they started.
fn specialist_prompts(host: &FakeHost) -> Vec<String> {
    let calls = host.calls.lock().unwrap();
    calls
        .iter()
        .filter(|c| agent_of(c) == "lang-rust")
        .map(prompt_of)
        .collect()
}

#[test]
fn a_refused_report_stands_and_its_run_is_not_paid_for_again() {
    let dir = workdir(false);

    let (runs, lines, host) = review_of(dir.path(), refused_once);

    let rust = &runs[0];
    assert!(rust.answered(), "{:?}", rust.failure);
    assert_eq!(rust.attempts, 1);
    assert_eq!(specialist_prompts(&host).len(), 1);
    assert!((rust.log.cost - 0.01).abs() < 1e-9, "{}", rust.log.cost);
    assert!(
        lines.iter().any(|l| l.starts_with("lang-rust (specialist)")
            && l.ends_with("not run again: its report is refused and the run was paid for")),
        "{lines:?}"
    );
    // What it stated is its report, refused as it stands.
    let report = fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap();
    assert!(
        report.contains(fake::FINDING) && report.contains(MALFORMED),
        "{report}"
    );
    assert!(refusal(&report, &rust.log.answer, &known()).is_some());
    assert!(
        !dir.path()
            .join("review/logs/lang-rust.first.jsonl")
            .exists()
    );
}

#[test]
fn a_run_cut_off_after_it_cost_money_keeps_what_it_stated_and_is_not_run_again() {
    let dir = workdir(false);

    let (runs, lines, host) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        fake::write_log(
            c,
            &fake::log_of(&[fake::FINDING], "error_max_turns", "", 0.1),
        );
        Ok(Output::default())
    });

    let rust = &runs[0];
    assert_eq!(
        rust.failure.as_deref(),
        Some("its run ended with error_max_turns")
    );
    assert_eq!(rust.attempts, 1);
    assert_eq!(specialist_prompts(&host).len(), 1);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("not run again: it cost 0.1000 USD")),
        "{lines:?}"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        format!(
            "incomplete: its run ended with error_max_turns\n{}\n",
            fake::FINDING
        )
    );
}

#[test]
fn a_run_that_cost_nothing_and_did_not_answer_is_run_once_more_told_why() {
    let dir = workdir(false);

    let (runs, lines, host) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        if prompt_of(c).contains("previous attempt") {
            return fake::specialist(c);
        }
        Err("claude did not start".to_owned())
    });

    let rust = &runs[0];
    assert!(rust.answered(), "{:?}", rust.failure);
    assert_eq!(rust.attempts, 2);
    assert!(
        specialist_prompts(&host)[1].ends_with(&note("claude did not start")),
        "{lines:?}"
    );
}

#[test]
fn a_report_that_cannot_be_written_fails_the_run_saying_so() {
    let dir = workdir(false);
    // A directory where the report goes, so the write fails.
    fs::create_dir_all(dir.path().join("review/reports/lang-rust.md")).unwrap();

    let (runs, _, _) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        fake::write_log(
            c,
            &fake::log_of(&[fake::FINDING], "error_max_turns", "", 0.1),
        );
        Ok(Output::default())
    });

    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure
            .starts_with("its run ended with error_max_turns; its report could not be written: "),
        "{failure}"
    );
}

#[test]
fn a_part_s_refused_report_stands_and_the_merge_names_it() {
    let dir = workdir(true);

    let (runs, lines, host) = review_of(dir.path(), |c| {
        if prompt_of(c).contains("you are part 2 of 2") {
            refused_once(c)
        } else {
            fake::specialist(c)
        }
    });

    assert_eq!(specialist_prompts(&host).len(), 2);
    let rust = &runs[0];
    assert_eq!((rust.parts, rust.attempts), (2, 2));
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("lang-rust part 2 of 2 (specialist)")
                && l.contains("not run again")),
        "{lines:?}"
    );
    let report = fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap();
    assert!(report.contains(MALFORMED), "{report}");
    assert!(refusal(&report, &rust.log.answer, &known()).is_some());
}

/// One model call that used tokens, and no result: a run killed at its
/// deadline or crashed after the model was called.
const CALLED: &str = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"output_tokens":5},"content":[{"type":"text","text":"src/a.rs:3 — if not addressed: a long line hides a bug — I9"}]}}"#;

/// A result that failed before any model call: no cost, one turn
/// counted, no usage.
const FREE: &str = r#"{"type":"result","subtype":"error_during_execution","is_error":true,"result":"","total_cost_usd":0,"num_turns":1}"#;

#[test]
fn a_run_is_charged_by_its_model_usage_or_its_cost_never_by_its_turns() {
    let parse = review_eval::log::parse;

    assert_eq!(
        charged(&parse(CALLED)).as_deref(),
        Some("it made 1 model call")
    );
    assert_eq!(charged(&parse(FREE)), None);
    assert_eq!(charged(&review_eval::log::RunLog::default()), None);
    let costly = parse(&fake::log("x", 0.25));
    assert_eq!(charged(&costly).as_deref(), Some("it cost 0.2500 USD"));
    let two = parse(&format!("{CALLED}\n{}", CALLED.replace("\"m1\"", "\"m2\"")));
    assert_eq!(charged(&two).as_deref(), Some("it made 2 model calls"));
}

#[test]
fn a_run_with_model_calls_and_no_result_is_charged_and_not_run_again() {
    let dir = workdir(false);

    let (runs, lines, host) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        fake::write_log(c, &format!("{CALLED}\n"));
        Ok(Output {
            timed_out: true,
            ..Output::default()
        })
    });

    let rust = &runs[0];
    assert_eq!(rust.failure.as_deref(), Some("its run ended with deadline"));
    assert_eq!(rust.attempts, 1);
    assert_eq!(specialist_prompts(&host).len(), 1);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("not run again: it made 1 model call")),
        "{lines:?}"
    );
}

#[test]
fn a_failure_that_cost_nothing_runs_once_more_whatever_turns_it_counts() {
    let dir = workdir(false);

    let (runs, _, host) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        if prompt_of(c).contains("previous attempt") {
            return fake::specialist(c);
        }
        fake::write_log(c, &format!("{FREE}\n"));
        Ok(Output::default())
    });

    assert!(runs[0].answered(), "{:?}", runs[0].failure);
    assert_eq!(runs[0].attempts, 2);
    assert_eq!(specialist_prompts(&host).len(), 2);
}
