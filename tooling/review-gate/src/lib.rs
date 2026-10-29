//! The review gate between a reviewing agent and the approval it may
//! earn (ENG-21, ENG-28). The agent only writes a structured review
//! outcome; this crate, run by the workflow step that alone holds the
//! reviewer app's key, decides what review to post. It approves only an
//! approving outcome with no blocking finding, on the head the agent
//! read, once CI passed there. The deterministic checks the review
//! workflow runs before the agent add their own findings, and a
//! blocking one withholds approval whatever the agent says. It is
//! repository tooling and never links into a shipped executable.

pub mod cli;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

use serde::{Deserialize, Deserializer, Serialize};

/// The `verdict` that asks for approval.
pub const APPROVE_VERDICT: &str = "approve";
/// The `verdict` that asks for changes.
pub const REQUEST_CHANGES_VERDICT: &str = "request_changes";

/// A finding that withholds approval whatever the verdict says.
pub const BLOCKING: &str = "blocking";
/// A finding that is reported but withholds nothing.
pub const NIT: &str = "nit";

/// The review event that approves, as GitHub names it.
pub const APPROVE: &str = "APPROVE";
/// The review event that requests changes, as GitHub names it.
pub const REQUEST_CHANGES: &str = "REQUEST_CHANGES";
/// The check the post job posts with the gate's decision (ENG-28),
/// which main's ruleset requires from the reviewer app alone (ENG-21).
pub const OWN_CHECK: &str = "review-gate";
/// The reviewer app, `jeduden-review-agent`, by its GitHub App id.
pub const REVIEWER_APP_ID: i64 = 5_140_085;

/// The check the branch ruleset requires: the job that passes only when
/// every CI job passed.
const CI_CHECK: &str = "CI";

/// The reviewing agent's structured output.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ReviewOutcome {
    pub verdict: String,
    pub summary: String,
    #[serde(deserialize_with = "null_as_default")]
    pub findings: Vec<Finding>,
}

/// One problem the agent found, at a path and line of the reviewed
/// tree; line 0 means the whole file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Finding {
    pub path: String,
    pub line: i64,
    pub severity: String,
    pub body: String,
}

/// One check run on the reviewed head, as GitHub reports it. A commit
/// can carry several runs of one check, when CI ran on it twice;
/// `started_at`, an RFC 3339 UTC time, orders them, and `id`, which
/// GitHub assigns in creation order, breaks a tie within one second.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CheckRun {
    pub id: i64,
    #[serde(deserialize_with = "null_as_default")]
    pub name: String,
    #[serde(deserialize_with = "null_as_default")]
    pub status: String,
    #[serde(deserialize_with = "null_as_default")]
    pub conclusion: String,
    #[serde(deserialize_with = "null_as_default")]
    pub started_at: String,
    /// The GitHub App that posted it, by id; 0 when unknown.
    #[serde(deserialize_with = "null_as_default")]
    pub app: i64,
}

/// Everything the gate decides from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Input {
    pub outcome: ReviewOutcome,
    /// The head commit the agent read.
    pub reviewed: String,
    /// The pull request's head commit now.
    pub head: String,
    pub check_runs: Vec<CheckRun>,
    /// The findings of the deterministic checks the workflow ran on the
    /// reviewed head, beside the agent's.
    pub checked: Vec<Finding>,
    /// Every agent the review bundle dispatched: the generalists and
    /// their specialists.
    pub dispatched: BTreeSet<String>,
    /// Every agent the orchestrator called, as its run log shows.
    pub consulted: BTreeSet<String>,
    /// The specialists the review bundle dispatched, whose reports the
    /// generalists read.
    pub specialists: BTreeSet<String>,
    /// Each specialist's report file, by name; one with no file is
    /// absent.
    pub reports: BTreeMap<String, String>,
    /// Each agent's final answer, by name, as the orchestrator's run log
    /// shows it.
    pub answers: BTreeMap<String, String>,
}

/// The pull request review to post, in the shape GitHub's
/// create-review endpoint takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub commit_id: String,
    pub event: String,
    pub body: String,
}

