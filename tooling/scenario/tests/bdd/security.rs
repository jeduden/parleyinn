//! The steps of SRS §6, security (SEC). `@SEC-02` and `@SEC-03` are
//! bound: the core refuses a home, or a file in it, that another OS user
//! can reach or owns, and a home bound to another home id, before it
//! opens any store file. `@SEC-05`'s first two rows are bound: a hook
//! input past 1 MiB or nested past depth 64 is refused and audited. Its
//! third, a long line streamed to the payload store, is raised: SEC-08
//! redacts each JSON string whole and PRV-08 parses the line whole.
//!
//! A path goes to another OS user directly when the scenarios run as
//! root, else through `sudo -n`, as CI's runner allows; each is given
//! back before the step that compares the home, and again when the
//! scenario ends, so its home can be removed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use cucumber::{given, then, when};
use record::digest::Digest;

use crate::World;
use crate::record::{first_start, home, last_run, run_core, set_env};

/// What the security steps track.
#[derive(Debug, Default)]
pub struct SecuritySteps {
    /// Every file and directory in the home, with its content and
    /// modification time, as it stood before the command ran.
    before: Option<Snapshot>,
    /// The path a step made one the core must refuse, and the words of
    /// the problem its error must name.
    refused: Option<(PathBuf, String)>,
    given: Given,
}

type Snapshot = BTreeMap<PathBuf, (Vec<u8>, Option<SystemTime>)>;

fn sec(w: &mut World) -> &mut SecuritySteps {
    w.section::<SecuritySteps>()
}

/// The paths a step gave another OS user, each with the OS user it
/// belonged to; dropping it gives them back.
#[derive(Debug, Default)]
struct Given(Vec<(PathBuf, u32)>);

impl Given {
    /// Gives every path back to the OS user it belonged to.
    fn give_back(&mut self) -> Result<(), String> {
        for (path, uid) in self.0.drain(..).rev() {
            give(&path, uid)?;
        }
        Ok(())
    }
}

impl Drop for Given {
    fn drop(&mut self) {
        // A path left with another OS user is the temporary
        // directory's to clean; the scenario's outcome stands.
        let _ = self.give_back();
    }
}

/// The OS user that owns `path`.
fn owner(path: &Path) -> Result<u32, String> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path)
        .map(|m| m.uid())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Gives `path` to the OS user `uid`: directly as root, else through
/// `sudo -n`.
pub(crate) fn give(path: &Path, uid: u32) -> Result<(), String> {
    if running_as_root()? {
        return std::os::unix::fs::chown(path, Some(uid), None)
            .map_err(|e| format!("chown {}: {e}", path.display()));
    }
    let status = Command::new("sudo")
        .args(["-n", "chown", &uid.to_string()])
        .arg(path)
        .status()
        .map_err(|e| format!("sudo -n chown {}: {e}", path.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "sudo -n chown {} exited {status}: the step needs root or sudo without a password",
            path.display()
        ))
    }
}

/// Whether the scenarios run as root: a file they make belongs to UID 0.
fn running_as_root() -> Result<bool, String> {
    Ok(current_uid()? == 0)
}

/// The OS user the scenarios run as: the owner of a file made now.
fn current_uid() -> Result<u32, String> {
    let dir = testkit::TempDir::new().map_err(|e| e.to_string())?;
    owner(dir.path())
}

/// Every file and directory under `root`, with its content and
/// modification time; nothing when `root` does not exist.
fn snapshot(root: &Path) -> Result<Snapshot, String> {
    let mut out = BTreeMap::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let listing = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for item in listing {
            let path = item.map_err(|e| e.to_string())?.path();
            let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            let content = if meta.is_dir() {
                pending.push(path.clone());
                Vec::new()
            } else {
                fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?
            };
            out.insert(path, (content, meta.modified().ok()));
        }
    }
    Ok(out)
}

#[given(
    regex = r#"^the store path "(PARINN_HOME|store database|payload store)" has mode "([0-7]{4})" and belongs to the OS user "(running UID|another UID)"$"#
)]
fn store_path(w: &mut World, which: String, mode: String, user: String) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let home = first_start(w)?;
    // A payload, as the payload store keeps one (§8.1).
    let payloads = home.root().join("payloads");
    let mut builder = fs::DirBuilder::new();
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(&payloads).map_err(|e| e.to_string())?;
    let payload = payloads.join(Digest::of(&[b"a payload"]).to_hex());
    fs::write(&payload, "a payload").map_err(|e| e.to_string())?;
    fs::set_permissions(&payload, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;

    let path = match which.as_str() {
        "PARINN_HOME" => home.root().to_path_buf(),
        "store database" => home.store(),
        _ => payload,
    };
    let bits = u32::from_str_radix(&mode, 8).map_err(|e| e.to_string())?;
    fs::set_permissions(&path, fs::Permissions::from_mode(bits)).map_err(|e| e.to_string())?;
    sec(w).before = Some(snapshot(home.root())?);
    let problem = if user == "another UID" {
        let mine = owner(&path)?;
        // nobody, unless that is who the scenarios run as.
        let other = if mine == 65_534 { 65_533 } else { 65_534 };
        give(&path, other)?;
        sec(w).given.0.push((path.clone(), mine));
        "belongs to UID ".to_owned()
    } else {
        format!("mode {mode}")
    };
    sec(w).refused = Some((path, problem));
    Ok(())
}

