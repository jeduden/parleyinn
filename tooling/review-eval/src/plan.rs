//! What a pull request's evaluation runs: every component whose file,
//! or whose suite's cases, differ between main's tree and the pull
//! request's, each with its reason. The shared sections are expanded
//! into each component's file, so a change to one reaches every
//! component that includes it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use review_bundle::owners;
use serde::{Deserialize, Serialize};

use crate::CASES;
use crate::component::{self, file_of, suite_of};

/// Where a tree names the stakeholder's paths.
const CODEOWNERS: &str = ".github/CODEOWNERS";

/// One component to run, and why.
///
/// Main's cases judge a component main already has, so a change cannot
/// soften the cases that judge it. The exception is the stakeholder's
/// to make (ENG-21): a case file the pull request changes, adds or
/// removes is judged as the pull request has it once a stakeholder
/// comment names it. [`consented`] sorts the case files that differ
/// into the two lists; both stay empty for a component main holds no
/// case for, which its own cases judge.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planned {
    pub component: String,
    pub reason: String,
    /// The case files of its suite that differ and that a stakeholder
    /// comment names: the pull request's version of each judges.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consented: Vec<String>,
    /// Those no stakeholder comment names: main's version of each
    /// judges.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unconsented: Vec<String>,
}

impl Planned {
    /// `component`, planned for `reason`, with no case file sorted yet.
    #[must_use]
    pub fn new(component: String, reason: String) -> Self {
        Self {
            component,
            reason,
            ..Self::default()
        }
    }

