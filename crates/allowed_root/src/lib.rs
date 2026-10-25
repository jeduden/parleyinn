//! The allowed roots (SEC-18). Every path ParleyInn receives from a hook
//! input, a transcript or configuration is resolved, symbolic links
//! included, and must lie within an allowed root once resolved: the
//! transcript roots (`transcript.roots`), the home, or, for reading
//! only, the run's worktree up to its repository's top level and the
//! git directory that worktree names. A path outside every root, or
//! one that leaves its root once its links are resolved, is refused.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// What a root allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Access {
    /// Reading only.
    Read,
    /// Reading and writing.
    ReadWrite,
}

/// A path refused: it does not resolve, or resolves outside every root
/// that allows the access asked. It names no path, so no audit entry
/// or message repeats what a hostile input chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathOutsideAllowedRoots;

impl fmt::Display for PathOutsideAllowedRoots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("path outside allowed roots")
    }
}

impl std::error::Error for PathOutsideAllowedRoots {}

/// A path resolved within an allowed root: every symbolic link in it
/// followed, so reading it reads no other file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedPath {
    path: PathBuf,
}

impl AllowedPath {
    /// The resolved path, to read or write instead of the one received.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The allowed roots, each resolved when it is added.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AllowedRoots {
    roots: Vec<(PathBuf, Access)>,
}

impl AllowedRoots {
    /// These roots and `root`, allowing `access`; a root that does not
    /// resolve allows nothing, so it is left out.
    #[must_use]
    pub fn with(mut self, root: &Path, access: Access) -> Self {
        if let Ok(resolved) = fs::canonicalize(root) {
            self.roots.push((resolved, access));
        }
        self
    }

    /// These roots and, for reading only, the worktree `cwd` lies in, up
    /// to its repository's top level, and the git directory it names;
    /// nothing when `cwd` lies in no repository.
    #[must_use]
    pub fn with_worktree(self, cwd: &Path) -> Self {
        let Some((top, git_dir)) = repository(cwd) else {
            return self;
        };

        self.with(&top, Access::Read).with(&git_dir, Access::Read)
    }

    /// `path` resolved, when it lies within a root allowing `access`.
    ///
    /// # Errors
    ///
    /// Fails when `path` does not resolve, or resolves outside every
    /// root that allows `access`.
    pub fn resolve(
        &self,
        path: &Path,
        access: Access,
    ) -> Result<AllowedPath, PathOutsideAllowedRoots> {
        let resolved = resolved(path).ok_or(PathOutsideAllowedRoots)?;
        let within = self
            .roots
            .iter()
            .any(|(root, allows)| *allows >= access && resolved.starts_with(root));
        if within {
            return Ok(AllowedPath { path: resolved });
        }

        Err(PathOutsideAllowedRoots)
    }
}

/// `path` with every symbolic link resolved; for one that does not
/// exist, not even as a dangling link, its parent resolved and its name
/// joined, so a missing file within a root is reported as missing, not
/// as outside.
fn resolved(path: &Path) -> Option<PathBuf> {
    match fs::canonicalize(path) {
        Ok(p) => Some(p),
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound && fs::symlink_metadata(path).is_err() =>
        {
            let name = path.file_name()?;
            let parent = fs::canonicalize(path.parent()?).ok()?;
            Some(parent.join(name))
        }
        Err(_) => None,
    }
}

/// The top level of the repository `cwd` lies in, and the git directory
/// it names: `.git` itself, or where a `.git` file's `gitdir:` line
/// points, relative to the top level when it is relative.
fn repository(cwd: &Path) -> Option<(PathBuf, PathBuf)> {
    let cwd = fs::canonicalize(cwd).ok()?;
    let top = cwd.ancestors().find(|d| d.join(".git").exists())?;
    let dot_git = top.join(".git");
    if dot_git.is_dir() {
        return Some((top.to_path_buf(), dot_git));
    }
    let text = fs::read_to_string(&dot_git).ok()?;
    let named = text.lines().find_map(|l| l.strip_prefix("gitdir:"))?.trim();
    match git_directory(top, &top.join(named)) {
        Some(git_dir) => Some((top.to_path_buf(), git_dir)),
        None => Some((top.to_path_buf(), dot_git)),
    }
}

/// The git directory `named` resolves to, judged after every symbolic
/// link and `..` in it is resolved: a directory holding `HEAD`, one of
/// whose parts is `.git` or ends in `.git`, as a repository's, a bare
/// one's, and a worktree's or submodule's inside them do, and no
/// ancestor of the worktree `top`. A `.git` file naming anything else,
/// as `gitdir: /`, a `*.git` link to the home or `sub.git/../..` would,
/// opens no root beyond the worktree.
fn git_directory(top: &Path, named: &Path) -> Option<PathBuf> {
    let resolved = fs::canonicalize(named).ok()?;
    let looks_like = resolved
        .components()
        .any(|c| c.as_os_str().to_str().is_some_and(|s| s.ends_with(".git")));
    let holds_head = resolved.join("HEAD").is_file();
    (looks_like && holds_head && !top.starts_with(&resolved)).then_some(resolved)
}

/// `path` with a leading `~` or `~/` taken as `home`, the way
/// `transcript.roots` writes its default.
#[must_use]
pub fn expand_home(path: &str, home: Option<&Path>) -> PathBuf {
    match (path.strip_prefix('~'), home) {
        (Some(""), Some(home)) => home.to_path_buf(),
        (Some(rest), Some(home)) if rest.starts_with('/') => home.join(&rest[1..]),
        _ => PathBuf::from(path),
    }
}

#[cfg(test)]
mod tests;
