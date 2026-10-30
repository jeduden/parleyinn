//! The reviewer components: every agent the review bundle can dispatch
//! (ENG-28), their deep instances, the orchestrator's skill, and every
//! other agent under `.claude/agents`, which a skill consults; each
//! read from the tree under evaluation with the model, effort, turn
//! budget and tools its front matter names.

use std::fs;
use std::path::Path;

use review_bundle::dispatch::{
    DEEP_GENERALISTS, FIXED_SPECIALISTS, GENERALISTS, MODEL, model_specialist, section_specialist,
};
use sha2::{Digest, Sha256};

/// The orchestrator, the review skill the workflow's agent follows.
pub const ORCHESTRATOR: &str = "review";
/// The orchestrator's file, relative to the tree.
pub const ORCHESTRATOR_FILE: &str = ".claude/skills/review/SKILL.md";
/// The model and effort the review workflow runs the orchestrator on.
pub const ORCHESTRATOR_MODEL: &str = "claude-sonnet-5-5";
pub const ORCHESTRATOR_EFFORT: &str = "low";
/// The orchestrator's turn budget for a standard review, before the
/// consulted agents add theirs.
pub const ORCHESTRATOR_TURNS: usize = 20;
/// A generalist's turn budget when its front matter names none.
pub const GENERALIST_TURNS: usize = 40;
/// The model a consultant runs on when its front matter names none:
/// the skills that consult one run it on the session's model, so the
/// evaluation holds it to the model the review workflow runs on.
pub const CONSULTANT_MODEL: &str = ORCHESTRATOR_MODEL;
/// The directory of the agents, relative to the tree.
const AGENTS: &str = ".claude/agents";
/// The prefixes of the specialists the dispatch instantiates per SRS
/// section and per domain-model file.
const SECTION_PREFIX: &str = "srs-";
const MODEL_PREFIX: &str = "domain-model-";
/// The sentences of the shared report sections whose specialists write
/// one finding per line in a fixed shape: the earlier wording and the
/// two of the current ones, which a file may wrap anywhere.
const ONE_LINE_EACH: [&str; 3] = [
    "each on one line",
    "every line of the report is a finding or the outcome",
    "every line is a finding or the outcome",
];

/// What a component is in the review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Orchestrator,
    Generalist,
    Specialist,
    /// An agent under `.claude/agents` the review bundle never
    /// dispatches: a skill consults it on a change, or a developer
    /// does. It answers in text, as a generalist does, and writes
    /// nothing.
    Consultant,
}

impl Kind {
    /// The word for it.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Orchestrator => "orchestrator",
            Self::Generalist => "generalist",
            Self::Specialist => "specialist",
            Self::Consultant => "consultant",
        }
    }
}

/// One reviewer component, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub kind: Kind,
    /// Its file, relative to the tree.
    pub file: String,
    pub text: String,
    /// The model its front matter names, or the workflow's for the
    /// orchestrator.
    pub model: String,
    pub effort: String,
    pub max_turns: usize,
    pub tools: Vec<String>,
    /// Whether its report holds one finding per line in the fixed
    /// shape, which the grader then holds it to.
    pub strict_lines: bool,
}

/// Every component's name: the orchestrator, the generalists, their
/// deep instances, the fixed specialists, a specialist per SRS section
/// and domain-model file under `root`, and every other agent under
/// `.claude/agents` there; sorted.
///
/// # Errors
///
/// Fails when the sections, the model files or the agents cannot be
/// listed.
pub fn names(root: &Path) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = vec![ORCHESTRATOR.to_owned()];
    out.extend(GENERALISTS.iter().map(|n| (*n).to_owned()));
    out.extend(DEEP_GENERALISTS.iter().map(|n| (*n).to_owned()));
    out.extend(FIXED_SPECIALISTS.iter().map(|(n, _)| (*n).to_owned()));
    for path in markdown_files(root, "docs/srs")? {
        out.extend(section_specialist(&path));
    }
    for path in markdown_files(root, MODEL.trim_end_matches('/'))? {
        let file = path
            .strip_prefix(MODEL)
            .and_then(|f| f.strip_suffix(".md"))
            .unwrap_or_default();
        out.extend(model_specialist(file));
    }
    for path in markdown_files(root, AGENTS)? {
        let name = path
            .strip_prefix(AGENTS)
            .and_then(|f| f.strip_prefix('/'))
            .and_then(|f| f.strip_suffix(".md"))
            .unwrap_or_default();
        out.push(name.to_owned());
    }
    out.sort();
    out.dedup();

    Ok(out)
}

/// The Markdown files directly under `dir` of `root`, as paths relative
/// to the root, sorted.
fn markdown_files(root: &Path, dir: &str) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(root.join(dir))
        .and_then(|entries| entries.collect::<Result<Vec<_>, _>>())
        .map_err(|e| format!("read {dir}: {e}"))?;
    let mut out = Vec::new();
    for name in entries.iter().map(fs::DirEntry::file_name) {
        let name = name
            .into_string()
            .map_err(|n| format!("{dir}/{} is not UTF-8", n.to_string_lossy()))?;
        if name.ends_with(".md") {
            out.push(format!("{dir}/{name}"));
        }
    }
    out.sort();

    Ok(out)
}

/// What `name` is in the review: a specialist when the dispatch can
/// name it, a consultant when it cannot.
#[must_use]
pub fn kind_of(name: &str) -> Kind {
    if name == ORCHESTRATOR {
        Kind::Orchestrator
    } else if GENERALISTS.contains(&name) || DEEP_GENERALISTS.contains(&name) {
        Kind::Generalist
    } else if FIXED_SPECIALISTS.iter().any(|(n, _)| *n == name)
        || name.starts_with(SECTION_PREFIX)
        || name.starts_with(MODEL_PREFIX)
    {
        Kind::Specialist
    } else {
        Kind::Consultant
    }
}