/// What the gate refuses.
#[derive(Debug)]
pub enum Error {
    /// The agent's output does not fit the review outcome's schema.
    Outcome(String),
    /// The deterministic checks' findings do not fit their schema.
    Checked(String),
    /// A line of the check runs is not a check run.
    CheckRun {
        line: usize,
        source: serde_json::Error,
    },
    /// The required CI check has not passed on the reviewed head.
    Ci(String),
    /// The review bundle's dispatch does not read.
    Dispatch(serde_json::Error),
    /// The list of agents the orchestrator called does not read.
    Consulted(serde_json::Error),
    /// The list of agents the orchestrator called holds an empty name.
    UnnamedAgent,
    /// The orchestrator left these dispatched agents out.
    Unconsulted(Vec<String>),
    /// The dispatch names an agent whose name is not lowercase letters,
    /// digits and hyphens, so it cannot name a report file.
    AgentName(String),
    /// The agents' answers from the run log do not read.
    Answers(serde_json::Error),
    /// A specialist's report is missing or empty, does not end with its
    /// outcome, or ends with another outcome than it answered.
    Report {
        agent: String,
        problem: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outcome(msg) => write!(f, "review: malformed review outcome: {msg}"),
            Self::Checked(msg) => write!(f, "review: malformed deterministic findings: {msg}"),
            Self::CheckRun { line, source } => write!(f, "review: check run line {line}: {source}"),
            Self::Ci(head) => write!(f, "review: CI has not passed on the reviewed head: {head}"),
            Self::Dispatch(source) => write!(f, "review: malformed dispatch: {source}"),
            Self::Consulted(source) => write!(f, "review: malformed consulted agents: {source}"),
            Self::UnnamedAgent => {
                write!(f, "review: malformed consulted agents: an empty agent name")
            }
            Self::Unconsulted(agents) => write!(
                f,
                "review: the orchestrator did not consult {}",
                agents.join(", ")
            ),
            Self::AgentName(name) => write!(f, "review: malformed dispatch: agent name {name:?}"),
            Self::Answers(source) => write!(f, "review: malformed agents' answers: {source}"),
            Self::Report { agent, problem } => write!(f, "review: {agent} {problem}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CheckRun { source, .. }
            | Self::Dispatch(source)
            | Self::Consulted(source)
            | Self::Answers(source) => Some(source),
            Self::Outcome(_)
            | Self::Checked(_)
            | Self::Ci(_)
            | Self::UnnamedAgent
            | Self::Unconsulted(_)
            | Self::AgentName(_)
            | Self::Report { .. } => None,
        }
    }
}

/// Reads a JSON `null` as the type's default, the way GitHub reports a
/// running check's conclusion.
fn null_as_default<'de, D: Deserializer<'de>, T: Default + Deserialize<'de>>(
    d: D,
) -> Result<T, D::Error> {
    Option::<T>::deserialize(d).map(Option::unwrap_or_default)
}

/// Decodes the agent's output strictly: one object, no unknown field,
/// and every value one the schema names.
///
/// # Errors
///
/// Fails on anything the schema does not allow.
pub fn parse_outcome(data: &str) -> Result<ReviewOutcome, Error> {
    let outcome: ReviewOutcome =
        serde_json::from_str(data).map_err(|e| Error::Outcome(e.to_string()))?;
    validate(&outcome)?;

    Ok(outcome)
}

/// Checks every value in `outcome` is one the schema names.
fn validate(outcome: &ReviewOutcome) -> Result<(), Error> {
    if outcome.verdict != APPROVE_VERDICT && outcome.verdict != REQUEST_CHANGES_VERDICT {
        return Err(Error::Outcome(format!(
            "verdict {:?} is neither {APPROVE_VERDICT} nor {REQUEST_CHANGES_VERDICT}",
            outcome.verdict
        )));
    }
    if outcome.summary.trim().is_empty() {
        return Err(Error::Outcome("no summary".into()));
    }
    validate_findings(&outcome.findings).map_err(Error::Outcome)
}

/// Decodes the deterministic checks' findings: a list of findings in
/// the review outcome's shape.
///
/// # Errors
///
/// Fails on anything that is not such a list.
pub fn parse_checked(data: &str) -> Result<Vec<Finding>, Error> {
    let findings: Vec<Finding> =
        serde_json::from_str(data).map_err(|e| Error::Checked(e.to_string()))?;
    validate_findings(&findings).map_err(Error::Checked)?;

    Ok(findings)
}

