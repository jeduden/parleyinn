//! The files a pull request adds, put into a world. A world is main's
//! tree with the component under test copied in, so a file the pull
//! request adds beside its component, a section a new specialist is
//! told to check, was not there: the specialist looked for it outside
//! what a run may read, and its case failed for it. Each file the
//! evaluated tree tracks and main lacks goes into both trees of the
//! world, as both hold it once the pull request is on main. An
//! instruction file among them goes into the pull request's tree alone,
//! where it moves out as text with the others, so nothing the pull
//! request wrote loads beside the agent but the component under test.
//! A file main holds stays as main has it.

use std::fs;
use std::path::{Component, Path};

use crate::host::{Command, Host};
use crate::world::INSTRUCTION_NAMES;

/// Copies each file the tree at `tree` tracks and `main` lacks into
/// `pr_head`, and into `main` too unless it is an instruction file.
/// One that is no regular file inside the tree, a link or reached
/// through one, is left out and named in `notes`.
///
/// # Errors
///
/// Fails when the tree's files cannot be listed or a file cannot be
/// copied.
pub fn copy(
    host: &dyn Host,
    tree: &Path,
    main: &Path,
    pr_head: &Path,
    notes: &mut Vec<String>,
) -> Result<(), String> {
    for rel in tracked(host, tree)? {
        let inside = Path::new(&rel)
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
        if inside && main.join(&rel).symlink_metadata().is_ok() {
            continue;
        }
        if !inside || !regular(tree, Path::new(&rel)) {
            notes.push(format!(
                "left out {rel}, which the evaluated tree adds: it is no regular file inside that tree"
            ));
            continue;
        }
        let from = tree.join(&rel);
        place(&from, &rel, pr_head)?;
        if !instruction(&rel) {
            place(&from, &rel, main)?;
        }
    }

    Ok(())
}

/// Whether `rel` names a regular file under `tree` that no link leads
/// to: each directory on its way there is one, not a link to another.
fn regular(tree: &Path, rel: &Path) -> bool {
    let is = |path: &Path, kind: fn(&fs::Metadata) -> bool| {
        tree.join(path).symlink_metadata().is_ok_and(|m| kind(&m))
    };

    is(rel, fs::Metadata::is_file)
        && rel
            .ancestors()
            .skip(1)
            .all(|dir| is(dir, fs::Metadata::is_dir))
}

/// The files git tracks in the tree at `tree`, relative to it.
fn tracked(host: &dyn Host, tree: &Path) -> Result<Vec<String>, String> {
    let out = host.run(&Command::new("git", &["ls-files", "-z"], tree))?;
    if !out.ok() {
        return Err(format!(
            "list the files of {}: {}",
            tree.display(),
            out.stderr.trim()
        ));
    }

    Ok(out
        .stdout
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Whether `rel` is an instruction file or lies under one, at any depth.
fn instruction(rel: &str) -> bool {
    Path::new(rel)
        .components()
        .any(|c| INSTRUCTION_NAMES.iter().any(|n| c.as_os_str() == *n))
}

/// Copies `from` to `rel` under `root`, making its directories.
fn place(from: &Path, rel: &str, root: &Path) -> Result<(), String> {
    let to = root.join(rel);
    to.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::copy(from, &to).map(|_| ()))
        .map_err(|e| format!("copy {rel} to {}: {e}", root.display()))
}

#[cfg(test)]
mod tests;
