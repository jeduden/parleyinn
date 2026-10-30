//! What a pull request's evaluation runs: every component whose file,
//! or whose suite's cases, differ between main's tree and the pull
//! request's, each with its reason. The shared sections are expanded
//! into each component's file, so a change to one reaches every
//! component that includes it.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::CASES;
use crate::component::{self, file_of, suite_of};

/// One component to run, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planned {
    pub component: String,
    pub reason: String,
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
            (Some(reason), ..) => out.push(Planned {
                component: name,
                reason: reason.to_owned(),
            }),
            (None, Some(b), Some(a)) if b != a => {
                shared.entry(sections_of(&a)).or_default().push(name);
            }
            _ => {}
        }
    }
    for names in shared.into_values() {
        let (first, rest) = names.split_first().map_or(("", &[][..]), |(f, r)| (f, r));
        out.push(Planned {
            component: first.to_owned(),
            reason: if rest.is_empty() {
                "shared sections changed".to_owned()
            } else {
                format!("shared sections changed; also covers {}", rest.join(", "))
            },
        });
    }
    out.sort_by(|a, b| a.component.cmp(&b.component));

    Ok(out)
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
    let (mut files, mut header) = (std::collections::BTreeSet::new(), false);
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
