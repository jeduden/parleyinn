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

const RUST: &str = "lang-rust";
const TOML: &str = "lang-toml";
const SHELL: &str = "lang-shell";
const ACTIONS: &str = "lang-github-actions";
const GHERKIN: &str = "lang-gherkin";
const MARKDOWN: &str = "lang-markdown";
const CI: &str = "ci-engineer";
const UNIT: &str = "test-engineer-unit";
const INTEGRATION: &str = "test-engineer-integration";
const END_TO_END: &str = "test-engineer-end-to-end";
/// The specialist for the performance budgets (NFR, ENG-15, I9).
pub const PERFORMANCE: &str = "performance";
/// The benchmark and calibration tooling, which the performance
/// specialist reads whatever the file's language.
const BENCHMARKS: [&str; 2] = ["tooling/benchmark/", "tooling/calibrate/"];
/// The prefix of the non-functional requirements, which set the budgets.
const NON_FUNCTIONAL: &str = "NFR-";
/// The invariant that ParleyInn never degrades the agent.
const NEVER_DEGRADES: &str = "I9";

/// Every specialist the dispatch names whatever the tree contains, each
/// with the generalist that weighs its findings: the single source of
/// both, so no specialist is listed without its route. The section and
/// domain-model specialists follow their files.
pub const FIXED_SPECIALISTS: [(&str, &str); 13] = [
    (RUST, CODE),
    (TOML, CODE),
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
    let mut out = Dispatch::new();
    let mut add = |specialist: &str| {
        if let Some(generalist) = generalist_of(specialist) {
            out.entry(generalist.to_owned())
                .or_default()
                .insert(specialist.to_owned());
        }
    };
    for path in paths {
        for specialist in by_path(path, security_crates) {
            add(&specialist);
        }
    }
    for row in rows {
        if let Some(section) = row.path.to_str().and_then(section_specialist) {
            add(&section);
        }
        if row
            .traces
            .iter()
            .any(|t| SECURITY_INVARIANTS.contains(&t.as_str()))
        {
            add(OFFICER);
        }
        if row.id.starts_with(NON_FUNCTIONAL) || row.traces.iter().any(|t| t == NEVER_DEGRADES) {
            add(PERFORMANCE);
        }
    }

    out
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
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let workflow = path.starts_with(".github/workflows/") || path.starts_with(".github/actions/");
    match ext {
        "rs" => {
            out.push(RUST.to_owned());
            out.push(test_layer(path).to_owned());
        }
        "toml" | "lock" => out.push(TOML.to_owned()),
        "sh" => out.push(SHELL.to_owned()),
        "yml" | "yaml" if workflow => {
            out.push(ACTIONS.to_owned());
            out.push(CI.to_owned());
        }
        "feature" => {
            out.push(GHERKIN.to_owned());
            out.push(MODEL_HUB.to_owned());
            out.push(END_TO_END.to_owned());
        }
        "md" => out.push(MARKDOWN.to_owned()),
        _ => {}
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
