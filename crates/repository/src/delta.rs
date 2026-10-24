//! Git's delta format: a result built from copies of a base and
//! inserted bytes.

use crate::Error;

const CUT: Error = Error::Corrupt("delta is cut short");
const INVALID: Error = Error::Corrupt("delta does not fit its base");

/// The bytes of a delta, taken from the front.
struct Bytes<'a>(&'a [u8]);

impl<'a> Bytes<'a> {
    fn byte(&mut self) -> Result<u8, Error> {
        let (&first, rest) = self.0.split_first().ok_or(CUT)?;
        self.0 = rest;
        Ok(first)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], Error> {
        let (taken, rest) = self.0.split_at_checked(count).ok_or(CUT)?;
        self.0 = rest;
        Ok(taken)
    }

    /// A size: seven bits at a time, the low bits first.
    fn size(&mut self) -> Result<usize, Error> {
        let mut size = 0_u64;
        for shift in (0..=56).step_by(7) {
            let byte = self.byte()?;
            size |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return usize::try_from(size).map_err(|_| Error::TooLarge("delta size"));
            }
        }
        Err(INVALID)
    }

    /// The operand bytes of a copy command that its low bits flag,
    /// `first` being the flag of the first.
    fn operand(&mut self, command: u8, first: u8, count: u32) -> Result<usize, Error> {
        let mut value = 0_usize;
        for i in 0..count {
            if command & (first << i) != 0 {
                value |= usize::from(self.byte()?) << (8 * i);
            }
        }
        Ok(value)
    }
}

/// One copy command: its bytes of the base.
fn copy<'b>(command: u8, delta: &mut Bytes<'_>, base: &'b [u8]) -> Result<&'b [u8], Error> {
    let offset = delta.operand(command, 0x01, 4)?;
    let size = match delta.operand(command, 0x10, 3)? {
        0 => 0x10000,
        size => size,
    };
    let end = offset.checked_add(size).ok_or(INVALID)?;
    base.get(offset..end).ok_or(INVALID)
}

/// The result of applying `delta` to `base`.
///
/// # Errors
///
/// [`Error::Corrupt`] when the delta does not fit its base or is cut
/// short; [`Error::TooLarge`] when it declares a result larger than
/// `max_size`, which is checked before anything is allocated.
pub(crate) fn apply(base: &[u8], delta: &[u8], max_size: usize) -> Result<Vec<u8>, Error> {
    let mut delta = Bytes(delta);
    if delta.size()? != base.len() {
        return Err(INVALID);
    }
    let result_size = delta.size()?;
    if result_size > max_size {
        return Err(Error::TooLarge("delta result"));
    }
    let mut out = Vec::new();
    while !delta.0.is_empty() {
        let command = delta.byte()?;
        let piece = match command {
            0 => return Err(INVALID),
            1..=0x7f => delta.take(usize::from(command))?,
            _ => copy(command, &mut delta, base)?,
        };
        if out.len() + piece.len() > result_size {
            return Err(INVALID);
        }
        out.extend_from_slice(piece);
    }
    if out.len() == result_size {
        Ok(out)
    } else {
        Err(INVALID)
    }
}

#[cfg(test)]
mod tests;
