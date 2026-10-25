//! The continuation cursor of a recall tool that stopped at its
//! per-call cap (RCL-03): where the next call goes on, as the writer,
//! the seq and the character offset into that event's content, and the
//! last seq of the range, so the continuation neither skips nor repeats
//! a character. It is a position and no authority: a call that passes
//! it is checked against its scope like any other, and a cursor that
//! does not read back is refused in fixed text.

use record::id::{Seq, WriterId};

/// The refusal for a cursor that reads back to no position.
pub const BAD_CURSOR: &str = "the cursor is not one a recall tool returned";

/// Where a recall goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub writer: WriterId,
    /// The event to go on at.
    pub seq: Seq,
    /// How many characters of its content were already returned.
    pub offset: usize,
    /// The last event of the range.
    pub to: Seq,
}

impl Cursor {
    /// The most characters a cursor's plain text holds: a writer id,
    /// a seq, an offset and a last seq, and their colons.
    pub const LONGEST: usize = 64 + 16 + 10 + 16 + 3;

    /// The cursor as text: lowercase hex of `<writer>:<seq>:<offset>:<to>`.
    #[must_use]
    pub fn encode(&self) -> String {
        let plain = format!("{}:{}:{}:{}", self.writer, self.seq, self.offset, self.to);

        plain.bytes().map(|b| format!("{b:02x}")).collect()
    }

    /// The cursor `text` spells.
    ///
    /// # Errors
    ///
    /// Fails, in fixed text, on any other text.
    pub fn decode(text: &str) -> Result<Self, &'static str> {
        let bytes: Option<Vec<u8>> = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
            .collect();
        let plain = String::from_utf8(bytes.ok_or(BAD_CURSOR)?).map_err(|_| BAD_CURSOR)?;
        let parts: Vec<&str> = plain.split(':').collect();
        let [writer, seq, offset, to] = parts.as_slice() else {
            return Err(BAD_CURSOR);
        };
        let number = |digits: &str| {
            let plain = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
            plain.then(|| digits.parse::<u64>().ok()).flatten()
        };

        Ok(Self {
            writer: WriterId::parse(writer).map_err(|_| BAD_CURSOR)?,
            seq: number(seq).and_then(Seq::new).ok_or(BAD_CURSOR)?,
            offset: number(offset)
                .and_then(|o| usize::try_from(o).ok())
                .ok_or(BAD_CURSOR)?,
            to: number(to).and_then(Seq::new).ok_or(BAD_CURSOR)?,
        })
    }
}

#[cfg(test)]
mod tests;
