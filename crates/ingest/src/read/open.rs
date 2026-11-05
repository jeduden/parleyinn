//! Opening a transcript source at its ingest marker (SEC-05, REC-13): the
//! window ingest reads, whose start the consumed-prefix hash vouches for,
//! the hash streamed so no more than a chunk of the prefix is held, and
//! the passes over a whole source that hold one line at a time.

use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use record::digest::Digest;
use store::source::Zeroed;
use store::{Source, source::Span};

use super::bounded::{LINE_LIMIT, Limits, Measure, Refused, SOURCE_LIMIT, window};
use super::{lines_at, regular_file};

/// How many bytes one read takes.
const CHUNK: usize = 64 * 1024;

/// Capacity reserved while an ingest window is resident: the larger of
/// one identifier-learning window, or a bounded parsed line plus its
/// whole-line redaction (SEC-05).
pub(crate) const LEARNING_WINDOW_LIMIT: usize = LINE_LIMIT + 1;
pub(crate) const LEARNING_RESERVE: usize = 3 * LEARNING_WINDOW_LIMIT;
/// Distinct harness identifiers retained while a source is ingested.
pub(crate) const IDENTIFIER_LIMIT: usize = 2 << 20;
/// Both the learned identifiers and the redactor compiled from them can
/// be resident while the redactor is built.
const IDENTIFIER_RESERVE: usize = 2 * IDENTIFIER_LIMIT;
/// Space for redacted event content and its planning metadata. Planning
/// stops at this budget and the source resumes at its ingest marker.
pub(crate) const PLANNING_RESERVE: usize = 32 << 20;
pub(crate) const INGEST_WINDOW_LIMIT: usize =
    SOURCE_LIMIT - LEARNING_RESERVE - IDENTIFIER_RESERVE - PLANNING_RESERVE;

/// A transcript source opened: the window to read, and how many bytes
/// the source held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub window: super::bounded::Window,
    /// The bytes the source held when opened.
    pub seen: u64,
}

/// Whether the transcript at `path` is longer than one window, so an
/// ingest of it needs its ingest marker to choose where to read from; a
/// source that fits one window is read whole, and its marker is not
/// looked up. A path that cannot be measured needs none: opening it
/// fails.
#[must_use]
pub fn needs_marker(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.len() > INGEST_WINDOW_LIMIT as u64)
}

/// The transcript at `path`, opened for an ingest of the source `known`,
/// the row its ingest marker is in when it has one. A source that fits
/// one window is read whole, from its first byte. A longer one is read
/// from the marker when the bytes before it still hash to `known`'s
/// consumed prefix, else from its first byte, as a new generation
/// (REC-07). So is a source whose position lies past a line over the
/// line ceiling, whatever its length.
///
/// # Errors
///
/// Fails as [`regular_file`] does, or when a read does.
pub fn open(path: &Path, known: Option<&Source>) -> io::Result<Opened> {
    let mut file = regular_file(path)?;
    let len = file.metadata()?.len();
    let mut from = 0;
    if let Some(source) =
        known.filter(|s| s.position.offset > 0 && (len > INGEST_WINDOW_LIMIT as u64 || passed(s)))
        && holds(&mut file, source)?
    {
        from = source.position.offset;
    }
    let limits = Limits {
        line: LINE_LIMIT,
        source: INGEST_WINDOW_LIMIT,
    };
    let window = window(&mut file, from, limits)?;
    let seen = if window.more {
        len.max(window.end() + 1)
    } else {
        window.end()
    };

    Ok(Opened { window, seen })
}

/// Whether `source`'s ingest position lies past a line over the line
/// ceiling: a window read from the first byte would end at that line, so
/// the source is read from its marker whatever its length (SEC-05).
#[must_use]
pub fn passed(source: &Source) -> bool {
    source.purged.iter().any(|s| s.length > LINE_LIMIT as u64)
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

/// The digest of the first `offset` bytes of the transcript at `path`,
/// `purged` zeroed, read a chunk at a time; none when it holds fewer
/// (SEC-05, SEC-31).
pub fn prefix(path: &Path, offset: u64, purged: &[Span]) -> io::Result<Option<Digest>> {
    prefixes(path, offset, purged, purged).map(|pair| pair.map(|(digest, _)| digest))
}

/// Two digests of the first `offset` bytes of one guarded read of the
/// transcript at `path`, each zeroing its own spans. Purge uses the pair
/// to validate the old prefix and replace it without a second read racing
/// a changed source (SEC-05, SEC-31).
pub fn prefixes(
    path: &Path,
    offset: u64,
    first: &[Span],
    second: &[Span],
) -> io::Result<Option<(Digest, Digest)>> {
    let mut file = regular_file(path)?;
    let (mut one, mut two) = (Zeroed::new(first), Zeroed::new(second));
    let mut buffer = vec![0; CHUNK];
    let mut left = offset;
    while left > 0 {
        let want = usize::try_from(left).map_or(CHUNK, |l| l.min(CHUNK));
        let n = file.read(&mut buffer[..want])?;
        if n == 0 {
            return Ok(None);
        }
        one.update(&buffer[..n]);
        two.update(&buffer[..n]);
        left -= n as u64;
    }

    Ok(Some((one.digest(), two.digest())))
}

/// Calls `each` with every line of the transcript at `path` that ends at
/// or before byte offset `until`, one window at a time; a line past the
/// line ceiling is stepped over, a line not UTF-8 read lossily.
///
/// # Errors
///
/// Fails as [`regular_file`] does, or when a read does.
pub fn for_each_line(path: &Path, until: u64, mut each: impl FnMut(&str)) -> io::Result<()> {
    for_each_line_between(path, 0, until, &mut each)
}

/// Calls `each` with every line beginning at `from` and ending at or
/// before `until`, one bounded window at a time. Both offsets are line
/// boundaries written by ingest (SEC-05, PRV-14).
pub fn for_each_line_between(
    path: &Path,
    from: u64,
    until: u64,
    mut each: impl FnMut(&str),
) -> io::Result<()> {
    for_each_line_with_limits(path, from, until, Limits::default(), &mut each).map(drop)
}

/// Calls `each` as [`for_each_line_between`] does while each read keeps
/// `limits`; returns the largest such read. Identifier learning uses a
/// one-line window while the ingest window is resident (SEC-05).
pub(crate) fn for_each_line_with_limits(
    path: &Path,
    mut from: u64,
    until: u64,
    limits: Limits,
    mut each: impl FnMut(&str),
) -> io::Result<usize> {
    let mut high_water = 0;
    loop {
        if from >= until {
            return regular_file(path).map(|_| high_water);
        }
        let got = window(regular_file(path)?, from, limits)?;
        high_water = high_water.max(got.high_water);
        let text = String::from_utf8_lossy(&got.bytes);
        for line in lines_at(&text, from) {
            if line.end > until {
                return Ok(high_water);
            }
            each(line.text);
        }
        from = match got.refused {
            Some(r) => r.offset + r.length + 1,
            None if got.more => got.end(),
            None => return Ok(high_water),
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