/// Checks every finding names a severity, a path, a body and a line.
fn validate_findings(findings: &[Finding]) -> Result<(), String> {
    for (i, f) in findings.iter().enumerate() {
        if f.severity != BLOCKING && f.severity != NIT {
            return Err(format!("finding {i}: severity {:?}", f.severity));
        }
        if f.path.is_empty() || f.body.trim().is_empty() {
            return Err(format!("finding {i} has no path or no body"));
        }
        if f.line < 0 {
            return Err(format!("finding {i}: line {}", f.line));
        }
    }

    Ok(())
}

/// Reads the review bundle's dispatch, each generalist with its
/// specialists, into the set of every agent it names.
///
/// # Errors
///
/// Fails on anything that is not such a map.
pub fn parse_dispatch(data: &str) -> Result<BTreeSet<String>, Error> {
    let dispatch: BTreeMap<String, BTreeSet<String>> =
        serde_json::from_str(data).map_err(Error::Dispatch)?;

    Ok(dispatch
        .into_iter()
        .flat_map(|(generalist, specialists)| std::iter::once(generalist).chain(specialists))
        .collect())
}

/// Reads the specialists the review bundle's dispatch names, each
/// generalist's list, refusing a name that could not name a file.
///
/// # Errors
///
/// Fails on anything that is not such a map, or a name that is not
/// lowercase letters, digits and hyphens.
pub fn parse_specialists(data: &str) -> Result<BTreeSet<String>, Error> {
    let dispatch: BTreeMap<String, BTreeSet<String>> =
        serde_json::from_str(data).map_err(Error::Dispatch)?;
    let names: BTreeSet<String> = dispatch.into_values().flatten().collect();
    if let Some(bad) = names.iter().find(|n| !agent_name(n)) {
        return Err(Error::AgentName(bad.clone()));
    }

    Ok(names)
}

/// Whether `name` is an agent's name: lowercase letters, digits and
/// hyphens, never empty.
fn agent_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Reads each agent's final answer from the run log, a map of name to
/// text.
///
/// # Errors
///
/// Fails on anything that is not such a map.
pub fn parse_answers(data: &str) -> Result<BTreeMap<String, String>, Error> {
    serde_json::from_str(data).map_err(Error::Answers)
}

/// The words a report's outcome line may give.
const OUTCOMES: [&str; 3] = ["sound", "nits", "blocking"];

/// The outcome line `text` ends with, trimmed: its last non-empty line,
/// when that line starts with `outcome:`, in any case, and its first
/// word, markup aside, is `sound`, `nits` or `blocking`.
fn outcome_line(text: &str) -> Option<&str> {
    let line = text.lines().map(str::trim).rfind(|l| !l.is_empty())?;
    let rest = line
        .get(..8)?
        .eq_ignore_ascii_case("outcome:")
        .then(|| &line[8..])?;
    let word = rest
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_ascii_alphanumeric());

    OUTCOMES.contains(&word).then_some(line)
}

/// The problem with `agent`'s report, if any: it has none, it is empty,
/// it does not end with an outcome line, or its run-log answer holds no
/// line that is the same outcome.
fn report_problem(input: &Input, agent: &str) -> Option<&'static str> {
    let report = input.reports.get(agent).map(|r| r.trim());
    let outcome = match report {
        None => return Some("wrote no report"),
        Some("") => return Some("wrote an empty report"),
        Some(text) => outcome_line(text),
    };
    let Some(outcome) = outcome else {
        return Some("has a report that does not end with its outcome");
    };

    (!input
        .answers
        .get(agent)
        .is_some_and(|answer| answer.lines().any(|l| l.trim() == outcome)))
    .then_some("has a report whose outcome differs from its answer in the run log")
}

/// Reads the agents the orchestrator called, a list of names.
///
/// # Errors
///
/// Fails on anything that is not a list of names.
pub fn parse_consulted(data: &str) -> Result<BTreeSet<String>, Error> {
    let names: Vec<String> = serde_json::from_str(data).map_err(Error::Consulted)?;
    if names.iter().any(String::is_empty) {
        return Err(Error::UnnamedAgent);
    }

    Ok(names.into_iter().collect())
}

/// Reads one check run per line, as `gh api --jq` prints them; blank
/// lines are skipped.
///
/// # Errors
///
/// Fails on a line that is not a check run.
pub fn parse_check_runs(data: &str) -> Result<Vec<CheckRun>, Error> {
    data.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            serde_json::from_str(line).map_err(|source| Error::CheckRun {
                line: i + 1,
                source,
            })
        })
        .collect()
}

