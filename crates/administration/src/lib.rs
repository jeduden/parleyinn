//! Administration of the home (§8.1, §9.5): where the home is, the
//! node's first start in it and its node identity, the seats the node
//! certifies, the principal acts its CLI takes, and `parinn verify`'s
//! check of the whole record.

use std::fmt;
use std::path::PathBuf;

pub mod act;
pub mod home;
pub mod node;
pub mod node_identity;
pub mod origin;
pub mod repository;
pub mod role;
pub mod trust_policy;
pub mod verify;

/// What can go wrong administering the home.
#[derive(Debug)]
pub enum AdminError {
    /// Neither `PARINN_HOME` nor `HOME` names a home.
    NoHome,
    /// A directory of the home cannot be made.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A key cannot be minted, kept or read, or a certificate signed.
    Key(key::KeyError),
    /// The store refused an operation.
    Store(store::StoreError),
    /// The home and the store disagree about the node.
    Node(&'static str),
    /// A repository's identity cannot be bound (LANE-02).
    Repository(&'static str),
    /// A repository's git data cannot be read (LANE-02).
    RepositoryRead(::repository::Error),
    /// A worktree or directory cannot be resolved to one name (LANE-02).
    Unresolved {
        what: &'static str,
        source: std::io::Error,
    },
    /// The device key is missing from the key store, but the store holds
    /// this node's record: no second genesis starts (PRV-12).
    DeviceKeyMissing,
    /// A directory or file of the home lets other OS users in (SEC-02).
    LoosePermissions { path: PathBuf, mode: u32 },
    /// A directory or file of the home belongs to OS user `uid`, not
    /// the `running` one (SEC-02, I8).
    OtherOsUser {
        path: PathBuf,
        uid: u32,
        running: u32,
    },
    /// A symbolic link inside the home, which could lead out of it.
    SymbolicLink { path: PathBuf },
    /// The running OS user cannot be learnt.
    OsUser(std::io::Error),
    /// The home's home id binding does not hold one home id (SEC-03).
    HomeIdBinding { path: PathBuf },
    /// The environment supplies another home id than the home's.
    HomeIdMismatch,
    /// The home is bound to a home id and the environment supplies none.
    HomeIdMissing,
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHome => f.write_str("no home: set PARINN_HOME or HOME"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Key(e) => write!(f, "{e}"),
            Self::Store(e) => write!(f, "{e}"),
            Self::Node(why) | Self::Repository(why) => f.write_str(why),
            Self::RepositoryRead(e) => write!(f, "the repository's git data cannot be read: {e}"),
            Self::Unresolved { what, source } => {
                write!(f, "the {what} cannot be resolved: {source}")
            }
            Self::DeviceKeyMissing => f.write_str(
                "the device key is missing from the key store, but the store holds this node's record; no second genesis starts",
            ),
            Self::LoosePermissions { path, mode } => write!(
                f,
                "{}: mode {mode:04o} lets other OS users in; the home wants owner-only permissions",
                path.display()
            ),
            Self::OtherOsUser { path, uid, running } => write!(
                f,
                "{}: belongs to UID {uid}, not the running UID {running}",
                path.display()
            ),
            Self::SymbolicLink { path } => {
                write!(f, "{}: a symbolic link inside the home", path.display())
            }
            Self::OsUser(e) => write!(f, "learn the running OS user: {e}"),
            Self::HomeIdBinding { path } => {
                write!(f, "{}: want {{\"id\": \"<home id>\"}}", path.display())
            }
            Self::HomeIdMismatch => {
                f.write_str("the home id PARINN_HOME_ID supplies does not match the home's")
            }
            Self::HomeIdMissing => {
                f.write_str("the home is bound to a home id, and PARINN_HOME_ID supplies none")
            }
        }
    }
}

impl std::error::Error for AdminError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::OsUser(source) | Self::Unresolved { source, .. } => {
                Some(source)
            }
            Self::RepositoryRead(e) => Some(e),
            Self::Key(e) => Some(e),
            Self::Store(e) => Some(e),
            Self::NoHome
            | Self::Node(_)
            | Self::Repository(_)
            | Self::DeviceKeyMissing
            | Self::LoosePermissions { .. }
            | Self::OtherOsUser { .. }
            | Self::SymbolicLink { .. }
            | Self::HomeIdBinding { .. }
            | Self::HomeIdMismatch
            | Self::HomeIdMissing => None,
        }
    }
}

impl AdminError {
    /// The failure to read, write or remove the file or directory at
    /// `path`.
    #[must_use]
    pub fn io(path: &std::path::Path, source: std::io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

impl From<key::KeyError> for AdminError {
    fn from(e: key::KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<store::StoreError> for AdminError {
    fn from(e: store::StoreError) -> Self {
        Self::Store(e)
    }
}

#[cfg(test)]
mod tests;
