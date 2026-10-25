//! The steps of SEC-10 that phase 5 of plan 2609292012 carries: the
//! core keeps its keys per the plan's ruling 2, never from the
//! environment, and no key or credential reaches the store, a log or
//! output.
//!
//! The scenario stays pending: encryption at rest is SEC-09's (M5),
//! `parinn backup create` ADM-06's (M3), the credentials of other
//! components come with them (M7), and the run-seat keys an MCP server
//! keeps in memory are phase 10's. Those steps are unbound.

use std::fs;
use std::path::Path;

use cucumber::{given, then};
use key::key_event::KeyEvent;
use key::key_store::{Entry, FileKeyStore, KeyStore, KeyStoreKind};
use record::digest::unhex;
use serde_json::Value;
use store::Store;

use crate::World;
use crate::record::{home, last_run, run_core, set_env};
use crate::record_keys::ingest_one_line;

/// What the key steps set in the parent environment, by name.
#[derive(Debug, Default)]
pub struct SecurityKeySteps {
    environment: Vec<(String, String)>,
}

fn sec(w: &mut World) -> &mut SecurityKeySteps {
    w.section::<SecurityKeySteps>()
}

/// The home's keys as files, as a home on an OS without a platform key
/// store keeps them.
fn files(w: &World) -> Result<FileKeyStore, String> {
    Ok(FileKeyStore::new(home(w)?.keys()))
}

/// Every file under `dir` but those under `skip`, with its bytes.
fn files_under(dir: &Path, skip: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(out);
    };
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.starts_with(skip) {
            continue;
        }
        if path.is_dir() {
            out.extend(files_under(&path, skip)?);
        } else {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            out.push((path.display().to_string(), bytes));
        }
    }
    Ok(out)
}

/// Whether `needle` occurs in `haystack`.
fn holds(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// `bytes` in standard base64, padded.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[given(
    regex = r#"^the parent environment sets "ANTHROPIC_API_KEY", "GITHUB_TOKEN", a device key value and an at-rest key value$"#
)]
fn parent_environment(w: &mut World) -> Result<(), String> {
    let set = [
        (
            "ANTHROPIC_API_KEY",
            "sk-ant-api03-parinn-scenario-0000000000000000",
        ),
        ("GITHUB_TOKEN", "ghp_parinnScenario000000000000000000000000"),
        ("PARINN_DEVICE_KEY", &"ab".repeat(32)),
        ("PARINN_AT_REST_KEY", &"cd".repeat(32)),
    ];
    for (name, value) in set {
        set_env(w, name, value);
        sec(w).environment.push((name.to_owned(), value.to_owned()));
    }
    Ok(())
}

#[given(regex = r"^no platform key store is available$")]
fn no_platform_key_store(w: &mut World) -> Result<(), String> {
    let kind = home(w)?.key_store().kind();
    if kind == KeyStoreKind::File {
        Ok(())
    } else {
        Err(format!("this OS keeps keys in its {}", kind.as_str()))
    }
}

/// Checks that `entry`'s key is a file only its owner reads, and returns
/// its seed.
fn kept_in_a_file(w: &World, entry: &Entry) -> Result<String, String> {
    let keys = files(w)?;
    let path = keys.path(entry);
    let mode = fs::metadata(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if mode.mode() & 0o777 != 0o600 {
            return Err(format!("{} has mode {:o}", path.display(), mode.mode()));
        }
    }
    #[cfg(not(unix))]
    let _ = mode;
    keys.get(entry).map_err(|e| e.to_string())
}

#[then(
    regex = r"^the device key was generated on the node into a file only the person's OS user can read, not taken from the environment$"
)]
fn device_key_file(w: &mut World) -> Result<(), String> {
    let seed = kept_in_a_file(w, &Entry::device())?;
    let taken = sec(w).environment.iter().any(|(_, v)| *v == seed);
    if taken {
        Err("the device key is a value the environment set".to_owned())
    } else {
        Ok(())
    }
}

#[then(
    regex = r"^the status says the key is a file an unsandboxed agent of the same OS user could read$"
)]
fn status_says_file(w: &mut World) -> Result<(), String> {
    run_core(w, &["status", "--json"], None)?;
    let out = last_run(w)?;
    let status: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let says = status["keys"]["store"] == "file"
        && status["keys"]["readable_by_agents"] == true
        && status["keys"]["where"]
            .as_str()
            .is_some_and(|t| t.contains("an unsandboxed agent of the same OS user could read"));
    if says {
        Ok(())
    } else {
        Err(format!("the status does not say so: {status}"))
    }
}

