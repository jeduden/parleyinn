//! Which agents the review's orchestrator consults (ENG-28): from the
//! paths a pull request changes and the requirement rows it touches,
//! the low-effort specialists that read them, grouped under the
//! medium-effort generalist that weighs their findings. It is a pure
//! function of its inputs, so the review gate can hold the orchestrator
//! to the same list.

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

/// Every specialist the dispatch names whatever the tree holds; the
/// section and domain-model specialists follow their files.
pub const FIXED_SPECIALISTS: [&str; 12] = [
    RUST,
    TOML,
    SHELL,
    ACTIONS,
    GHERKIN,
    MARKDOWN,
    CI,
    UNIT,
    INTEGRATION,
    END_TO_END,
    OFFICER,
    MODEL_HUB,
];

/// Each generalist the review consults, with the specialists whose
/// findings it weighs.
pub type Dispatch = BTreeMap<String, BTreeSet<String>>;

/// The SRS directory, whose files each have a section specialist.
const SRS: &str = "docs/srs/";
/// The SRS file that holds no section: its table of contents.
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
    let mut add = |generalist: &str, specialist: &str| {
        out.entry(generalist.to_owned())
            .or_default()
            .insert(specialist.to_owned());
    };
    for path in paths {
        for (generalist, specialist) in by_path(path, security_crates) {
            add(generalist, &specialist);
        }
    }
    for row in rows {
        if let Some(section) = row.path.to_str().and_then(section_specialist) {
            add(SPEC, &section);
        }
        if row
            .traces
            .iter()
            .any(|t| SECURITY_INVARIANTS.contains(&t.as_str()))
        {
            add(SECURITY, OFFICER);
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

/// The generalist and specialist pairs one changed `path` brings.
fn by_path(path: &str, security_crates: &[String]) -> Vec<(&'static str, String)> {
    let mut out: Vec<(&'static str, String)> = Vec::new();
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let workflow = path.starts_with(".github/workflows/") || path.starts_with(".github/actions/");
    match ext {
        "rs" => {
            out.push((CODE, RUST.to_owned()));
            out.push((CODE, test_layer(path).to_owned()));
        }
        "toml" | "lock" => out.push((CODE, TOML.to_owned())),
        "sh" => out.push((CODE, SHELL.to_owned())),
        "yml" | "yaml" if workflow => {
            out.push((CODE, ACTIONS.to_owned()));
            out.push((CODE, CI.to_owned()));
        }
        "feature" => {
            out.push((SPEC, GHERKIN.to_owned()));
            out.push((SPEC, MODEL_HUB.to_owned()));
            out.push((CODE, END_TO_END.to_owned()));
        }
        "md" => out.push((SPEC, MARKDOWN.to_owned())),
        _ => {}
    }
    if let Some(section) = section_specialist(path) {
        out.push((SPEC, section));
        out.push((SPEC, MODEL_HUB.to_owned()));
    }
    if let Some(file) = path.strip_prefix(MODEL).and_then(|f| f.strip_suffix(".md")) {
        out.extend(model_specialist(file).map(|s| (SPEC, s)));
    }
    if path == INVARIANTS || security_code(path, security_crates) {
        out.push((SECURITY, OFFICER.to_owned()));
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

/// Whether `path` lies in a security-sensitive crate or is a crate's
/// reach test.
pub(crate) fn security_code(path: &str, security_crates: &[String]) -> bool {
    path.strip_prefix("crates/")
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(name, tail)| security_crates.iter().any(|c| c == name) || tail == REACH_TEST)
}

#[cfg(test)]
mod tests;
