//! A case: one change to main's tree, as edits to its files, with the
//! report the reviewer component under evaluation must give for it: a
//! specialist's as the review assembles it from its run log.
//! Cases are TOML files under `tooling/review-eval/cases/<component>/`,
//! one per file; the section and domain-model specialists' cases are
//! derived from the tree instead (`derived`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::diff::Change;
use crate::{PASSES, REPETITIONS};

/// What a case is for: the kinds every component must have one of,
/// and `other` for the rest.
pub const KINDS: [&str; 4] = ["defect", "clean", "injection", "other"];

/// The outcomes a specialist's report ends with.
pub const OUTCOMES: [&str; 2] = crate::report::OUTCOMES;

/// The outcomes a case written while specialists weighed their own
/// findings names, each with the outcome it is read as: a sound report
/// held no finding, a report of nits or blocking findings held some.
pub const LEGACY_OUTCOMES: [(&str, &str); 3] = [
    ("sound", crate::report::OUTCOMES[1]),
    ("nits", crate::report::OUTCOMES[0]),
    ("blocking", crate::report::OUTCOMES[0]),
];

/// The severity a case written while specialists weighed names for a
/// specialist's linked finding.
pub const BLOCKING: &str = "blocking";

/// The form of a specialist's finding linked to its requirements.
pub const LINKED: &str = "linked";
/// The form of a specialist's requirement request.
pub const REQUEST: &str = "request";
/// The forms of a specialist's finding line a case may name.
pub const FORMS: [&str; 2] = [LINKED, REQUEST];

/// One edit to a file of main's tree, its path relative to the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Creates `file`, which must not exist, holding `content`.
    Create { file: String, content: String },
    /// Swaps the first `old` in `file` for `new`.
    Replace {
        file: String,
        old: String,
        new: String,
    },
    /// Deletes `file`.
    Remove { file: String },
}

impl Edit {
    /// The file the edit touches.
    #[must_use]
    pub fn file(&self) -> &str {
        match self {
            Self::Create { file, .. } | Self::Replace { file, .. } | Self::Remove { file } => file,
        }
    }
}

/// A finding the report must hold, or must not: matched against each
/// line of the report, which must name the path and hold every word of
/// `mentions`, one of `mentions_any`, each id, and the source, where
/// the case names them; a line number within `within` of `line` where
/// the case names one; and, for a generalist, the severity word it
/// gives. A specialist gives no severity: `form` names the form of its
/// line, `linked` or `request`, and a case written before reads its
/// `severity`, `blocking` as a linked finding and `nit` as either form.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Expected {
    pub path: String,
    pub severity: Option<String>,
    pub form: Option<String>,
    pub line: Option<usize>,
    pub within: Option<usize>,
    pub mentions: Vec<String>,
    pub mentions_any: Vec<String>,
    pub ids: Vec<String>,
    pub source: Option<String>,
}

/// What the report must say.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Expect {
    /// The outcomes the report may end with, each a word of
    /// [`OUTCOMES`] once the case is read.
    #[serde(deserialize_with = "one_or_many")]
    pub outcome: Vec<String>,
    /// Findings the report must hold.
    pub finding: Vec<Expected>,
    /// Findings the report must not hold.
    pub none: Vec<Expected>,
    /// Findings a generalist must drop, each named by the words of its
    /// dropped line.
    pub dropped: Vec<Expected>,
    /// Findings a specialist must state in its text before its final
    /// answer, as it finds them, not only at the end.
    pub stated: Vec<Expected>,
    /// Findings a specialist must withdraw or revise in its text, each
    /// named by the place its `withdrawn:` or `revised:` line names.
    pub withdrawn: Vec<Expected>,
    /// Text the report must not hold anywhere.
    pub absent: Vec<String>,
    /// The most findings a generalist may keep as nits, when given.
    pub nits: Option<usize>,
    /// Whether the run must read and search the repository's files in
    /// the pull request's tree alone: one it opens in main's tree, the
    /// work directory's own root, fails the run.
    pub main_unread: bool,
}

/// A specialist's report the fixture seeds for a generalist's case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    pub agent: String,
    pub content: String,
}

