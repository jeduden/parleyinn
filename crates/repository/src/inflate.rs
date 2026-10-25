//! zlib decompression (RFC 1950, RFC 1951), written here because the
//! core is at its dependency budget (ENG-18). It reads a stream and
//! stops at the end of the zlib data, never past it; every output is
//! bounded by a maximum size the caller sets, so a hostile object
//! cannot make the core allocate without limit.

use std::io::BufRead;

use crate::Error;

/// The longest code, in bits.
const MAX_BITS: usize = 15;

/// The base length of each length symbol, from 257.
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];

/// The extra bits of each length symbol.
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

/// The base distance of each distance symbol.
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];

/// The extra bits of each distance symbol.
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// The order the code length code's lengths are stored in.
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

const TRUNCATED: Error = Error::Corrupt("zlib stream is cut short");
const INVALID: Error = Error::Corrupt("zlib stream is invalid");

/// The next byte of `reader`; it fails when the data is cut short.
pub(crate) fn read_byte<R: BufRead>(reader: &mut R) -> Result<u8, Error> {
    let chunk = reader.fill_buf().map_err(Error::io("a zlib stream"))?;
    let byte = *chunk.first().ok_or(TRUNCATED)?;
    reader.consume(1);
    Ok(byte)
}

/// A canonical Huffman code, built from the lengths of its symbols.
pub(crate) struct Huffman {
    /// How many symbols have each length.
    count: [u16; MAX_BITS + 1],
    /// The symbols, ordered by length and then by value.
    symbol: Vec<u16>,
}

impl Huffman {
    /// The code whose symbol `n` has `lengths[n]` bits, or an error
    /// when the lengths give more codes than the bits can tell apart.
    /// A code with spare room is accepted: a bit pattern no symbol has
    /// fails when it is read.
    pub(crate) fn build(lengths: &[u8]) -> Result<Self, Error> {
        let mut count = [0_u16; MAX_BITS + 1];
        for &length in lengths {
            *count.get_mut(usize::from(length)).ok_or(INVALID)? += 1;
        }
        let mut left = 1_i32;
        for &n in count.iter().skip(1) {
            left = (left << 1) - i32::from(n);
            if left < 0 {
                return Err(INVALID);
            }
        }
        let mut symbol = Vec::with_capacity(lengths.len());
        for length in 1..=MAX_BITS {
            let of_length = lengths
                .iter()
                .enumerate()
                .filter(|(_, l)| usize::from(**l) == length);
            symbol.extend(of_length.filter_map(|(sym, _)| u16::try_from(sym).ok()));
        }
        Ok(Self { count, symbol })
    }
}

/// A reader of bits, least significant first, over a byte stream.
pub(crate) struct Bits<R> {
    reader: R,
    /// Bits taken from the stream and not yet used.
    held: u32,
    /// How many bits `held` has.
    holding: u32,
}

impl<R: BufRead> Bits<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            held: 0,
            holding: 0,
        }
    }

    /// The next whole byte of the stream.
    fn byte(&mut self) -> Result<u8, Error> {
        read_byte(&mut self.reader)
    }

    /// The next `count` bits (at most 16) as a number.
    fn bits(&mut self, count: u32) -> Result<u32, Error> {
        let mut value = self.held;
        while self.holding < count {
            value |= u32::from(self.byte()?) << self.holding;
            self.holding += 8;
        }
        self.held = value >> count;
        self.holding -= count;
        Ok(value & ((1 << count) - 1))
    }

    /// Drops the bits left of the current byte.
    fn align(&mut self) {
        self.held = 0;
        self.holding = 0;
    }

    /// The next symbol of `code`.
    pub(crate) fn decode(&mut self, code: &Huffman) -> Result<u16, Error> {
        let (mut bits, mut first, mut index) = (0_i32, 0_i32, 0_i32);
        for &n in code.count.iter().skip(1) {
            bits |= self.bits(1)? as i32;
            let n = i32::from(n);
            if bits - n < first {
                let at = usize::try_from(index + (bits - first)).map_err(|_| INVALID)?;
                return code.symbol.get(at).copied().ok_or(INVALID);
            }
            index += n;
            first = (first + n) << 1;
            bits <<= 1;
        }
        Err(INVALID)
    }
}

/// The output so far, held to its maximum size.
struct Output {
    data: Vec<u8>,
    max: usize,
}

impl Output {
    fn push(&mut self, byte: u8) -> Result<(), Error> {
        if self.data.len() >= self.max {
            return Err(Error::TooLarge("inflated data"));
        }
        self.data.push(byte);
        Ok(())
    }

    /// Repeats `length` bytes from `distance` bytes back.
    fn copy(&mut self, distance: usize, length: usize) -> Result<(), Error> {
        if distance == 0 || distance > self.data.len() {
            return Err(INVALID);
        }
        for _ in 0..length {
            let byte = self.data[self.data.len() - distance];
            self.push(byte)?;
        }
        Ok(())
    }
}

