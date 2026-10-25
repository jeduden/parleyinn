//! The node identity (REC-24): a value outside the home that a cloned
//! image or a restored snapshot cannot carry over, read before the first
//! append of each start. The home keeps only its digest, never the value.
//!
//! On Linux it is the machine id systemd keeps, `/etc/machine-id`, or
//! D-Bus's copy of it. On macOS it is the hardware UUID, on Windows the
//! `MachineGuid`, both read through the `platform` crate.

use std::fs;
use std::path::PathBuf;

use key::file::{staging, sync_directory, write_secret};
use platform::identity::{HostId, Os};
use record::digest::Digest;

use crate::AdminError;
use crate::home::Home;

/// The label a node identity's digest is derived under.
const LABEL: &[u8] = b"parinn node identity v1\0";

/// Where a node identity is read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentitySource {
    /// The first of these files that holds a value.
    Files(Vec<PathBuf>),
    /// The macOS hardware UUID.
    HardwareUuid,
    /// The Windows `MachineGuid`.
    MachineGuid,
    /// This OS has no source ParleyInn reads: the identity is not checked.
    Unsupported,
}

impl IdentitySource {
    /// This OS's source: the machine id on Linux.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn platform() -> Self {
        Self::linux()
    }

    /// This OS's source: the hardware UUID on macOS.
    #[cfg(target_os = "macos")]
    #[must_use]
    pub fn platform() -> Self {
        Self::HardwareUuid
    }

    /// This OS's source: the `MachineGuid` on Windows.
    #[cfg(windows)]
    #[must_use]
    pub fn platform() -> Self {
        Self::MachineGuid
    }

    /// This OS's source: none.
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    #[must_use]
    pub fn platform() -> Self {
        Self::Unsupported
    }

    /// What the identity is checked against, or none when it is not
    /// checked.
    #[must_use]
    pub fn describe(&self) -> Option<&'static str> {
        match self {
            Self::Files(_) if *self == Self::linux() => Some("Linux machine id"),
            Self::Files(_) => Some("a file"),
            Self::HardwareUuid => Some("macOS hardware UUID"),
            Self::MachineGuid => Some("Windows MachineGuid"),
            Self::Unsupported => None,
        }
    }

    /// Linux's source, whatever the running OS.
    fn linux() -> Self {
        Self::Files(vec![
            PathBuf::from("/etc/machine-id"),
            PathBuf::from("/var/lib/dbus/machine-id"),
        ])
    }

    /// The value the first file that holds one holds, trimmed, or the
    /// running OS's host id; none when unavailable.
    #[must_use]
    pub fn read(&self) -> Option<String> {
        self.read_with(&Os)
    }

    /// As [`read`](Self::read), with `host` standing for the running
    /// OS's host id.
    #[must_use]
    pub fn read_with(&self, host: &dyn HostId) -> Option<String> {
        match self {
            Self::Files(files) => files.iter().find_map(|f| {
                fs::read_to_string(f)
                    .ok()
                    .map(|v| v.trim().to_owned())
                    .filter(|v| !v.is_empty())
            }),
            Self::HardwareUuid | Self::MachineGuid => host.read(),
            Self::Unsupported => None,
        }
    }

    /// The value, read again once when the first read finds none
    /// (REC-24 lets ParleyInn retry within the hook budgets).
    #[must_use]
    pub fn observe(&self) -> Option<String> {
        self.read().or_else(|| self.read())
    }
}

/// The digest a home keeps of the node identity `value`; an unavailable
/// value has a digest of its own, so a node that never reads one changes
/// once, not at every start.
#[must_use]
pub fn digest(value: Option<&str>) -> Digest {
    match value {
        Some(v) => Digest::of(&[LABEL, b"value\0", v.as_bytes()]),
        None => Digest::of(&[LABEL, b"unavailable"]),
    }
}

/// Whether the node identity changed since `home` last recorded it:
/// the digest of the one it reads now when it did. A home that recorded
/// none yet records it now and counts as unchanged; an unsupported
/// source is never checked.
///
/// # Errors
///
/// Fails when the recorded digest cannot be read or written.
pub fn changed(home: &Home) -> Result<Option<Digest>, AdminError> {
    if *home.identity() == IdentitySource::Unsupported {
        return Ok(None);
    }
    let current = digest(home.identity().observe().as_deref());
    let path = home.node_identity();
    let recorded = match fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            record(home, current)?;
            return Ok(None);
        }
        recorded => recorded.map_err(|source| AdminError::io(&path, source))?,
    };

    Ok((recorded.trim() != current.to_hex()).then_some(current))
}

/// Records `current` as the digest of the node identity `home` is
/// started under, replacing the one it held in one rename.
///
/// # Errors
///
/// Fails when the file cannot be written.
pub fn record(home: &Home, current: Digest) -> Result<(), AdminError> {
    let path = home.node_identity();
    let temporary = staging(&path);
    match fs::remove_file(&temporary) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            return Err(AdminError::io(&temporary, e));
        }
        _ => {}
    }
    write_secret(&temporary, &current.to_hex())?;
    fs::rename(&temporary, &path).map_err(|source| AdminError::io(&path, source))?;
    sync_directory(&path).map_err(|source| AdminError::io(&path, source))
}

#[cfg(test)]
mod tests;