#[given(regex = r#"^the home is configured with home id "([^"]+)"$"#)]
fn configured_home_id(w: &mut World, id: String) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let home = home(w)?;
    home.create().map_err(|e| e.to_string())?;
    let binding = home.binding();
    fs::write(&binding, serde_json::json!({"id": id}).to_string()).map_err(|e| e.to_string())?;
    fs::set_permissions(&binding, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    sec(w).before = Some(snapshot(home.root())?);
    Ok(())
}

#[given(regex = r#"^the environment supplies home id "([^"]+)"$"#)]
fn supplied_home_id(w: &mut World, id: String) {
    set_env(w, "PARINN_HOME_ID", &id);
}

#[then(regex = r#"^the error names "([^"]+)" and the permission problem$"#)]
fn names_path_and_problem(w: &mut World, which: String) -> Result<(), String> {
    let (path, problem) = sec(w)
        .refused
        .clone()
        .ok_or("no step made a path the core must refuse")?;
    let err = String::from_utf8_lossy(&last_run(w)?.stderr).into_owned();
    let named = format!("{}: ", path.display());
    if err.contains(&named) && err.contains(&problem) {
        return Ok(());
    }
    Err(format!(
        "the error does not name {which} ({}) and \"{problem}\": {}",
        path.display(),
        err.trim()
    ))
}

#[then(regex = r"^the error states that the home id does not match$")]
fn home_id_mismatch(w: &mut World) -> Result<(), String> {
    let err = String::from_utf8_lossy(&last_run(w)?.stderr).into_owned();
    if err.contains("home id") && err.contains("does not match") {
        return Ok(());
    }
    Err(format!(
        "the error does not state that the home id does not match: {}",
        err.trim()
    ))
}

/// The home as it stands is the home as it stood before the command:
/// no store file was made, written or removed. Whatever a step gave
/// another OS user is given back first, so the home can be read.
#[then(regex = r"^no store file is opened( for writing)?$")]
fn no_store_file_opened(w: &mut World) -> Result<(), String> {
    sec(w).given.give_back()?;
    let root = home(w)?.root().to_path_buf();
    let before = sec(w)
        .before
        .take()
        .ok_or("no step took the home's state before the command")?;
    let after = snapshot(&root)?;
    if after == before {
        return Ok(());
    }
    let changed: Vec<String> = before
        .keys()
        .chain(after.keys())
        .filter(|p| before.get(*p) != after.get(*p))
        .map(|p| p.display().to_string())
        .collect();
    Err(format!(
        "the command changed the home: {}",
        changed.join(", ")
    ))
}

/// A hook input that is `bytes` long: a JSON object with one string.
fn hook_input_of(bytes: usize) -> Vec<u8> {
    let mut input = b"{\"prompt\":\"".to_vec();
    input.resize(bytes - 2, b'x');
    input.extend_from_slice(b"\"}");
    input
}

/// `@SEC-05`: the hook runs with an oversized or deeply nested hook
/// input. The third row, a transcript line streamed to the payload
/// store without holding it whole, is raised (plan 2609292012, phase 6).
#[when(
    regex = r#"^the hook "([A-Za-z]+)" runs with (stdin of 1 MiB plus one byte|a JSON object nested 65 levels deep)$"#
)]
fn hook_runs_with(w: &mut World, hook: String, input: String) -> Result<(), String> {
    let input = if input.starts_with("stdin") {
        hook_input_of((1 << 20) + 1)
    } else {
        format!("{}1{}", "{\"a\":".repeat(65), "}".repeat(65)).into_bytes()
    };
    run_core(w, &["hook", &hook], Some(&input))
}

#[then(regex = r"^the hook handler exits 0 with no injection$")]
fn hook_exits_0_with_no_injection(w: &mut World) -> Result<(), String> {
    let out = last_run(w)?;
    if out.status.success() && out.stdout.is_empty() && out.stderr.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the hook exited {} with stdout {:?} and stderr {:?}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}
