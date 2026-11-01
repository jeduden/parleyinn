//! The model access token, kept out of every agent's reach (ENG-28).
//! A step of its own stages it, from its environment, into a
//! credentials file only its owner reads, in a directory outside the
//! work directory; the step that runs the agents holds it in no
//! variable. Each `claude` the review starts finds it there through
//! [`CONFIG_VAR`], never in its environment, which the process host
//! also clears of [`TOKEN_VAR`], and is refused every read of that
//! directory, so an agent's own Read cannot return it. The review reads
//! the token back only to scrub its outputs of it.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::host::{Command, Host, Output};
use crate::runner::PROGRAM;

/// The variable the workflow hands the token in, to the staging step
/// alone.
pub const TOKEN_VAR: &str = "CLAUDE_CODE_OAUTH_TOKEN";

/// The variable that points `claude` at its configuration directory,
/// where it reads its credentials file.
pub const CONFIG_VAR: &str = "CLAUDE_CONFIG_DIR";

/// The credentials file `claude` reads under its configuration
/// directory.
pub const FILE: &str = ".credentials.json";

/// What the token of `claude setup-token` may do: inference alone.
const SCOPE: &str = "user:inference";

/// Writes `token` as the credentials file under `dir`, the directory
/// made only its owner's and the file only its owner's to read.
///
/// # Errors
///
/// Fails, never naming the token, when it is empty or holds a
/// character no token has, or the directory or file cannot be written.
pub fn stage(dir: &Path, token: &str) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() || !token.chars().all(token_char) {
        return Err("the access token is empty or holds a character no token has".to_owned());
    }
    fs::create_dir_all(dir).map_err(|e| format!("make {}: {e}", dir.display()))?;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("make {} its owner's alone: {e}", dir.display()))?;
    let path = dir.join(FILE);
    let body = json!({
        "claudeAiOauth": {
            "accessToken": token,
            "refreshToken": null,
            "expiresAt": null,
            "scopes": [SCOPE],
        }
    });
    let write = || -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
        file.write_all(format!("{body}\n").as_bytes())
    };

    write().map_err(|e| format!("write {}: {e}", path.display()))
}

/// The token staged under `dir`.
///
/// # Errors
///
/// Fails, never naming the token, when the file does not read, is not
/// JSON or holds none.
pub fn read(dir: &Path) -> Result<String, String> {
    let path = dir.join(FILE);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    // The parser's message can quote the text, so only its place is kept.
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{} is not JSON (line {}, column {})",
            path.display(),
            e.line(),
            e.column()
        )
    })?;
    value["claudeAiOauth"]["accessToken"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{} holds no access token", path.display()))
}

/// `dir` as the absolute path a read rule names.
///
/// # Errors
///
/// Fails when it does not resolve.
pub fn locate(dir: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(dir).map_err(|e| format!("locate {}: {e}", dir.display()))
}

/// Whether `c` can be part of an access token.
fn token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "-_.~+/=".contains(c)
}

/// Points `command` at the configuration under `dir` and refuses its
/// agent every read of it: `Read(//<dir>/**)` and the directory itself
/// join the tools it refuses.
pub fn confine(command: &mut Command, dir: &Path) {
    let shown = dir.display().to_string();
    command.env.push((CONFIG_VAR.to_owned(), shown.clone()));
    let rules = format!("Read(/{shown}/**),Read(/{shown})");
    let at = command
        .args
        .iter()
        .position(|a| a == "--disallowedTools")
        .map(|at| at + 1);
    match at.and_then(|at| command.args.get_mut(at)) {
        Some(tools) => {
            tools.push(',');
            tools.push_str(&rules);
        }
        None => command.args.extend(["--disallowedTools".to_owned(), rules]),
    }
}

/// A host that confines every `claude` it starts to the credential
/// staged under its directory, and starts everything else as given.
pub struct Confined<'a> {
    inner: &'a dyn Host,
    dir: PathBuf,
}

impl<'a> Confined<'a> {
    /// `inner`, its `claude` runs confined to the credential under
    /// `dir`.
    #[must_use]
    pub fn new(inner: &'a dyn Host, dir: &Path) -> Self {
        Self {
            inner,
            dir: dir.to_path_buf(),
        }
    }
}

impl Host for Confined<'_> {
    fn run(&self, command: &Command) -> Result<Output, String> {
        if command.program != PROGRAM {
            return self.inner.run(command);
        }
        let mut confined = command.clone();
        confine(&mut confined, &self.dir);
        self.inner.run(&confined)
    }
}

#[cfg(test)]
mod tests;
