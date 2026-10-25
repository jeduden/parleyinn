//! Where the home is and what it holds (§8.1): `PARINN_HOME` when set,
//! else `.parinn` in `HOME`. The home and its `keys` directory are
//! readable only by their owner. Its keys are kept in this OS's key
//! store, each in an entry named per home (SEC-10), and it remembers
//! the digest of the node identity it was last started under (REC-24).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use key::key_store::{self, KeyStore};

use crate::AdminError;
use crate::node_identity::IdentitySource;

/// A ParleyInn home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    root: PathBuf,
    identity: IdentitySource,
}

impl Home {
    /// The home `PARINN_HOME` names, else `.parinn` in `HOME`; an empty
    /// variable counts as unset.
    ///
    /// # Errors
    ///
    /// Fails when neither names one.
    pub fn locate(parinn_home: Option<&OsStr>, home: Option<&OsStr>) -> Result<Self, AdminError> {
        let set = |v: Option<&OsStr>| v.filter(|v| !v.is_empty()).map(PathBuf::from);
        let root = set(parinn_home)
            .or_else(|| set(home).map(|h| h.join(".parinn")))
            .ok_or(AdminError::NoHome)?;

        Ok(Self {
            root,
            identity: IdentitySource::platform(),
        })
    }

    /// The home, its node identity read from `identity` instead of this
    /// OS's source: a development build's seam for the tests.
    #[must_use]
    pub fn with_identity(self, identity: IdentitySource) -> Self {
        Self { identity, ..self }
    }

    /// Where its node identity is read from (REC-24).
    #[must_use]
    pub fn identity(&self) -> &IdentitySource {
        &self.identity
    }

    /// The home's directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The store.
    #[must_use]
    pub fn store(&self) -> PathBuf {
        self.root.join("store.db")
    }

    /// The key store its keys are kept in: this OS's, each key in an
    /// entry named for this home (SEC-10).
    #[must_use]
    pub fn key_store(&self) -> Box<dyn KeyStore> {
        key_store::platform(&self.keys(), &self.root)
    }

    /// The file whose presence says a start staged new keys and has not
    /// yet recorded and placed them: a first start's, or a node identity
    /// change's. It names which, and holds no secret.
    #[must_use]
    pub fn staged(&self) -> PathBuf {
        self.keys().join("staged")
    }

    /// The file holding the digest of the node identity the home was last
    /// started under (REC-24).
    #[must_use]
    pub fn node_identity(&self) -> PathBuf {
        self.root.join("node-identity")
    }

    /// The audit log's directory (OPS-02).
    #[must_use]
    pub fn audit(&self) -> PathBuf {
        self.root.join("audit")
    }

    /// The home id binding (§8.1, SEC-03): `{"id": "<home id>"}`.
    #[must_use]
    pub fn binding(&self) -> PathBuf {
        self.root.join("home.json")
    }

    /// Checks the home before anything reads or writes in it (SEC-02,
    /// I8): the home and every directory and file in it belong to the
    /// running OS user and give no group or other bits, and none in it
    /// is a symbolic link. The home itself may be reached through one.
    /// A home that does not exist yet passes. Windows' ACLs are not
    /// checked yet.
    ///
    /// # Errors
    ///
    /// Fails, naming the path, at the first that breaks the rule, and
    /// when the home cannot be listed or the running OS user learnt.
    pub fn check(&self) -> Result<(), AdminError> {
        #[cfg(unix)]
        return self.check_as(running_uid()?);
        #[cfg(not(unix))]
        Ok(())
    }