/// Returns the review to post, or `None` when the pull request moved
/// past the reviewed head: the run for the newer head decides instead.
///
/// # Errors
///
/// A head whose CI has not passed, or a review whose orchestrator left
/// out an agent the review bundle dispatched, is an error, so the step
/// fails loudly rather than posting anything.
pub fn decide(input: &Input) -> Result<Option<Review>, Error> {
    if input.head != input.reviewed {
        return Ok(None);
    }
    let left_out: Vec<String> = input
        .dispatched
        .difference(&input.consulted)
        .cloned()
        .collect();
    if !left_out.is_empty() {
        return Err(Error::Unconsulted(left_out));
    }
    if let Some((agent, problem)) = input
        .specialists
        .iter()
        .find_map(|a| report_problem(input, a).map(|p| (a, p)))
    {
        return Err(Error::Report {
            agent: agent.clone(),
            problem,
        });
    }
    let runs = latest(&input.check_runs);
    if !runs.iter().any(|r| passed_ci(r)) {
        return Err(Error::Ci(input.reviewed.clone()));
    }
    let failed: Vec<&CheckRun> = runs.into_iter().filter(|r| failed(r)).collect();
    let approves = input.outcome.verdict == APPROVE_VERDICT
        && failed.is_empty()
        && !input.outcome.findings.iter().any(is_blocking)
        && !input.checked.iter().any(is_blocking);
    let event = if approves { APPROVE } else { REQUEST_CHANGES };

    Ok(Some(Review {
        commit_id: input.reviewed.clone(),
        event: event.to_owned(),
        body: render(
            &input.outcome,
            &input.checked,
            &failed,
            event,
            &input.reviewed,
        ),
    }))
}

/// Keeps the newest run of each check, in the order the checks first
/// appear. An older run that a newer one replaced, cancelled or failed,
/// says nothing about the head as it stands.
fn latest(runs: &[CheckRun]) -> Vec<&CheckRun> {
    let mut out: Vec<&CheckRun> = Vec::new();
    let mut at: BTreeMap<&str, usize> = BTreeMap::new();
    for r in runs {
        match at.get(r.name.as_str()) {
            None => {
                at.insert(&r.name, out.len());
                out.push(r);
            }
            Some(&i) if newer(r, out[i]) => out[i] = r,
            Some(_) => {}
        }
    }

    out
}

/// Whether run `a` started after run `b`, by start time and then by
/// run id.
fn newer(a: &CheckRun, b: &CheckRun) -> bool {
    (&a.started_at, a.id) > (&b.started_at, b.id)
}

/// Whether `r` is the required CI check, passed.
fn passed_ci(r: &CheckRun) -> bool {
    r.name == CI_CHECK && r.status == "completed" && r.conclusion == "success"
}

/// Whether `f` withholds approval.
fn is_blocking(f: &Finding) -> bool {
    f.severity == BLOCKING
}

/// Whether `r` completed and neither passed nor was skipped. A check
/// still running, this review among them, is not counted: the required
/// CI check has already passed. Nor is the gate's own [`OWN_CHECK`]
/// from an earlier review of this head, which a re-review, after the
/// stakeholder's consent, would otherwise never get past.
fn failed(r: &CheckRun) -> bool {
    let own = r.name == OWN_CHECK && r.app == REVIEWER_APP_ID;
    !own && r.status == "completed"
        && !["success", "neutral", "skipped"].contains(&r.conclusion.as_str())
}

/// Writes the review body. A title names the gate's outcome and the
/// head it covers; a line gives the review agent's verdict and, when
/// the gate overrules it, why. Then come the sections, each under a
/// fixed heading and only when it has content: the review agent's
/// summary and findings (always), the deterministic checks' findings,
/// and the checks that failed. Findings list blocking before nits.
/// Agent text is kept on one line after a label, so it cannot open a
/// section.
fn render(
    outcome: &ReviewOutcome,
    checked: &[Finding],
    failed: &[&CheckRun],
    event: &str,
    head: &str,
) -> String {
    let title = if event == APPROVE {
        "Approved"
    } else {
        "Changes requested"
    };
    let short = head.get(..12).unwrap_or(head);
    let verdict = if outcome.verdict == APPROVE_VERDICT {
        "approve"
    } else {
        "request changes"
    };
    let mut b = format!(
        "**{title}** by the review gate on `{short}`.\n\n\
         The review agent's verdict: {verdict}."
    );
    if let Some(reason) = overruled(outcome, checked, failed, event) {
        let _ = write!(b, " The gate requests changes: {reason}.");
    }
    let _ = write!(
        b,
        "\n\n### Review agent\n\nSummary: {}\n",
        defuse_mentions(&one_line(&outcome.summary))
    );
    list_findings(&mut b, &outcome.findings);
    if !checked.is_empty() {
        b.push_str("\n### Deterministic checks (review-bundle)\n");
        list_findings(&mut b, checked);
    }
    if !failed.is_empty() {
        b.push_str("\n### Failed checks\n\n");
        for r in failed {
            let _ = writeln!(b, "- check `{}`: {}", code_span(&r.name), r.conclusion);
        }
    }

    b
}

