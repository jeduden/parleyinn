//! Compares main's tree with the pull request's: which requirements and
//! scenarios changed, which the diff names, the review class, and the
//! findings a machine can decide alone. It reads nothing but its inputs:
//! the stakeholder's consenting comments arrive as the command line read
//! them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{BLOCKING, Finding, NIT, Snapshot, diff, dispatch, owners, size_list};

/// The invariants file ENG-29 guards.
pub const INVARIANTS: &str = "docs/srs/invariants.md";

/// The requirement whose text names the requirements marked for I2
/// review.
const ENG_29: &str = "ENG-29";
/// How a finding names a change to ENG-29 itself: it holds the list of
/// requirements marked for I2 review, so a change to it could take one
/// off that list.
const OWN: &str = "ENG-29 itself";

/// Where the decision records live.
const ADR_DIR: &str = "docs/adr/";

/// Markdown under these directories is read as more than prose: the
/// domain model, the plans and the scenarios.
const NOT_LIGHT: [&str; 3] = ["docs/domain-model/", "plan/", "features/"];

/// The code no stakeholder path holds that still reviews deep: the
/// gates. Each crate's reach test and the security-sensitive crates
/// ENG-11 names join it.
const DEEP_CODE: [&str; 1] = ["tooling/"];

/// The text no stakeholder path holds that still reviews deep, since
/// it lands on the review's approval alone (ENG-21): the specification,
/// the decision records and the scenarios.
const DEEP_TEXT: [&str; 3] = ["docs/srs/", "docs/adr/", "features/"];

/// The files, at any depth, no stakeholder path holds that still review
/// deep: the build, lint, coverage and toolchain configuration and the
/// supply-chain and disclosure policy.
const DEEP_FILES: [&str; 11] = [
    "Cargo.toml",
    "clippy.toml",
    "deny.toml",
    "codecov.yml",
    "rust-toolchain.toml",
    "mise.toml",
    "mise.lock",
    "mise.dev.toml",
    "mise.dev.lock",
    "SECURITY.md",
    "DEPENDENCIES.md",
];

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
    /// Each changed file with the touched ids it holds or names: an SRS
    /// file each requirement it changes, a feature file each scenario it
    /// changes, and any file each id an added or removed line names.
    pub named: BTreeMap<String, BTreeSet<String>>,
    pub findings: Vec<Finding>,
    /// The record a guarded change lands with (ENG-29), when it has
    /// one, whatever the comments say.
    pub eng_29_record: Option<Record>,
    /// Those of `findings` that turn on the stakeholder's consent
    /// (ENG-21, ENG-29, ENG-34), in either state. The gate weighs them
    /// and no agent is shown one: a comment changes them, and what an
    /// agent reads does not turn on a comment (ENG-28).
    pub on_consent: Vec<Finding>,
    /// The agents the review runs (ENG-28).
    pub dispatch: dispatch::Dispatch,
    /// Each specialist with the changed files it reads.
    pub scopes: dispatch::Scopes,
}

/// Compares `base`, main's tree, with `head`, the pull request's, whose
/// diff against main is `diff`. `consents` are the bodies of the
/// stakeholder's comments on the pull request that consent: each to the
/// owned paths it names (ENG-21) and to a lengthened size list when it
/// names the size list (ENG-34), and any one of them to an ENG-29
/// change.
#[must_use]
pub fn analyse(base: &Snapshot, head: &Snapshot, diff: &str, consents: &[String]) -> Analysis {
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
    let guard = eng_29(base, head, &paths, &requirements, !consents.is_empty());
    findings.extend(guard.unrecorded);
    let unconsented: Vec<&String> = owned
        .iter()
        .filter(|p| !consents.iter().any(|c| owners::names(&patterns, c, p)))
        .collect();
    let names_the_size_list = consents
        .iter()
        .any(|c| owners::names(&patterns, c, size_list::SIZE_LIST));
    let on_consent: Vec<Finding> = (guard.unconsented.into_iter())
        .chain(stakeholder_paths(&owned, &unconsented))
        .chain(size_list::finding(diff, names_the_size_list))
        .collect();
    findings.extend(on_consent.iter().cloned());
    findings.extend(unparsed.iter().map(|h| nit(h, UNPARSED)));
    findings.extend(lifted_bans(diff));
    let crates = security_crates(base);
    if crates.is_none() {
        let path = requirement_path(base, ENG_11).unwrap_or_else(|| "docs/srs".to_owned());
        findings.push(nit(&path, NO_SECURITY_CRATES));
    }
    findings.sort();

    // Without the list, every crate counts as security-sensitive, so the
    // class and the dispatch fail closed.
    let crates = crates.unwrap_or_else(|| vec![dispatch::EVERY_CRATE.to_owned()]);
    let deep = paths
        .iter()
        .any(|p| deep_code(&crates, p) || deep_unowned(p));
    // The requirements that bring their section's specialist: those the
    // change edits, and those code names; one Markdown only quotes is in
    // the bundle but brings no one.
    let named: BTreeSet<String> = requirements
        .keys()
        .chain(scenarios.keys())
        .cloned()
        .chain(diff::ids_in_code(diff))
        .collect();
    let bringing: Vec<&srs::Requirement> = touched
        .iter()
        .filter(|id| named.contains(*id))
        .filter_map(|id| requirement(head, id).or_else(|| requirement(base, id)))
        .collect();
    let class = classify(&paths, deep || !owned.is_empty(), &unparsed);
    let scopes = dispatch::scopes(&paths, &bringing, &crates, &diff::ids_by_file(diff));
    let mut routed = dispatch::grouped(&scopes);
    if routed.is_empty() {
        // No rule routes the change: one generalist still reads it.
        routed.insert(dispatch::CODE.to_owned(), BTreeSet::new());
    }
    let dispatch = match class {
        Class::Deep => dispatch::deepened(routed),
        Class::Light | Class::Standard => routed,
    };

    Analysis {
        class,
        owned,
        paths,
        unparsed,
        named: named_by_file(base, head, &requirements, &scenarios, diff, &touched),
        requirements,
        scenarios,
        touched: touched.into_iter().collect(),
        findings,
        eng_29_record: guard.record,
        on_consent,
        dispatch,
        scopes,
    }
}