    /// What the plan says of its sorted case files, a line per list
    /// that holds one: which the pull request's version judges, and
    /// which main's.
    #[must_use]
    pub fn judged(&self) -> Vec<String> {
        let line = |how: &str, files: &[String]| {
            (!files.is_empty()).then(|| format!("{how}: {}", files.join(", ")))
        };
        let theirs = "the pull request's version judges, a stakeholder comment names it";
        let mains = "main's version judges, no stakeholder comment names it";

        [
            line(theirs, &self.consented),
            line(mains, &self.unconsented),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// The components to run for a pull request whose tree is `head`,
/// against main's tree `base`, in name order.
///
/// A component whose own text or cases changed is always planned. One
/// whose file differs only inside the bodies of the shared sections it
/// includes is planned through one component per distinct set of
/// sections, the first by name, which stands for the rest: they hold the
/// same text, and the scheduled run covers each of them.
///
/// # Errors
///
/// Fails when the head's components cannot be listed or a case
/// directory cannot be read.
pub fn between(base: &Path, head: &Path) -> Result<Vec<Planned>, String> {
    let mut out = Vec::new();
    let mut shared: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    for name in component::names(head)? {
        let file = file_of(&name);
        let (before, after) = (
            fs::read_to_string(base.join(&file)).ok(),
            fs::read_to_string(head.join(&file)).ok(),
        );
        let own = match (&before, &after) {
            (None, Some(_)) => Some("new component"),
            (Some(b), Some(a)) if b != a && strip_includes(b) != strip_includes(a) => {
                Some("instructions changed")
            }
            _ => None,
        };
        let suite = suite_of(&name);
        let cases = format!("{CASES}/{suite}");
        let reason = match own {
            Some(reason) => Some(reason),
            None if files(&base.join(&cases))? != files(&head.join(&cases))? => {
                Some("cases changed")
            }
            None => None,
        };
        match (reason, before, after) {
            (Some(reason), ..) => out.push(Planned::new(name, reason.to_owned())),
            (None, Some(b), Some(a)) if b != a => {
                shared.entry(sections_of(&a)).or_default().push(name);
            }
            _ => {}
        }
    }
    for names in shared.into_values() {
        let (first, rest) = names.split_first().map_or(("", &[][..]), |(f, r)| (f, r));
        out.push(Planned::new(
            first.to_owned(),
            if rest.is_empty() {
                "shared sections changed".to_owned()
            } else {
                format!("shared sections changed; also covers {}", rest.join(", "))
            },
        ));
    }
    out.sort_by(|a, b| a.component.cmp(&b.component));

    Ok(out)
}

/// `planned`, each with the case files of its suite that differ between
/// main's tree `base` and the pull request's `head` sorted by consent:
/// those a comment of `consents` names, by the patterns `base`'s
/// CODEOWNERS assigns (ENG-21), and those none names.
///
/// # Errors
///
/// Fails when `base`'s CODEOWNERS or a case directory cannot be read.
pub fn consented(
    base: &Path,
    head: &Path,
    planned: Vec<Planned>,
    consents: &[String],
) -> Result<Vec<Planned>, String> {
    let owners_file = base.join(CODEOWNERS);
    let patterns = match fs::read_to_string(&owners_file) {
        Ok(text) => owners::patterns(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(format!("read {}: {e}", owners_file.display())),
    };
    planned
        .into_iter()
        .map(|mut p| {
            let dir = format!("{CASES}/{}", suite_of(&p.component));
            let named = |file: &String| {
                let path = format!("{dir}/{file}");
                consents.iter().any(|c| owners::names(&patterns, c, &path))
            };
            (p.consented, p.unconsented) = changed_cases(&base.join(&dir), &head.join(&dir))?
                .into_iter()
                .partition(named);

            Ok(p)
        })
        .collect()
}

/// The case files that differ between main's directory of a suite,
/// `base`, and the pull request's, `head`: changed, added or removed,
/// by name, sorted. None when main holds no case there: the suite is
/// new, and its own cases judge it.
fn changed_cases(base: &Path, head: &Path) -> Result<Vec<String>, String> {
    let case = |name: &&String| name.ends_with(".toml");
    let (before, after) = (files(base)?, files(head)?);
    if !before.keys().any(|name| case(&name)) {
        return Ok(Vec::new());
    }
    let names: BTreeSet<&String> = before.keys().chain(after.keys()).filter(case).collect();

    Ok(names
        .into_iter()
        .filter(|name| before.get(*name) != after.get(*name))
        .cloned()
        .collect())
}

/// What an evaluation of some cases costs, from the last full evaluation:
/// a case took 2.1 runs, a run 21 seconds and about 4 cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Estimate {
    pub runs: usize,
    pub minutes: usize,
    pub cents: usize,
}

/// The estimate for `cases` cases run `parallel` at a time.
#[must_use]
pub fn estimate(cases: usize, parallel: usize) -> Estimate {
    let runs = (cases * 21).div_ceil(10);

    Estimate {
        runs,
        minutes: (runs * 21 / parallel.max(1)).div_ceil(60),
        cents: runs * 4,
    }
}

impl std::fmt::Display for Estimate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "about {} runs, about {} min, about ${}.{:02}",
            self.runs,
            self.minutes,
            self.cents / 100,
            self.cents % 100
        )
    }
}

/// `text` without the expanded bodies of its `<?include` sections: the
/// directive and its closing line stay, so what is left is the
/// component's own text.
fn strip_includes(text: &str) -> String {
    let (mut out, mut header, mut body) = (String::new(), false, false);
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if body && trimmed != "<?/include?>" {
            continue;
        }
        body = false;
        if trimmed.starts_with("<?include") {
            header = true;
        } else if header && trimmed == "?>" {
            header = false;
            body = true;
        }
        out.push_str(line);
    }

    out
}

/// The files `text` includes, each once, sorted.
fn sections_of(text: &str) -> Vec<String> {
    let (mut files, mut header) = (BTreeSet::new(), false);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<?include") {
            header = true;
        } else if header && trimmed == "?>" {
            header = false;
        } else if let Some(file) = header.then(|| trimmed.strip_prefix("file:")).flatten() {
            files.insert(file.trim().to_owned());
        }
    }

    files.into_iter().collect()
}

/// Each file directly under `dir` with its text; nothing for a missing
/// directory.
fn files(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let entries = match fs::read_dir(dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        listed => listed.map_err(|e| format!("read {}: {e}", dir.display()))?,
    };
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", dir.display()))?;
        if entry.path().is_file() {
            let text = fs::read_to_string(entry.path())
                .map_err(|e| format!("read {}: {e}", entry.path().display()))?;
            out.insert(entry.file_name().to_string_lossy().into_owned(), text);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
