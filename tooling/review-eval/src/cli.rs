//! The `review-eval` command line: `check` runs the deterministic
//! check of the cases; `plan` names the components a pull request
//! changed; `run` evaluates components on their model, or on one the
//! run names, and writes the scorecard, the findings the review gate
//! weighs and the report; `compare` reads a scorecard against the
//! published one; `publish` writes a scorecard's lines into it.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::case::Case;
use crate::component::{self, Component, file_of, hash, suite_of};
use crate::host::Host;
use crate::plan::{self, Planned};
use crate::score::{CaseRecord, RunRecord, Scorecard, SuiteRecord, decided};
use crate::{CASES, DEADLINE_SECONDS, check, compare, grade, published, runner, world};

/// Every case passed, or the command had nothing to fail on.
pub const EXIT_OK: u8 = 0;
/// A case failed, the check found a problem, or a file could not be
/// read or written.
pub const EXIT_FAILURE: u8 = 1;
/// The command line is wrong.
pub const EXIT_USAGE: u8 = 2;

/// How to call it.
pub const USAGE: &str = "usage: review-eval check [--checkout DIR] [--tree DIR] [--cases DIR]
       review-eval plan --base DIR --head DIR [--out FILE]
       review-eval run --out DIR (--component NAME)... | --all | --plan FILE
                       [--checkout DIR] [--tree DIR] [--cases DIR] [--base-cases DIR]
                       [--model MODEL]
                       [--repetitions N] [--passes K] [--parallel P] [--deadline SECONDS]
       review-eval compare --published FILE --scorecard FILE
       review-eval publish --scorecard FILE --into FILE";

/// The files a run writes under its `--out` directory.
pub const SCORECARD_JSON: &str = "scorecard.json";
pub const SCORECARD_TEXT: &str = "scorecard.txt";
pub const FINDINGS_JSON: &str = "findings.json";
pub const REPORT_MD: &str = "report.md";
/// Where a run keeps each run's log, report and grade.
pub const RUNS: &str = "runs";

/// How many cases run at once unless the run says otherwise.
const PARALLEL: usize = 4;

/// What `run` evaluates with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The checkout the worlds are worktrees of.
    pub checkout: PathBuf,
    /// The tree the components are read from.
    pub tree: PathBuf,
    /// The written cases.
    pub cases: PathBuf,
    /// Written cases that judge a component instead of `cases`'s when
    /// they hold a directory for it: main's, in the review of a pull
    /// request.
    pub base_cases: Option<PathBuf>,
    /// A model every component runs on instead of its own.
    pub model: Option<String>,
    /// Repetitions and passes in place of each case's own.
    pub repetitions: Option<usize>,
    pub passes: Option<usize>,
    pub parallel: usize,
    pub deadline: Duration,
    pub out: PathBuf,
}

/// Runs `args` and returns the exit code; `main` without the process.
pub fn run(
    host: &dyn Host,
    args: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let (command, flags) = match parse(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            let _ = writeln!(stderr, "review-eval: {e}\n{USAGE}");
            return EXIT_USAGE;
        }
    };
    let result = match command.as_str() {
        "check" => check_command(&flags, stdout),
        "plan" => plan_command(&flags, stdout),
        "run" => run_command(host, &flags, stdout, stderr),
        "compare" => compare_command(&flags, stdout),
        "publish" => publish_command(&flags, stdout),
        other => Err(Usage(format!("no command {other}"))),
    };
    match result {
        Ok(code) => code,
        Err(Usage(e)) => {
            let _ = writeln!(stderr, "review-eval: {e}\n{USAGE}");
            EXIT_USAGE
        }
        Err(Failed(e)) => {
            let _ = writeln!(stderr, "review-eval: {e}");
            EXIT_FAILURE
        }
    }
}

/// Why a command did not run to its end.
enum Problem {
    Usage(String),
    Failed(String),
}

use Problem::{Failed, Usage};

impl From<String> for Problem {
    fn from(e: String) -> Self {
        Failed(e)
    }
}

/// The flags a command got, each with its values in order; a flag
/// without a value holds an empty string.
type Flags = BTreeMap<String, Vec<String>>;