/// One case, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    /// The component the case evaluates, which names its suite.
    pub component: String,
    /// The file's stem.
    pub name: String,
    pub kind: String,
    pub title: String,
    /// A stakeholder comment on the pull request, when the case grants
    /// consent.
    pub consent: Option<String>,
    pub edits: Vec<Edit>,
    pub seeds: Vec<Seed>,
    pub expect: Expect,
    pub repetitions: usize,
    pub passes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    file: String,
    content: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replace {
    file: String,
    old: String,
    new: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Remove {
    file: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Runs {
    #[serde(default = "repetitions")]
    repetitions: usize,
    #[serde(default = "passes")]
    passes: usize,
}

fn repetitions() -> usize {
    REPETITIONS
}

fn passes() -> usize {
    PASSES
}

/// The file as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    component: String,
    kind: String,
    title: String,
    consent: Option<String>,
    #[serde(default)]
    create: Vec<Create>,
    #[serde(default)]
    replace: Vec<Replace>,
    #[serde(default)]
    remove: Vec<Remove>,
    #[serde(default)]
    report: Vec<Seed>,
    runs: Option<Runs>,
    expect: Expect,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

fn one_or_many<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(v) => v,
    })
}

impl Case {
    /// Reads the case `name` of `component` from `text`.
    ///
    /// # Errors
    ///
    /// Fails, naming the case, when the TOML does not read, names
    /// another component, has an unknown kind, title, outcome or
    /// repetition rule, or makes no edit.
    pub fn parse(component: &str, name: &str, text: &str) -> Result<Self, String> {
        let at = format!("case {component}/{name}");
        let file: File = toml::from_str(text).map_err(|e| format!("{at}: {}", e.message()))?;
        if file.component != component {
            return Err(format!(
                "{at}: names component {}, but lies under {component}",
                file.component
            ));
        }
        if !KINDS.contains(&file.kind.as_str()) {
            return Err(format!(
                "{at}: kind {} is none of {}",
                file.kind,
                KINDS.join(", ")
            ));
        }
        if file.title.trim().is_empty() {
            return Err(format!("{at}: has no title"));
        }
        let mut edits: Vec<Edit> = file
            .create
            .into_iter()
            .map(|c| Edit::Create {
                file: c.file,
                content: c.content,
            })
            .collect();
        edits.extend(file.replace.into_iter().map(|r| Edit::Replace {
            file: r.file,
            old: r.old,
            new: r.new,
        }));
        edits.extend(
            file.remove
                .into_iter()
                .map(|r| Edit::Remove { file: r.file }),
        );
        if edits.is_empty() {
            return Err(format!("{at}: makes no edit"));
        }
        let mut expect = file.expect;
        expect.outcome = outcomes(&expect.outcome).map_err(|e| format!("{at}: {e}"))?;
        if let Some(bad) = [
            &expect.finding,
            &expect.none,
            &expect.dropped,
            &expect.stated,
            &expect.withdrawn,
        ]
        .into_iter()
        .flatten()
        .filter_map(|e| e.form.as_deref())
        .find(|f| !FORMS.contains(f))
        {
            return Err(format!("{at}: form {bad} is none of {}", FORMS.join(", ")));
        }
        let (repetitions, passes) = file
            .runs
            .map_or((REPETITIONS, PASSES), |r| (r.repetitions, r.passes));
        if passes == 0 || passes > repetitions {
            return Err(format!(
                "{at}: must pass {passes} of {repetitions} runs, which is no rule"
            ));
        }

        Ok(Self {
            component: component.to_owned(),
            name: name.to_owned(),
            kind: file.kind,
            title: file.title,
            consent: file.consent,
            edits,
            seeds: file.report,
            expect,
            repetitions,
            passes,
        })
    }

    /// `component/name`.
    #[must_use]
    pub fn id(&self) -> String {
        format!("{}/{}", self.component, self.name)
    }

