//! The review outcome, decided from the runs alone: a pure function of
//! the plan, each agent's run, each specialist's report as the run read
//! it from the specialist's log, and the requirement ids the reports may
//! cite. Every agent that ran is consulted, whatever result its run
//! ended with. It approves exactly when every dispatched agent ran,
//! every specialist read its whole diff and stated no line that does
//! not read, every report takes the form the gate takes, no
//! generalist's weighing is partial and none kept a blocking finding. A
//! specialist carries no severity: its findings reach the verdict only
//! as its generalist weighs them, and a finding a generalist keeps from
//! a specialist's requirement request stays a nit. An agent that did
//! not run, each range of a diff a specialist did not read, each line of
//! a report that does not read, a report the gate would refuse, and a
//! generalist's finding line the review cannot read are each named in a
//! blocking finding of its own, so the review is decided all the same
//! and the author sees why it was not approved; each line a specialist
//! set aside as no finding is counted. The gate still refuses to post a
//! review a dispatched agent did not run in or whose report it refuses
//! (ENG-28), as its consulted list and reports show it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use review_eval::component::Kind;
use review_eval::report;
use review_gate::report::Line;
use review_gate::{
    APPROVE_VERDICT, BLOCKING, Finding, NIT, REQUEST_CHANGES_VERDICT, ReviewOutcome, report_check,
};

use crate::gaps::{self, Gap};
use crate::plan::{Plan, report_file};
use crate::reading::ReportLine;
use crate::run::AgentRun;

/// Where a finding about the review itself points: the review's own
/// directory, not a file of the change.
pub const REVIEW_PATH: &str = "review/";

/// What the review decided, and what the gate holds it to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub outcome: ReviewOutcome,
    /// The agents that ran, whose run log the review read.
    pub consulted: BTreeSet<String>,
    /// Each of their final answers, by name: a specialist's, the
    /// outcome line its report ends with.
    pub answers: BTreeMap<String, String>,
    /// Each generalist's weighing, answered or not, by name: what the
    /// review kept its findings from.
    pub weighings: BTreeMap<String, String>,
    /// Every dispatched agent, or part of one, the review did not
    /// review whole, by name: never clean.
    pub not_reviewed: Vec<Gap>,
    /// Every line of a specialist's report set aside as no finding.
    pub ignored: Vec<ReportLine>,
}

/// Decides the review of `plan` from `runs` and `reports`, each
/// specialist's assembled report by name, when the run wrote one,
/// holding each linked finding to the ids in `known`.
#[must_use]
pub fn decide(
    plan: &Plan,
    runs: &[AgentRun],
    reports: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
) -> Decision {
    let mut findings = Vec::new();
    let (answers, silent) = answers(plan, runs, reports, &mut findings);
    let (partial, unreadable) = read_partly(plan, runs, &mut findings);
    let refused = refused_reports(plan, &answers, reports, known, &mut findings);
    let weighings = weighings(plan, runs, &answers);
    if plan.dispatch.is_empty() {
        findings.push(blocking(
            REVIEW_PATH,
            "the review bundle dispatched no agent, so no reviewer read this change".to_owned(),
        ));
    }
    let Weighed {
        kept: kept_counts,
        closings,
        unread,
    } = weighed(plan, &weighings, (&answers, known), reports, &mut findings);
    let consulted: BTreeSet<String> = answers.keys().cloned().collect();
    let mut not_reviewed = not_reviewed(runs, &silent, &refused, &unread);
    not_reviewed.extend(partial.iter().chain(&unreadable).cloned());
    not_reviewed.sort_by(|a, b| (&a.name, a.part).cmp(&(&b.name, b.part)));
    let ignored: Vec<ReportLine> = of_specialists(plan, runs)
        .flat_map(|run| run.ignored.iter().cloned())
        .collect();
    let approves = silent.is_empty()
        && refused.is_empty()
        && unread.is_empty()
        && partial.is_empty()
        && unreadable.is_empty()
        && !plan.dispatch.is_empty()
        && !findings.iter().any(|f| f.severity == BLOCKING);
    let counts = Counts {
        plan,
        consulted: &consulted,
        kept: kept_counts,
        cut: specialist_runs(plan, runs)
            .filter(|run| {
                run.cut.is_some() && run.partial.is_empty() && run.stated_nothing.is_empty()
            })
            .count(),
        cut_generalists: runs
            .iter()
            .filter(|run| {
                run.answered() && run.cut.is_some() && plan.generalists().contains(&run.name)
            })
            .count(),
        ignored: ignored.len(),
    };

    Decision {
        outcome: ReviewOutcome {
            verdict: if approves {
                APPROVE_VERDICT
            } else {
                REQUEST_CHANGES_VERDICT
            }
            .to_owned(),
            summary: summary(
                &counts,
                &Named {
                    silent: &silent,
                    partial: &labels(&partial),
                    unreadable: &labels(&unreadable),
                    refused: &labelled(&refused),
                    unread: &unread.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
                    not_reviewed: &labels(&not_reviewed),
                },
                &closings,
            ),
            findings,
        },
        consulted,
        answers,
        weighings,
        not_reviewed,
        ignored,
    }
}