/// The command and its flags, from `args`.
fn parse(args: impl IntoIterator<Item = String>) -> Result<(String, Flags), String> {
    let mut args = args.into_iter().peekable();
    let command = args.next().ok_or("no command")?;
    let mut flags = Flags::new();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument {arg:?}"));
        };
        let (name, value) = match flag.split_once('=') {
            Some((name, value)) => (name.to_owned(), value.to_owned()),
            None => {
                let value = match args.peek() {
                    Some(next) if !next.starts_with("--") => args.next().unwrap_or_default(),
                    _ => String::new(),
                };
                (flag.to_owned(), value)
            }
        };
        flags.entry(name).or_default().push(value);
    }

    Ok((command, flags))
}

/// The one value of `flag`, when given.
fn one(flags: &Flags, flag: &str) -> Result<Option<String>, Problem> {
    match flags.get(flag).map(Vec::as_slice) {
        None => Ok(None),
        Some([value]) if !value.is_empty() => Ok(Some(value.clone())),
        Some([_]) => Err(Usage(format!("--{flag} needs a value"))),
        Some(_) => Err(Usage(format!("--{flag} given more than once"))),
    }
}

/// The one value of `flag`, which must be given.
fn required(flags: &Flags, flag: &str) -> Result<String, Problem> {
    one(flags, flag)?.ok_or_else(|| Usage(format!("--{flag} is required")))
}

/// Refuses a flag the command does not take.
fn only(flags: &Flags, allowed: &[&str]) -> Result<(), Problem> {
    match flags.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(flag) => Err(Usage(format!("unknown flag --{flag}"))),
        None => Ok(()),
    }
}

/// A number `flag` names.
fn number(flags: &Flags, flag: &str) -> Result<Option<usize>, Problem> {
    one(flags, flag)?
        .map(|v| {
            v.parse()
                .map_err(|_| Usage(format!("--{flag} {v} is not a number")))
        })
        .transpose()
}

/// The checkout, the tree and the cases directory a command reads.
fn places(flags: &Flags) -> Result<(PathBuf, PathBuf, PathBuf), Problem> {
    let checkout = PathBuf::from(one(flags, "checkout")?.unwrap_or_else(|| ".".to_owned()));
    let tree = one(flags, "tree")?.map_or_else(|| checkout.clone(), PathBuf::from);
    let cases = one(flags, "cases")?.map_or_else(|| checkout.join(CASES), PathBuf::from);

    Ok((checkout, tree, cases))
}

fn check_command(flags: &Flags, stdout: &mut dyn Write) -> Result<u8, Problem> {
    only(flags, &["checkout", "tree", "cases"])?;
    let (checkout, tree, cases) = places(flags)?;
    let problems = check::problems(&checkout, &tree, &cases, None);
    if problems.is_empty() {
        let cases = check::all_cases(&checkout, &cases)?;
        let components = component::names(&checkout)?;
        let _ = writeln!(
            stdout,
            "every case applies and every component has its cases: {} cases, {} components",
            cases.len(),
            components.len()
        );
        return Ok(EXIT_OK);
    }
    for problem in &problems {
        let _ = writeln!(stdout, "{problem}");
    }

    Ok(EXIT_FAILURE)
}

fn plan_command(flags: &Flags, stdout: &mut dyn Write) -> Result<u8, Problem> {
    only(flags, &["base", "head", "out"])?;
    let base = required(flags, "base")?;
    let head = required(flags, "head")?;
    let planned = plan::between(Path::new(&base), Path::new(&head))?;
    if let Some(out) = one(flags, "out")? {
        let text = serde_json::to_string_pretty(&planned).map_err(|e| e.to_string())?;
        fs::write(&out, format!("{text}\n")).map_err(|e| format!("write {out}: {e}"))?;
    }
    if planned.is_empty() {
        let _ = writeln!(stdout, "nothing to evaluate: no component or case changed");
    }
    for p in &planned {
        let _ = writeln!(stdout, "{}: {}", p.component, p.reason);
    }

    Ok(EXIT_OK)
}