/// How many unconsented paths the ENG-21 finding lists before it counts
/// the rest.
const LISTED: usize = 10;

/// ENG-21: a change to a path CODEOWNERS assigns needs the stakeholder's
/// consent, by a comment that names it. Blocks at the first of the
/// `unconsented` paths, listing them; when every `owned` path is
/// consented, a nit that names the paths.
fn stakeholder_paths(owned: &[String], unconsented: &[&String]) -> Option<Finding> {
    let first = owned.first()?;
    let Some(blocked) = unconsented.first() else {
        return Some(Finding {
            path: first.clone(),
            line: 0,
            severity: NIT,
            body: format!(
                "this change touches paths CODEOWNERS assigns ({}) (ENG-21), and a \
                 stakeholder comment names each of them, consenting to it",
                owned.join(", ")
            ),
        });
    };
    let listed: Vec<&str> = unconsented
        .iter()
        .take(LISTED)
        .map(|p| p.as_str())
        .collect();
    let more = match unconsented.len().saturating_sub(LISTED) {
        0 => String::new(),
        n => format!(" and {n} more"),
    };
    Some(Finding {
        path: (*blocked).clone(),
        line: 0,
        severity: BLOCKING,
        body: format!(
            "this change touches paths CODEOWNERS assigns (ENG-21) that no stakeholder \
             comment names, so none consents to them: {}{more}",
            listed.join(", ")
        ),
    })
}

/// What an unparsed diff header weighs.
const UNPARSED: &str = "the paths this diff header names do not read, so the review class is deep";

/// What main's ENG-11 weighs when it names no security-sensitive crate
/// list the analysis can read.
const NO_SECURITY_CRATES: &str =
    "ENG-11 names no security-sensitive crates, so every crate reviews deep";

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

/// The requirement `id` of `snapshot`, when it defines one.
fn requirement<'a>(snapshot: &'a Snapshot, id: &str) -> Option<&'a srs::Requirement> {
    snapshot.requirements.iter().find(|r| r.id == id)
}

/// Each changed file with the `touched` ids it holds or names
/// ([`Analysis::named`]): the file of each requirement in
/// `requirements` and of each scenario in `scenarios`, as the pull
/// request has it or main when the pull request removes it, and each
/// file of `diff` with the ids its added or removed lines name.
fn named_by_file(
    base: &Snapshot,
    head: &Snapshot,
    requirements: &BTreeMap<String, Change>,
    scenarios: &BTreeMap<String, Change>,
    diff: &str,
    touched: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let defined = |id: &String| requirement(head, id).or_else(|| requirement(base, id));
    let scenario = |id: &String| {
        let find = |s: &Snapshot| {
            s.scenarios
                .iter()
                .find(|x| &x.id == id)
                .map(|x| x.path.clone())
        };
        find(head).or_else(|| find(base))
    };
    let held = (requirements.keys())
        .filter_map(|id| defined(id).map(|r| (r.path.clone(), id)))
        .chain(
            scenarios
                .keys()
                .filter_map(|id| scenario(id).map(|p| (p, id))),
        );
    let mut out = diff::ids_by_file(diff);
    for (path, id) in held {
        let path = path.to_string_lossy().into_owned();
        out.entry(path).or_default().insert(id.clone());
    }
    for ids in out.values_mut() {
        ids.retain(|id| touched.contains(id));
    }
    out.retain(|_, ids| !ids.is_empty());

    out
}