/// The component whose cases `name` runs: itself, or for a deep
/// instance the generalist it is an instance of.
#[must_use]
pub fn suite_of(name: &str) -> String {
    GENERALISTS
        .iter()
        .zip(DEEP_GENERALISTS)
        .find(|(_, deep)| *deep == name)
        .map_or(name, |(normal, _)| normal)
        .to_owned()
}

/// `name`'s file, relative to the tree.
#[must_use]
pub fn file_of(name: &str) -> String {
    if name == ORCHESTRATOR {
        ORCHESTRATOR_FILE.to_owned()
    } else {
        format!("{AGENTS}/{name}.md")
    }
}

/// A short hash of `text`, to say what was evaluated.
#[must_use]
pub fn hash(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// The front matter keys a reviewer agent may carry, and the keys the
/// orchestrator's skill may. Any other key, such as `hooks`,
/// `mcpServers` or `permissionMode`, could start a process or widen the
/// agent's reach when the harness runs the definition under test with
/// the review's token, so the harness refuses it before any run (I4).
pub const AGENT_KEYS: [&str; 7] = [
    "name",
    "description",
    "tools",
    "model",
    "effort",
    "maxTurns",
    "generalist",
];
pub const SKILL_KEYS: [&str; 2] = ["name", "description"];
/// The tools a specialist may list, and the tools a generalist or a
/// consultant may: those two write nothing.
pub const SPECIALIST_TOOL_SET: [&str; 4] = ["Read", "Grep", "Glob", "Write"];
pub const GENERALIST_TOOL_SET: [&str; 3] = ["Read", "Grep", "Glob"];

/// Reads component `name` from the tree at `tree`.
///
/// # Errors
///
/// Fails when its file cannot be read, its front matter does not read,
/// names a key or a tool the component may not carry, or names no
/// model while the component is no consultant, or its turn budget is
/// not a number.
pub fn load(tree: &Path, name: &str) -> Result<Component, String> {
    let file = file_of(name);
    let text = fs::read_to_string(tree.join(&file)).map_err(|e| format!("read {file}: {e}"))?;
    let kind = kind_of(name);
    let strict_lines = one_finding_per_line(&text);
    let (fields, _) = adr::front_matter(&text).map_err(|e| format!("{file}: {e}"))?;
    let keys: &[&str] = if kind == Kind::Orchestrator {
        &SKILL_KEYS
    } else {
        &AGENT_KEYS
    };
    allowed_keys(&file, &fields, keys)?;
    if kind == Kind::Orchestrator {
        return Ok(Component {
            name: name.to_owned(),
            kind,
            file,
            text,
            model: ORCHESTRATOR_MODEL.to_owned(),
            effort: ORCHESTRATOR_EFFORT.to_owned(),
            max_turns: ORCHESTRATOR_TURNS,
            tools: vec![
                "Read".into(),
                "Glob".into(),
                "Grep".into(),
                "Agent".into(),
                "Write".into(),
            ],
            strict_lines,
        });
    }
    let field = |key: &str| fields.get(key).map_or("", String::as_str).to_owned();
    let model = match (kind, field("model")) {
        (Kind::Consultant, model) if model.is_empty() => CONSULTANT_MODEL.to_owned(),
        (_, model) if model.is_empty() => return Err(format!("{file}: names no model")),
        (_, model) => model,
    };
    let max_turns = match field("maxTurns").as_str() {
        "" => GENERALIST_TURNS,
        turns => turns
            .parse()
            .map_err(|_| format!("{file}: maxTurns {turns} is not a number"))?,
    };

    let tools: Vec<String> = field("tools")
        .split(',')
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
        .collect();
    allowed_tools(&file, kind, &tools)?;

    Ok(Component {
        name: name.to_owned(),
        kind,
        file,
        text,
        model,
        effort: field("effort"),
        max_turns,
        tools,
        strict_lines,
    })
}

/// Whether `text` holds a sentence that has a report hold one finding
/// per line, however the file wraps it.
fn one_finding_per_line(text: &str) -> bool {
    let flat = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    ONE_LINE_EACH.iter().any(|marker| flat.contains(marker))
}

/// Refuses a front matter key of `file` outside `allowed`.
fn allowed_keys(
    file: &str,
    fields: &std::collections::BTreeMap<String, String>,
    allowed: &[&str],
) -> Result<(), String> {
    match fields.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => Err(format!(
            "{file}: front matter names {key}, which a reviewer component may not"
        )),
        None => Ok(()),
    }
}

/// Refuses a tool of `file` that a component of `kind` may not use.
fn allowed_tools(file: &str, kind: Kind, tools: &[String]) -> Result<(), String> {
    let allowed: &[&str] = match kind {
        Kind::Generalist | Kind::Consultant => &GENERALIST_TOOL_SET,
        Kind::Specialist | Kind::Orchestrator => &SPECIALIST_TOOL_SET,
    };
    match tools.iter().find(|t| !allowed.contains(&t.as_str())) {
        Some(tool) => Err(format!(
            "{file}: lists tool {tool}, which a {} may not",
            kind.word()
        )),
        None => Ok(()),
    }
}

/// Every component of `root`'s review, read from the tree at `tree`.
///
/// # Errors
///
/// Fails as [`names`] and [`load`] do.
pub fn catalog(root: &Path, tree: &Path) -> Result<Vec<Component>, String> {
    names(root)?.iter().map(|name| load(tree, name)).collect()
}

#[cfg(test)]
mod tests;