    /// Each file the edits change, from its text on main, which `read`
    /// gives, to its text in the fixture: every edit applied in order,
    /// creates first, then replacements, then removals.
    ///
    /// # Errors
    ///
    /// Fails, naming the case and the file, when a file to create
    /// exists, a text to replace is not in the file, or a file to
    /// replace in or remove does not exist.
    pub fn changes(&self, read: &dyn Fn(&str) -> Option<String>) -> Result<Vec<Change>, String> {
        let mut texts: BTreeMap<&str, (Option<String>, Option<String>)> = BTreeMap::new();
        for edit in &self.edits {
            let file = edit.file();
            let entry = texts.entry(file).or_insert_with(|| {
                let old = read(file);
                (old.clone(), old)
            });
            let at = format!("{}: {file}", self.id());
            match (edit, &entry.1) {
                (Edit::Create { .. }, Some(_)) => return Err(format!("{at}: exists already")),
                (Edit::Create { content, .. }, None) => entry.1 = Some(content.clone()),
                (Edit::Replace { old, new, .. }, Some(text)) => {
                    if old.is_empty() || !text.contains(old.as_str()) {
                        return Err(format!("{at}: does not contain {old:?}"));
                    }
                    entry.1 = Some(text.replacen(old.as_str(), new, 1));
                }
                (Edit::Remove { .. }, Some(_)) => entry.1 = None,
                (Edit::Replace { .. } | Edit::Remove { .. }, None) => {
                    return Err(format!("{at}: does not exist"));
                }
            }
        }

        Ok(texts
            .into_iter()
            .map(|(path, (old, new))| Change {
                path: path.to_owned(),
                old,
                new,
            })
            .collect())
    }

    /// Checks the edits against the tree at `root` without changing it.
    ///
    /// # Errors
    ///
    /// Fails as [`Case::changes`] does.
    pub fn validate(&self, root: &Path) -> Result<(), String> {
        self.changes(&|file| fs::read_to_string(root.join(file)).ok())
            .map(drop)
    }

    /// Makes the edits in the tree at `root`.
    ///
    /// # Errors
    ///
    /// Fails as [`Case::changes`] does, or when a file cannot be
    /// written or removed.
    pub fn apply(&self, root: &Path) -> Result<Vec<Change>, String> {
        let changes = self.changes(&|file| fs::read_to_string(root.join(file)).ok())?;
        for change in &changes {
            let path = root.join(&change.path);
            let at = format!("{}: {}", self.id(), change.path);
            let written = match &change.new {
                Some(text) => path
                    .parent()
                    .map_or(Ok(()), fs::create_dir_all)
                    .and_then(|()| fs::write(&path, text)),
                None => fs::remove_file(&path),
            };
            written.map_err(|e| format!("{at}: {e}"))?;
        }

        Ok(changes)
    }
}

/// `words`, a case's outcomes, each read as a word of [`OUTCOMES`] and
/// named once, in order.
fn outcomes(words: &[String]) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for word in words {
        let read = OUTCOMES
            .iter()
            .find(|o| **o == word)
            .or_else(|| {
                LEGACY_OUTCOMES
                    .iter()
                    .find(|(old, _)| old == word)
                    .map(|(_, now)| now)
            })
            .ok_or_else(|| {
                let all: Vec<&str> = OUTCOMES
                    .into_iter()
                    .chain(LEGACY_OUTCOMES.map(|(old, _)| old))
                    .collect();
                format!("outcome {word} is none of {}", all.join(", "))
            })?;
        if !out.iter().any(|o| o == read) {
            out.push((*read).to_owned());
        }
    }

    Ok(out)
}

/// Every case under `dir`: `<component>/<name>.toml`, in name order.
/// A missing directory holds no case.
///
/// # Errors
///
/// Fails when a directory or file cannot be read, or a case does not
/// parse.
pub fn load_dir(dir: &Path) -> Result<Vec<Case>, String> {
    let mut out = Vec::new();
    let components = match fs::read_dir(dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        listed => listed.map_err(|e| format!("read {}: {e}", dir.display()))?,
    };
    let mut dirs: Vec<_> = components
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("read {}: {e}", dir.display()))?;
    dirs.sort();
    for component_dir in dirs.into_iter().filter(|p| p.is_dir()) {
        let component = component_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("{} is not UTF-8", component_dir.display()))?
            .to_owned();
        let mut files: Vec<_> = fs::read_dir(&component_dir)
            .and_then(|entries| {
                entries
                    .map(|e| e.map(|e| e.path()))
                    .collect::<Result<_, _>>()
            })
            .map_err(|e| format!("read {}: {e}", component_dir.display()))?;
        files.sort();
        for file in files
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        {
            let name = file
                .file_stem()
                .and_then(|n| n.to_str())
                .ok_or_else(|| format!("{} is not UTF-8", file.display()))?;
            let text =
                fs::read_to_string(&file).map_err(|e| format!("read {}: {e}", file.display()))?;
            out.push(Case::parse(&component, name, &text)?);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
