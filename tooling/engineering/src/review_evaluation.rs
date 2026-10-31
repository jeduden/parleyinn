//! ENG-32: every reviewer component has its evaluation cases, every
//! case applies to the tree and makes the review bundle dispatch its
//! component, and the harness grades a run and decides a case as the
//! requirement says. The checks call the `review-eval` crate, which the
//! evaluate job runs; the decision tables here drive its grader and its
//! two-of-three rule without a model.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use review_eval::case::{Case, Expect, Expected};
use review_eval::component::{Component, Kind};
use review_eval::grade::{Run, grade};
use review_eval::log::RunLog;
use review_eval::plan::{self, Planned};
use review_eval::published::{self, Line};
use review_eval::score::{CaseRecord, GateFinding, RunRecord, Scorecard, SuiteRecord, decided};
use review_eval::{CASES, check};

use crate::{Checkout, list};

/// The component's name and report the decision table grades.
const COMPONENT: &str = "lang-rust";
const REPORT: &str = "review/reports/lang-rust.md";
/// The paths the table's case expects a finding at, and forbids one at.
const WANTED: &str = "crates/x/src/a.rs";
const FORBIDDEN: &str = "crates/x/src/b.rs";

/// The components without one of their three kinds of case (ENG-32).
#[must_use]
pub fn components_have_their_cases(checkout: &Checkout) -> Vec<String> {
    check::kind_problems(&checkout.root, &checkout.root, &checkout.root.join(CASES))
}

/// The cases that do not read, apply to the checkout, or make the
/// review bundle dispatch their component (ENG-32).
#[must_use]
pub fn cases_apply_and_dispatch(checkout: &Checkout) -> Vec<String> {
    check::case_problems(&checkout.root, &checkout.root, &checkout.root.join(CASES))
}

/// Grades one run per row of the scenario's table and compares the
/// grader's answer with the row's `run` column.
#[must_use]
pub fn run_graded(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let got = run_outcome(row);
            (row.get("run").map(String::as_str) != Some(got.as_str()))
                .then(|| format!("row {} {row:?}: the grader says {got}", i + 1))
        })
        .collect()
}

/// The specialist the decision table grades: a strict-lined one on
/// haiku, as the language specialists are.
fn table_component() -> Component {
    Component {
        name: COMPONENT.to_owned(),
        kind: Kind::Specialist,
        file: format!(".claude/agents/{COMPONENT}.md"),
        text: "findings, each on one line".to_owned(),
        model: "haiku".to_owned(),
        effort: "low".to_owned(),
        max_turns: 20,
        tools: Vec::new(),
        strict_lines: true,
    }
}

/// The decision table's case: a defect that wants an unwrap finding at
/// one path and forbids a finding at another.
fn table_case() -> Case {
    Case {
        component: COMPONENT.to_owned(),
        name: "table".to_owned(),
        kind: "defect".to_owned(),
        title: "the table's case".to_owned(),
        consent: None,
        edits: Vec::new(),
        seeds: Vec::new(),
        expect: Expect {
            outcome: vec!["blocking".to_owned()],
            finding: vec![Expected {
                path: WANTED.to_owned(),
                mentions: vec!["unwrap".to_owned()],
                ..Expected::default()
            }],
            none: vec![Expected {
                path: FORBIDDEN.to_owned(),
                ..Expected::default()
            }],
            ..Expect::default()
        },
        repetitions: 3,
        passes: 2,
    }
}

