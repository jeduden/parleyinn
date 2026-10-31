use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use review_eval::host::{Command, Output};
use testkit::TempDir;

use super::*;
use crate::fake::{self, FakeHost, agent_of, prompt_of};
use crate::plan::parse_dispatch;
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
    let report = "a.rs:1 — if not addressed: the range matches — no finding.\n\
                  outcome: none — clean.\n";

    assert_eq!(
        refusal(report, "outcome: none — clean.", &known()).as_deref(),
        Some(
            "has a report whose line 1 cites no requirement or invariant id: a line saying all \
             is well is not a finding, so leave it out: \
             `a.rs:1 — if not addressed: the range matches — no finding.`"
        )
    );
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
fn a_refused_report_s_note_names_what_was_refused_and_restates_the_forms() {
    let note = note(&Why::Refused("has a report whose line 2 is malformed: `x`"));

    assert_eq!(
        note,
        format!(
            "The review refused your previous attempt's report: it has a report whose line 2 is \
             malformed: `x`. This attempt is judged alone, so state every finding again. {FORMS}"
        )
    );
}

#[test]
fn a_run_that_did_not_answer_is_told_why_and_to_state_as_it_goes() {
    let note = note(&Why::Silent("its run ended with error_max_turns"));

    assert!(
        note.starts_with(
            "Your previous attempt did not answer: its run ended with error_max_turns. \
             This attempt is judged alone: state each finding line the moment you find it, \
             read each file once, search no whole tree, and send your outcome line \
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
    let runs = crate::run::review(&host, dir, &plan, &known(), options, &|l| {
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
fn a_refused_report_is_run_once_more_told_its_problem_and_judged_alone() {
    let dir = workdir(false);

    let (runs, lines, host) = review_of(dir.path(), refused_once);

    let rust = &runs[0];
    assert!(rust.answered(), "{:?}", rust.failure);
    assert_eq!(rust.attempts, 2);
    assert!((rust.log.cost - 0.02).abs() < 1e-9, "{}", rust.log.cost);
    assert_eq!(runs[1].attempts, 1);
    let prompts = specialist_prompts(&host);
    assert_eq!(prompts.len(), 2);
    assert!(!prompts[0].contains("previous attempt"));
    let problem = format!(
        "has a report whose line 2 is neither a linked finding nor a requirement request: \
         `{MALFORMED}`"
    );
    assert!(
        prompts[1].ends_with(&note(&Why::Refused(&problem))),
        "{}",
        prompts[1]
    );
    assert_eq!(
        lines[0],
        format!("lang-rust (specialist) {problem}; running it once more")
    );
    // Judged alone: the first attempt's finding is not carried over.
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        "outcome: none — clean.\n"
    );
    assert!(
        dir.path()
            .join("review/logs/lang-rust.first.jsonl")
            .is_file()
    );
}

#[test]
fn a_report_refused_twice_stays_refused_and_the_first_s_findings_do_not_rescue_it() {
    let dir = workdir(false);

    let (runs, _, host) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        // Refused each time, for a problem the other's lines would
        // cure: a finding beside none, then findings beside none.
        if prompt_of(c).contains("previous attempt") {
            fake::answer(c, &[], "outcome: findings — one.")
        } else {
            fake::answer(c, &[fake::FINDING], "outcome: none — clean.")
        }
    });

    assert_eq!(specialist_prompts(&host).len(), 2);
    let rust = &runs[0];
    assert!(rust.answered());
    assert_eq!(rust.attempts, 2);
    let report = fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap();
    assert_eq!(report, "outcome: findings — one.\n");
    assert_eq!(
        refusal(&report, &rust.log.answer, &known()).as_deref(),
        Some("has a report whose outcome is findings but which holds none")
    );
}

#[test]
fn a_second_attempt_that_does_not_answer_shows_what_both_stated() {
    let dir = workdir(false);

    let (runs, lines, host) = review_of(dir.path(), |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        let stated = if prompt_of(c).contains("previous attempt") {
            "b.rs:2 — if not addressed: y — I2"
        } else {
            fake::FINDING
        };
        fake::write_log(c, &fake::log_of(&[stated], "error_max_turns", "", 0.1));
        Ok(Output::default())
    });

    let rust = &runs[0];
    assert_eq!(
        rust.failure.as_deref(),
        Some("its run ended with error_max_turns")
    );
    assert_eq!(rust.attempts, 2);
    assert!(
        specialist_prompts(&host)[1]
            .ends_with(&note(&Why::Silent("its run ended with error_max_turns"))),
        "{lines:?}"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap(),
        format!(
            "incomplete: its run ended with error_max_turns\n{}\n\
             b.rs:2 — if not addressed: y — I2\n",
            fake::FINDING
        )
    );
}

#[test]
fn a_salvaged_report_that_cannot_be_written_fails_the_run_saying_so() {
    let dir = workdir(false);
    let report = dir.path().join("review/reports/lang-rust.md");

    let (runs, _, _) = review_of(dir.path(), move |c| {
        if !prompt_of(c).contains("a specialist") {
            return fake::generalist(c);
        }
        let stated = if prompt_of(c).contains("previous attempt") {
            // The second run's own report is not written, nor its salvage.
            fs::remove_file(&report).unwrap();
            fs::create_dir_all(&report).unwrap();
            "b.rs:2 — if not addressed: y — I2"
        } else {
            fake::FINDING
        };
        fake::write_log(c, &fake::log_of(&[stated], "error_max_turns", "", 0.1));
        Ok(Output::default())
    });

    let failure = runs[0].failure.clone().unwrap();
    assert!(
        failure
            .starts_with("its run ended with error_max_turns; its report could not be written: "),
        "{failure}"
    );
    assert_eq!(
        failure.matches("could not be written").count(),
        2,
        "{failure}"
    );
}

#[test]
fn a_part_s_refused_report_is_run_once_more_and_the_merge_takes_the_second() {
    let dir = workdir(true);

    let (runs, lines, host) = review_of(dir.path(), |c| {
        if prompt_of(c).contains("you are part 2 of 2") {
            refused_once(c)
        } else {
            fake::specialist(c)
        }
    });

    assert_eq!(specialist_prompts(&host).len(), 3);
    let rust = &runs[0];
    assert!(rust.answered(), "{:?}", rust.failure);
    assert_eq!((rust.parts, rust.attempts), (2, 3));
    assert!(
        lines.iter().any(|l| l.starts_with(
            "lang-rust part 2 of 2 (specialist) has a report whose line 2 is neither"
        )),
        "{lines:?}"
    );
    let report = fs::read_to_string(dir.path().join("review/reports/lang-rust.md")).unwrap();
    assert!(!report.contains(MALFORMED), "{report}");
    assert_eq!(refusal(&report, &rust.log.answer, &known()), None);
    assert!(
        dir.path()
            .join("review/logs/lang-rust.2.first.jsonl")
            .is_file()
    );
}
