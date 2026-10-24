//! Repository identity (LANE-02), read inside the core from a git
//! clone's own files.
//!
//! The crate starts no program and opens no socket (I4, SEC-01): it
//! reads loose objects, packfiles and references itself, and writes
//! nothing, so it needs no `git` binary and cannot be steered by a
//! hostile clone's hooks or configuration.

mod deadline;
mod delta;
mod error;
#[cfg(test)]
mod fixture;
mod fsio;
mod identity;
mod inflate;
mod locate;
mod loose;
mod object;
mod objects;
mod pack;
mod refs;
mod repo;
mod sha1;
mod walk;

pub use deadline::Deadline;
pub use error::Error;
pub use identity::{CommitId, RepositoryIdentity};
pub use locate::{Located, git_dir, locate};

use std::path::Path;

use objects::Objects;
use repo::Repo;

/// The id git gives the commit whose object body is `body`: the SHA-1
/// of `commit <length>\0` and the body. A test builds a commit under
/// its real id with it.
#[must_use]
pub fn commit_object_id(body: &[u8]) -> CommitId {
    sha1::object_id("commit", body)
}

/// The root commit that names the repository whose git directory is
/// `git_dir`: the parentless commit with the lowest id among those
/// reachable from the default branch.
///
/// # Errors
///
/// See [`Error`].
pub fn root_commit(git_dir: &Path, deadline: &dyn Deadline) -> Result<Option<CommitId>, Error> {
    let repo = Repo::open(git_dir)?;
    let Some(tip) = refs::default_branch_tip(&repo, deadline)? else {
        return Ok(None);
    };
    let objects = Objects::open(&repo.objects_dir())?;
    let shallow = walk::read_shallow(&repo, walk::MAX_SHALLOW)?;
    walk::lowest_root(&objects, &shallow, tip, deadline, walk::MAX_COMMITS)
}

/// Every parentless commit reachable from the default branch of the
/// repository whose git directory is `git_dir`, the lowest id first: none
/// for a repository with no commit, or a shallow clone that reaches no
/// root. A commit the branch does not reach is not one of them, whatever
/// the clone holds.
///
/// # Errors
///
/// See [`Error`].
pub fn roots(git_dir: &Path, deadline: &dyn Deadline) -> Result<Vec<CommitId>, Error> {
    let repo = Repo::open(git_dir)?;
    let Some(tip) = refs::default_branch_tip(&repo, deadline)? else {
        return Ok(Vec::new());
    };
    let objects = Objects::open(&repo.objects_dir())?;
    let shallow = walk::read_shallow(&repo, walk::MAX_SHALLOW)?;
    walk::roots(&objects, &shallow, tip, deadline, walk::MAX_COMMITS)
}

/// Whether the clone whose git directory is `git_dir` holds the commit
/// `id`, loose or packed. The object is read and hashed: one that does
/// not hash to `id` is an error, and one that is not a commit is not a
/// commit the clone holds.
///
/// # Errors
///
/// See [`Error`].
pub fn has_commit(git_dir: &Path, id: &CommitId) -> Result<bool, Error> {
    let repo = Repo::open(git_dir)?;
    let object = Objects::open(&repo.objects_dir())?.read(id)?;
    Ok(object.is_some_and(|object| object.kind == object::Kind::Commit))
}

/// The identity of the repository whose git directory is `git_dir`.
///
/// # Errors
///
/// See [`Error`].
pub fn identity(git_dir: &Path, deadline: &dyn Deadline) -> Result<RepositoryIdentity, Error> {
    root_commit(git_dir, deadline).map(RepositoryIdentity::from_root)
}

#[cfg(test)]
mod tests;