/// What the grader says of the run `row` describes: `passes`, `passes,
/// slips on` what it slips on first, or `fails on` what it fails on
/// first.
fn run_outcome(row: &BTreeMap<String, String>) -> String {
    let column = |key: &str| row.get(key).map_or("", String::as_str);
    let (component, case) = (table_component(), table_case());
    let outcome_line = if column("outcome") == "as expected" {
        "outcome: blocking — it unwraps."
    } else {
        "outcome: sound — it is fine."
    };
    let mut report = String::new();
    if column("finding") == "found" {
        report.push_str(&format!(
            "{WANTED}:6 — blocking — an unwrap outside tests — ENG-16\n"
        ));
    }
    if column("forbidden") == "found" {
        report.push_str(&format!("{FORBIDDEN}:2 — nit — a long line — ENG-16\n"));
    }
    report.push_str(&format!("\n{outcome_line}\n"));
    let answer = if column("answer") == "more" {
        format!("I read the diff.\n{outcome_line}\n{REPORT}\n")
    } else {
        format!("{outcome_line}\n{REPORT}\n")
    };
    let log = RunLog {
        answered: column("answered") == "yes",
        subtype: if column("answered") == "yes" {
            "success".to_owned()
        } else {
            "error_max_turns".to_owned()
        },
        answer,
        writes: vec![REPORT.to_owned()],
        ..RunLog::default()
    };
    let graded = grade(
        &component,
        &case,
        &Run {
            log: &log,
            report: (column("report") == "written").then_some(report.as_str()),
        },
    );
    match (graded.failures.first(), graded.slips.first()) {
        (Some(f), _) => format!("fails on {}", failed_on(f)),
        (None, Some(s)) => format!("passes, slips on {}", failed_on(s)),
        (None, None) => "passes".to_owned(),
    }
}

/// The problems with how a failed case reaches the gate: one blocking
/// finding at the component's file, naming the case, the model it ran
/// on and its first failure, and none for a case that passed.
#[must_use]
pub fn failed_case_is_a_blocking_finding() -> Vec<String> {
    let card = Scorecard {
        suites: vec![table_suite()],
    };
    finding_problems(&card.findings(&|c| format!(".claude/agents/{c}.md")))
}

/// The problems with `findings` as the table suite's: one, blocking, at
/// the component's file, naming the case, the model and the failure.
fn finding_problems(findings: &[GateFinding]) -> Vec<String> {
    let [f] = findings else {
        return vec![format!(
            "{} findings for one failed case and one passed, want one",
            findings.len()
        )];
    };
    let mut problems = Vec::new();
    if f.path != ".claude/agents/lang-rust.md" {
        problems.push(format!(
            "the finding is at {}, want the component's file",
            f.path
        ));
    }
    if f.severity != "blocking" {
        problems.push(format!("the finding is {}, want blocking", f.severity));
    }
    for word in [
        "lang-rust/unwrap",
        "claude-haiku-5-5",
        "outcome sound, want blocking",
    ] {
        if !f.body.contains(word) {
            problems.push(format!("the finding does not name {word:?}: {}", f.body));
        }
    }

    problems
}

/// The problems with a scorecard line: rendered and read back, it holds
/// the component's cases and runs passed, the runs that slipped, the
/// cost, the mean turns and seconds, and the model the runs resolved to.
#[must_use]
pub fn scorecard_line_records_the_suite() -> Vec<String> {
    let card = Scorecard {
        suites: vec![table_suite()],
    };
    line_problems(published::parse(&published::render(&published::lines(
        &card,
    ))))
}

/// The problems with `read`, a scorecard read back, as the table
/// suite's line.
fn line_problems(read: Result<Vec<Line>, String>) -> Vec<String> {
    let lines = match read {
        Ok(lines) => lines,
        Err(e) => return vec![format!("the scorecard does not read back: {e}")],
    };
    let Some(line) = lines.first() else {
        return vec!["the scorecard holds no line".to_owned()];
    };
    let got = (
        line.component.as_str(),
        line.model.as_str(),
        line.resolved.as_str(),
        (line.cases_passed, line.cases),
        (line.runs_passed, line.runs),
        line.slipped,
    );
    let want = ("lang-rust", "haiku", "claude-haiku-5-5", (1, 2), (2, 4), 1);
    let mut problems = Vec::new();
    if got != want {
        problems.push(format!("the line reads {got:?}, want {want:?}"));
    }
    let numbers = (line.cost, line.turns, line.seconds);
    if (line.cost - 0.4).abs() > 1e-6
        || (line.turns - 6.0).abs() > 1e-6
        || (line.seconds - 3.0).abs() > 1e-6
    {
        problems.push(format!(
            "the line's cost, turns and seconds read {numbers:?}, want (0.4, 6.0, 3.0)"
        ));
    }

    problems
}