/// Why the gate requests changes on an approving verdict: the first of
/// a blocking deterministic finding, a failed check and a blocking
/// finding of the agent's. `None` when the gate follows the verdict.
fn overruled(
    outcome: &ReviewOutcome,
    checked: &[Finding],
    failed: &[&CheckRun],
    event: &str,
) -> Option<&'static str> {
    let reasons = [
        (
            checked.iter().any(is_blocking),
            "a deterministic check found a blocking finding",
        ),
        (!failed.is_empty(), "a check failed"),
        (
            outcome.findings.iter().any(is_blocking),
            "the agent reported a blocking finding",
        ),
    ];
    (event != APPROVE && outcome.verdict == APPROVE_VERDICT)
        .then(|| reasons.into_iter().find(|(holds, _)| *holds))
        .flatten()
        .map(|(_, reason)| reason)
}

/// Appends `findings` to `b` as a list after a blank line, one line
/// each, blocking ones first; nothing when there are none.
fn list_findings(b: &mut String, findings: &[Finding]) {
    if findings.is_empty() {
        return;
    }
    b.push('\n');
    for severity in [BLOCKING, NIT] {
        for f in findings.iter().filter(|f| f.severity == severity) {
            let _ = writeln!(
                b,
                "- {severity} `{}`: {}",
                code_span(&location(f)),
                defuse_mentions(&one_line(&f.body))
            );
        }
    }
}

/// `s` with every run of whitespace that breaks a line, at `\n` or
/// `\r`, made one space, so no part of it starts a line of the body.
/// Whitespace within a line stays as it is.
fn one_line(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if !c.is_whitespace() {
            out.push(c);
            continue;
        }
        let mut run = String::from(c);
        while let Some(w) = chars.next_if(|w| w.is_whitespace()) {
            run.push(w);
        }
        if run.contains(['\n', '\r']) {
            out.push(' ');
        } else {
            out.push_str(&run);
        }
    }

    out
}

/// Names a finding's file, and its line when it has one.
fn location(f: &Finding) -> String {
    if f.line == 0 {
        f.path.clone()
    } else {
        format!("{}:{}", f.path, f.line)
    }
}

/// Wraps each bare `@word` outside a code span in backticks, so a review
/// quoting a scenario tag such as `@I2` mentions no GitHub user of that
/// name. Text inside backticks and the `@` of an email address stay as
/// they are.
fn defuse_mentions(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut in_code = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let bare = !in_code
            && c == '@'
            && !(i > 0 && word_char(chars[i - 1]))
            && chars.get(i + 1).is_some_and(|next| word_char(*next));
        if !bare {
            // A backtick opens a code span only when another closes it;
            // GitHub prints a lone one as text, mentions and all.
            in_code = match c {
                '`' if in_code => false,
                '`' => chars[i + 1..].contains(&'`'),
                _ => in_code,
            };
            out.push(c);
            i += 1;
            continue;
        }
        let end = (i + 1..chars.len())
            .find(|&j| !word_char(chars[j]) && chars[j] != '-')
            .unwrap_or(chars.len());
        out.push('`');
        out.extend(&chars[i..end]);
        out.push('`');
        i = end;
    }

    out
}

/// `text` made safe to print between backticks: a backtick inside
/// would close the code span early and let a mention out, so it
/// becomes a quote, and a line break would start a line of the body,
/// so it is collapsed as [`one_line`] does.
fn code_span(text: &str) -> String {
    one_line(text).replace('`', "'")
}

/// Whether `c` can be part of a GitHub user name.
fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests;
