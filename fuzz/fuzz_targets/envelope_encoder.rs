//! ENG-07, SEC-06: the envelope's encoder on hostile content. The input
//! is split at U+0001 into the content of one item each, which also
//! fills every free field of its item. The rendering must parse, hold
//! no top-level field but the closed set, and give each content back
//! exactly as one string value, so no recalled text alters the
//! envelope's structure. The checks are in the file the corpus replay
//! test shares.
#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../crates/envelope/tests/support/mod.rs"]
mod harness;

fuzz_target!(|data: &[u8]| {
    harness::encode(data);
});