/// The problems with the scheduled run's workflow, `text`: it must
/// record the run's exit code, write the report that names each failed
/// case into its summary, and fail on a non-zero code after that.
#[must_use]
pub fn scheduled_run_fails_on_failed_cases(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !text.contains("echo \"code=$code\" >> \"$GITHUB_OUTPUT\"") {
        problems.push("the scheduled run does not record the run's exit code".to_owned());
    }
    if !text.contains("cat review-eval/report.md >> \"$GITHUB_STEP_SUMMARY\"") {
        problems.push(
            "the scheduled run's summary does not hold the report naming each failed case"
                .to_owned(),
        );
    }
    let fails = text
        .split("if: steps.run.outputs.code != '0'")
        .nth(1)
        .is_some_and(|rest| rest.lines().take(6).any(|l| l.trim() == "exit 1"));
    if !fails {
        problems.push("no step fails the scheduled run on a failed case".to_owned());
    }

    problems
}

/// The suite the scorecard steps read: `lang-rust` on haiku, one case
/// failed in two runs and one passed in two, one of them with a slip.
fn table_suite() -> SuiteRecord {
    let run = |passed: bool, slips: Vec<String>| RunRecord {
        passed,
        failures: if passed {
            Vec::new()
        } else {
            vec!["outcome sound, want blocking".to_owned()]
        },
        slips,
        answered: true,
        cost: 0.1,
        turns: 6,
        duration_ms: 3000,
        resolved_model: "claude-haiku-5-5".to_owned(),
        ..RunRecord::default()
    };
    let case = |name: &str, passed: bool, runs: Vec<RunRecord>| CaseRecord {
        component: COMPONENT.to_owned(),
        name: name.to_owned(),
        kind: "defect".to_owned(),
        title: format!("the {name} case"),
        repetitions: 3,
        passes: 2,
        runs,
        passed,
    };
    SuiteRecord {
        component: COMPONENT.to_owned(),
        model: "haiku".to_owned(),
        hash: "0123456789abcdef".to_owned(),
        cases: vec![
            case(
                "unwrap",
                false,
                vec![run(false, Vec::new()), run(false, Vec::new())],
            ),
            case(
                "clean",
                true,
                vec![
                    run(
                        true,
                        vec![
                            "a line that is neither a finding nor the outcome: Findings:"
                                .to_owned(),
                        ],
                    ),
                    run(true, Vec::new()),
                ],
            ),
        ],
    }
}

/// What a failure is about, by its first words.
fn failed_on(failure: &str) -> &'static str {
    if failure.starts_with("did not answer") {
        "no answer"
    } else if failure.starts_with("wrote no report")
        || failure.starts_with("wrote an empty report")
        || failure.starts_with("report does not end")
    {
        "report"
    } else if failure.starts_with("outcome ") {
        "outcome"
    } else if failure.starts_with("no finding") {
        "finding"
    } else if failure.starts_with("a finding at") {
        "forbidden"
    } else if failure.starts_with("answer ") {
        "answer"
    } else {
        "something else"
    }
}

/// Decides one case per row of the scenario's table, each run in turn
/// until decided, and compares with the row's `case` and `runs made`
/// columns. A run of `none` did not answer, which fails it.
#[must_use]
pub fn case_decided(rows: &[BTreeMap<String, String>]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            let column = |key: &str| row.get(key).map_or("", String::as_str);
            let (verdict, made) = decide(column("runs"));
            let want = (column("case"), column("runs made"));
            (want != (verdict, made.to_string().as_str())).then(|| {
                format!(
                    "row {} {row:?}: the case {verdict} after {made} runs",
                    i + 1
                )
            })
        })
        .collect()
}

/// The case's verdict after the runs `runs` lists, and how many it
/// took: `passes` or `fails`, with two passes needed of three.
fn decide(runs: &str) -> (&'static str, usize) {
    let (mut passes, mut fails) = (0, 0);
    for (made, run) in runs.split(',').map(str::trim).enumerate() {
        if run == "pass" {
            passes += 1;
        } else {
            fails += 1;
        }
        if let Some(passed) = decided(passes, fails, 3, 2) {
            return (if passed { "passes" } else { "fails" }, made + 1);
        }
    }

    ("fails", runs.split(',').count())
}

/// The line a pull request adds to a shared section's text, as every
/// component that includes it holds it expanded.
const SHARED_EDIT: &str = "A line the pull request adds to the shared section.";

/// The trees the evaluation's plan reads: the components, the sections
/// and files that name them, and their cases.
const PLANNED_TREES: [&str; 4] = [".claude/agents", "docs/srs", "docs/domain-model", CASES];