/// The runs of `plan`'s specialists in `runs` that ran.
fn specialist_runs<'a>(plan: &Plan, runs: &'a [AgentRun]) -> impl Iterator<Item = &'a AgentRun> {
    of_specialists(plan, runs).filter(|run| run.answered())
}

/// The runs of `plan`'s specialists in `runs`, whether they ran or not:
/// a specialist run in parts that did not run, a part of it having not
/// run, still holds what its parts that ran read and stated.
fn of_specialists<'a>(plan: &Plan, runs: &'a [AgentRun]) -> impl Iterator<Item = &'a AgentRun> {
    let specialists = plan.specialists();
    runs.iter()
        .filter(move |run| specialists.contains(&run.name))
}

/// The gaps of the specialists in `runs`, or their parts, that ran but
/// did not read every line of their diff, and those whose report holds
/// a line that does not read, by part, each named in a blocking finding
/// added to `findings`: what they did not read, with why the run ended
/// when it was cut off, and each line quoted with its problem. A part's
/// gap stands beside another part that did not run, so a resume runs it
/// again rather than keep it. Their other findings stay in their report
/// for the generalist.
fn read_partly(
    plan: &Plan,
    runs: &[AgentRun],
    findings: &mut Vec<Finding>,
) -> (Vec<Gap>, Vec<Gap>) {
    let (mut partial, mut unreadable) = (Vec::new(), Vec::new());
    for run in of_specialists(plan, runs) {
        let cut = run
            .cut
            .as_deref()
            .map_or_else(String::new, |why| format!(" ({why})"));
        for (part, why) in &run.partial {
            let gap = Gap::new(&run.name, *part, gaps::partial(why));
            findings.push(blocking(
                REVIEW_PATH,
                format!(
                    "{} did not read its whole diff{cut}: {why}, so the review of it is partial",
                    gap.label()
                ),
            ));
            partial.push(gap);
        }
        for (part, ended) in &run.stated_nothing {
            let why = gaps::nothing_stated(ended);
            let gap = Gap::new(&run.name, *part, gaps::stated_nothing(ended));
            findings.push(blocking(
                REVIEW_PATH,
                format!("{} {why}, so the review of it is partial", gap.label()),
            ));
            partial.push(gap);
        }
        for line in run.unreadable.iter().take(MOST_UNREADABLE) {
            findings.push(blocking(
                &report_file(&run.name),
                format!("{} {}: `{}`", run.name, line.why, gaps::quoted(&line.text)),
            ));
            unreadable.push(Gap::new(&run.name, line.part, gaps::unreadable(line)));
        }
        let more = run.unreadable.len().saturating_sub(MOST_UNREADABLE);
        if more > 0 {
            let why = format!("and {} that do not read", counted(more, "more line"));
            findings.push(blocking(
                &report_file(&run.name),
                format!("{} {why}", run.name),
            ));
            unreadable.push(Gap::new(
                &run.name,
                None,
                format!("{} {why}", gaps::UNREADABLE),
            ));
        }
    }

    (partial, unreadable)
}

