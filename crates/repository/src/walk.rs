//! The walk from the default branch's tip to the root commits behind it.

use std::collections::BTreeSet;

use crate::deadline::check;
use crate::fsio::read_limited;
use crate::object::Kind;
use crate::objects::Objects;
use crate::repo::Repo;
use crate::{CommitId, Deadline, Error};

/// The most commits one walk visits. The Linux kernel has about one and
/// a third million; the bound keeps the set of commits seen from
/// growing past what the deadline would stop anyway.
pub(crate) const MAX_COMMITS: usize = 2_000_000;

/// The largest `shallow` file read.
pub(crate) const MAX_SHALLOW: u64 = 16 << 20;

const BAD_PARENT: Error = Error::Corrupt("commit has a parent that is not a commit id");
const BAD_SHALLOW: Error = Error::Corrupt("shallow file lists something that is not a commit id");
const NOT_A_COMMIT: Error = Error::Corrupt("object is not a commit");

/// Confirms that `commit` parses as a commit: it opens with a `tree`
/// header that names an id, as git writes it.
///
/// # Errors
///
/// [`Error::Corrupt`] for bytes that do not.
pub(crate) fn check_commit(commit: &[u8]) -> Result<(), Error> {
    let first = commit.split(|&b| b == b'\n').next().unwrap_or_default();
    first
        .strip_prefix(b"tree ")
        .and_then(|text| std::str::from_utf8(text).ok())
        .and_then(CommitId::parse)
        .map(|_| ())
        .ok_or(NOT_A_COMMIT)
}

/// The commits `parent` headers name in a commit object's bytes.
///
/// # Errors
///
/// [`Error::Corrupt`] for a `parent` header that is not a commit id.
pub(crate) fn parents(commit: &[u8]) -> Result<Vec<CommitId>, Error> {
    let mut found = Vec::new();
    for line in commit.split(|&b| b == b'\n').take_while(|l| !l.is_empty()) {
        if let Some(text) = line.strip_prefix(b"parent ") {
            let id = std::str::from_utf8(text).ok().and_then(CommitId::parse);
            found.push(id.ok_or(BAD_PARENT)?);
        }
    }
    Ok(found)
}

/// The commits the clone's `shallow` file lists: where its history was
/// cut, whose parents it does not hold.
///
/// # Errors
///
/// [`Error::Corrupt`] for a line that is not a commit id;
/// [`Error::TooLarge`] for a file past `limit` bytes.
pub(crate) fn read_shallow(repo: &Repo, limit: u64) -> Result<BTreeSet<CommitId>, Error> {
    let bytes = read_limited(&repo.common_dir, "shallow", limit)?.unwrap_or_default();
    String::from_utf8_lossy(&bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| CommitId::parse(line).ok_or(BAD_SHALLOW))
        .collect()
}

/// The parentless commit with the lowest id among those reachable from
/// `tip`. A commit in `shallow` is a boundary, not a root: its parents
/// are not followed, and it is not a candidate.
///
/// # Errors
///
/// [`Error::Missing`] for a parent the clone lacks and does not list as
/// shallow; [`Error::Corrupt`] for a commit that is not one;
/// [`Error::TooLarge`] past `max_commits`; [`Error::Deadline`] when the
/// deadline passes.
pub(crate) fn roots(
    objects: &Objects,
    shallow: &BTreeSet<CommitId>,
    tip: CommitId,
    deadline: &dyn Deadline,
    max_commits: usize,
) -> Result<Vec<CommitId>, Error> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![tip];
    let mut found = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id.to_bytes()) {
            continue;
        }
        if seen.len() > max_commits {
            return Err(Error::TooLarge("number of commits"));
        }
        check(deadline)?;
        let object = objects
            .read(&id)?
            .ok_or_else(|| Error::Missing(id.clone()))?;
        if object.kind != Kind::Commit {
            return Err(NOT_A_COMMIT);
        }
        if shallow.contains(&id) {
            continue;
        }
        check_commit(&object.data)?;
        let behind = parents(&object.data)?;
        if behind.is_empty() {
            found.insert(id);
        }
        pending.extend(behind);
    }
    Ok(found.into_iter().collect())
}

/// The parentless commit with the lowest id among those [`roots`] finds.
///
/// # Errors
///
/// As [`roots`].
pub(crate) fn lowest_root(
    objects: &Objects,
    shallow: &BTreeSet<CommitId>,
    tip: CommitId,
    deadline: &dyn Deadline,
    max_commits: usize,
) -> Result<Option<CommitId>, Error> {
    Ok(roots(objects, shallow, tip, deadline, max_commits)?
        .into_iter()
        .next())
}

#[cfg(test)]
mod tests;
