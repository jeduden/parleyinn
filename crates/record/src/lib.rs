//! The record's laws (§8.2, §8.3): what an event's header holds, how
//! each writer's events chain, how an event commits to its content, and
//! the canonical encoding all three hash. Nothing here reads a file, a
//! clock or randomness: a caller hands in the commitment key and the
//! events, so every result is a function of its inputs (I10).
//!
//! - [`canonical`]: RFC 8785, the JSON Canonicalization Scheme.
//! - [`digest`]: SHA-256 and HMAC-SHA-256 over it.
//! - [`id`]: writer, room and seat ids, `seq` and the address.
//! - [`event`]: the event kinds and the provenance classes (PRV-01).
//! - [`commitment`]: the commitment to an event's content (REC-17).
//! - [`header`]: the event header and its hash (REC-10).
//! - [`fork`]: two different events at one address (PEER-10).
//! - [`chain`]: each writer's hash chain, the heads it cites, and the
//!   causal order they give (REC-06, REC-10).

pub mod canonical;
pub mod chain;
pub mod commitment;
pub mod digest;
pub mod event;
pub mod fork;
pub mod header;
pub mod id;