/// The most lines of one agent's report that do not read the outcome
/// names one by one, each in a blocking finding and a gap; the rest are
/// counted in one more of each, so no report floods the posted review.
pub const MOST_UNREADABLE: usize = 10;

/// Each generalist's weighing, by name: an answered one's answer, as
/// [`answers`] assembled it; one cut off before its answer, what it
/// stated, so nothing it kept is lost.
fn weighings(
    plan: &Plan,
    runs: &[AgentRun],
    answers: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    plan.generalists()
        .into_iter()
        .filter_map(|name| {
            let weighing = match answers.get(&name) {
                Some(answer) => answer.clone(),
                None => {
                    let run = runs.iter().find(|r| r.name == name)?;
                    report::weighing(&run.log.texts, "")
                }
            };
            Some((name, weighing))
        })
        .collect()
}

/// The labels of `gaps`, each once, in their order: the summary names
/// an agent or part once however many of its lines do not read.
fn labels(gaps: &[Gap]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for label in gaps.iter().map(Gap::label) {
        if !out.contains(&label) {
            out.push(label);
        }
    }
    out
}

/// Each refused specialist as the summary names it, with its problem.
fn labelled(refused: &[(String, String)]) -> Vec<String> {
    refused
        .iter()
        .map(|(name, problem)| format!("{name} ({problem})"))
        .collect()
}

/// The gaps: each `silent` agent, a specialist run in parts by each
/// part of its run in `runs` that did not answer, each `refused`
/// specialist and each `unread` generalist, with why, sorted.
fn not_reviewed(
    runs: &[AgentRun],
    silent: &[String],
    refused: &[(String, String)],
    unread: &[(String, String)],
) -> Vec<Gap> {
    let mut out = Vec::new();
    for name in silent {
        match runs.iter().find(|r| &r.name == name) {
            Some(run) if !run.failed_parts.is_empty() => {
                out.extend(
                    run.failed_parts
                        .iter()
                        .map(|(part, why)| Gap::new(name, Some(*part), gaps::silent(why))),
                );
            }
            run => {
                let why = run
                    .and_then(|r| r.failure.as_deref())
                    .unwrap_or("it was not run");
                out.push(Gap::new(name, None, gaps::silent(why)));
            }
        }
    }
    for (name, problem) in refused {
        out.push(Gap::new(name, None, format!("refused: {problem}")));
    }
    for (name, why) in unread {
        out.push(Gap::new(name, None, format!("unread: {why}")));
    }
    out.sort_by(|a, b| (&a.name, a.part).cmp(&(&b.name, b.part)));

    out
}

/// Each dispatched agent's answer, when it ran, a specialist's the
/// outcome line its report in `reports` ends with, and the agents that
/// did not run, each with a blocking finding added to `findings`.
fn answers(
    plan: &Plan,
    runs: &[AgentRun],
    reports: &BTreeMap<String, String>,
    findings: &mut Vec<Finding>,
) -> (BTreeMap<String, String>, Vec<String>) {
    let mut answers = BTreeMap::new();
    let mut silent = Vec::new();
    for name in plan.agents() {
        match runs.iter().find(|r| r.name == name) {
            Some(run) if run.answered() => {
                let text = match (run.kind, reports.get(&name)) {
                    (Kind::Generalist, _) => report::weighing(&run.log.texts, &run.log.answer),
                    (_, Some(report)) => crate::reading::answer_of(report),
                    _ => run.log.answer.clone(),
                };
                answers.insert(name, text);
            }
            other => {
                let why = other
                    .and_then(|r| r.failure.clone())
                    .unwrap_or_else(|| "it was not run".to_owned());
                findings.push(blocking(
                    REVIEW_PATH,
                    format!("{name} did not run ({why}), so the review is incomplete"),
                ));
                silent.push(name);
            }
        }
    }

    (answers, silent)
}