    /// [`check`](Self::check) for the OS user `running`.
    ///
    /// # Errors
    ///
    /// As [`check`](Self::check).
    #[cfg(unix)]
    pub fn check_as(&self, running: u32) -> Result<(), AdminError> {
        let meta = match fs::metadata(&self.root) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            meta => meta.map_err(|source| AdminError::Io {
                path: self.root.clone(),
                source,
            })?,
        };
        owner_only(&self.root, &meta, running)?;
        let mut pending = vec![self.root.clone()];
        while let Some(dir) = pending.pop() {
            let io = |source| AdminError::Io {
                path: dir.clone(),
                source,
            };
            // What another process removes while the walk runs, such as
            // the store's WAL files as its last connection closes, is
            // no longer in the home: it is skipped, not refused.
            let listing = present(fs::read_dir(&dir)).map_err(io)?;
            let mut items: Vec<(PathBuf, fs::Metadata)> = Vec::new();
            for item in listing.into_iter().flatten() {
                let item = item.map_err(io)?;
                // Each item's own metadata, a link's rather than its
                // target's.
                if let Some(meta) = present(item.metadata()).map_err(io)? {
                    items.push((item.path(), meta));
                }
            }
            // A listing's order is the file system's; the first
            // refusal named should not be.
            items.sort_by(|a, b| a.0.cmp(&b.0));
            for (path, meta) in items {
                if meta.file_type().is_symlink() {
                    return Err(AdminError::SymbolicLink { path });
                }
                owner_only(&path, &meta, running)?;
                if meta.is_dir() {
                    pending.push(path);
                }
            }
        }

        Ok(())
    }

    /// Checks the home id `supplied` by the environment against the
    /// home's binding, if it has one (SEC-03).
    ///
    /// # Errors
    ///
    /// Fails when the home is bound to another home id, when it is
    /// bound and none is supplied, and when the binding cannot be read
    /// or holds no home id.
    pub fn check_home_id(&self, supplied: Option<&OsStr>) -> Result<(), AdminError> {
        let path = self.binding();
        let text = match fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            text => text.map_err(|source| AdminError::Io {
                path: path.clone(),
                source,
            })?,
        };
        let id = home_id(&text).ok_or(AdminError::HomeIdBinding { path })?;
        match supplied {
            None => Err(AdminError::HomeIdMissing),
            Some(s) if s.to_str() == Some(id.as_str()) => Ok(()),
            Some(_) => Err(AdminError::HomeIdMismatch),
        }
    }

    /// Its keys directory, where the files of a file-based key store
    /// lie.
    #[must_use]
    pub fn keys(&self) -> PathBuf {
        self.root.join("keys")
    }

    /// Makes the home and its `keys` directory, each readable only by
    /// its owner, where they do not exist yet.
    ///
    /// # Errors
    ///
    /// Fails when a directory cannot be made.
    pub fn create(&self) -> Result<(), AdminError> {
        let keys = self.keys();
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);

        builder
            .create(&keys)
            .map_err(|source| AdminError::Io { path: keys, source })
    }
}

/// The home id a binding's `text` holds: an object with one member,
/// `id`, a string that is not empty.
fn home_id(text: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let object = value.as_object().filter(|o| o.len() == 1)?;

    object
        .get("id")?
        .as_str()
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

/// What `result` holds, or none when what it reads no longer exists.
#[cfg(unix)]
fn present<T>(result: std::io::Result<T>) -> std::io::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Refuses `path` unless `meta` shows it belongs to `running` and
/// gives no group or other bits.
#[cfg(unix)]
fn owner_only(path: &Path, meta: &fs::Metadata, running: u32) -> Result<(), AdminError> {
    use std::os::unix::fs::MetadataExt;

    if meta.uid() != running {
        return Err(AdminError::OtherOsUser {
            path: path.to_path_buf(),
            uid: meta.uid(),
            running,
        });
    }
    let mode = meta.mode() & 0o7777;
    if mode & 0o077 != 0 {
        return Err(AdminError::LoosePermissions {
            path: path.to_path_buf(),
            mode,
        });
    }

    Ok(())
}

/// The OS user this process runs as: the one a pipe it makes belongs
/// to. That needs no `unsafe` and no dependency, and inside a user
/// namespace gives the user as that namespace maps it, as it does the
/// owners of the home's files.
///
/// # Errors
///
/// Fails when no pipe can be made or its owner read.
#[cfg(unix)]
pub fn running_uid() -> Result<u32, AdminError> {
    use std::os::fd::OwnedFd;
    use std::os::unix::fs::MetadataExt;

    let (reader, _writer) = std::io::pipe().map_err(AdminError::OsUser)?;
    let pipe = fs::File::from(OwnedFd::from(reader));

    pipe.metadata().map(|m| m.uid()).map_err(AdminError::OsUser)
}

#[cfg(test)]
mod tests;
