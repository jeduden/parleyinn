//! Compares main's tree with the pull request's: which requirements and
//! scenarios changed, which the diff names, the review class, and the
//! findings a machine can decide alone. It reads nothing but its inputs:
//! the stakeholder's consent arrives as a flag the command line read.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{BLOCKING, Finding, NIT, Snapshot, diff, dispatch, owners};

/// The invariants file ENG-29 guards.
pub const INVARIANTS: &str = "docs/srs/invariants.md";

/// The requirement whose text names the requirements marked for I2
/// review.
const ENG_29: &str = "ENG-29";

/// Where the decision records live.
const ADR_DIR: &str = "docs/adr/";

/// Markdown under these directories is read as more than prose: the
/// domain model, the plans and the scenarios.
const NOT_LIGHT: [&str; 3] = ["docs/domain-model/", "plan/", "features/"];

/// The code no stakeholder path holds that still reviews deep: the
/// gates. Each crate's reach test and the security-sensitive crates the
/// ENG-11 row names join it.
const DEEP_CODE: [&str; 1] = ["tooling/"];

/// The requirement whose text names the security-sensitive crates.
const ENG_11: &str = "ENG-11";

/// What an added line that lifts a ban weighs.
const LIFTS_A_BAN: &str = "lifts a reach or determinism ban: weigh its reason";

/// What lifts a ban: an allow or expect of a `disallowed_*` lint, or of
/// a lint group that holds them.
const BANS: [&str; 3] = ["clippy::disallowed_", "clippy::style", "clippy::all"];

/// Whether `line` names a lint of [`BANS`]: a group only as a whole
/// word, so `clippy::allow_attributes` is not `clippy::all`.
fn lifts_a_ban(line: &str) -> bool {
    BANS.iter().any(|ban| {
        line.match_indices(ban).any(|(at, _)| {
            ban.ends_with('_')
                || !line[at + ban.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        })
    })
}

/// How deep a review goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// Only Markdown no stakeholder path holds, outside the domain
    /// model, the plans and the scenarios: notes and guides.
    Light,
    /// Anything else no stakeholder path holds: product code, scenarios,
    /// plans, or a diff that names no path.
    Standard,
    /// A path CODEOWNERS assigns: the SRS, the scenarios, CI, the agent
    /// instructions; code that gates the build or is security-sensitive;
    /// or a diff header whose paths do not read, so the class fails
    /// closed.
    Deep,
}

impl Class {
    /// The name the review workflow reads.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }
}

/// How one requirement or scenario differs between the two trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Added,
    Removed,
    Changed,
}

/// The decision record that satisfies ENG-29, as the review bundle
/// quotes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub path: String,
    pub summary: String,
}

/// What the pull request does, as the review bundle and the gate read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub class: Class,
    /// The paths CODEOWNERS assigns among the changed ones.
    pub owned: Vec<String>,
    pub paths: Vec<String>,
    /// The diff headers whose paths do not read.
    pub unparsed: Vec<String>,
    pub requirements: BTreeMap<String, Change>,
    pub scenarios: BTreeMap<String, Change>,
    /// Every id the change touches: a changed requirement or scenario,
    /// or one a changed line names, that either tree defines.
    pub touched: Vec<String>,
    pub findings: Vec<Finding>,
    /// The record that satisfies ENG-29, when a guarded change has one
    /// and the stakeholder consented.
    pub eng_29_record: Option<Record>,
    /// The agents the review's orchestrator consults (ENG-28).
    pub dispatch: dispatch::Dispatch,
}

