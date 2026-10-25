//! The keys a node holds and what they sign (PRV-11, PRV-12): its
//! device key, its seats' keys and its recovery keys, each minted from
//! the OS's random source; signatures that name their kind, so one
//! kind never verifies as another; seat certificates; and key events.
//! On Linux the keys live in files only the principal's OS user reads
//! (the plan's ruling 2; phase 5 builds the other platforms' key
//! stores).

pub mod certificate;
pub mod file;
pub mod key_event;
pub mod key_store;
pub mod random;
pub mod signing;

pub use signing::KeyError;