#[then(
    regex = r"^no key or credential value appears in the store, a segment, a derived artifact, a backup, an export, the audit log or any log output$"
)]
fn no_key_anywhere(w: &mut World) -> Result<(), String> {
    let home = home(w)?;
    let mut secrets: Vec<String> = sec(w).environment.iter().map(|(_, v)| v.clone()).collect();
    for entry in fs::read_dir(home.keys()).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() {
            secrets.push(fs::read_to_string(&path).map_err(|e| e.to_string())?);
        }
    }
    let mut places = files_under(home.root(), &home.keys())?;
    let out = last_run(w)?;
    places.push(("the last output".to_owned(), out.stdout.clone()));
    places.push(("the last error output".to_owned(), out.stderr.clone()));
    for secret in &secrets {
        if let Some((path, _)) = places.iter().find(|(_, b)| holds(b, secret.as_bytes())) {
            return Err(format!("a key or credential appears in {path}"));
        }
    }
    Ok(())
}

#[then(
    regex = r#"^a structural key event in the device seat in the personal room carries only its kind "minted", the device key's fingerprint, the digest of its recovery key's public key and its signatures$"#
)]
fn minted_carries_only(w: &mut World) -> Result<(), String> {
    let home = home(w)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let node = administration::node::Node::find(&home, &store).map_err(|e| e.to_string())?;
    let log = store.log(node.device_writer()).map_err(|e| e.to_string())?;
    let content = log
        .first()
        .and_then(|s| s.content.clone())
        .ok_or("the device seat holds no event")?;
    let event = KeyEvent::from_value(&content).map_err(|e| e.to_string())?;
    let recovery = key::signing::RecoveryKey::fetch(home.key_store().as_ref(), &Entry::recovery())
        .map_err(|e| e.to_string())?;
    // An inception follows and replaces nothing: those fields stay empty.
    let only = event.kind.as_str() == "minted"
        && event.key == node.device().public().fingerprint()
        && event.recovery == recovery.public().digest()
        && event.follows.is_none()
        && event.replaces.is_empty()
        && event.signatures.len() == 1
        && content.as_object().map(|o| o.len()) == Some(6);
    if only {
        Ok(())
    } else {
        Err(format!("the key event carries more: {content}"))
    }
}

#[then(
    regex = r"^the device key's recovery key is kept in an entry of its own, and no file in the home, backup or export, and no log or output line, contains its bytes raw, hex or base64$"
)]
fn recovery_key_kept_apart(w: &mut World) -> Result<(), String> {
    let home = home(w)?;
    let seed = kept_in_a_file(w, &Entry::recovery())?;
    let raw: [u8; 32] = unhex(&seed).map_err(|e| e.to_string())?;
    let forms = [
        raw.to_vec(),
        seed.clone().into_bytes(),
        base64(&raw).into_bytes(),
    ];
    let recovery_file = files(w)?.path(&Entry::recovery());
    let mut places = files_under(home.root(), &recovery_file)?;
    let out = last_run(w)?;
    places.push(("the last output".to_owned(), out.stdout.clone()));
    places.push(("the last error output".to_owned(), out.stderr.clone()));
    for form in &forms {
        if let Some((path, _)) = places.iter().find(|(_, b)| holds(b, form)) {
            return Err(format!("the recovery key's bytes appear in {path}"));
        }
    }
    Ok(())
}

#[then(
    regex = r#"^a run ingested by "parinn ingest --path" has a seat key that ingest minted, kept like the device key in a file only the person's OS user can read$"#
)]
fn ingest_seat_key(w: &mut World) -> Result<(), String> {
    ingest_one_line(w, "ingested")?;
    let out = last_run(w)?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let writer = text
        .split_whitespace()
        .last()
        .ok_or("parinn ingest named no writer")?
        .to_owned();
    let home = home(w)?;
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let writer = record::id::WriterId::parse(&writer).map_err(|e| e.to_string())?;
    let seat = store
        .seat(writer)
        .map_err(|e| e.to_string())?
        .ok_or("the ingest seat is not recorded")?;
    kept_in_a_file(w, &Entry::seat(seat.seat)).map(drop)
}
