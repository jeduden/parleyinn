//! Which agents the review's orchestrator consults (ENG-28): from the
//! paths a pull request changes and the requirement rows it touches,
//! the low-effort specialists that read them, grouped under the
//! medium-effort generalist that weighs their findings. It is a pure
//! function of its inputs, so the review gate can check the
//! orchestrator against the same list.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The generalist for code, its tests and the build configuration.
pub const CODE: &str = "review-code";
/// The generalist for the specification, the scenarios and the model.
pub const SPEC: &str = "review-spec";
/// The generalist for the invariants and the security requirements.
pub const SECURITY: &str = "review-security";

/// The generalists, which weigh at medium effort.
pub const GENERALISTS: [&str; 3] = [CODE, SPEC, SECURITY];

/// Each generalist's deep instance: the same template, run on the
/// strongest model at high effort with more turns, which a deep review
/// consults in its place.
pub const CODE_DEEP: &str = "review-code-deep";
pub const SPEC_DEEP: &str = "review-spec-deep";
pub const SECURITY_DEEP: &str = "review-security-deep";

/// The deep instances, in the order of [`GENERALISTS`].
pub const DEEP_GENERALISTS: [&str; 3] = [CODE_DEEP, SPEC_DEEP, SECURITY_DEEP];

const RUST: &str = "lang-rust";
const TOML: &str = "lang-toml";
const SHELL: &str = "lang-shell";
const ACTIONS: &str = "lang-github-actions";
const GHERKIN: &str = "lang-gherkin";
const MARKDOWN: &str = "lang-markdown";
const JSON: &str = "lang-json";
const YAML: &str = "lang-yaml";
const CI: &str = "ci-engineer";
const UNIT: &str = "test-engineer-unit";
const INTEGRATION: &str = "test-engineer-integration";
const END_TO_END: &str = "test-engineer-end-to-end";
/// The specialist for the performance budgets (NFR, ENG-15, I9).
pub const PERFORMANCE: &str = "performance";
/// The benchmark and calibration tooling, which the performance
/// specialist reads whatever the file's language.
const BENCHMARKS: [&str; 2] = ["tooling/benchmark/", "tooling/calibrate/"];
/// The fixtures, corpora, research notes and spikes: data, whose files
/// no language specialist reads, Markdown aside.
pub const DATA_ROOTS: [&str; 4] = ["testdata/", "fuzz/corpus/", "research/", "plan/"];
/// The files written in no language: license texts, lists, checksums
/// and git's own files, by extension or by name.
const PLAIN_TEXT: [&str; 4] = ["txt", "sha256", "gitignore", "gitattributes"];
const PLAIN_NAMES: [&str; 3] = ["LICENSE", "THIRD_PARTY_LICENSES", "CODEOWNERS"];
/// The files that decide who approves what and what the agents load:
/// the officer and the CI engineer read them whatever their language.
const GOVERNANCE: [&str; 3] = [
    ".github/CODEOWNERS",
    ".github/rulesets/",
    ".claude/settings.json",
];
/// The prefix of the non-functional requirements, which set the budgets.
const NON_FUNCTIONAL: &str = "NFR-";
/// The invariant that ParleyInn never degrades the agent.
const NEVER_DEGRADES: &str = "I9";

/// Every specialist the dispatch names whatever the tree contains, each
/// with the generalist that weighs its findings: the single source of
/// both, so no specialist is listed without its route. The section and
/// domain-model specialists follow their files.
pub const FIXED_SPECIALISTS: [(&str, &str); 15] = [
    (RUST, CODE),
    (TOML, CODE),
    (JSON, CODE),
    (YAML, CODE),
    (SHELL, CODE),
    (ACTIONS, CODE),
    (GHERKIN, SPEC),
    (MARKDOWN, SPEC),
    (CI, CODE),
    (UNIT, CODE),
    (INTEGRATION, CODE),
    (END_TO_END, CODE),
    (OFFICER, SECURITY),
    (MODEL_HUB, SPEC),
    (PERFORMANCE, CODE),
];

/// Each generalist the review consults, with the specialists whose
/// findings it weighs.
pub type Dispatch = BTreeMap<String, BTreeSet<String>>;

/// Each specialist with the changed files it reads, so its diff holds
/// only those.
pub type Scopes = BTreeMap<String, BTreeSet<String>>;

/// The SRS directory, whose files each have a section specialist.
const SRS: &str = "docs/srs/";
/// The SRS file that contains no section: its table of contents.
const SRS_INDEX: &str = "docs/srs/index.md";
/// The domain model directory and its hub.
pub const MODEL: &str = "docs/domain-model/";
pub const MODEL_HUB: &str = "domain-model-hub";
/// The invariants, whose change reaches the security generalist.
const INVARIANTS: &str = "docs/srs/invariants.md";
/// The officer every security-relevant change consults.
const OFFICER: &str = "persona-security-officer";
/// The invariants whose rows reach the security generalist.
const SECURITY_INVARIANTS: [&str; 4] = ["I2", "I4", "I6", "I8"];
/// The test, within a crate, that refuses a crate linked into it.
const REACH_TEST: &str = "tests/reach.rs";
/// The security-sensitive crate name that stands for every crate, for
/// when main's ENG-11 row names none the analysis can read.
pub const EVERY_CRATE: &str = "*";

