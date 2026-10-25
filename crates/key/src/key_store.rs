//! Key stores (SEC-10, the plan's ruling 2): where the core keeps the
//! keys SEC-10 binds to a home, each in its own entry named per home.
//!
//! - macOS: the Keychain, through `security-framework`, under an
//!   account named for the home.
//! - Windows: a file in the home whose bytes DPAPI protects for the OS
//!   user, through the `platform` crate.
//! - Linux and other Unix: a file in the home only the principal's OS
//!   user can read. Linux's Secret Service needs D-Bus, a socket the
//!   core may not open (I4), and its kernel keyring does not survive a
//!   reboot.
//!
//! A key missing from its entry is told apart from a key store that is
//! locked or cannot be read: only the first counts as missing (PRV-12,
//! REC-24).

use std::path::{Path, PathBuf};

use record::digest::{Digest, hex, unhex};
use record::id::SeatId;

use crate::KeyError;
use crate::file::{read_secret, write_secret};

/// The label a home's keychain account is derived under.
const ACCOUNT: &[u8] = b"parinn key store account v1\0";

/// One entry of a key store: the place of one key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry(String);

impl Entry {
    /// The device key's entry.
    #[must_use]
    pub fn device() -> Self {
        Self("device".to_owned())
    }

    /// The device key's recovery key's own entry (PRV-12).
    #[must_use]
    pub fn recovery() -> Self {
        Self("recovery".to_owned())
    }

    /// The entry of the key of the seat `seat`.
    #[must_use]
    pub fn seat(seat: SeatId) -> Self {
        Self(format!("seat-{seat}"))
    }

    /// The entry a start stages `entry`'s new key in before it records
    /// anything: a first start's, or a node identity change's.
    #[must_use]
    pub fn staged(entry: &Self) -> Self {
        Self(format!("staged-{}", entry.0))
    }

    /// The entry a start stages a new device seat's key in, before the
    /// seat's id is known.
    #[must_use]
    pub fn staged_seat() -> Self {
        Self("staged-seat".to_owned())
    }

    /// Its name, which names no secret.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Which kind of key store keeps a home's keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStoreKind {
    /// Files only the principal's OS user can read.
    File,
    /// The macOS Keychain.
    Keychain,
    /// Files DPAPI protects for the principal's OS user.
    Dpapi,
}

impl KeyStoreKind {
    /// The kind's name, as `parinn status --json` shows it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Keychain => "keychain",
            Self::Dpapi => "dpapi",
        }
    }

    /// Whether a key it keeps is a file an unsandboxed agent of the same
    /// OS user could read (SEC-10).
    #[must_use]
    pub fn readable_by_agents(self) -> bool {
        matches!(self, Self::File | Self::Dpapi)
    }

    /// Where it keeps keys, as `parinn status` says it.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            Self::File => {
                "files only this OS user can read; an unsandboxed agent of the same OS user could read them"
            }
            Self::Keychain => "the macOS Keychain",
            Self::Dpapi => {
                "files DPAPI protects for this OS user; an unsandboxed agent of the same OS user could read them"
            }
        }
    }
}

/// A place the core keeps keys, each in its own entry.
pub trait KeyStore {
    /// Which kind it is.
    fn kind(&self) -> KeyStoreKind;

    /// Keeps `secret` in `entry`, which must hold none yet.
    ///
    /// # Errors
    ///
    /// Fails when the entry holds a key, or the key store cannot be
    /// written.
    fn put(&self, entry: &Entry, secret: &str) -> Result<(), KeyError>;

    /// The secret `entry` holds.
    ///
    /// # Errors
    ///
    /// Fails with [`KeyError::is_missing`] true when the entry holds no
    /// key, and otherwise when the key store is locked or cannot be read.
    fn get(&self, entry: &Entry) -> Result<String, KeyError>;

    /// Empties `entry`; one that holds nothing is no failure.
    ///
    /// # Errors
    ///
    /// Fails when the key store cannot be written.
    fn remove(&self, entry: &Entry) -> Result<(), KeyError>;
}