fn run_command(
    host: &dyn Host,
    flags: &Flags,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Problem> {
    only(
        flags,
        &[
            "out",
            "component",
            "all",
            "plan",
            "checkout",
            "tree",
            "cases",
            "base-cases",
            "model",
            "repetitions",
            "passes",
            "parallel",
            "deadline",
        ],
    )?;
    let (checkout, tree, cases) = places(flags)?;
    let out = PathBuf::from(required(flags, "out")?);
    let options = Options {
        checkout: checkout.clone(),
        tree,
        cases,
        base_cases: one(flags, "base-cases")?.map(PathBuf::from),
        model: one(flags, "model")?,
        repetitions: number(flags, "repetitions")?,
        passes: number(flags, "passes")?,
        parallel: number(flags, "parallel")?.unwrap_or(PARALLEL).max(1),
        deadline: Duration::from_secs(
            number(flags, "deadline")?.map_or(DEADLINE_SECONDS, |s| s as u64),
        ),
        out,
    };
    let components = selected(flags, &checkout)?;
    if components.is_empty() {
        let _ = writeln!(stdout, "nothing to evaluate");
        write_outputs(&options.out, &Scorecard::default())?;
        return Ok(EXIT_OK);
    }
    let progress = Mutex::new(Vec::new());
    let scorecard = evaluate(host, &options, &components, &|line| {
        if let Ok(mut lines) = progress.lock() {
            lines.push(line.to_owned());
        }
    })?;
    for line in progress.lock().map(|l| l.clone()).unwrap_or_default() {
        let _ = writeln!(stderr, "{line}");
    }
    write_outputs(&options.out, &scorecard)?;
    let _ = write!(stdout, "{}", scorecard.markdown());

    Ok(if scorecard.passed() {
        EXIT_OK
    } else {
        EXIT_FAILURE
    })
}

/// The components `run` was asked for: named, all, or a plan's.
fn selected(flags: &Flags, checkout: &Path) -> Result<Vec<String>, Problem> {
    let named = flags.get("component").cloned().unwrap_or_default();
    let all = flags.contains_key("all");
    let planned = one(flags, "plan")?;
    let mut out: Vec<String> = match (named.is_empty(), all, planned) {
        (false, false, None) => named,
        (true, true, None) => component::names(checkout)?,
        (true, false, Some(file)) => {
            let text = fs::read_to_string(&file).map_err(|e| format!("read {file}: {e}"))?;
            let planned: Vec<Planned> =
                serde_json::from_str(&text).map_err(|e| format!("read {file}: {e}"))?;
            planned.into_iter().map(|p| p.component).collect()
        }
        _ => {
            return Err(Usage(
                "name components with --component, or --all, or --plan FILE, one of the three"
                    .to_owned(),
            ));
        }
    };
    if out.iter().any(String::is_empty) {
        return Err(Usage("--component needs a value".to_owned()));
    }
    out.sort();
    out.dedup();

    Ok(out)
}

/// Evaluates `components` as `options` say, each case of each up to
/// its repetitions until decided, `options.parallel` cases at once,
/// telling `progress` about each run.
///
/// # Errors
///
/// Fails when the cases have a problem, a component is unknown or has
/// no cases, or the run directory cannot be made.
pub fn evaluate(
    host: &dyn Host,
    options: &Options,
    components: &[String],
    progress: &(dyn Fn(&str) + Sync),
) -> Result<Scorecard, String> {
    let problems = check::problems_from(
        &options.checkout,
        &options.tree,
        &options.cases,
        options.base_cases.as_deref(),
        Some(components),
    );
    if !problems.is_empty() {
        return Err(format!(
            "the cases have problems; fix them before a run:\n{}",
            problems.join("\n")
        ));
    }
    let base = options.base_cases.as_deref();
    let cases = check::all_cases_from(&options.checkout, &options.cases, base)?;
    let mut jobs: Vec<(Component, Case)> = Vec::new();
    let mut suites: Vec<SuiteRecord> = Vec::new();
    for name in components {
        let component = component::load(&options.tree, name)?;
        let own: Vec<&Case> = cases
            .iter()
            .filter(|c| c.component == suite_of(name))
            .collect();
        suites.push(SuiteRecord {
            component: name.clone(),
            model: options
                .model
                .clone()
                .unwrap_or_else(|| component.model.clone()),
            hash: hash(&component.text),
            cases: Vec::new(),
        });
        jobs.extend(own.into_iter().map(|c| (component.clone(), c.clone())));
    }
    fs::create_dir_all(options.out.join(RUNS))
        .map_err(|e| format!("make {}: {e}", options.out.join(RUNS).display()))?;
    let queue = Mutex::new(jobs);
    let records: Mutex<Vec<CaseRecord>> = Mutex::new(Vec::new());
    let worktrees = Mutex::new(());
    thread::scope(|scope| {
        for _ in 0..options.parallel {
            scope.spawn(|| worker(host, options, &queue, &records, &worktrees, progress));
        }
    });
    let mut records = records.into_inner().unwrap_or_default();
    records.sort_by(|a, b| (&a.component, &a.name).cmp(&(&b.component, &b.name)));
    for suite in &mut suites {
        suite.cases = records
            .iter()
            .filter(|r| r.component == suite.component)
            .cloned()
            .collect();
    }

    Ok(Scorecard { suites })
}

/// Takes cases off `queue` until it is empty, running each and
/// keeping its record.
fn worker(
    host: &dyn Host,
    options: &Options,
    queue: &Mutex<Vec<(Component, Case)>>,
    records: &Mutex<Vec<CaseRecord>>,
    worktrees: &Mutex<()>,
    progress: &(dyn Fn(&str) + Sync),
) {
    loop {
        let next = queue.lock().ok().and_then(|mut q| q.pop());
        let Some((component, case)) = next else {
            return;
        };
        let record = run_case(host, options, &component, &case, worktrees, progress);
        if let Ok(mut records) = records.lock() {
            records.push(record);
        }
    }
}

/// Runs `case` for `component` until decided, each run in its own
/// directory under the run directory, the world torn down after it.
fn run_case(
    host: &dyn Host,
    options: &Options,
    component: &Component,
    case: &Case,
    worktrees: &Mutex<()>,
    progress: &(dyn Fn(&str) + Sync),
) -> CaseRecord {
    let repetitions = options.repetitions.unwrap_or(case.repetitions);
    let passes = options
        .passes
        .unwrap_or(case.passes)
        .min(repetitions)
        .max(1);
    let mut record = CaseRecord {
        component: component.name.clone(),
        name: case.name.clone(),
        kind: case.kind.clone(),
        title: case.title.clone(),
        repetitions,
        passes,
        runs: Vec::new(),
        passed: false,
    };
    for n in 1..=repetitions {
        let dir = options
            .out
            .join(RUNS)
            .join(&component.name)
            .join(&case.name)
            .join(n.to_string());
        let run = one_run(host, options, component, case, &dir, worktrees);
        progress(&format!(
            "{}/{} run {n}: {} ({:.4} USD, {} turns){}{}",
            component.name,
            case.name,
            if run.passed { "passed" } else { "failed" },
            run.cost,
            run.turns,
            if run.failures.is_empty() {
                String::new()
            } else {
                format!(": {}", run.failures.join("; "))
            },
            if run.slips.is_empty() {
                String::new()
            } else {
                format!("; slipped: {}", run.slips.join("; "))
            }
        ));
        record.runs.push(run);
        let (passed, failed) = (
            record.runs.iter().filter(|r| r.passed).count(),
            record.runs.iter().filter(|r| !r.passed).count(),
        );
        if let Some(verdict) = decided(passed, failed, repetitions, passes) {
            record.passed = verdict;
            break;
        }
    }

    record
}

/// One run of `case` for `component` under `dir`: the world built, the
/// component run, its report and log kept, the grade written, the
/// world torn down.
fn one_run(
    host: &dyn Host,
    options: &Options,
    component: &Component,
    case: &Case,
    dir: &Path,
    worktrees: &Mutex<()>,
) -> RunRecord {
    if let Err(e) = fs::create_dir_all(dir) {
        return RunRecord::failed(&format!("make {}: {e}", dir.display()));
    }
    let world_dir = dir.join("world");
    let result = world::build(
        host,
        &options.checkout,
        &options.tree,
        component,
        case,
        &world_dir,
        worktrees,
    )
    .and_then(|world| {
        let log_path = dir.join("log.jsonl");
        let command = runner::command(
            component,
            &world,
            options.model.as_deref(),
            &log_path,
            options.deadline,
        );
        let log = runner::run(host, &command)?;
        let report = world.report(&component.name);
        if let Some(report) = &report {
            let _ = fs::write(dir.join("report.md"), report);
        }
        let graded = grade::grade(
            component,
            case,
            &grade::Run {
                log: &log,
                report: report.as_deref(),
            },
        );
        let mut record = RunRecord::new(&log, graded.failures, graded.slips);
        record.notes.clone_from(&world.notes);
        let mut grade = if record.passed {
            "passed\n".to_owned()
        } else {
            format!("{}\n", record.failures.join("\n"))
        };
        for slip in &record.slips {
            grade.push_str(&format!("slip: {slip}\n"));
        }
        for note in &record.notes {
            grade.push_str(&format!("note: {note}\n"));
        }
        let _ = fs::write(dir.join("grade.txt"), grade);
        Ok(record)
    });
    let _ = world::teardown(host, &options.checkout, &world_dir, worktrees);
    let _ = fs::remove_dir_all(&world_dir);

    result.unwrap_or_else(|e| RunRecord::failed(&e))
}

/// Writes the scorecard, its published lines, the gate's findings and
/// the report under `out`.
fn write_outputs(out: &Path, scorecard: &Scorecard) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|e| format!("make {}: {e}", out.display()))?;
    let write = |name: &str, text: String| {
        fs::write(out.join(name), text)
            .map_err(|e| format!("write {}: {e}", out.join(name).display()))
    };
    let json = serde_json::to_string_pretty(scorecard).map_err(|e| e.to_string())?;
    write(SCORECARD_JSON, format!("{json}\n"))?;
    write(
        SCORECARD_TEXT,
        published::render(&published::lines(scorecard)),
    )?;
    let findings =
        serde_json::to_string_pretty(&scorecard.findings(&file_of)).map_err(|e| e.to_string())?;
    write(FINDINGS_JSON, format!("{findings}\n"))?;
    write(REPORT_MD, scorecard.markdown())
}