/// The generalist that weighs `specialist`'s findings, the single source
/// of the routing the agents' `generalist` front matter repeats. None for
/// a name that is no specialist.
#[must_use]
pub fn generalist_of(specialist: &str) -> Option<&'static str> {
    FIXED_SPECIALISTS
        .iter()
        .find(|(name, _)| *name == specialist)
        .map(|(_, generalist)| *generalist)
        .or_else(|| {
            (specialist.starts_with("srs-") || specialist.starts_with("domain-model-"))
                .then_some(SPEC)
        })
}

/// The specialist for the SRS section at `path`: `srs-` and the file's
/// name without its leading section number. None for a path that is
/// not a section file.
#[must_use]
pub fn section_specialist(path: &str) -> Option<String> {
    let name = path.strip_prefix(SRS)?.strip_suffix(".md")?;
    if name.contains('/') || path == SRS_INDEX {
        return None;
    }
    let digits = name
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(name.len());
    let bare = if digits == 0 {
        name
    } else {
        let rest = &name[digits..];
        let rest = rest
            .strip_prefix(|c: char| c.is_ascii_lowercase())
            .filter(|r| r.starts_with('-'))
            .unwrap_or(rest);
        rest.strip_prefix('-').unwrap_or(name)
    };

    agent_name(&format!("srs-{bare}"))
}

/// The specialist for the domain-model file named `file`, without its
/// `.md`: the hub for the index, `domain-model-` and the name otherwise.
#[must_use]
pub fn model_specialist(file: &str) -> Option<String> {
    if file == "index" {
        Some(MODEL_HUB.to_owned())
    } else {
        agent_name(&format!("domain-model-{file}"))
    }
}

/// `name` when it reads as an agent's name, lowercase letters, digits
/// and hyphens, so no path a pull request names writes markup into the
/// review bundle.
fn agent_name(name: &str) -> Option<String> {
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        .then(|| name.to_owned())
}

/// The agents to consult for `paths`, the changed paths, and `rows`,
/// the requirement rows the change touches; `security_crates` are the
/// crates ENG-11 names.
#[must_use]
pub fn dispatch(
    paths: &[String],
    rows: &[&srs::Requirement],
    security_crates: &[String],
) -> Dispatch {
    grouped(&scopes(paths, rows, security_crates, &BTreeMap::new()))
}

/// Each specialist to consult, with the changed files it reads: those
/// of `paths` that bring it, and for a specialist one of `rows` brings,
/// that row's own file among `paths` and every file `naming`, each
/// file's ids, says names the row. A specialist a row brings with no
/// such file reads none.
#[must_use]
pub fn scopes(
    paths: &[String],
    rows: &[&srs::Requirement],
    security_crates: &[String],
    naming: &BTreeMap<String, BTreeSet<String>>,
) -> Scopes {
    let mut out = Scopes::new();
    let mut add = |specialist: &str, files: &mut dyn Iterator<Item = &String>| {
        if generalist_of(specialist).is_some() {
            out.entry(specialist.to_owned())
                .or_default()
                .extend(files.cloned());
        }
    };
    for path in paths {
        for specialist in by_path(path, security_crates) {
            add(&specialist, &mut std::iter::once(path));
        }
    }
    for row in rows {
        let own = row.path.to_str().unwrap_or_default();
        let files: Vec<&String> = paths
            .iter()
            .filter(|p| p.as_str() == own)
            .chain(
                naming
                    .iter()
                    .filter(|(_, ids)| ids.contains(&row.id))
                    .map(|(f, _)| f),
            )
            .collect();
        if let Some(section) = row.path.to_str().and_then(section_specialist) {
            add(&section, &mut files.iter().copied());
        }
        if row
            .traces
            .iter()
            .any(|t| SECURITY_INVARIANTS.contains(&t.as_str()))
        {
            add(OFFICER, &mut files.iter().copied());
        }
        if row.id.starts_with(NON_FUNCTIONAL) || row.traces.iter().any(|t| t == NEVER_DEGRADES) {
            add(PERFORMANCE, &mut files.iter().copied());
        }
    }

    out
}

/// `scopes` as a dispatch: each generalist with the specialists it
/// weighs.
#[must_use]
pub fn grouped(scopes: &Scopes) -> Dispatch {
    let mut out = Dispatch::new();
    for specialist in scopes.keys() {
        if let Some(generalist) = generalist_of(specialist) {
            out.entry(generalist.to_owned())
                .or_default()
                .insert(specialist.clone());
        }
    }

    out
}