/// The specialists that ran whose report the gate refuses, or whose
/// linked finding cites an id outside `known`, each with its problem,
/// and a blocking finding on its report file added to `findings`, the
/// refused line quoted. A report read from its log never is: this holds
/// a report that could not be written, or one kept from an earlier run,
/// to the gate's rule.
fn refused_reports(
    plan: &Plan,
    answers: &BTreeMap<String, String>,
    reports: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) -> Vec<(String, String)> {
    let mut refused = Vec::new();
    for name in plan.specialists() {
        let Some(answer) = answers.get(&name) else {
            continue;
        };
        let report = reports.get(&name).map(String::as_str);
        if let Some(problem) = report_check(report, Some(answer), Some(known)) {
            // The finding quotes the line refused, so the author sees it.
            let quoted = report
                .and_then(|text| crate::retry::refusal(text, answer, known))
                .unwrap_or_else(|| problem.to_string());
            findings.push(blocking(&report_file(&name), format!("{name} {quoted}")));
            refused.push((name, problem.to_string()));
        }
    }

    refused
}

/// What weighing the generalists' answers gave.
struct Weighed {
    /// The kept findings: blocking, then nits.
    kept: (usize, usize),
    /// Each generalist's closing sentence, named.
    closings: Vec<String>,
    /// The generalists whose answer holds no finding, no dropped one and
    /// no `none —` line, or a finding line it cannot read, with why.
    unread: Vec<(String, String)>,
}

/// Adds each answering generalist's kept findings to `findings`, each
/// once, counts them, and names each generalist whose answer says
/// neither a finding nor that it has none, or holds a finding line it
/// cannot read, with a blocking finding of its own: silence is no clean
/// pass, and a finding the review cannot read is never dropped. A blocking finding kept from a
/// specialist's requirement request, in `reports`, is kept as a nit.
fn weighed(
    plan: &Plan,
    weighings: &BTreeMap<String, String>,
    (answers, known): (&BTreeMap<String, String>, &BTreeSet<String>),
    reports: &BTreeMap<String, String>,
    findings: &mut Vec<Finding>,
) -> Weighed {
    let mut counts = (0, 0);
    let mut closings = Vec::new();
    let mut unread = Vec::new();
    for name in plan.generalists() {
        let answered = answers.contains_key(&name);
        // A run cut off before its answer still has what it stated, and
        // that is weighed as it stands: the run is named as one that did
        // not answer, and nothing it kept is lost.
        let Some(answer) = weighings.get(&name) else {
            continue;
        };
        let heard = Heard {
            reports,
            answers,
            known,
        };
        unread.extend(unread_by(plan, &name, answer, answered, heard, findings));
        for mut f in kept(answer) {
            if f.severity == BLOCKING && from_request(plan, &name, reports, &f) {
                f.severity = NIT.to_owned();
                f.body = format!("{} (kept from a requirement request, so a nit)", f.body);
            }
            if findings.contains(&f) {
                continue;
            }
            if f.severity == BLOCKING {
                counts.0 += 1;
            } else {
                counts.1 += 1;
            }
            findings.push(f);
        }
        if let Some(sentence) = closing(answer) {
            closings.push(format!("{name}: {sentence}"));
        }
    }

    Weighed {
        kept: counts,
        closings,
        unread,
    }
}