/// The security-sensitive crates ENG-11 names in `snapshot`;
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

/// Whether `path` reviews deep though no stakeholder path holds it and
/// it is no code: the specification, a decision record, a scenario, or
/// a configuration or policy file of [`DEEP_FILES`].
fn deep_unowned(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    DEEP_TEXT.iter().any(|d| path.starts_with(d)) || DEEP_FILES.contains(&name)
}

/// The review class: deep when `deep` says a stakeholder path, deep
/// code or deep text changed, or a header does not read; light when only prose
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

/// The requirements whose fields or text differ: priority, method,
/// traces or text. Where a requirement sits does not count.
fn requirement_changes(base: &Snapshot, head: &Snapshot) -> BTreeMap<String, Change> {
    let compared = |r: &srs::Requirement| {
        (
            r.priority.clone(),
            r.verify.clone(),
            r.traces.clone(),
            r.text.clone(),
        )
    };
    changes(
        &keyed(&base.requirements, |r| r.id.as_str(), compared),
        &keyed(&head.requirements, |r| r.id.as_str(), compared),
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

/// ENG-29: a change to an invariant, to a requirement ENG-29 marks for
/// I2 review, or to ENG-29 itself, which holds that list, lands only
/// with an accepted decision record, changed
/// in the same pull request, that names ENG-29 and what it covers, and
/// with the stakeholder's consent: `consent`, a consenting comment on
/// the pull request, whatever paths it names. Returns a finding for
/// each that is missing, and the record when the change has one.
fn eng_29(
    base: &Snapshot,
    head: &Snapshot,
    paths: &[String],
    requirements: &BTreeMap<String, Change>,
    consent: bool,
) -> Guard {
    let marked = marked_for_i2_review(base);
    let changed: Vec<&String> = requirements
        .keys()
        .filter(|id| marked.contains(id))
        .collect();
    let invariant = base.invariants != head.invariants || paths.iter().any(|p| p == INVARIANTS);
    let own = requirements.contains_key(ENG_29);
    if changed.is_empty() && !invariant && !own {
        return Guard::default();
    }
    let mut names = vec![ENG_29];
    let mut guarded = Vec::new();
    if invariant {
        names.push("invariant");
        guarded.push("an invariant");
    }
    if own {
        guarded.push(OWN);
    }
    names.extend(changed.iter().map(|id| id.as_str()));
    guarded.extend(changed.iter().map(|id| id.as_str()));
    let record = paths
        .iter()
        .filter(|p| p.starts_with(ADR_DIR))
        .filter_map(|p| head.decisions.get(p).map(|body| (p, body)))
        .find_map(|(p, body)| covering(p, body, &names[1..]));
    let path = if invariant {
        INVARIANTS.to_owned()
    } else {
        changed
            .first()
            .map(|id| id.as_str())
            .or(own.then_some(ENG_29))
            .and_then(|id| requirement_path(head, id).or_else(|| requirement_path(base, id)))
            .unwrap_or_default()
    };
    let finding = |missing: &str| Finding {
        path: path.clone(),
        line: 0,
        severity: BLOCKING,
        body: format!(
            "this change touches {} (ENG-29), but {missing}",
            guarded.join(", ")
        ),
    };
    let unrecorded = record.is_none().then(|| {
        finding(&format!(
            "no accepted ADR under docs/adr that this change changes names {}",
            and_list(&names)
        ))
    });
    let unconsented = (!consent).then(|| finding("no stakeholder comment consents to it"));

    Guard {
        unrecorded,
        unconsented,
        record,
    }
}

/// What ENG-29 finds of a change it guards, one finding a cause: the
/// record is the change's to bring, the consent the stakeholder's to
/// give in a comment.
#[derive(Debug, Default)]
struct Guard {
    /// The change lands with no record.
    unrecorded: Option<Finding>,
    /// No stakeholder comment consents to it.
    unconsented: Option<Finding>,
    /// The record it lands with.
    record: Option<Record>,
}

/// `items` as "a, b and c".
fn and_list(items: &[&str]) -> String {
    match items.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => items.join(""),
    }
}

/// Where `tree` holds the requirement `id`.
fn requirement_path(tree: &Snapshot, id: &str) -> Option<String> {
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

/// The ids ENG-29 names besides itself: the requirements marked
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
