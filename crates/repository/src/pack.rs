//! Packfiles and their version 2 indexes, read through a reader that
//! can seek, so a unit test feeds bytes from memory.

use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::inflate::{read_byte, zlib_inflate};
use crate::object::{Kind, Object};
use crate::{CommitId, Error};

/// An entry of a pack: a whole object, or a delta and what it is
/// against.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// An object stored whole.
    Whole(Object),
    /// A delta against the entry at this offset of the same pack.
    OffsetDelta { base: u64, delta: Vec<u8> },
    /// A delta against the object with this id, wherever it is.
    ReferenceDelta { base: CommitId, delta: Vec<u8> },
}

/// The bytes before an index's fanout table: its magic and version.
const INDEX_HEADER: u64 = 8;
/// The fanout table: 256 counts of four bytes.
const FANOUT: u64 = 256 * 4;
/// The two checksums that end an index.
const INDEX_TRAILER: u64 = 40;
/// The widest an entry's size may be, in bits.
const SIZE_BITS: u32 = 60;

const CUT: Error = Error::Corrupt("pack data is cut short");
const BAD_INDEX: Error = Error::Corrupt("pack index contradicts its size");
const BAD_ENTRY: Error = Error::Corrupt("pack entry is invalid");

/// Fills `buf` from `at`; a short file is corrupt, not an I/O failure.
fn read_at<R: Read + Seek>(reader: &mut R, at: u64, buf: &mut [u8]) -> Result<(), Error> {
    reader
        .seek(SeekFrom::Start(at))
        .map_err(Error::io("a pack file"))?;
    reader.read_exact(buf).map_err(|e| match e.kind() {
        std::io::ErrorKind::UnexpectedEof => CUT,
        _ => Error::io("a pack file")(e),
    })
}

