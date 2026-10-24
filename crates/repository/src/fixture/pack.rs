//! Packs, pack indexes and deltas built in memory for the unit tests.

use crate::fixture::zlib_stored;

/// The bytes of a delta's varint: seven bits at a time, low first.
pub(crate) fn delta_varint(mut value: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let low = (value & 0x7f) as u8;
        value >>= 7;
        out.push(if value > 0 { low | 0x80 } else { low });
        if value == 0 {
            return out;
        }
    }
}

/// A delta's copy operation: `size` bytes from `offset` of the base.
/// A size of zero means 0x10000, as git has it.
pub(crate) fn delta_copy(offset: usize, size: usize) -> Vec<u8> {
    let mut command = 0x80_u8;
    let mut operands = Vec::new();
    for i in 0..4 {
        let byte = (offset >> (8 * i)) as u8;
        if byte != 0 {
            command |= 1 << i;
            operands.push(byte);
        }
    }
    for i in 0..3 {
        let byte = (size >> (8 * i)) as u8;
        if byte != 0 {
            command |= 0x10 << i;
            operands.push(byte);
        }
    }
    [vec![command], operands].concat()
}

/// A delta's insert operation (at most 127 bytes).
pub(crate) fn delta_insert(data: &[u8]) -> Vec<u8> {
    [vec![u8::try_from(data.len()).unwrap()], data.to_vec()].concat()
}

/// A whole delta: the base and result sizes, then the operations.
pub(crate) fn delta(base_size: usize, result_size: usize, ops: &[Vec<u8>]) -> Vec<u8> {
    [
        delta_varint(base_size),
        delta_varint(result_size),
        ops.concat(),
    ]
    .concat()
}

/// An entry's header: its type and size.
pub(crate) fn entry_header(kind: u8, size: usize) -> Vec<u8> {
    let mut rest = size >> 4;
    let first = (kind << 4) | (size & 15) as u8;
    let mut out = vec![if rest > 0 { first | 0x80 } else { first }];
    while rest > 0 {
        let low = (rest & 0x7f) as u8;
        rest >>= 7;
        out.push(if rest > 0 { low | 0x80 } else { low });
    }
    out
}

/// How an offset delta writes the distance back to its base.
pub(crate) fn ofs_distance(mut distance: u64) -> Vec<u8> {
    let mut out = vec![(distance & 0x7f) as u8];
    distance >>= 7;
    while distance > 0 {
        distance -= 1;
        out.push(0x80 | (distance & 0x7f) as u8);
        distance >>= 7;
    }
    out.reverse();
    out
}

/// A version 2 pack index of `entries`, each an id and its offset in
/// the pack; offsets from 2^31 up go to the large offset table.
pub(crate) fn pack_index(entries: &[([u8; 20], u64)]) -> Vec<u8> {
    let mut sorted = entries.to_vec();
    sorted.sort();
    let mut out = vec![0xff, b't', b'O', b'c', 0, 0, 0, 2];
    for first in 0..=255_u8 {
        let n = sorted.iter().filter(|e| e.0[0] <= first).count();
        out.extend(u32::try_from(n).unwrap().to_be_bytes());
    }
    out.extend(sorted.iter().flat_map(|e| e.0));
    out.extend(vec![0; 4 * sorted.len()]);
    let mut large = Vec::new();
    for entry in &sorted {
        let word = if entry.1 >= 1 << 31 {
            large.push(entry.1);
            0x8000_0000 | u32::try_from(large.len() - 1).unwrap()
        } else {
            u32::try_from(entry.1).unwrap()
        };
        out.extend(word.to_be_bytes());
    }
    out.extend(large.iter().flat_map(|o| o.to_be_bytes()));
    out.extend([0; 40]);
    out
}

/// A reader every read of which fails, as a disk that went away does.
pub(crate) struct Failing;

impl std::io::Read for Failing {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }
}

impl std::io::Seek for Failing {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

/// A pack under construction, and the index that goes with it.
pub(crate) struct PackBuilder {
    pub(crate) bytes: Vec<u8>,
    entries: Vec<([u8; 20], u64)>,
}

impl PackBuilder {
    pub(crate) fn new() -> Self {
        Self {
            bytes: [b"PACK".to_vec(), vec![0, 0, 0, 2, 0, 0, 0, 0]].concat(),
            entries: Vec::new(),
        }
    }

    fn add(&mut self, id: [u8; 20], header: &[u8], payload: &[u8]) -> u64 {
        let offset = self.bytes.len() as u64;
        self.bytes.extend(header);
        self.bytes.extend(zlib_stored(payload));
        self.entries.push((id, offset));
        offset
    }

    /// A whole object of pack type `kind` (1 commit, 2 tree, 3 blob,
    /// 4 tag).
    pub(crate) fn whole(&mut self, id: [u8; 20], kind: u8, data: &[u8]) -> u64 {
        self.add(id, &entry_header(kind, data.len()), data)
    }

    /// A delta against the entry at `base`.
    pub(crate) fn offset_delta(&mut self, id: [u8; 20], base: u64, delta: &[u8]) -> u64 {
        let distance = self.bytes.len() as u64 - base;
        let header = [entry_header(6, delta.len()), ofs_distance(distance)].concat();
        self.add(id, &header, delta)
    }

    /// A delta against the object `base`, wherever it is.
    pub(crate) fn reference_delta(&mut self, id: [u8; 20], base: [u8; 20], delta: &[u8]) -> u64 {
        let header = [entry_header(7, delta.len()), base.to_vec()].concat();
        self.add(id, &header, delta)
    }

    /// The pack and its index.
    pub(crate) fn finish(mut self) -> (Vec<u8>, Vec<u8>) {
        let count = u32::try_from(self.entries.len()).unwrap();
        self.bytes[8..12].copy_from_slice(&count.to_be_bytes());
        self.bytes.extend([0; 20]);
        let index = pack_index(&self.entries);
        (self.bytes, index)
    }
}