/// The Adler-32 checksum of `data` (RFC 1950).
pub(crate) fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let (mut a, mut b) = (1_u32, 0_u32);
    for &byte in data {
        a = (a + u32::from(byte)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

/// Checks the two header bytes of a zlib stream.
fn header<R: BufRead>(bits: &mut Bits<R>) -> Result<(), Error> {
    let cmf = u32::from(bits.byte()?);
    let flags = u32::from(bits.byte()?);
    let deflate = cmf & 15 == 8 && cmf >> 4 <= 7;
    let checked = ((cmf << 8) | flags) % 31 == 0;
    if !deflate || !checked {
        return Err(INVALID);
    }
    if flags & 0x20 != 0 {
        return Err(Error::Corrupt("zlib stream wants a preset dictionary"));
    }
    Ok(())
}

/// A stored block: bytes copied as they are.
fn stored<R: BufRead>(bits: &mut Bits<R>, out: &mut Output) -> Result<(), Error> {
    bits.align();
    let length = u16::from_le_bytes([bits.byte()?, bits.byte()?]);
    let complement = u16::from_le_bytes([bits.byte()?, bits.byte()?]);
    if length != !complement {
        return Err(INVALID);
    }
    for _ in 0..length {
        let byte = bits.byte()?;
        out.push(byte)?;
    }
    Ok(())
}

/// The codes of RFC 1951's fixed block.
fn fixed_codes() -> Result<(Huffman, Huffman), Error> {
    let mut lengths = [8_u8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    Ok((Huffman::build(&lengths)?, Huffman::build(&[5; 30])?))
}

/// The literal and distance codes a dynamic block's header describes.
fn dynamic_codes<R: BufRead>(bits: &mut Bits<R>) -> Result<(Huffman, Huffman), Error> {
    let literals = bits.bits(5)? as usize + 257;
    let distances = bits.bits(5)? as usize + 1;
    let header_codes = bits.bits(4)? as usize + 4;
    if literals > 286 || distances > 30 {
        return Err(INVALID);
    }
    let mut header_lengths = [0_u8; 19];
    for &sym in CODE_LENGTH_ORDER.iter().take(header_codes) {
        header_lengths[sym] = bits.bits(3)? as u8;
    }
    let header_code = Huffman::build(&header_lengths)?;
    let lengths = code_lengths(bits, &header_code, literals + distances)?;
    let (literal_lengths, distance_lengths) = lengths.split_at(literals);
    Ok((
        Huffman::build(literal_lengths)?,
        Huffman::build(distance_lengths)?,
    ))
}

/// `total` code lengths, in the code length code's run-length form.
fn code_lengths<R: BufRead>(
    bits: &mut Bits<R>,
    header_code: &Huffman,
    total: usize,
) -> Result<Vec<u8>, Error> {
    let mut lengths: Vec<u8> = Vec::with_capacity(total);
    while lengths.len() < total {
        let sym = bits.decode(header_code)?;
        let (value, repeat) = match sym {
            0..=15 => (sym as u8, 1),
            16 => (*lengths.last().ok_or(INVALID)?, 3 + bits.bits(2)? as usize),
            17 => (0, 3 + bits.bits(3)? as usize),
            _ => (0, 11 + bits.bits(7)? as usize),
        };
        if lengths.len() + repeat > total {
            return Err(INVALID);
        }
        lengths.resize(lengths.len() + repeat, value);
    }
    Ok(lengths)
}

/// The bytes of a match: its length symbol, then its distance.
fn matched<R: BufRead>(
    bits: &mut Bits<R>,
    out: &mut Output,
    symbol: u16,
    distances: &Huffman,
) -> Result<(), Error> {
    let at = usize::from(symbol - 257);
    let base = LENGTH_BASE.get(at).ok_or(INVALID)?;
    let extra = LENGTH_EXTRA.get(at).ok_or(INVALID)?;
    let length = usize::from(*base) + bits.bits(u32::from(*extra))? as usize;
    let at = usize::from(bits.decode(distances)?);
    let base = DIST_BASE.get(at).ok_or(INVALID)?;
    let extra = DIST_EXTRA.get(at).ok_or(INVALID)?;
    let distance = usize::from(*base) + bits.bits(u32::from(*extra))? as usize;
    out.copy(distance, length)
}

/// A block of literals and matches, up to its end-of-block symbol.
fn compressed<R: BufRead>(
    bits: &mut Bits<R>,
    out: &mut Output,
    (literals, distances): &(Huffman, Huffman),
) -> Result<(), Error> {
    loop {
        match bits.decode(literals)? {
            256 => return Ok(()),
            symbol @ 0..=255 => out.push(symbol as u8)?,
            symbol => matched(bits, out, symbol, distances)?,
        }
    }
}

/// The data of the zlib stream `reader` holds, read up to the end of
/// the stream and no further.
///
/// # Errors
///
/// [`Error::Corrupt`] when the stream is not valid zlib, is cut short
/// or fails its checksum; [`Error::TooLarge`] when the data would pass
/// `max_size` bytes, which is checked before each byte is stored.
pub(crate) fn zlib_inflate<R: BufRead>(reader: R, max_size: usize) -> Result<Vec<u8>, Error> {
    let mut bits = Bits::new(reader);
    header(&mut bits)?;
    let mut out = Output {
        data: Vec::new(),
        max: max_size,
    };
    loop {
        let last = bits.bits(1)? == 1;
        match bits.bits(2)? {
            0 => stored(&mut bits, &mut out)?,
            1 => compressed(&mut bits, &mut out, &fixed_codes()?)?,
            2 => {
                let codes = dynamic_codes(&mut bits)?;
                compressed(&mut bits, &mut out, &codes)?;
            }
            _ => return Err(INVALID),
        }
        if last {
            break;
        }
    }
    bits.align();
    let stored_sum = u32::from_be_bytes([bits.byte()?, bits.byte()?, bits.byte()?, bits.byte()?]);
    if stored_sum != adler32(&out.data) {
        return Err(Error::Corrupt("zlib checksum does not match"));
    }
    Ok(out.data)
}

#[cfg(test)]
mod tests;