/// Why the review cannot take generalist `name`'s `answer` as a whole
/// weighing, each with a blocking finding added to `findings`: it says
/// nothing, it weighs none of the specialists' findings, or holds a
/// finding line it cannot read. A run that did not answer is named
/// already, and gives none of these.
fn unread_by(
    plan: &Plan,
    name: &str,
    answer: &str,
    answered: bool,
    heard: Heard,
    findings: &mut Vec<Finding>,
) -> Vec<(String, String)> {
    let mut unread = Vec::new();
    if !answered {
        return unread;
    }
    if unreadable(answer) == Some(SAYS_NOTHING) {
        findings.push(blocking(
            REVIEW_PATH,
            format!(
                "{name} answered no finding, no dropped one and no `none —` line, so its \
                 weighing cannot be read"
            ),
        ));
        unread.push((name.to_owned(), SAYS_NOTHING.to_owned()));
    } else if unreadable(answer).is_none() {
        let missed = unweighed(plan, name, answer, heard);
        if !missed.is_empty() {
            findings.push(blocking(
                REVIEW_PATH,
                format!(
                    "{name} weighed no finding at {}, which its specialists reported, so its \
                     weighing is partial",
                    missed.join(", ")
                ),
            ));
            unread.push((
                name.to_owned(),
                format!("it weighed no finding at {}", missed.join(", ")),
            ));
        }
    }
    for line in answer.lines().filter(|l| report::unread(l)) {
        findings.push(blocking(
            REVIEW_PATH,
            format!(
                "{name} answered a finding line the review cannot read, so its weighing \
                 cannot be read: {}",
                line.trim()
            ),
        ));
        if !unread.iter().any(|(n, _)| n == name) {
            unread.push((name.to_owned(), UNREAD_LINE.to_owned()));
        }
    }

    unread
}

/// What the specialists gave: the answers and the reports, and the ids
/// a report may cite.
#[derive(Clone, Copy)]
struct Heard<'a> {
    reports: &'a BTreeMap<String, String>,
    answers: &'a BTreeMap<String, String>,
    known: &'a BTreeSet<String>,
}

/// The places of `generalist`'s specialists' findings and requests that
/// its `answer` weighs nowhere: no finding or dropped line of it is at
/// the place, or covers it as a range of at most [`MAX_RANGE`] lines,
/// and none names it exactly. Decided from the reports and the answer
/// alone, never by what the generalist says of its own scope; empty
/// when it weighed every one, sorted.
fn unweighed(plan: &Plan, generalist: &str, answer: &str, heard: Heard) -> Vec<String> {
    let Heard {
        reports,
        answers,
        known,
    } = heard;

    missed(held_to(plan, generalist, answers, reports, known), answer)
}

/// The reports `generalist`'s weighing is held to: of each of its
/// specialists that answered, `answers` by name, its report in
/// `reports`, when the gate takes it beside that answer, its ids held
/// to `known`. A report the gate refuses is named as refused, not as
/// unweighed. The one rule the outcome and the answer store hold a
/// weighing to.
#[must_use]
pub fn held_to<'a>(
    plan: &Plan,
    generalist: &str,
    answers: &BTreeMap<String, String>,
    reports: &'a BTreeMap<String, String>,
    known: &BTreeSet<String>,
) -> Vec<&'a str> {
    plan.dispatch
        .get(generalist)
        .into_iter()
        .flatten()
        .filter_map(|s| {
            let (answer, report) = (answers.get(s)?, reports.get(s)?);
            report_check(Some(report), Some(answer), Some(known))
                .is_none()
                .then_some(report.as_str())
        })
        .collect()
}

/// The places of the findings and requests in `reports` that the
/// weighing `answer` weighs nowhere, sorted and each once; empty when it
/// weighs every one. The answer store holds a generalist to the same
/// rule, so a partial weighing is neither stored nor replayed.
#[must_use]
pub fn missed<'a>(reports: impl IntoIterator<Item = &'a str>, answer: &str) -> Vec<String> {
    let weighing: Vec<&str> = answer
        .lines()
        .filter(|l| report::finding(l).is_some() || report::dropped(l))
        .collect();
    let reports: Vec<&str> = reports.into_iter().collect();
    let known = reported_paths(&reports);
    let mut missed = BTreeSet::new();
    for report in reports {
        for line in report.lines().map(review_gate::report::line) {
            let (path, number) = match &line {
                Line::Linked(k) => (&k.path, k.line),
                Line::Request(r) => (&r.path, r.line),
                _ => continue,
            };
            let at = (path.clone(), number);
            let weighed = weighing.iter().any(|l| {
                report::place_of(l) == Some(at.clone()) || named_by(l, path, number, &known)
            });
            if !weighed {
                missed.insert(format!("{path}:{number}"));
            }
        }
    }

    missed.into_iter().collect()
}

