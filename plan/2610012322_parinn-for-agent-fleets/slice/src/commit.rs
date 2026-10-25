//! The commitment of REC-17: `C = HMAC-SHA256(k_e, canonical content)`
//! under a random 256-bit commitment key `k_e` kept with the content.
//!
//! Hiding: HMAC under a key the reader lacks is a PRF, so `C` says
//! nothing about the content. Binding: two openings `(k, m) != (k', m')`
//! with one `C` would give a SHA-256 collision. Erase `k_e` with the
//! content and no guess at the content can be checked against `C`.

use sha2::{Digest, Sha256};
use std::io::Read;

use crate::Result;

pub type Key = [u8; 32];
pub type Commitment = [u8; 32];

/// HMAC-SHA256 (RFC 2104) for a key of at most one block.
pub fn hmac(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let pad = |x: u8| block.map(|b| b ^ x);
    let inner = Sha256::new()
        .chain_update(pad(0x36))
        .chain_update(msg)
        .finalize();
    Sha256::new()
        .chain_update(pad(0x5c))
        .chain_update(inner)
        .finalize()
        .into()
}

/// The commitment to canonical content under its commitment key.
pub fn commit(key: &Key, canonical: &[u8]) -> Commitment {
    hmac(key, canonical)
}

/// A fresh commitment key from the OS's random source.
pub fn fresh_key() -> Result<Key> {
    let mut k = [0u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut k)?;
    Ok(k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    #[test]
    fn hmac_matches_rfc_4231_case_2() {
        assert_eq!(
            hex(&hmac(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn hmac_hashes_a_key_longer_than_a_block() {
        // RFC 4231 case 6: a 131-byte key of 0xaa.
        assert_eq!(
            hex(&hmac(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn fresh_keys_differ() {
        assert_ne!(fresh_key().unwrap(), fresh_key().unwrap());
    }
}