/// Compares `base`, main's tree, with `head`, the pull request's, whose
/// diff against main is `diff`. `consent` says whether a stakeholder
/// comment on the pull request consents to it.
#[must_use]
pub fn analyse(base: &Snapshot, head: &Snapshot, diff: &str, consent: bool) -> Analysis {
    let diff::Paths { paths, unparsed } = diff::changed_paths(diff);
    let patterns = owners::patterns(&base.codeowners);
    let owned: Vec<String> = paths
        .iter()
        .filter(|p| patterns.iter().any(|pat| owners::owns(pat, p)))
        .cloned()
        .collect();
    let requirements = requirement_changes(base, head);
    let scenarios = scenario_changes(base, head);
    let known: BTreeSet<&str> = base.ids().chain(head.ids()).collect();
    let touched: BTreeSet<String> = requirements
        .keys()
        .chain(scenarios.keys())
        .cloned()
        .chain(diff::mentioned_ids(diff))
        .filter(|id| known.contains(id.as_str()))
        .collect();
    let mut findings = ratchet(base, head);
    let (guard, eng_29_record) = eng_29(base, head, &paths, &requirements, consent);
    findings.extend(guard);
    findings.extend(unparsed.iter().map(|h| nit(h, UNPARSED)));
    findings.extend(lifted_bans(diff));
    let crates = security_crates(base);
    if crates.is_none() {
        let path = row_path(base, ENG_11).unwrap_or_else(|| "docs/srs".to_owned());
        findings.push(nit(&path, NO_SECURITY_CRATES));
    }
    findings.sort();

    // Without the list, every crate counts as security-sensitive, so the
    // class and the dispatch fail closed.
    let crates = crates.unwrap_or_else(|| vec![dispatch::EVERY_CRATE.to_owned()]);
    let deep = paths.iter().any(|p| deep_code(&crates, p));
    // The rows that bring their section's specialist: those the change
    // edits, and those code names; a row Markdown only quotes is in the
    // bundle but brings no one.
    let named: BTreeSet<String> = requirements
        .keys()
        .chain(scenarios.keys())
        .cloned()
        .chain(diff::ids_in_code(diff))
        .collect();
    let rows: Vec<&srs::Requirement> = touched
        .iter()
        .filter(|id| named.contains(*id))
        .filter_map(|id| requirement(head, id).or_else(|| requirement(base, id)))
        .collect();
    let class = classify(&paths, deep || !owned.is_empty(), &unparsed);
    let dispatch = match class {
        Class::Deep => dispatch::deepened(dispatch::dispatch(&paths, &rows, &crates)),
        Class::Light | Class::Standard => dispatch::dispatch(&paths, &rows, &crates),
    };

    Analysis {
        class,
        owned,
        paths,
        unparsed,
        requirements,
        scenarios,
        touched: touched.into_iter().collect(),
        findings,
        eng_29_record,
        dispatch,
    }
}

/// What an unparsed diff header weighs.
const UNPARSED: &str = "the paths this diff header names do not read, so the review class is deep";

/// What main's ENG-11 row weighs when it names no security-sensitive
/// crate list the analysis can read.
const NO_SECURITY_CRATES: &str =
    "the ENG-11 row names no security-sensitive crates, so every crate reviews deep";

/// A nit at `path` as a whole.
fn nit(path: &str, body: &str) -> Finding {
    Finding {
        path: path.to_owned(),
        line: 0,
        severity: NIT,
        body: body.to_owned(),
    }
}

/// One nit per file the diff adds a line to that lifts a reach or
/// determinism ban, which a reviewer weighs.
fn lifted_bans(diff: &str) -> BTreeSet<Finding> {
    diff::added_lines(diff)
        .into_iter()
        .filter(|(_, line)| lifts_a_ban(line))
        .map(|(file, _)| nit(&file, LIFTS_A_BAN))
        .collect()
}

/// The row `id` of `snapshot`, when it defines one.
fn requirement<'a>(snapshot: &'a Snapshot, id: &str) -> Option<&'a srs::Requirement> {
    snapshot.requirements.iter().find(|r| r.id == id)
}