/// The key store of this OS for the home at `home`, whose keys
/// directory is `keys`.
#[must_use]
pub fn platform(keys: &Path, home: &Path) -> Box<dyn KeyStore> {
    #[cfg(target_os = "macos")]
    {
        let _ = keys;
        Box::new(KeychainKeyStore::new(macos::MacKeychain, home))
    }
    #[cfg(windows)]
    {
        let _ = home;
        Box::new(ProtectedFileKeyStore::new(
            windows::Dpapi,
            keys.to_path_buf(),
        ))
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        let _ = home;
        Box::new(FileKeyStore::new(keys.to_path_buf()))
    }
}

/// The failure of an entry whose key store is locked or cannot be read.
fn unavailable(entry: &Entry, reason: String) -> KeyError {
    KeyError::Unavailable {
        entry: entry.0.clone(),
        reason,
    }
}

/// A missing file's read as a missing entry.
fn missing_file(entry: &Entry, e: KeyError) -> KeyError {
    match e {
        KeyError::File { source, .. } if source.kind() == std::io::ErrorKind::NotFound => {
            KeyError::Missing {
                entry: entry.0.clone(),
            }
        }
        e => e,
    }
}

/// Writes `secret` to a new file at `path` that only its owner reads,
/// making its directory, readable only by its owner, first.
fn write_kept(path: &Path, secret: &str) -> Result<(), KeyError> {
    let dir = path.parent().unwrap_or(path);
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    if let Err(source) = builder.create(dir) {
        return Err(KeyError::File {
            path: dir.to_path_buf(),
            source,
        });
    }
    write_secret(path, secret)
}

/// Removes the file at `path`; one not there is no failure.
fn remove_file(path: &Path) -> Result<(), KeyError> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(KeyError::File {
            path: path.to_path_buf(),
            source: e,
        }),
        _ => Ok(()),
    }
}

/// Keys in files only the principal's OS user can read, one per entry,
/// in the home's keys directory.
#[derive(Debug, Clone)]
pub struct FileKeyStore {
    dir: PathBuf,
}

impl FileKeyStore {
    /// The key store in the directory `dir`.
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The file `entry` is kept in.
    #[must_use]
    pub fn path(&self, entry: &Entry) -> PathBuf {
        self.dir.join(format!("{}.key", entry.0))
    }
}

impl KeyStore for FileKeyStore {
    fn kind(&self) -> KeyStoreKind {
        KeyStoreKind::File
    }

    fn put(&self, entry: &Entry, secret: &str) -> Result<(), KeyError> {
        write_kept(&self.path(entry), secret)
    }

    fn get(&self, entry: &Entry) -> Result<String, KeyError> {
        read_secret(&self.path(entry)).map_err(|e| missing_file(entry, e))
    }

    fn remove(&self, entry: &Entry) -> Result<(), KeyError> {
        remove_file(&self.path(entry))
    }
}

/// The account a home's `entry` is kept under in a keychain: named for
/// the home, so two homes of one OS user never share a key.
#[must_use]
pub fn account(home: &Path, entry: &Entry) -> String {
    let path = home.as_os_str().as_encoded_bytes();
    let digest = Digest::of(&[ACCOUNT, path]).to_hex();

    format!("home-{}/{}", &digest[..32], entry.0)
}

/// What a keychain does for a key store: ParleyInn's own generic passwords,
/// by account.
pub trait Keychain {
    /// The secret kept under `account`, none when nothing is.
    ///
    /// # Errors
    ///
    /// Fails when the keychain is locked or cannot be read.
    fn get(&self, account: &str) -> Result<Option<String>, String>;

    /// Keeps `secret` under `account`, false when one is already kept
    /// there.
    ///
    /// # Errors
    ///
    /// Fails when the keychain is locked or cannot be written.
    fn add(&self, account: &str, secret: &str) -> Result<bool, String>;

    /// Removes what is kept under `account`, if anything.
    ///
    /// # Errors
    ///
    /// Fails when the keychain is locked or cannot be written.
    fn delete(&self, account: &str) -> Result<(), String>;
}

