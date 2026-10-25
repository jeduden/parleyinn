//! The real [`Host`]: the cargo that runs this tool, and the file
//! system. A process and file boundary, so the integration tests in
//! `tests/host.rs` prove it rather than unit tests.

use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Host;

/// The real machine: the cargo that runs this tool, and the file system.
#[derive(Debug, Clone)]
pub struct Cargo {
    program: OsString,
}

impl Cargo {
    /// The cargo named by `$CARGO`, which cargo sets for what it runs,
    /// or else the `cargo` on PATH.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            program: std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()),
        }
    }

    fn command(&self, args: &[String]) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(args)
            .stdin(Stdio::null())
            .env(crate::MEASURING, "1");
        command
    }
}

impl Host for Cargo {
    fn cargo(&self, args: &[String]) -> Result<(), String> {
        let status = self
            .command(args)
            .status()
            .map_err(|e| format!("run cargo {}: {e}", args.join(" ")))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("cargo {}: {status}", args.join(" ")))
        }
    }

    fn cargo_output(&self, args: &[String]) -> Result<String, String> {
        let out = self
            .command(args)
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| format!("run cargo {}: {e}", args.join(" ")))?;
        if !out.status.success() {
            return Err(format!("cargo {}: {}", args.join(" "), out.status));
        }

        String::from_utf8(out.stdout).map_err(|e| format!("cargo {}: {e}", args.join(" ")))
    }

    fn write(&self, path: &Path, text: &str) -> Result<(), String> {
        let dir = path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(dir)
            .and_then(|()| fs::write(path, text))
            .map_err(|e| format!("write {}: {e}", path.display()))
    }

    fn read(&self, path: &Path) -> Result<Option<String>, String> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("read {}: {e}", path.display())),
        }
    }

    fn files(&self, dir: &Path) -> Result<Vec<String>, String> {
        let read = |e: std::io::Error| format!("read {}: {e}", dir.display());
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(read(e)),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(read)?;
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
        names.sort();

        Ok(names)
    }

    fn append(&self, path: &Path, text: &str) -> Result<(), String> {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| f.write_all(text.as_bytes()))
            .map_err(|e| format!("append to {}: {e}", path.display()))
    }

    fn sources(&self, dir: &Path) -> Result<Vec<(PathBuf, String)>, String> {
        let mut out = Vec::new();
        let entries = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|e| format!("read {}: {e}", dir.display()))?
                .path();
            if path.is_dir() && !path.ends_with("target") {
                out.extend(self.sources(&path)?);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = fs::read_to_string(&path)
                    .map_err(|e| format!("read {}: {e}", path.display()))?;
                out.push((path, text));
            }
        }

        Ok(out)
    }

    fn bound_scenarios(&self, features: &Path) -> Result<usize, String> {
        let all = scenario::scenarios(features).map_err(|e| e.to_string())?;

        Ok(all.iter().filter(|s| !s.pending).count())
    }
}

#[cfg(test)]
mod tests;
