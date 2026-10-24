//! References: the tip of a repository's default branch, found in loose
//! reference files and in `packed-refs`.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::deadline::check;
use crate::fsio::{open_below, read_limited};
use crate::repo::Repo;
use crate::{CommitId, Deadline, Error};

/// The largest `packed-refs` read.
pub(crate) const MAX_PACKED_REFS: u64 = 64 << 20;

/// The symbolic references followed before a chain is called a loop.
pub(crate) const MAX_SYMBOLIC: usize = 5;

/// The largest loose reference file read.
const MAX_REFERENCE: u64 = 4096;

const BAD_NAME: Error = Error::Corrupt("reference name is invalid");
const BAD_REFERENCE: Error = Error::Corrupt("reference is invalid");
const BAD_PACKED: Error = Error::Corrupt("packed-refs line is invalid");
const LOOP: Error = Error::Corrupt("symbolic references loop");

/// Whether `name` is `HEAD` or lies under `refs/` without leaving it: no
/// empty, `.` or `..` component, and no control character or backslash.
fn valid_name(name: &str) -> bool {
    let plain = |part: &str| {
        !part.is_empty()
            && part != "."
            && part != ".."
            && !part.contains(|c: char| c.is_control() || c == '\\')
    };
    name == "HEAD"
        || name
            .strip_prefix("refs/")
            .is_some_and(|rest| rest.split('/').all(plain))
}

/// The commit `line` of `packed-refs` gives for `name`, if it is the
/// line for `name`.
fn packed_line(line: &str, name: &str) -> Result<Option<CommitId>, Error> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
        return Ok(None);
    }
    let (sha, found) = line.split_once(' ').ok_or(BAD_PACKED)?;
    let id = CommitId::parse(sha).ok_or(BAD_PACKED)?;
    Ok((found == name).then_some(id))
}

/// The commit `packed-refs` gives for `name`, read line by line up to
/// `limit` bytes.
fn packed(
    dir: &Path,
    name: &str,
    deadline: &dyn Deadline,
    limit: u64,
) -> Result<Option<CommitId>, Error> {
    let Some(file) = open_below(dir, "packed-refs")? else {
        return Ok(None);
    };
    let mut reader = BufReader::new(file.take(limit.saturating_add(1)));
    let (mut line, mut total) = (Vec::new(), 0_u64);
    loop {
        check(deadline)?;
        line.clear();
        let read = reader
            .read_until(b'\n', &mut line)
            .map_err(Error::io("packed references"))?;
        total += read as u64;
        if total > limit {
            return Err(Error::TooLarge("packed references"));
        }
        if read == 0 {
            return Ok(None);
        }
        if let Some(id) = packed_line(&String::from_utf8_lossy(&line), name)? {
            return Ok(Some(id));
        }
    }
}

/// `resolve`, `depth` symbolic references down.
fn resolve_at(
    repo: &Repo,
    name: &str,
    deadline: &dyn Deadline,
    limit: u64,
    depth: usize,
) -> Result<Option<CommitId>, Error> {
    if !valid_name(name) {
        return Err(BAD_NAME);
    }
    let home = if name == "HEAD" {
        &repo.head_dir
    } else {
        &repo.common_dir
    };
    let Some(bytes) = read_limited(home, name, MAX_REFERENCE)? else {
        return packed(&repo.common_dir, name, deadline, limit);
    };
    let text = String::from_utf8_lossy(&bytes);
    let text = text.trim();
    match text.strip_prefix("ref:") {
        Some(_) if depth + 1 >= MAX_SYMBOLIC => Err(LOOP),
        Some(target) => resolve_at(repo, target.trim(), deadline, limit, depth + 1),
        None => CommitId::parse(text).map(Some).ok_or(BAD_REFERENCE),
    }
}

/// The commit the reference `name` ends at, following symbolic
/// references, reading `packed-refs` up to `limit` bytes.
pub(crate) fn resolve(
    repo: &Repo,
    name: &str,
    deadline: &dyn Deadline,
    limit: u64,
) -> Result<Option<CommitId>, Error> {
    resolve_at(repo, name, deadline, limit, 0)
}

/// The tip of the default branch: where the remote's `HEAD` points, if
/// the clone has one that resolves, else where `HEAD` points. `None`
/// when that branch has no commit yet.
///
/// # Errors
///
/// [`Error::Corrupt`] for a reference that is not one;
/// [`Error::TooLarge`] for a `packed-refs` past the bound;
/// [`Error::Deadline`] when the deadline passes while `packed-refs` is
/// scanned.
pub(crate) fn default_branch_tip(
    repo: &Repo,
    deadline: &dyn Deadline,
) -> Result<Option<CommitId>, Error> {
    let remote = resolve(repo, "refs/remotes/origin/HEAD", deadline, MAX_PACKED_REFS)?;
    if remote.is_some() {
        return Ok(remote);
    }
    resolve(repo, "HEAD", deadline, MAX_PACKED_REFS)
}

#[cfg(test)]
mod tests;