fn read_u32<R: Read + Seek>(reader: &mut R, at: u64) -> Result<u32, Error> {
    let mut buf = [0; 4];
    read_at(reader, at, &mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// Where the sorted ids, the offsets and the large offsets of an index
/// of `count` objects start.
struct Layout {
    count: u64,
}

impl Layout {
    fn ids(&self) -> u64 {
        INDEX_HEADER + FANOUT
    }

    fn offsets(&self) -> u64 {
        self.ids() + 24 * self.count
    }

    fn large_offsets(&self) -> u64 {
        self.offsets() + 4 * self.count
    }
}

/// The index's layout, once its header and its size agree.
fn index_layout<R: Read + Seek>(index: &mut R) -> Result<Layout, Error> {
    let mut header = [0; 8];
    read_at(index, 0, &mut header)?;
    if header != [0xff, b't', b'O', b'c', 0, 0, 0, 2] {
        return Err(Error::Unsupported("pack index is not version 2"));
    }
    let count = u64::from(read_u32(index, INDEX_HEADER + FANOUT - 4)?);
    let layout = Layout { count };
    let length = index
        .seek(SeekFrom::End(0))
        .map_err(Error::io("a pack file"))?;
    if length < layout.large_offsets() + INDEX_TRAILER {
        return Err(BAD_INDEX);
    }
    Ok(layout)
}

/// The positions in the sorted ids that objects starting with `first`
/// can be at.
fn bucket<R: Read + Seek>(index: &mut R, layout: &Layout, first: u8) -> Result<(u64, u64), Error> {
    let end = u64::from(read_u32(index, INDEX_HEADER + 4 * u64::from(first))?);
    let start = match first {
        0 => 0,
        _ => u64::from(read_u32(index, INDEX_HEADER + 4 * u64::from(first - 1))?),
    };
    if start > end || end > layout.count {
        return Err(BAD_INDEX);
    }
    Ok((start, end))
}

/// The position among the sorted ids of `wanted`, searching
/// `start..end`.
fn search<R: Read + Seek>(
    index: &mut R,
    layout: &Layout,
    (mut start, mut end): (u64, u64),
    wanted: &[u8; 20],
) -> Result<Option<u64>, Error> {
    while start < end {
        let middle = start + (end - start) / 2;
        let mut found = [0; 20];
        read_at(index, layout.ids() + 20 * middle, &mut found)?;
        match found.cmp(wanted) {
            std::cmp::Ordering::Less => start = middle + 1,
            std::cmp::Ordering::Greater => end = middle,
            std::cmp::Ordering::Equal => return Ok(Some(middle)),
        }
    }
    Ok(None)
}

/// The offset in the pack of the object `id`, if the index lists it.
///
/// # Errors
///
/// [`Error::Unsupported`] for an index that is not version 2;
/// [`Error::Corrupt`] for one that contradicts its own size.
pub(crate) fn find_offset<R: Read + Seek>(
    index: &mut R,
    id: &CommitId,
) -> Result<Option<u64>, Error> {
    let layout = index_layout(index)?;
    let wanted = id.to_bytes();
    let range = bucket(index, &layout, wanted[0])?;
    let Some(position) = search(index, &layout, range, &wanted)? else {
        return Ok(None);
    };
    let word = read_u32(index, layout.offsets() + 4 * position)?;
    if word & 0x8000_0000 == 0 {
        return Ok(Some(u64::from(word)));
    }
    let at = layout.large_offsets() + 8 * u64::from(word & 0x7fff_ffff);
    let length = index
        .seek(SeekFrom::End(0))
        .map_err(Error::io("a pack file"))?;
    if at + 8 > length - INDEX_TRAILER {
        return Err(BAD_INDEX);
    }
    let mut large = [0; 8];
    read_at(index, at, &mut large)?;
    Ok(Some(u64::from_be_bytes(large)))
}

/// Checks the pack's 12-byte header.
///
/// # Errors
///
/// [`Error::Corrupt`] when it is not a pack; [`Error::Unsupported`]
/// for a version other than 2 and 3.
pub(crate) fn check_header<R: Read + Seek>(pack: &mut R) -> Result<(), Error> {
    let mut header = [0; 12];
    read_at(pack, 0, &mut header)?;
    if &header[..4] != b"PACK" {
        return Err(Error::Corrupt("file is not a pack"));
    }
    match header[4..8] {
        [0, 0, 0, 2 | 3] => Ok(()),
        _ => Err(Error::Unsupported("pack version")),
    }
}

/// An entry's type and size.
fn entry_header<R: std::io::BufRead>(reader: &mut R) -> Result<(u8, u64), Error> {
    let first = read_byte(reader)?;
    let (mut size, mut shift, mut more) = (u64::from(first & 15), 4, first & 0x80 != 0);
    while more {
        if shift >= SIZE_BITS {
            return Err(BAD_ENTRY);
        }
        let byte = read_byte(reader)?;
        size |= u64::from(byte & 0x7f) << shift;
        shift += 7;
        more = byte & 0x80 != 0;
    }
    Ok(((first >> 4) & 7, size))
}

/// The distance back to an offset delta's base.
fn base_distance<R: std::io::BufRead>(reader: &mut R) -> Result<u64, Error> {
    let mut byte = read_byte(reader)?;
    let mut distance = u64::from(byte & 0x7f);
    while byte & 0x80 != 0 {
        byte = read_byte(reader)?;
        let moved = distance.checked_add(1).and_then(|d| d.checked_mul(128));
        distance = moved.ok_or(BAD_ENTRY)? | u64::from(byte & 0x7f);
    }
    Ok(distance)
}

/// The inflated data of an entry, which must be `size` bytes.
fn payload<R: std::io::BufRead>(reader: &mut R, size: usize) -> Result<Vec<u8>, Error> {
    let data = zlib_inflate(reader, size)?;
    if data.len() == size {
        Ok(data)
    } else {
        Err(BAD_ENTRY)
    }
}

/// The entry at `offset`, its data inflated to no more than `max_size`.
///
/// # Errors
///
/// [`Error::Corrupt`] for an entry that is cut short or contradicts its
/// own header; [`Error::TooLarge`] for one that declares more than
/// `max_size` bytes.
pub(crate) fn read_entry<R: Read + Seek>(
    pack: &mut R,
    offset: u64,
    max_size: usize,
) -> Result<Entry, Error> {
    pack.seek(SeekFrom::Start(offset))
        .map_err(Error::io("a pack file"))?;
    let mut reader = BufReader::new(pack);
    let (code, size) = entry_header(&mut reader)?;
    let size = usize::try_from(size)
        .ok()
        .filter(|&size| size <= max_size)
        .ok_or(Error::TooLarge("pack entry"))?;
    match code {
        6 => {
            let base = base_distance(&mut reader)?;
            let base = offset.checked_sub(base).filter(|_| base > 0);
            let base = base.ok_or(BAD_ENTRY)?;
            let delta = payload(&mut reader, size)?;
            Ok(Entry::OffsetDelta { base, delta })
        }
        7 => {
            let mut base = [0; 20];
            for byte in &mut base {
                *byte = read_byte(&mut reader)?;
            }
            let delta = payload(&mut reader, size)?;
            let base = CommitId::from_bytes(&base);
            Ok(Entry::ReferenceDelta { base, delta })
        }
        _ => {
            let kind = Kind::from_pack(code).ok_or(BAD_ENTRY)?;
            let data = payload(&mut reader, size)?;
            Ok(Entry::Whole(Object { kind, data }))
        }
    }
}

#[cfg(test)]
mod tests;
