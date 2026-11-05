//! Reading a transcript source within its memory ceilings (SEC-05): a
//! line is at most 4 MiB, and at most 64 MiB of one source is held in
//! memory at once. A longer source is read in windows, each resumed at
//! its ingest marker (REC-13). A line past the line ceiling is refused
//! whole and the window ends before it, so the source is left short at
//! that line and resumable (REC-03); the line is counted over, never
//! buffered.

use std::io::{self, Read, Seek, SeekFrom};

/// The most bytes of one line, without its line break, ingest takes
/// (SEC-05).
pub const LINE_LIMIT: usize = 4 * 1024 * 1024;

/// The most bytes of one source ingest holds in memory at once
/// (SEC-05).
pub const SOURCE_LIMIT: usize = 64 * 1024 * 1024;

/// How many bytes one read takes.
const CHUNK: usize = 64 * 1024;

/// The ceilings a read keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The most bytes of a line, without its line break.
    pub line: usize,
    /// The most bytes of the source held at once.
    pub source: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            line: LINE_LIMIT,
            source: SOURCE_LIMIT,
        }
    }
}

/// A line past the line ceiling, refused whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refused {
    /// The byte offset it starts at, in the source.
    pub offset: u64,
    /// Its bytes without a line break; those up to the end of the source
    /// when it ends before one.
    pub length: u64,
}

/// One window of a source: the lines from a byte offset that fit the
/// ceilings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// The byte offset in the source the window starts at.
    pub start: u64,
    /// Whole lines, ending after a line break, bar a last line the source
    /// ends in before one.
    pub bytes: Vec<u8>,
    /// The line the window ends before, past the line ceiling.
    pub refused: Option<Refused>,
    /// Whether the source holds bytes past the window: a line that
    /// would not fit with the lines before it, or a refused line.
    pub more: bool,
    /// Whether the refused line's line break is in the source, so a read
    /// can go on past it; false while the harness may still be writing
    /// it, and when no line is refused.
    pub refused_ended: bool,
    /// The most bytes the read held at once: at most the source
    /// ceiling.
    pub high_water: usize,
}

impl Window {
    /// The byte offset in the source just past the window.
    #[must_use]
    pub fn end(&self) -> u64 {
        self.start + self.bytes.len() as u64
    }
}

/// How far a read of one line has got.
struct Reading {
    window: Window,
    /// Where the line being read starts in `window.bytes`.
    line_start: usize,
    /// The bytes counted of a refused line still to its end, no more
    /// kept.
    refusing: Option<u64>,
    done: bool,
}

/// The window of `source` from byte offset `from`: whole lines up to
/// `limits.source` bytes, ending before a line past `limits.line`. The
/// source is moved to `from` by a seek: the bytes before it are never
/// read, so reading a source window by window reads each byte once.
///
/// # Errors
///
/// Fails when a read does.
pub fn window(mut source: impl Read + Seek, from: u64, limits: Limits) -> io::Result<Window> {
    source.seek(SeekFrom::Start(from))?;
    let mut reading = Reading {
        window: Window {
            start: from,
            bytes: Vec::new(),
            refused: None,
            more: false,
            refused_ended: false,
            high_water: 0,
        },
        line_start: 0,
        refusing: None,
        done: false,
    };
    let mut scratch = vec![0; CHUNK];
    while !reading.done {
        let n = source.read(&mut scratch)?;
        if n == 0 {
            break;
        }
        reading.take(&scratch[..n], limits);
    }
    reading.finish(from);

    Ok(reading.window)
}

impl Reading {
    /// Takes the bytes `chunk` holds, a piece of a line at a time.
    fn take(&mut self, chunk: &[u8], limits: Limits) {
        let mut rest = chunk;
        while !rest.is_empty() && !self.done {
            let newline = rest.iter().position(|b| *b == b'\n');
            let piece = newline.map_or(rest.len(), |i| i + 1);
            match self.refusing {
                Some(counted) => self.count(counted, newline, rest.len()),
                None => self.keep(&rest[..piece], newline, limits),
            }
            rest = &rest[piece..];
        }
    }

    /// Counts a refused line's bytes through `newline`, or through the
    /// `len` bytes of the piece when it holds none.
    fn count(&mut self, counted: u64, newline: Option<usize>, len: usize) {
        let more = newline.unwrap_or(len) as u64;
        self.refusing = Some(counted + more);
        self.done = newline.is_some();
    }

    /// Keeps `piece`, the end of the line being read when `newline` is
    /// where it ends, unless it passes a ceiling.
    fn keep(&mut self, piece: &[u8], newline: Option<usize>, limits: Limits) {
        let held = self.window.bytes.len();
        let text = held - self.line_start + newline.unwrap_or(piece.len());
        if text > limits.line {
            self.window.bytes.truncate(self.line_start);
            self.refusing = Some(text as u64);
            self.done = newline.is_some();
            return;
        }
        if held + piece.len() > limits.source {
            self.window.bytes.truncate(self.line_start);
            self.window.more = true;
            self.done = true;
            return;
        }
        self.window.bytes.extend_from_slice(piece);
        self.window.high_water = self.window.high_water.max(self.window.bytes.len());
        if newline.is_some() {
            self.line_start = self.window.bytes.len();
        }
    }

    /// Ends the read: a refused line is named by where it starts.
    fn finish(&mut self, from: u64) {
        if let Some(length) = self.refusing {
            self.window.refused = Some(Refused {
                offset: from + self.line_start as u64,
                length,
            });
            self.window.more = true;
            self.window.refused_ended = self.done;
        }
    }
}

/// Counts the bytes of the first line of a source, a piece at a time,
/// holding none: how `parinn verify` finds a line the limit refused.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Measure {
    length: u64,
    ended: bool,
}

impl Measure {
    /// Counts `piece`, which follows the bytes counted so far; true once
    /// the line has ended.
    pub fn feed(&mut self, piece: &[u8]) -> bool {
        if !self.ended {
            match piece.iter().position(|b| *b == b'\n') {
                Some(i) => {
                    self.length += i as u64;
                    self.ended = true;
                }
                None => self.length += piece.len() as u64,
            }
        }

        self.ended
    }

    /// The bytes of the line counted, without its line break.
    #[must_use]
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Whether the line is past `limit` bytes.
    #[must_use]
    pub fn over(&self, limit: usize) -> bool {
        self.length > limit as u64
    }
}

#[cfg(test)]
mod tests;
