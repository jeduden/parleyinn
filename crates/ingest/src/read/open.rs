//! Opening a transcript source at its ingest marker (SEC-05, REC-13): the
//! window ingest reads, whose start the consumed-prefix hash vouches for,
//! the hash streamed so no more than a chunk of the prefix is held, and
//! the passes over a whole source that hold one line at a time.

use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use store::source::Zeroed;
use store::{Source, source::Span};

use super::bounded::{LINE_LIMIT, Limits, Measure, Refused, SOURCE_LIMIT, window};
use super::{lines_at, regular_file};

/// How many bytes one read takes.
const CHUNK: usize = 64 * 1024;

/// A transcript source opened: the window to read, and how many bytes
/// the source held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub window: super::bounded::Window,
    /// The bytes the source held when opened.
    pub seen: u64,
}

/// The transcript at `path`, opened for an ingest of the source `known`,
/// the row its ingest marker is in when it has one. A source that fits
/// one window is read whole, from its first byte. A longer one is read
/// from the marker when the bytes before it still hash to `known`'s
/// consumed prefix, else from its first byte, as a new generation
/// (REC-07).
///
/// # Errors
///
/// Fails as [`regular_file`] does, or when a read does.
pub fn open(path: &Path, known: Option<&Source>) -> io::Result<Opened> {
    let mut file = regular_file(path)?;
    let len = file.metadata()?.len();
    let mut from = 0;
    if let Some(source) = known.filter(|s| len > SOURCE_LIMIT as u64 && s.position.offset > 0) {
        if holds(&mut file, source)? {
            from = source.position.offset;
        }
        file.seek(SeekFrom::Start(0))?;
    }
    let window = window(&mut file, from, Limits::default())?;
    let seen = if window.more {
        len.max(window.end() + 1)
    } else {
        window.end()
    };

    Ok(Opened { window, seen })
}

/// Whether the first bytes of `file` still hash to the consumed prefix
/// of `source`, its purged lines zeroed (SEC-31, REC-07).
///
/// # Errors
///
/// Fails when a read does.
pub fn holds(file: impl Read, source: &Source) -> io::Result<bool> {
    let hash = zeroed(file, source.position.offset, &source.purged)?;

    Ok(hash.is_some_and(|h| h.digest() == source.position.prefix))
}

/// Whether the transcript at `path` still begins with the prefix
/// `source` consumed; false when it cannot be read.
#[must_use]
pub fn continues(path: &Path, source: &Source) -> bool {
    regular_file(path).is_ok_and(|file| holds(file, source).unwrap_or(false))
}

/// The running hash of the first `offset` bytes of `file`, `purged`
/// zeroed, a chunk at a time, ready to take the bytes that follow; none
/// when `file` holds fewer.
///
/// # Errors
///
/// Fails when a read does.
pub fn zeroed(mut file: impl Read, offset: u64, purged: &[Span]) -> io::Result<Option<Zeroed<'_>>> {
    let mut hash = Zeroed::new(purged);
    let mut buffer = vec![0; CHUNK];
    let mut left = offset;
    while left > 0 {
        let want = usize::try_from(left).map_or(CHUNK, |l| l.min(CHUNK));
        let n = file.read(&mut buffer[..want])?;
        if n == 0 {
            return Ok(None);
        }
        hash.update(&buffer[..n]);
        left -= n as u64;
    }

    Ok(Some(hash))
}

/// Calls `each` with every line of the transcript at `path` that ends at
/// or before byte offset `until`, one window at a time; a line past the
/// line ceiling is stepped over, a line not UTF-8 read lossily.
///
/// # Errors
///
/// Fails as [`regular_file`] does, or when a read does.
pub fn for_each_line(path: &Path, until: u64, mut each: impl FnMut(&str)) -> io::Result<()> {
    let mut from = 0;
    loop {
        let got = window(regular_file(path)?, from, Limits::default())?;
        let text = String::from_utf8_lossy(&got.bytes);
        for line in lines_at(&text, from) {
            if line.end > until {
                return Ok(());
            }
            each(line.text);
        }
        from = match got.refused {
            Some(r) => r.offset + r.length + 1,
            None if got.more => got.end(),
            None => return Ok(()),
        };
    }
}

/// The line at byte offset `at` of the transcript at `path` when it is
/// past the line ceiling: ingest refuses it and stays before it.
///
/// # Errors
///
/// Fails as [`regular_file`] does, or when a read does.
pub fn refused_at(path: &Path, at: u64) -> io::Result<Option<Refused>> {
    let mut file = regular_file(path)?;
    file.seek(SeekFrom::Start(at))?;
    let mut line = Measure::default();
    let mut buffer = vec![0; CHUNK];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 || line.feed(&buffer[..n]) {
            break;
        }
    }

    Ok(line.over(LINE_LIMIT).then(|| Refused {
        offset: at,
        length: line.length(),
    }))
}

#[cfg(test)]
mod tests;