/// `dispatch` for a deep review: each generalist's specialists handed
/// to its deep instance instead.
#[must_use]
pub fn deepened(dispatch: Dispatch) -> Dispatch {
    dispatch
        .into_iter()
        .map(|(generalist, specialists)| {
            let deep = GENERALISTS
                .iter()
                .zip(DEEP_GENERALISTS)
                .find(|(g, _)| **g == generalist)
                .map_or(generalist, |(_, deep)| deep.to_owned());
            (deep, specialists)
        })
        .collect()
}

/// Every agent `dispatch` names: the generalists and their specialists.
#[must_use]
pub fn agents(dispatch: &Dispatch) -> BTreeSet<String> {
    dispatch
        .iter()
        .flat_map(|(generalist, specialists)| {
            std::iter::once(generalist.clone()).chain(specialists.iter().cloned())
        })
        .collect()
}

/// The specialists one changed `path` brings.
fn by_path(path: &str, security_crates: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let ext = extension(path);
    match language_specialist(path) {
        Some(RUST) => {
            out.push(RUST.to_owned());
            out.push(test_layer(path).to_owned());
        }
        Some(ACTIONS) => {
            out.push(ACTIONS.to_owned());
            out.push(CI.to_owned());
        }
        Some(GHERKIN) => {
            out.push(GHERKIN.to_owned());
            out.push(MODEL_HUB.to_owned());
            out.push(END_TO_END.to_owned());
        }
        Some(language) => out.push(language.to_owned()),
        None => {}
    }
    if GOVERNANCE.iter().any(|g| path.starts_with(g)) {
        out.push(OFFICER.to_owned());
        out.push(CI.to_owned());
    }
    if let Some(section) = section_specialist(path) {
        out.push(section);
        out.push(MODEL_HUB.to_owned());
    }
    if let Some(file) = path.strip_prefix(MODEL).and_then(|f| f.strip_suffix(".md")) {
        out.extend(model_specialist(file));
    }
    if BENCHMARKS.iter().any(|b| path.starts_with(b)) || (ext == "rs" && product_code(path)) {
        out.push(PERFORMANCE.to_owned());
    }
    if path == INVARIANTS || security_code(path, security_crates) {
        out.push(OFFICER.to_owned());
    }

    out
}

/// The language specialist for the file at `path`: by its extension,
/// workflows apart from other YAML. None for a file in no language the
/// specialists read, and for data under [`DATA_ROOTS`], Markdown aside.
#[must_use]
pub fn language_specialist(path: &str) -> Option<&'static str> {
    let ext = extension(path);
    if ext == "md" {
        return Some(MARKDOWN);
    }
    if DATA_ROOTS.iter().any(|d| path.starts_with(d)) {
        return None;
    }
    let workflow = path.starts_with(".github/workflows/") || path.starts_with(".github/actions/");
    match ext {
        "rs" => Some(RUST),
        "toml" | "lock" => Some(TOML),
        "sh" => Some(SHELL),
        "yml" | "yaml" if workflow => Some(ACTIONS),
        "yml" | "yaml" => Some(YAML),
        "json" => Some(JSON),
        "feature" => Some(GHERKIN),
        _ => None,
    }
}

/// Whether the file at `path` is source or configuration, which ENG-28
/// wants a language specialist for: anything outside [`DATA_ROOTS`]
/// that is not plain text.
#[must_use]
pub fn needs_a_language(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    !DATA_ROOTS.iter().any(|d| path.starts_with(d))
        && !PLAIN_NAMES.contains(&name)
        && !PLAIN_TEXT.contains(&extension(path))
}

/// The extension of the file at `path`, or of a dot file its name.
fn extension(path: &str) -> &str {
    Path::new(path)
        .extension()
        .or_else(|| Path::new(path).file_name())
        .and_then(|e| e.to_str())
        .map_or("", |e| e.trim_start_matches('.'))
}

/// The test-layer specialist for a Rust file at `path`.
fn test_layer(path: &str) -> &'static str {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if path.contains("/tests/bdd/") || name.starts_with("e2e") {
        END_TO_END
    } else if path.contains("/tests/") {
        INTEGRATION
    } else {
        UNIT
    }
}

/// Whether `path` is a product crate's shipped code: under its `src/`,
/// and not a module's `tests.rs`.
fn product_code(path: &str) -> bool {
    path.strip_prefix("crates/")
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(_, tail)| tail.starts_with("src/") && !tail.ends_with("/tests.rs"))
}

/// Whether `path` lies in a security-sensitive crate or is a crate's
/// reach test. [`EVERY_CRATE`] among `security_crates` names them all.
pub(crate) fn security_code(path: &str, security_crates: &[String]) -> bool {
    path.strip_prefix("crates/")
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(name, tail)| {
            security_crates
                .iter()
                .any(|c| c == name || c == EVERY_CRATE)
                || tail == REACH_TEST
        })
}

#[cfg(test)]
mod tests;