/// The paths the findings and requests in `reports` name, each once.
fn reported_paths(reports: &[&str]) -> BTreeSet<String> {
    reports
        .iter()
        .flat_map(|r| r.lines().map(review_gate::report::line))
        .filter_map(|line| match line {
            Line::Linked(k) => Some(k.path),
            Line::Request(r) => Some(r.path),
            _ => None,
        })
        .collect()
}

/// Whether `a` is `b`, or `b` with directories before it.
fn tail_of(a: &str, b: &str) -> bool {
    a == b || a.strip_suffix(b).is_some_and(|d| d.ends_with('/'))
}

/// Whether `line` names the place `path:number`, spelled as `path`, a
/// tail of it at a `/` that no other path of `known` ends in, or, for a
/// `path` that is a bare file name, a path that ends in it when no other
/// path of `known` is a tail of it or has it as one: an agent that
/// shortens or lengthens a path still weighs the finding, while an
/// ambiguous basename names none. A reported path with directories is
/// not lengthened, since the longer path may be another file.
fn named_by(line: &str, path: &str, number: usize, known: &BTreeSet<String>) -> bool {
    if names(line, path, number) {
        return true;
    }
    let others = || known.iter().filter(|q| q.as_str() != path);
    let shorter = path
        .match_indices('/')
        .map(|(i, _)| &path[i + 1..])
        .filter(|t| !t.is_empty() && !others().any(|q| tail_of(q, t)))
        .any(|t| names(line, t, number));
    if shorter {
        return true;
    }
    let alone = !path.contains('/') && !others().any(|q| tail_of(q, path) || tail_of(path, q));
    let head = format!("/{path}:");

    alone
        && line.match_indices(&head).any(|(at, _)| {
            let start = line[..at]
                .rfind(|c: char| !(c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-')))
                .map_or(0, |i| {
                    i + line[i..].chars().next().map_or(1, char::len_utf8)
                });
            names(line, &line[start..at + 1 + path.len()], number)
        })
}

/// The places of the findings and requests in `reports`, each once, in
/// the form a weighing line names them, `path:line`, sorted: what a
/// generalist is told it must answer before it starts, the same places
/// [`missed`] holds its weighing to.
#[must_use]
pub fn places<'a>(reports: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut out = BTreeSet::new();
    for report in reports {
        for line in report.lines().map(review_gate::report::line) {
            match &line {
                Line::Linked(k) => out.insert(format!("{}:{}", k.path, k.line)),
                Line::Request(r) => out.insert(format!("{}:{}", r.path, r.line)),
                _ => continue,
            };
        }
    }

    out.into_iter().collect()
}

/// The note a generalist's prompt ends with for the `places` its
/// specialists reported, none when they reported none: each must be
/// answered by a line at it, or at a range holding it of at most
/// [`MAX_RANGE`] lines, however alike its findings.
#[must_use]
pub fn answerable(places: &[String]) -> Option<String> {
    if places.is_empty() {
        return None;
    }
    let list = places
        .iter()
        .map(|p| format!("`{p}`"))
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!(
        "Your specialists reported at these places: {list}. The review counts a place weighed \
         only when a finding or dropped line of yours is at it, or at a range holding it of \
         at most {MAX_RANGE} lines, such as `path:29-30`, so answer every one, a duplicate \
         or adjacent finding too."
    ))
}

/// The most lines a range a weighing line names, `start-end`, may span
/// and cover each of: a wider one names its first line alone, as a
/// finding at it is read, so no one line can cover a whole file.
pub const MAX_RANGE: usize = 20;

/// Whether `line` names the place `path:number`: `path:` where no path
/// character comes before it, then the number or a range,
/// `start-end`, holding it, of at most [`MAX_RANGE`] lines, and no
/// digit after.
fn names(line: &str, path: &str, number: usize) -> bool {
    let pathy = |c: char| c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-');
    let head = format!("{path}:");
    line.match_indices(&head).any(|(at, _)| {
        if line[..at].chars().next_back().is_some_and(pathy) {
            return false;
        }
        let rest = &line[at + head.len()..];
        let (start, rest) = digits(rest);
        let Some(start) = start else {
            return false;
        };
        let end = rest
            .strip_prefix(['-', '–'])
            .and_then(|r| digits(r).0)
            .filter(|end| end.saturating_sub(start) < MAX_RANGE)
            .unwrap_or(start);
        (start..=end).contains(&number)
    })
}

