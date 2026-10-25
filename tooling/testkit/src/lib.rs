//! Test-only helpers the tooling crates share: a temporary directory,
//! the path of the repository the tests inspect, and the confinement
//! the end-to-end suite starts each component in (ENG-12). Only test code
//! depends on this crate; nothing here links into a shipped
//! executable.

pub mod guard;

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod boundary;
pub mod relative;

/// How many names a [`TempDir`] tries before it gives up.
const ATTEMPTS: u32 = 1000;

/// The repository checkout the tests inspect: two levels above this
/// crate's manifest, as for every crate under `tooling/`.
#[must_use]
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Points `command` at `home` (ENG-14): its `HOME`, a `PARINN_HOME`
/// inside it, and its working directory, so an executable a test starts
/// cannot reach the real `HOME`, a ParleyInn home, Claude Code's
/// configuration, or the repository configuration of the checkout the
/// test runs in.
pub fn isolate<'a>(command: &'a mut Command, home: &Path) -> &'a mut Command {
    command
        .env("HOME", home)
        .env("PARINN_HOME", home.join(".parinn"))
        .current_dir(home)
}

/// Copies the transcript at `source` under `home`'s default transcript
/// root, `~/.claude/projects` (SEC-18), keeping its project directory's
/// and its own name, and returns where it now lies.
///
/// # Errors
///
/// Fails when the transcript cannot be read or written.
pub fn place_transcript(home: &Path, source: &Path) -> io::Result<PathBuf> {
    let project = source
        .parent()
        .and_then(Path::file_name)
        .unwrap_or_else(|| std::ffi::OsStr::new("project"));
    let name = source
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?;
    let dir = home.join(".claude").join("projects").join(project);
    std::fs::create_dir_all(&dir)?;
    let placed = dir.join(name);
    std::fs::copy(source, &placed)?;

    Ok(placed)
}

/// The `parinn` executable cargo built beside the running test. `cargo
/// test` builds a workspace member's executables into the profile
/// directory whose `deps` directory holds every test, so a scenario
/// starts the same build of the core the end-to-end tests do.
///
/// # Errors
///
/// Fails, saying how to build it, when it is not there.
pub fn core_executable() -> Result<PathBuf, String> {
    let test = std::env::current_exe().map_err(|e| format!("locate the running test: {e}"))?;
    sibling_executable(&test, "parinn")
}

/// The environment variable that names the release build of the core, a
/// path, when it is not at `target/release/parinn` of the checkout.
pub const RELEASE_CORE_VAR: &str = "PARINN_RELEASE_CORE";

/// The release build of the `parinn` core, which every step that holds
/// the core to a time budget runs (NFR-02): a development build is
/// slower by an order of magnitude and flakes on a loaded runner. It is
/// the path `PARINN_RELEASE_CORE` names, else `target/release/parinn` of
/// the checkout; the test run never builds it.
///
/// # Errors
///
/// Fails, saying how to build it, when it is not there.
pub fn release_core_executable() -> Result<PathBuf, String> {
    release_executable(std::env::var_os(RELEASE_CORE_VAR), &repo_root(), "parinn")
}

/// The environment variable that names the release build of the
/// calibration workload, a path, when it is not at
/// `target/release/calibrate` of the checkout.
pub const CALIBRATE_VAR: &str = "PARINN_CALIBRATE";

/// The release build of the `calibrate` workload, the reference the
/// hook budget rows measure a hook handler against (NFR-02): the path
/// `PARINN_CALIBRATE` names, else `target/release/calibrate` of the
/// checkout; the test run never builds it.
///
/// # Errors
///
/// Fails, saying how to build it, when it is not there.
pub fn calibrate_executable() -> Result<PathBuf, String> {
    release_executable(std::env::var_os(CALIBRATE_VAR), &repo_root(), "calibrate")
}

/// The release build of `name`: `named` when given, else under `root`.
fn release_executable(
    named: Option<std::ffi::OsString>,
    root: &Path,
    name: &str,
) -> Result<PathBuf, String> {
    let path = named.map_or_else(
        || {
            root.join("target/release")
                .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
        },
        PathBuf::from,
    );
    if path.is_file() {
        Ok(path)
    } else {
        let var = if name == "calibrate" {
            CALIBRATE_VAR
        } else {
            RELEASE_CORE_VAR
        };
        Err(format!(
            "no release {name} executable at {}: build it first, with cargo build --release -p {name}, or name it in {var}",
            path.display()
        ))
    }
}

/// The executable `name` in the directory above the one holding `test`.
fn sibling_executable(test: &Path, name: &str) -> Result<PathBuf, String> {
    let profile = test
        .parent()
        .and_then(Path::parent)
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| {
            format!(
                "the running test {} lies in no deps directory",
                test.display()
            )
        })?;
    let path = profile.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "no {name} executable at {}: build it first, with cargo build -p {name}",
            path.display()
        ))
    }
}

/// A directory under the system's temporary directory, removed with
/// everything in it when dropped.
#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Creates a fresh, empty directory readable only by its owner.
    ///
    /// # Errors
    ///
    /// Fails when the directory cannot be created.
    pub fn new() -> io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut names = (0..ATTEMPTS)
            .map(|n| OsString::from(format!("parinn-{}-{stamp}-{n}", std::process::id())));
        create_unique(&std::env::temp_dir(), &mut names).map(|path| Self { path })
    }

    /// The directory's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // A directory left behind by a failed removal is the system
        // temporary directory's to clean; a test has nothing to do.
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Creates the first of `names` under `base` that does not exist yet.
/// `create_dir` refuses an existing name, so another process or user
/// cannot hand the test a directory it planted.
fn create_unique(base: &Path, names: &mut dyn Iterator<Item = OsString>) -> io::Result<PathBuf> {
    for name in names {
        let path = base.join(name);
        match owner_only().create(&path) {
            Ok(()) => return Ok(path),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("testkit: no free directory name under {}", base.display()),
    ))
}

#[cfg(unix)]
fn owner_only() -> fs::DirBuilder {
    use std::os::unix::fs::DirBuilderExt;

    let mut b = fs::DirBuilder::new();
    b.mode(0o700);
    b
}

#[cfg(not(unix))]
fn owner_only() -> fs::DirBuilder {
    fs::DirBuilder::new()
}

#[cfg(test)]
mod tests;
