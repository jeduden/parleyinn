//! The recorded shapes, generated from the recordings (REC-26, ENG-17):
//! each version's transcripts give its shapes, and the file the
//! `transcript` crate compiles in must be exactly what they give.

use std::fs;
use std::path::{Path, PathBuf};

use transcript::Line;
use transcript::shape::{HARNESS, RecordedShapes};

use crate::recording;

/// Where the `transcript` crate keeps each version's recorded shapes,
/// relative to the repository.
pub const SHAPES_DIR: &str = "crates/transcript/shapes";

/// The transcripts `version` recorded: every `.jsonl` file under a
/// directory named `transcripts`, in path order.
///
/// # Errors
///
/// Fails when a directory cannot be listed.
pub fn transcripts(version: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(recording::files(version, ".jsonl")?
        .into_iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "transcripts"))
        .collect())
}

/// The shapes `version`'s transcripts show, named by the directory's
/// name.
///
/// # Errors
///
/// Fails when a transcript cannot be read or holds a line that is not
/// JSON.
pub fn generate(version: &Path) -> Result<RecordedShapes, String> {
    let mut lines: Vec<Line> = Vec::new();
    for path in transcripts(version)? {
        lines.extend(
            recording::lines(&path)?
                .into_iter()
                .filter_map(|v| v.as_object().cloned()),
        );
    }
    let name = version
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(RecordedShapes::generate(&name, &lines))
}

/// The file holding `version`'s recorded shapes under `shapes`.
#[must_use]
pub fn file(shapes: &Path, version: &str) -> PathBuf {
    shapes.join(HARNESS).join(format!("{version}.json"))
}

/// Regenerates every version under `recordings` and compares each with
/// its file under `shapes`, or, when `write`, writes it there. Returns
/// one problem per version whose file differs or is missing, and per
/// file no recorded version gives.
///
/// # Errors
///
/// Fails when a recording or a file cannot be read, or a file cannot
/// be written.
pub fn regenerate(recordings: &Path, shapes: &Path, write: bool) -> Result<Vec<String>, String> {
    regenerate_with(recordings, shapes, write, generate)
}

/// Where the `hook` crate keeps each version's hook input shapes,
/// relative to the repository.
pub const HOOK_SHAPES_DIR: &str = "crates/hook/shapes";

/// The hook input shapes `version`'s recorded hook inputs show, each
/// under its hook, named by the directory's name.
///
/// # Errors
///
/// Fails when a hook input cannot be read or is not JSON.
pub fn generate_hooks(version: &Path) -> Result<RecordedShapes, String> {
    let mut keyed = Vec::new();
    for path in recording::files(&version.join("hooks"), ".json")? {
        let value = recording::json(&path)?;
        let hook = value.get("hook_event_name").and_then(|h| h.as_str());
        if let (Some(hook), Some(object)) = (hook, value.as_object()) {
            keyed.push((hook.to_owned(), object.clone()));
        }
    }
    let name = version
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    Ok(RecordedShapes::generate_keyed(&name, &keyed))
}

/// [`regenerate`] for the hook input shapes under `shapes`.
///
/// # Errors
///
/// As [`regenerate`].
pub fn regenerate_hooks(
    recordings: &Path,
    shapes: &Path,
    write: bool,
) -> Result<Vec<String>, String> {
    regenerate_with(recordings, shapes, write, generate_hooks)
}

/// [`regenerate`] with each version's shapes given by `generate`.
fn regenerate_with(
    recordings: &Path,
    shapes: &Path,
    write: bool,
    generate: fn(&Path) -> Result<RecordedShapes, String>,
) -> Result<Vec<String>, String> {
    let mut problems = Vec::new();
    let mut versions = Vec::new();
    if write {
        let dir = shapes.join(HARNESS);
        fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    for dir in recording::versions(recordings)? {
        let generated = generate(&dir)?;
        let path = file(shapes, &generated.version);
        versions.push(path.clone());
        let text = generated.to_text();
        if write {
            fs::write(&path, &text).map_err(|e| format!("write {}: {e}", path.display()))?;
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(committed) if committed == text => {}
            Ok(_) => problems.push(format!(
                "{} differs from the shapes {} records; regenerate it",
                path.display(),
                generated.version
            )),
            Err(_) => problems.push(format!(
                "{} is missing for the recorded version {}",
                path.display(),
                generated.version
            )),
        }
    }
    for path in recording::files(&shapes.join(HARNESS), ".json")? {
        if !versions.contains(&path) {
            problems.push(format!(
                "{} has no recorded version under {}",
                path.display(),
                recordings.display()
            ));
        }
    }

    Ok(problems)
}

#[cfg(test)]
mod tests;