/// Reads a scorecard a run wrote.
fn read_scorecard(file: &str) -> Result<Scorecard, String> {
    let text = fs::read_to_string(file).map_err(|e| format!("read {file}: {e}"))?;

    serde_json::from_str(&text).map_err(|e| format!("read {file}: {e}"))
}

fn compare_command(flags: &Flags, stdout: &mut dyn Write) -> Result<u8, Problem> {
    only(flags, &["published", "scorecard"])?;
    let published_file = required(flags, "published")?;
    let scorecard = read_scorecard(&required(flags, "scorecard")?)?;
    let text =
        fs::read_to_string(&published_file).map_err(|e| format!("read {published_file}: {e}"))?;
    let published = published::parse(&text).map_err(|e| format!("{published_file}: {e}"))?;
    let comparisons = compare::compare(&published, &published::lines(&scorecard));
    let _ = write!(stdout, "{}", compare::render(&comparisons));

    Ok(EXIT_OK)
}

fn publish_command(flags: &Flags, stdout: &mut dyn Write) -> Result<u8, Problem> {
    only(flags, &["scorecard", "into"])?;
    let scorecard = read_scorecard(&required(flags, "scorecard")?)?;
    let into = required(flags, "into")?;
    let existing = match fs::read_to_string(&into) {
        Ok(text) => published::parse(&text).map_err(|e| format!("{into}: {e}"))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(Failed(format!("read {into}: {e}"))),
    };
    let new = published::lines(&scorecard);
    let merged = published::merge(existing, new.clone());
    fs::write(&into, published::render(&merged)).map_err(|e| format!("write {into}: {e}"))?;
    let _ = writeln!(stdout, "published {} components into {into}", new.len());

    Ok(EXIT_OK)
}

#[cfg(test)]
mod tests;
