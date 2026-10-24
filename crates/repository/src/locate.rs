//! Finding the git directory of the repository a directory is in.

use std::fs::{self, Metadata};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::fsio::read_limited;

/// The largest `.git` file read.
const MAX_GITDIR_FILE: u64 = 4096;

const BAD_GITDIR: Error = Error::Corrupt("gitdir file names no directory");

/// Whether `path` lies where git keeps a linked worktree's or a
/// submodule's git directory: below `.git/worktrees` or `.git/modules`.
/// Any other directory a `.git` file names, an unrelated repository's
/// included, is not followed, so a clone cannot bind someone else's
/// commit as its identity.
fn is_linked_git_dir(path: &Path) -> bool {
    let parts: Vec<_> = path.components().map(|c| c.as_os_str()).collect();

    parts
        .windows(3)
        .any(|w| w[0] == ".git" && (w[1] == "worktrees" || w[1] == "modules") && w[2] != "..")
}

/// The directory a `.git` file in `dir` names with `gitdir: <path>`.
fn gitdir_file(dir: &Path) -> Result<PathBuf, Error> {
    let bytes = read_limited(dir, ".git", MAX_GITDIR_FILE)?.unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    let target = text
        .lines()
        .next()
        .and_then(|line| line.trim().strip_prefix("gitdir:"))
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .ok_or(BAD_GITDIR)?;
    let path = dir.join(target);
    if !is_linked_git_dir(&path) {
        return Err(Error::Unsupported(
            "gitdir that is no linked worktree's or submodule's",
        ));
    }
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_symlink() => {
            Err(Error::Unsupported("symbolic link as a git directory"))
        }
        Ok(meta) if meta.is_dir() => Ok(path),
        Ok(_) => Err(BAD_GITDIR),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(BAD_GITDIR),
        Err(e) => Err(Error::io("a git directory")(e)),
    }
}

/// The git directory a `.git` entry of `dir`, with `meta`, stands for.
fn from_dot_git(dir: &Path, dot_git: &Path, meta: &Metadata) -> Result<Option<PathBuf>, Error> {
    if meta.is_symlink() {
        Err(Error::Unsupported("symbolic link as .git"))
    } else if meta.is_dir() {
        Ok(Some(dot_git.to_path_buf()))
    } else if meta.is_file() {
        gitdir_file(dir).map(Some)
    } else {
        Ok(None)
    }
}

/// The git directory of the repository `directory` is in, found the way
/// git finds it: `.git` in `directory` or in the nearest directory
/// above it. `None` when no directory above has one.
///
/// A `.git` that is a directory is the git directory. One that is a
/// file holds `gitdir: <path>`, as a linked worktree's and a
/// submodule's do; the path, relative to the directory the file is in,
/// is the git directory. A symbolic link is never followed: a `.git`
/// that is one, or a `gitdir:` that names one, is refused, so a clone
/// cannot point this crate at files outside itself by that route. The
/// directories above `directory` are walked as the path spells them,
/// without resolving links.
///
/// The walk stops before `ceiling`, which callers set to the OS user's
/// home directory: the directory `ceiling` names is never looked in,
/// nor is anything above it, so a dotfiles repository at `~/.git` is
/// never taken for the repository of every directory below it. A
/// `ceiling` that is not above `directory` changes nothing. It is
/// compared as the path spells it, like the walk. `None` is for tests.
///
/// A `gitdir:` file's target is followed only as git's linked
/// worktrees and submodules need it: it must lie below a `.git/worktrees`
/// or a `.git/modules` directory, exist as a real directory and not be a
/// symbolic link. A separate git directory is refused. The git directory it
/// names is read by [`crate::root_commit`] and [`crate::has_commit`],
/// which hold its `commondir`, if it has one, to an ancestor of it or
/// itself, as a linked worktree's `../..` is, and refuse any other.
///
/// # Errors
///
/// [`Error::Unsupported`] for a symbolic link; [`Error::Corrupt`] for a
/// `.git` file that names no directory; [`Error::TooLarge`] for one
/// that is not a few bytes long; [`Error::Io`] when the file system
/// cannot say.
pub fn git_dir(directory: &Path, ceiling: Option<&Path>) -> Result<Option<PathBuf>, Error> {
    Ok(locate(directory, ceiling)?.map(|found| found.git_dir))
}

/// Where a repository's worktree and git directory are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    /// The directory holding `.git`: the root of the worktree, however
    /// deep below it the search began.
    pub worktree: PathBuf,
    /// The git directory, as [`git_dir`] finds it.
    pub git_dir: PathBuf,
}

/// [`git_dir`], with the worktree it was found for.
///
/// # Errors
///
/// As [`git_dir`].
pub fn locate(directory: &Path, ceiling: Option<&Path>) -> Result<Option<Located>, Error> {
    for dir in directory.ancestors() {
        if ceiling == Some(dir) {
            break;
        }
        let dot_git = dir.join(".git");
        match fs::symlink_metadata(&dot_git) {
            Ok(meta) => {
                if let Some(git_dir) = from_dot_git(dir, &dot_git, &meta)? {
                    return Ok(Some(Located {
                        worktree: dir.to_path_buf(),
                        git_dir,
                    }));
                }
            }
            Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {}
            Err(e) => return Err(Error::io("a git directory")(e)),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
