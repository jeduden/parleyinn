//! Reading a transcript source: the file at a resolved path, opened
//! without blocking on a FIFO and refused unless it is the regular file
//! the path resolved to (SEC-18), then split into the lines ingest may
//! consume, each with the byte offsets it spans (REC-03).

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::Path;

/// One line ingest may consume: its text without the newline, and the
/// bytes it spans with the newline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line<'a> {
    pub text: &'a str,
    /// The byte offset it starts at.
    pub offset: u64,
    /// The byte offset just past it and its newline.
    pub end: u64,
}

/// The bytes of the transcript at `path`, a path already resolved
/// within the allowed roots. It is opened only when it is a regular
/// file, without blocking should it become a FIFO, and read only when
/// the file opened is the one looked at: a link or other file swapped in
/// between is refused.
///
/// # Errors
///
/// Fails when the path is not a regular file, cannot be opened or read,
/// or changed between the look and the open.
pub fn transcript(path: &Path) -> io::Result<Vec<u8>> {
    let looked = fs::symlink_metadata(path)?;
    if !looked.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a regular file",
        ));
    }
    let mut file = open(path)?;
    same_file(&looked, &file.metadata()?)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    Ok(bytes)
}

/// Opens `path` for reading, never waiting for a FIFO's writer.
fn open(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(NONBLOCK);
    }
    options.open(path)
}

/// `O_NONBLOCK` where this parinn knows it; an open with it never waits
/// for a FIFO's writer.
#[cfg(any(target_os = "linux", target_os = "android"))]
const NONBLOCK: i32 = 0o4000;
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
const NONBLOCK: i32 = 0x0004;
#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))
))]
const NONBLOCK: i32 = 0;

/// Refuses `opened` unless it is the regular file `looked` described.
fn same_file(looked: &fs::Metadata, opened: &fs::Metadata) -> io::Result<()> {
    #[cfg(unix)]
    let same = {
        use std::os::unix::fs::MetadataExt;
        looked.dev() == opened.dev() && looked.ino() == opened.ino()
    };
    #[cfg(not(unix))]
    let same = looked.len() == opened.len();
    if opened.is_file() && same {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "the file changed while it was opened",
    ))
}

/// The lines of `text` ingest may consume, in order: each ended by a
/// newline, and a last one without its newline only when it parses as
/// JSON, as the harness may still be writing it.
#[must_use]
pub fn lines(text: &str) -> Vec<Line<'_>> {
    let mut out = Vec::new();
    let mut offset = 0;
    for piece in text.split_inclusive('\n') {
        let end = offset + piece.len();
        let line = piece.strip_suffix('\n');
        let consumable = line.or_else(|| {
            serde_json::from_str::<serde_json::Value>(piece)
                .is_ok()
                .then_some(piece)
        });
        match consumable {
            Some(text) => out.push(Line {
                text,
                offset: offset as u64,
                end: end as u64,
            }),
            None => break,
        }
        offset = end;
    }

    out
}

#[cfg(test)]
mod tests;
