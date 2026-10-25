//! The workspace's one crate that may hold unsafe code (SEC-10, REC-24,
//! ENG-09): the calls into the OS that safe Rust cannot make, each in a
//! module of its own behind a trait a test fakes, so Miri checks every
//! pointer the module handles on any OS.
//!
//! - [`identity`]: the host id behind the node identity, the hardware UUID on
//!   macOS and the `MachineGuid` on Windows (REC-24).
//! - [`dpapi`]: Windows' DPAPI, protecting a key's bytes for the running
//!   OS user before they reach a file.
//!
//! The workspace forbids unsafe code; this crate denies it in its own
//! lints, and only the modules that allow it hold any.

#[allow(unsafe_code)]
pub mod dpapi;

#[allow(unsafe_code)]
pub mod identity;