/// Keys in a keychain, each under an account named for the home.
#[derive(Debug)]
pub struct KeychainKeyStore<K> {
    keychain: K,
    home: PathBuf,
}

impl<K: Keychain> KeychainKeyStore<K> {
    /// The key store of the home at `home` in `keychain`.
    #[must_use]
    pub fn new(keychain: K, home: &Path) -> Self {
        Self {
            keychain,
            home: home.to_path_buf(),
        }
    }
}

impl<K: Keychain> KeyStore for KeychainKeyStore<K> {
    fn kind(&self) -> KeyStoreKind {
        KeyStoreKind::Keychain
    }

    fn put(&self, entry: &Entry, secret: &str) -> Result<(), KeyError> {
        let added = self
            .keychain
            .add(&account(&self.home, entry), secret)
            .map_err(|e| unavailable(entry, e))?;
        if added {
            Ok(())
        } else {
            Err(KeyError::Occupied {
                entry: entry.0.clone(),
            })
        }
    }

    fn get(&self, entry: &Entry) -> Result<String, KeyError> {
        self.keychain
            .get(&account(&self.home, entry))
            .map_err(|e| unavailable(entry, e))?
            .ok_or_else(|| KeyError::Missing {
                entry: entry.0.clone(),
            })
    }

    fn remove(&self, entry: &Entry) -> Result<(), KeyError> {
        self.keychain
            .delete(&account(&self.home, entry))
            .map_err(|e| unavailable(entry, e))
    }
}

/// What protects a key's bytes for the OS user before they reach a file:
/// DPAPI on Windows.
pub trait Protect {
    /// `plain`, protected.
    ///
    /// # Errors
    ///
    /// Fails when the protector does.
    fn protect(&self, plain: &[u8]) -> Result<Vec<u8>, String>;

    /// The bytes `blob` protects.
    ///
    /// # Errors
    ///
    /// Fails when the blob does not unprotect for this OS user.
    fn unprotect(&self, blob: &[u8]) -> Result<Vec<u8>, String>;
}

/// Keys in files in the home's keys directory, each holding the key's
/// bytes as a protector protected them, only its owner reading it.
#[derive(Debug, Clone)]
pub struct ProtectedFileKeyStore<P> {
    protector: P,
    dir: PathBuf,
}

impl<P: Protect> ProtectedFileKeyStore<P> {
    /// The key store in the directory `dir`, protected by `protector`.
    #[must_use]
    pub fn new(protector: P, dir: PathBuf) -> Self {
        Self { protector, dir }
    }

    /// The file `entry` is kept in.
    #[must_use]
    pub fn path(&self, entry: &Entry) -> PathBuf {
        self.dir.join(format!("{}.dpapi", entry.0))
    }
}

impl<P: Protect> KeyStore for ProtectedFileKeyStore<P> {
    fn kind(&self) -> KeyStoreKind {
        KeyStoreKind::Dpapi
    }

    fn put(&self, entry: &Entry, secret: &str) -> Result<(), KeyError> {
        let blob = self
            .protector
            .protect(secret.as_bytes())
            .map_err(|e| unavailable(entry, e))?;
        write_kept(&self.path(entry), &hex(&blob))
    }

    fn get(&self, entry: &Entry) -> Result<String, KeyError> {
        let text = read_secret(&self.path(entry)).map_err(|e| missing_file(entry, e))?;
        let blob = unhex_vec(&text)
            .ok_or_else(|| unavailable(entry, "the protected key is not hex".to_owned()))?;
        let plain = self
            .protector
            .unprotect(&blob)
            .map_err(|e| unavailable(entry, e))?;
        String::from_utf8(plain)
            .map_err(|_| unavailable(entry, "the protected key is not UTF-8".to_owned()))
    }

    fn remove(&self, entry: &Entry) -> Result<(), KeyError> {
        remove_file(&self.path(entry))
    }
}

/// The bytes an even number of lowercase hex digits spell.
fn unhex_vec(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| unhex::<1>(&text[i..i + 2]).ok().map(|[b]| b))
        .collect()
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(test)]
mod tests;