/// The number `text` starts with, and the text after it.
fn digits(text: &str) -> (Option<usize>, &str) {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    (text[..end].parse().ok(), &text[end..])
}

/// Whether `f`, a finding `generalist` kept, came from a requirement
/// request: its source, the last part of its line, names one of the
/// generalist's specialists, whose report asks for a requirement at
/// the same path and line and links no finding there. A generalist
/// that finds a blocking defect at that line itself names itself.
fn from_request(
    plan: &Plan,
    generalist: &str,
    reports: &BTreeMap<String, String>,
    f: &Finding,
) -> bool {
    let source = f
        .body
        .rsplit(" — ")
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches(['`', '*', '_', '.'])
        .to_lowercase();
    let Some(report) = plan
        .dispatch
        .get(generalist)
        .filter(|specialists| specialists.contains(&source))
        .and_then(|_| reports.get(&source))
    else {
        return false;
    };
    let at = |path: &str, line: usize| path == f.path && i64::try_from(line).ok() == Some(f.line);
    let lines: Vec<Line> = report.lines().map(review_gate::report::line).collect();
    lines
        .iter()
        .any(|l| matches!(l, Line::Request(r) if at(&r.path, r.line)))
        && !lines
            .iter()
            .any(|l| matches!(l, Line::Linked(k) if at(&k.path, k.line)))
}

/// A blocking finding on the whole of `path`.
fn blocking(path: &str, body: String) -> Finding {
    Finding {
        path: path.to_owned(),
        line: 0,
        severity: BLOCKING.to_owned(),
        body,
    }
}

/// Why the review cannot read a generalist's `answer`: it says no
/// finding, no dropped one and no `none —` line, or holds a finding line
/// the review cannot read; none when it reads.
#[must_use]
pub fn unreadable(answer: &str) -> Option<&'static str> {
    let says = answer
        .lines()
        .any(|l| report::finding(l).is_some() || report::dropped(l) || report::nothing(l));
    if !says {
        return Some(SAYS_NOTHING);
    }
    answer.lines().any(report::unread).then_some(UNREAD_LINE)
}

/// Why a generalist's answer that says nothing cannot be read.
const SAYS_NOTHING: &str = "it answered no finding, no dropped one and no `none —` line";
/// Why one holding a finding line the review cannot read cannot.
const UNREAD_LINE: &str = "it answered a finding line the review cannot read";

/// The findings a generalist's `answer` keeps: each line that reads as
/// a finding and is no dropped one, `nit` or `blocking` as it says. A
/// severity that is neither is read as blocking, the safe side, with
/// the word it gave kept in its body.
#[must_use]
pub fn kept(answer: &str) -> Vec<Finding> {
    answer
        .lines()
        .filter(|l| !report::dropped(l))
        .filter_map(report::finding)
        .map(|f| {
            let text = if f.text.is_empty() { f.raw } else { f.text };
            let (severity, body) = match f.severity.as_str() {
                BLOCKING | NIT => (f.severity.clone(), text),
                other => (BLOCKING.to_owned(), format!("{text} (reported as {other})")),
            };
            Finding {
                path: f.path,
                line: i64::try_from(f.line).unwrap_or(i64::MAX),
                severity,
                body,
            }
        })
        .collect()
}

/// A generalist's closing sentence on its scope: the last line of its
/// `answer` that is neither blank, a heading, a finding nor a dropped
/// one.
#[must_use]
pub fn closing(answer: &str) -> Option<String> {
    answer
        .lines()
        .map(str::trim)
        .rfind(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !report::dropped(l)
                && !report::nothing(l)
                && report::finding(l).is_none()
        })
        .map(str::to_owned)
}