/// The security-sensitive crates the ENG-11 row of `snapshot` names;
/// `None` when it names no list or an empty one, so the caller fails
/// closed rather than treat no crate as security-sensitive.
fn security_crates(snapshot: &Snapshot) -> Option<Vec<String>> {
    snapshot
        .requirements
        .iter()
        .find(|r| r.id == ENG_11)
        .and_then(|r| r.text.split_once("security-sensitive crates ("))
        .and_then(|(_, after)| after.split_once(')'))
        .map(|(list, _)| {
            list.split(',')
                .map(|c| c.trim().trim_matches('`').to_owned())
                .filter(|c| !c.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|crates| !crates.is_empty())
}

/// Whether `path` is code that reviews deep though no stakeholder path
/// holds it: a gate, a reach test, or a security-sensitive crate.
fn deep_code(crates: &[String], path: &str) -> bool {
    DEEP_CODE.iter().any(|d| path.starts_with(d)) || dispatch::security_code(path, crates)
}

/// The review class: deep when `deep` says a stakeholder path or deep
/// code changed, or a header does not read; light when only prose
/// Markdown did; standard otherwise.
fn classify(paths: &[String], deep: bool, unparsed: &[String]) -> Class {
    let light = |p: &String| p.ends_with(".md") && !NOT_LIGHT.iter().any(|d| p.starts_with(d));
    if deep || !unparsed.is_empty() {
        Class::Deep
    } else if !paths.is_empty() && paths.iter().all(light) {
        Class::Light
    } else {
        Class::Standard
    }
}

/// Keys each of `items` by `id`, keeping what `value` reads of it.
fn keyed<'a, T, V>(
    items: &'a [T],
    id: impl Fn(&'a T) -> &'a str,
    value: impl Fn(&'a T) -> V,
) -> BTreeMap<&'a str, V> {
    items.iter().map(|i| (id(i), value(i))).collect()
}

/// How `base` and `head` differ, id by id.
fn changes<V: PartialEq>(
    base: &BTreeMap<&str, V>,
    head: &BTreeMap<&str, V>,
) -> BTreeMap<String, Change> {
    let ids: BTreeSet<&&str> = base.keys().chain(head.keys()).collect();
    ids.into_iter()
        .filter_map(|id| {
            let change = match (base.get(*id), head.get(*id)) {
                (None, _) => Change::Added,
                (_, None) => Change::Removed,
                (Some(a), Some(b)) if a != b => Change::Changed,
                _ => return None,
            };
            Some(((*id).to_owned(), change))
        })
        .collect()
}

/// The requirements whose row differs: priority, method, traces or
/// text. Where a row sits does not count.
fn requirement_changes(base: &Snapshot, head: &Snapshot) -> BTreeMap<String, Change> {
    let row = |r: &srs::Requirement| {
        (
            r.priority.clone(),
            r.verify.clone(),
            r.traces.clone(),
            r.text.clone(),
        )
    };
    changes(
        &keyed(&base.requirements, |r| r.id.as_str(), row),
        &keyed(&head.requirements, |r| r.id.as_str(), row),
    )
}

/// The scenarios whose name, tags or steps differ.
fn scenario_changes(base: &Snapshot, head: &Snapshot) -> BTreeMap<String, Change> {
    let body = |s: &scenario::Scenario| {
        (
            s.name.clone(),
            s.priority.clone(),
            s.invariants.clone(),
            s.pending,
            s.steps.clone(),
        )
    };
    changes(
        &keyed(&base.scenarios, |s| s.id.as_str(), body),
        &keyed(&head.scenarios, |s| s.id.as_str(), body),
    )
}

/// A scenario live on main stays live: the pull request may not delete
/// it or put it back on `@pending` (CLAUDE.md, Requirements and
/// Scenarios). The backlog only shrinks.
fn ratchet(base: &Snapshot, head: &Snapshot) -> Vec<Finding> {
    let now: BTreeMap<&str, &scenario::Scenario> =
        head.scenarios.iter().map(|s| (s.id.as_str(), s)).collect();
    base.scenarios
        .iter()
        .filter(|s| !s.pending)
        .filter_map(|was| match now.get(was.id.as_str()) {
            None => Some(Finding {
                path: was.path.display().to_string(),
                line: was.line,
                severity: BLOCKING,
                body: format!(
                    "the {} scenario is live on main and this change removes it",
                    was.id
                ),
            }),
            Some(is) if is.pending => Some(Finding {
                path: is.path.display().to_string(),
                line: is.line,
                severity: BLOCKING,
                body: format!(
                    "the {} scenario is live on main and this change puts it back on @pending",
                    was.id
                ),
            }),
            Some(_) => None,
        })
        .collect()
}

/// ENG-29: a change to an invariant, or to a requirement ENG-29 marks
/// for I2 review, lands only with an accepted decision record, changed
/// in the same pull request, that names ENG-29 and what it covers, and
/// with the stakeholder's consent. Returns the finding when either is
/// missing, else the record that satisfies it.
fn eng_29(
    base: &Snapshot,
    head: &Snapshot,
    paths: &[String],
    requirements: &BTreeMap<String, Change>,
    consent: bool,
) -> (Vec<Finding>, Option<Record>) {
    let marked = marked_for_i2_review(base);
    let rows: Vec<&String> = requirements
        .keys()
        .filter(|id| marked.contains(id))
        .collect();
    let invariant = base.invariants != head.invariants || paths.iter().any(|p| p == INVARIANTS);
    if rows.is_empty() && !invariant {
        return (Vec::new(), None);
    }
    let mut names = vec![ENG_29];
    let mut guarded = Vec::new();
    if invariant {
        names.push("invariant");
        guarded.push("an invariant");
    }
    names.extend(rows.iter().map(|id| id.as_str()));
    guarded.extend(rows.iter().map(|id| id.as_str()));
    let record = paths
        .iter()
        .filter(|p| p.starts_with(ADR_DIR))
        .filter_map(|p| head.decisions.get(p).map(|body| (p, body)))
        .find_map(|(p, body)| covering(p, body, &names[1..]));
    if let (Some(record), true) = (&record, consent) {
        return (Vec::new(), Some(record.clone()));
    }
    let mut missing = Vec::new();
    if record.is_none() {
        missing.push(format!(
            "no accepted ADR under docs/adr that this change changes names {}",
            and_list(&names)
        ));
    }
    if !consent {
        missing.push("no stakeholder comment consents to it".to_owned());
    }
    let path = if invariant {
        INVARIANTS.to_owned()
    } else {
        rows.first()
            .and_then(|id| row_path(head, id).or_else(|| row_path(base, id)))
            .unwrap_or_default()
    };
    let finding = Finding {
        path,
        line: 0,
        severity: BLOCKING,
        body: format!(
            "this change touches {} (ENG-29), but {}",
            guarded.join(", "),
            missing.join("; and ")
        ),
    };

    (vec![finding], None)
}

/// `items` as "a, b and c".
fn and_list(items: &[&str]) -> String {
    match items.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => items.join(""),
    }
}

/// Where `tree` holds the row `id`.
fn row_path(tree: &Snapshot, id: &str) -> Option<String> {
    tree.requirements
        .iter()
        .find(|r| r.id == id)
        .map(|r| r.path.display().to_string())
}

/// The record at `path` when `body` is an accepted decision record that
/// names ENG-29 and each of `names`: the word "invariant" or a
/// requirement id.
fn covering(path: &str, body: &str, names: &[&str]) -> Option<Record> {
    let adr = adr::parse(Path::new(path), body).ok()?;
    let ids: BTreeSet<&str> = diff::ids_in(body).collect();
    let lower = body.to_lowercase();
    let names_all = names.iter().all(|n| {
        if srs::is_requirement_id(n) {
            ids.contains(n)
        } else {
            lower.contains(n)
        }
    });
    (adr.status == adr::ACCEPTED && ids.contains(ENG_29) && names_all).then(|| Record {
        path: path.to_owned(),
        summary: adr.summary,
    })
}

/// The ids ENG-29's row names besides itself: the requirements marked
/// for I2 review.
fn marked_for_i2_review(base: &Snapshot) -> Vec<String> {
    base.requirements
        .iter()
        .find(|r| r.id == ENG_29)
        .map(|r| {
            let ids: BTreeSet<&str> = diff::ids_in(&r.text).filter(|id| *id != ENG_29).collect();
            ids.into_iter().map(str::to_owned).collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
