//! The payload store (REC-09, §8.1): `payloads/` beside `store.db`,
//! holding the content of each event whose canonical encoding is longer
//! than the payload threshold. A payload's name is HMAC-SHA-256 under
//! the event's commitment key over a fixed label, so without that key no
//! one confirms a guess at the content from it, and two equal contents
//! get two names. The event row keeps a preview, the name and the size;
//! its commitment covers the whole content.
//!
//! Each payload is written to a temporary file in the payload store,
//! synced, renamed into place and the directory synced (REC-09), through
//! a [`Disk`] a test fakes.

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use record::commitment::{CommitmentKey, hmac_sha256};
use serde_json::Value;

/// The label every payload name's message is.
const NAME_LABEL: &[u8] = b"parinn payload name v1\0";

/// The most bytes a preview holds (REC-09).
pub const PREVIEW_BYTES: usize = 512;

/// The prefix of a temporary file in the payload store.
const TEMPORARY: &str = ".tmp-";

/// The name of the payload of the event whose commitment key is `key`.
#[must_use]
pub fn name(key: &CommitmentKey) -> String {
    hmac_sha256(key.as_bytes(), &[NAME_LABEL]).to_hex()
}

/// The preview of `content`, whose canonical encoding is `canonical`:
/// its `text`, or else that encoding, cut to at most [`PREVIEW_BYTES`]
/// at a character boundary.
#[must_use]
pub fn preview(content: &Value, canonical: &str) -> String {
    let text = content["text"].as_str().unwrap_or(canonical);
    let mut end = text.len().min(PREVIEW_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }

    text[..end].to_owned()
}

/// The file operations the payload store makes.
pub trait Disk: Send {
    /// Writes `bytes` to a new or truncated file at `path`, readable
    /// only by its owner, and syncs it.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written or synced.
    fn write_synced(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;

    /// Renames `from` to `to`, replacing it.
    ///
    /// # Errors
    ///
    /// Fails when the rename does.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;

    /// Syncs the directory `dir`, so a rename in it lasts.
    ///
    /// # Errors
    ///
    /// Fails when the directory cannot be opened or synced.
    fn sync_dir(&self, dir: &Path) -> io::Result<()>;

    /// The bytes of the file at `path`.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
}

/// The file system.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsDisk;

impl Disk for OsDisk {
    fn write_synced(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(path)?;
        file.write_all(bytes)?;
        file.sync_all()
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn sync_dir(&self, dir: &Path) -> io::Result<()> {
        // A directory opens for reading on Unix, and syncs; elsewhere
        // the rename alone is what the platform offers.
        #[cfg(unix)]
        fs::File::open(dir)?.sync_all()?;
        #[cfg(not(unix))]
        let _ = dir;

        Ok(())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
}

/// The payload store at one directory.
pub struct Payloads {
    dir: PathBuf,
    disk: Box<dyn Disk>,
}

impl fmt::Debug for Payloads {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Payloads").field("dir", &self.dir).finish()
    }
}

impl Payloads {
    /// The payload store at `dir`, written through `disk`.
    #[must_use]
    pub fn new(dir: PathBuf, disk: Box<dyn Disk>) -> Self {
        Self { dir, disk }
    }

    /// Writes `bytes` as the payload `name`: to a temporary file in the
    /// payload store, synced, then renamed into place, the directory
    /// synced after. The payload store is made, readable only by its
    /// owner, when it does not exist.
    ///
    /// # Errors
    ///
    /// Fails when the payload store cannot be made, or a step fails.
    pub fn write(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(&self.dir)?;
        let temporary = self.dir.join(format!("{TEMPORARY}{name}"));
        self.disk.write_synced(&temporary, bytes)?;
        self.disk.rename(&temporary, &self.dir.join(name))?;

        self.disk.sync_dir(&self.dir)
    }

    /// The bytes of the payload `name`.
    ///
    /// # Errors
    ///
    /// Fails when `name` is no payload name, or the payload cannot be
    /// read.
    pub fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        let named = name.len() == 64
            && name
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !named {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no payload name",
            ));
        }

        self.disk.read(&self.dir.join(name))
    }
}

#[cfg(test)]
mod tests;
