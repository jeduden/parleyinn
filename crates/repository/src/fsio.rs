//! The one way this crate opens a file of a clone, which is for
//! reading, and only after the path has been looked at.
//!
//! A clone is attacker-controlled (I2). A FIFO named `HEAD` would block
//! an open for ever, a device would never end, and a symbolic link would
//! send the read anywhere on the machine. So every open goes through
//! [`open_below`], which looks at each directory on the way and at the
//! file with `symlink_metadata` first, and refuses anything that is not
//! a regular file in a real directory before it opens it.

use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::Error;

const SYMLINK: Error = Error::Unsupported("symbolic link inside a repository");
const NOT_REGULAR: Error = Error::Unsupported("repository file that is not a regular file");

/// Refuses `path` when it is a symbolic link. A path that is not there
/// passes.
///
/// # Errors
///
/// [`Error::Unsupported`] for a symbolic link; [`Error::Io`] when the
/// file system cannot say.
pub(crate) fn refuse_symlink(path: &Path) -> Result<(), Error> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_symlink() => Err(SYMLINK),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io("a repository file")(e)),
    }
}

/// Whether a directory on the way to `rel` is missing, after refusing
/// each one that is a symbolic link. `rel` is `/`-separated.
fn directory_missing(base: &Path, rel: &str) -> Result<bool, Error> {
    let mut at = base.to_path_buf();
    let mut parts = rel.split('/').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            break;
        }
        at.push(part);
        match fs::symlink_metadata(&at) {
            Ok(meta) if meta.is_symlink() => return Err(SYMLINK),
            Ok(meta) if !meta.is_dir() => break,
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(true),
            Err(e) => return Err(Error::io("a repository file")(e)),
        }
    }
    Ok(false)
}

/// The regular file `rel` below the directory `base`, open for reading,
/// or `None` when there is no such file. `rel` is `/`-separated and
/// holds no `.` or `..`.
///
/// # Errors
///
/// [`Error::Unsupported`] when the file, or a directory on the way to
/// it below `base`, is a symbolic link, or the file is a FIFO, a
/// device, a socket or a directory; [`Error::Io`] when the file system
/// cannot say or the file cannot be opened.
pub(crate) fn open_below(base: &Path, rel: &str) -> Result<Option<File>, Error> {
    if directory_missing(base, rel)? {
        return Ok(None);
    }
    let path = base.join(rel);
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() => {}
        Ok(meta) if meta.is_symlink() => return Err(SYMLINK),
        Ok(_) => return Err(NOT_REGULAR),
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::io("a repository file")(e)),
    }
    open_checked(&path).map(Some)
}

/// Opens the regular file at `path` for reading without ever waiting on
/// it or following a link, and looks at what was opened: a path swapped
/// for a FIFO, a device or a link since it was looked at is refused, not
/// waited on or followed. A file gone since is an error like any other
/// open's.
///
/// # Errors
///
/// [`Error::Unsupported`] for a link or anything that is not a regular
/// file; [`Error::Io`] when it cannot be opened.
pub(crate) fn open_checked(path: &Path) -> Result<File, Error> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    no_wait_no_follow(&mut options);
    let file = match options.open(path) {
        Ok(file) => file,
        // The open refused a link: it was swapped in after the look.
        Err(_) if fs::symlink_metadata(path).is_ok_and(|m| m.is_symlink()) => return Err(SYMLINK),
        Err(e) => return Err(Error::io("a repository file")(e)),
    };
    let meta = file.metadata().map_err(Error::io("a repository file"))?;
    if meta.is_file() {
        Ok(file)
    } else {
        Err(NOT_REGULAR)
    }
}

/// Makes `options` open without waiting on a FIFO and without following
/// a link. `libc` would name these flags; the values are the platform's
/// own, which std takes as plain integers.
#[cfg(target_os = "linux")]
fn no_wait_no_follow(options: &mut fs::OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    const O_NONBLOCK: i32 = 0o4000;
    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    const O_NOFOLLOW: i32 = 0o100_000;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
    const O_NOFOLLOW: i32 = 0o400_000;
    options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
}

/// See the Linux version.
#[cfg(target_os = "macos")]
fn no_wait_no_follow(options: &mut fs::OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    const O_NONBLOCK: i32 = 0x4;
    const O_NOFOLLOW: i32 = 0x100;
    options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
}

/// Elsewhere the look with `symlink_metadata` is all there is.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn no_wait_no_follow(_options: &mut fs::OpenOptions) {}

/// The bytes of the regular file `rel` below `base`, or `None` when
/// there is no such file.
///
/// # Errors
///
/// [`Error::TooLarge`] when the file is longer than `limit` bytes,
/// which is checked while reading, so a huge file is never held; the
/// refusals of [`open_below`]; [`Error::Io`] when it cannot be read.
pub(crate) fn read_limited(base: &Path, rel: &str, limit: u64) -> Result<Option<Vec<u8>>, Error> {
    let Some(file) = open_below(base, rel)? else {
        return Ok(None);
    };
    let mut data = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut data)
        .map_err(Error::io("a repository file"))?;
    if data.len() as u64 > limit {
        return Err(Error::TooLarge("repository file"));
    }
    Ok(Some(data))
}

#[cfg(test)]
mod tests;