/// What the summary counts.
struct Counts<'a> {
    plan: &'a Plan,
    consulted: &'a BTreeSet<String>,
    /// The generalists' kept findings: blocking, then nits.
    kept: (usize, usize),
    /// The specialists cut off that read their whole diff.
    cut: usize,
    /// The generalists cut off, whose weighing is what their log states.
    cut_generalists: usize,
    /// The lines set aside as no finding.
    ignored: usize,
}

/// The agents the summary names for what went wrong.
struct Named<'a> {
    /// The agents that did not run.
    silent: &'a [String],
    /// The specialists, or parts, that did not read their whole diff.
    partial: &'a [String],
    /// The specialists, or parts, whose report holds a line that does
    /// not read.
    unreadable: &'a [String],
    /// The specialists whose report the gate refuses.
    refused: &'a [String],
    /// The generalists whose answer the review cannot read.
    unread: &'a [String],
    /// Every agent or part not reviewed whole.
    not_reviewed: &'a [String],
}

/// The summary: who ran of whom, what the generalists kept, how many
/// specialists were cut off with their whole diff read, how many
/// generalists were cut off, how many lines
/// were set aside, who did not run, who read part of their diff, whose
/// report holds a line that does not read or the gate refuses, whose
/// answer the review cannot read, who was consulted, and each
/// generalist's closing sentence.
fn summary(counts: &Counts, named: &Named, closings: &[String]) -> String {
    let plan = counts.plan;
    let answered_of = |names: &[String]| {
        names
            .iter()
            .filter(|n| counts.consulted.contains(*n))
            .count()
    };
    let (specialists, generalists) = (plan.specialists(), plan.generalists());
    let mut s = if plan.dispatch.is_empty() {
        "The review bundle dispatched no agent.".to_owned()
    } else {
        format!(
            "{} of {} dispatched agents ran and their logs were read: {} of {} {} and {} of {} \
             {}. The generalists kept {} blocking and {} nit findings.",
            counts.consulted.len(),
            plan.agents().len(),
            answered_of(&specialists),
            specialists.len(),
            plural(Kind::Specialist, specialists.len()),
            answered_of(&generalists),
            generalists.len(),
            plural(Kind::Generalist, generalists.len()),
            counts.kept.0,
            counts.kept.1,
        )
    };
    if counts.cut > 0 {
        let _ = write!(
            s,
            " {} ended without a successful result, cut off by a turn budget, deadline or \
             cost cap, with every line of their diff read: what each one's log states is its \
             report.",
            counted(counts.cut, "specialist")
        );
    }
    if counts.cut_generalists > 0 {
        let _ = write!(
            s,
            " {} ended without a successful result, cut off by a turn budget, deadline or \
             cost cap: what each one's log states is its weighing.",
            counted(counts.cut_generalists, "generalist")
        );
    }
    if counts.ignored > 0 {
        let _ = write!(
            s,
            " Set aside as no finding: {} of the specialists' reports ({}).",
            counted(counts.ignored, "line"),
            crate::reading::IGNORED
        );
    }
    for (label, names) in [
        ("Did not run", named.silent),
        ("Read part of their diff", named.partial),
        (
            "Reports holding a line that does not read",
            named.unreadable,
        ),
        ("Reports the gate refuses", named.refused),
        ("Answers the review cannot read", named.unread),
        ("Not reviewed", named.not_reviewed),
    ] {
        if !names.is_empty() {
            let _ = write!(s, " {label}: {}.", names.join(", "));
        }
    }
    if !counts.consulted.is_empty() {
        let names: Vec<&str> = counts.consulted.iter().map(String::as_str).collect();
        let _ = write!(s, " Consulted: {}.", names.join(", "));
    }
    for line in closings {
        let _ = write!(s, " {line}");
    }

    s
}

/// `n` `word`s, said: `1 line`, `2 lines`.
fn counted(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// The word for `n` agents of `kind`.
fn plural(kind: Kind, n: usize) -> String {
    let word = kind.word();
    if n == 1 {
        word.to_owned()
    } else {
        format!("{word}s")
    }
}

#[cfg(test)]
mod tests;