/// Requires the evaluation's plan, for a pull request that changes only
/// the shared `section` (ENG-32), to run one component per distinct set
/// of sections among those that include it, standing for the rest of
/// its set: it copies the checkout's components into `head`, adds a
/// line to the section's text in each that includes it, and plans
/// `head` against the checkout.
///
/// # Errors
///
/// Fails when the trees cannot be copied or planned, or no component
/// includes `section`.
pub fn shared_section_plans_one_per_set(
    checkout: &Checkout,
    section: &str,
    head: &Path,
) -> Result<Vec<String>, String> {
    for tree in PLANNED_TREES {
        copy_dir(&checkout.root.join(tree), &head.join(tree))?;
    }
    let agents = head.join(".claude/agents");
    let mut sets: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(&agents).map_err(|e| format!("read {}: {e}", agents.display()))? {
        let path = entry
            .map_err(|e| format!("read {}: {e}", agents.display()))?
            .path();
        let Some(name) = path
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_suffix(".md"))
        else {
            continue;
        };
        let text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if let Some((edited, sections)) = edit_section(&text, section) {
            fs::write(&path, edited).map_err(|e| format!("write {}: {e}", path.display()))?;
            sets.entry(sections).or_default().push(name.to_owned());
        }
    }
    if sets.is_empty() {
        return Err(format!("no reviewer component includes {section}"));
    }

    Ok(plan_problems(&sets, &plan::between(&checkout.root, head)?))
}

/// `text` with [`SHARED_EDIT`] added to the expanded text of its
/// include of `section`, and every section it includes, sorted; `None`
/// when it does not include `section`.
fn edit_section(text: &str, section: &str) -> Option<(String, Vec<String>)> {
    let (mut out, mut sections) = (String::new(), BTreeSet::new());
    let (mut header, mut hit, mut edited) = (false, false, false);
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        out.push_str(line);
        if trimmed.starts_with("<?include") {
            header = true;
        } else if header && trimmed == "?>" {
            header = false;
            if hit {
                out.push_str(&format!("{SHARED_EDIT}\n"));
                (hit, edited) = (false, true);
            }
        } else if let Some(file) = header.then(|| trimmed.strip_prefix("file:")).flatten() {
            let file = file.trim();
            hit = file == section || file.ends_with(&format!("/{section}"));
            sections.insert(file.to_owned());
        }
    }

    edited.then(|| (out, sections.into_iter().collect()))
}

/// What is wrong with `planned` for a change to one shared section that
/// the components of each set in `sets` include: each planned component
/// must be planned for that change alone and include it, and each set
/// must have exactly one planned, which covers the rest of its set.
fn plan_problems(sets: &BTreeMap<Vec<String>, Vec<String>>, planned: &[Planned]) -> Vec<String> {
    let mut problems = Vec::new();
    for p in planned {
        if !p.reason.starts_with("shared sections changed") {
            problems.push(format!(
                "component {} planned for {}, want the shared section's change alone",
                p.component, p.reason
            ));
        }
        if !sets.values().any(|names| names.contains(&p.component)) {
            problems.push(format!(
                "component {} planned, but it includes no changed section",
                p.component
            ));
        }
    }
    for (sections, names) in sets {
        let members: Vec<&Planned> = planned
            .iter()
            .filter(|p| names.contains(&p.component))
            .collect();
        let [one] = members.as_slice() else {
            problems.push(format!(
                "{} components planned for the sections {}, want one",
                members.len(),
                list(sections)
            ));
            continue;
        };
        let mut covered: Vec<String> = one
            .reason
            .split_once("; also covers ")
            .map(|(_, rest)| rest.split(", ").map(str::to_owned).collect())
            .unwrap_or_default();
        covered.push(one.component.clone());
        covered.sort();
        let mut want = names.clone();
        want.sort();
        if covered != want {
            problems.push(format!(
                "component {} covers {}, want {}",
                one.component,
                list(&covered),
                list(&want)
            ));
        }
    }

    problems
}

/// Copies the tree at `from` to `to`, directories included.
fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    let read = |e: std::io::Error| format!("read {}: {e}", from.display());
    fs::create_dir_all(to).map_err(|e| format!("write {}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(read)? {
        let path = entry.map_err(read)?.path();
        let target = to.join(path.file_name().unwrap_or_default());
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|e| format!("copy {}: {e}", path.display()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
