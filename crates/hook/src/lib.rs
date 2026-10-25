//! The hook handlers' entry point (§9.1): it names the hooks ParleyInn
//! installs, decodes each hook input, and catches a handler's panic, so
//! `parinn hook` fails open on every path (ENG-04, I9). Each failure is
//! typed for the counter its audit entry feeds (OPS-01).
//!
//! - [`input`]: the hook input decoder. A hook input is held to its
//!   limits (SEC-05), must be a JSON object, and each field a handler
//!   uses must pass the schema (SEC-16); its keys are then checked
//!   against the hook input shapes recorded for the supported versions
//!   ([`shape`], REC-26).
//! - [`ancestry`]: the process ancestry a hook handler reads inside the
//!   core, which tells a nested harness session (PRV-08).

pub mod ancestry;
pub mod input;
pub mod shape;

use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::time::Duration;

use audit::entry::{Counter, Kind, Occurrence};

pub use input::{Decoded, HookInput, decode};

/// The most bytes a hook input may hold (SEC-05).
pub const HOOK_INPUT_LIMIT: usize = 1 << 20;

/// The deepest a hook input's JSON may nest (SEC-05).
pub const NESTING_LIMIT: usize = 64;

/// A hook ParleyInn installs: only those through which it writes to an
/// agent (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hook {
    SessionStart,
    UserPromptSubmit,
    PermissionRequest,
}

impl Hook {
    /// The hook the harness names `name`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "SessionStart" => Some(Self::SessionStart),
            "UserPromptSubmit" => Some(Self::UserPromptSubmit),
            "PermissionRequest" => Some(Self::PermissionRequest),
            _ => None,
        }
    }

    /// The name the harness gives it, its hook input's
    /// `hook_event_name`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::UserPromptSubmit => "UserPromptSubmit",
            Self::PermissionRequest => "PermissionRequest",
        }
    }

    /// The operation its audit entries name.
    #[must_use]
    pub fn operation(self) -> &'static str {
        match self {
            Self::SessionStart => "hook SessionStart",
            Self::UserPromptSubmit => "hook UserPromptSubmit",
            Self::PermissionRequest => "hook PermissionRequest",
        }
    }

    /// Its hook budget (§9.1, NFR-02): how long its handler may take,
    /// from its start, before it answers empty.
    #[must_use]
    pub fn budget(self) -> Duration {
        match self {
            Self::SessionStart => Duration::from_millis(150),
            Self::UserPromptSubmit | Self::PermissionRequest => Duration::from_millis(50),
        }
    }

    /// The harness timeout ParleyInn installs it with (ASM-10), above its
    /// hook budget: a start marker older than this was left by a
    /// handler the harness stopped (NFR-02).
    #[must_use]
    pub fn harness_timeout(self) -> Duration {
        match self {
            Self::SessionStart | Self::UserPromptSubmit => Duration::from_secs(5),
            Self::PermissionRequest => Duration::from_secs(30),
        }
    }
}

/// Why a hook handler injected nothing; each feeds its own counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// `parinn hook` named no hook ParleyInn installs.
    UnknownHook,
    /// The hook input is no JSON object.
    Malformed,
    /// A field the handler uses failed the schema (SEC-16).
    Invalid,
    /// The handler panicked, and the panic was caught.
    Panicked,
    /// The hook input exceeds [`HOOK_INPUT_LIMIT`].
    TooLarge,
    /// The hook input nests deeper than [`NESTING_LIMIT`].
    TooDeep,
}

impl Failure {
    /// The audit entry it makes for `operation`.
    #[must_use]
    pub fn occurrence(self, operation: &'static str) -> Occurrence<'static> {
        let (kind, counter) = match self {
            Self::UnknownHook | Self::Malformed => (Kind::Rejected, Counter::HookRejected),
            Self::Invalid => (Kind::Rejected, Counter::HookInputRejected),
            Self::Panicked => (Kind::Failed, Counter::HookPanicked),
            Self::TooLarge => (Kind::Rejected, Counter::HookInputTooLarge),
            Self::TooDeep => (Kind::Rejected, Counter::HookInputTooDeep),
        };

        Occurrence::Degraded {
            kind,
            counter,
            operation,
            detail: self.detail(),
        }
    }

    /// Its fixed text.
    fn detail(self) -> &'static str {
        match self {
            Self::UnknownHook => "no such hook",
            Self::Malformed => "not a JSON object",
            Self::Invalid => "hook input failed schema validation",
            Self::Panicked => "panic recovered",
            Self::TooLarge => "hook input exceeds 1 MiB",
            Self::TooDeep => "hook input nesting exceeds depth 64",
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.detail())
    }
}

impl std::error::Error for Failure {}

/// Runs `handler`, catching a panic in it as [`Failure::Panicked`] so
/// no panic leaves the hook's entry point (ENG-04). The caller keeps the
/// default panic message off stderr.
///
/// # Errors
///
/// What `handler` fails with, or the caught panic.
pub fn guarded<T>(handler: impl FnOnce() -> Result<T, Failure>) -> Result<T, Failure> {
    panic::catch_unwind(AssertUnwindSafe(handler)).unwrap_or(Err(Failure::Panicked))
}

#[cfg(test)]
mod tests;
