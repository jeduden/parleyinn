//! SHA-1, written here so the crate names an object by the hash of what
//! it read and takes no dependency for it (ENG-18). Git's object ids are
//! SHA-1 of `<kind> <length>\0<body>`; what this guards against is a bit
//! flip or a planted file, not a chosen-prefix collision.

use crate::CommitId;

const START: [u32; 5] = [
    0x6745_2301,
    0xefcd_ab89,
    0x98ba_dcfe,
    0x1032_5476,
    0xc3d2_e1f0,
];

const BLOCK: usize = 64;

/// A SHA-1 computation in progress.
pub(crate) struct Sha1 {
    state: [u32; 5],
    block: [u8; BLOCK],
    filled: usize,
    length: u64,
}

/// The round function and constant of round `t`.
fn round(t: usize, b: u32, c: u32, d: u32) -> (u32, u32) {
    match t / 20 {
        0 => ((b & c) | (!b & d), 0x5a82_7999),
        1 => (b ^ c ^ d, 0x6ed9_eba1),
        2 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
        _ => (b ^ c ^ d, 0xca62_c1d6),
    }
}

/// The 80 words a block expands to.
fn schedule(block: &[u8; BLOCK]) -> [u32; 80] {
    let mut w = [0_u32; 80];
    for (word, bytes) in w.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    for t in 16..80 {
        w[t] = (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1);
    }
    w
}

impl Sha1 {
    pub(crate) fn new() -> Self {
        Self {
            state: START,
            block: [0; BLOCK],
            filled: 0,
            length: 0,
        }
    }

    /// Mixes the full block in `self.block` into the state.
    fn compress(&mut self) {
        let w = schedule(&self.block);
        let [mut a, mut b, mut c, mut d, mut e] = self.state;
        for (t, word) in w.iter().enumerate() {
            let (f, k) = round(t, b, c, d);
            let next = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            (a, b, c, d, e) = (next, a, b.rotate_left(30), c, d);
        }
        for (state, add) in self.state.iter_mut().zip([a, b, c, d, e]) {
            *state = state.wrapping_add(add);
        }
    }

    pub(crate) fn update(&mut self, data: &[u8]) {
        self.length = self.length.wrapping_add(data.len() as u64);
        let mut rest = data;
        while !rest.is_empty() {
            let take = rest.len().min(BLOCK - self.filled);
            let (now, later) = rest.split_at(take);
            self.block[self.filled..self.filled + take].copy_from_slice(now);
            self.filled += take;
            rest = later;
            if self.filled == BLOCK {
                self.compress();
                self.filled = 0;
            }
        }
    }

    pub(crate) fn finish(mut self) -> [u8; 20] {
        let bits = self.length.wrapping_mul(8);
        self.update(&[0x80]);
        while self.filled != BLOCK - 8 {
            self.update(&[0]);
        }
        self.update(&bits.to_be_bytes());
        let mut out = [0; 20];
        for (bytes, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(self.state) {
            *bytes = word.to_be_bytes();
        }
        out
    }
}

/// The id git gives the object of `kind` with `body`.
pub(crate) fn object_id(kind: &str, body: &[u8]) -> CommitId {
    let mut sha = Sha1::new();
    sha.update(format!("{kind} {}\0", body.len()).as_bytes());
    sha.update(body);
    CommitId::from_bytes(&sha.finish())
}

#[cfg(test)]
mod tests;
