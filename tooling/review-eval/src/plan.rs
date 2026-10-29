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
/// # Errors
///
/// Fails when the head's components cannot be listed or a case
/// directory cannot be read.
pub fn between(base: &Path, head: &Path) -> Result<Vec<Planned>, String> {
    let mut out = Vec::new();
    for name in component::names(head)? {
        let file = file_of(&name);
        let reason = match (
            fs::read_to_string(base.join(&file)).ok(),
            fs::read_to_string(head.join(&file)).ok(),
        ) {
            (None, Some(_)) => Some("new component".to_owned()),
            (Some(before), Some(after)) if before != after => {
                Some("instructions changed".to_owned())
            }
            _ => None,
        };
        let suite = suite_of(&name);
        let cases = format!("{CASES}/{suite}");
        let reason = match reason {
            Some(reason) => Some(reason),
            None if files(&base.join(&cases))? != files(&head.join(&cases))? => {
                Some("cases changed".to_owned())
            }
            None => None,
        };
        if let Some(reason) = reason {
            out.push(Planned {
                component: name,
                reason,
            });
        }
    }

    Ok(out)
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
