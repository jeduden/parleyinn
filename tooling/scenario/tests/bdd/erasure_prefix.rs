//! What a purge leaves of a transcript source's consumed-prefix hash
//! (SEC-31, ADM-07, REC-17): the hash is recomputed with the bytes of
//! each purged transcript line zeroed, so it confirms no guess at the
//! purged bytes while the source still resumes; only each purged line's
//! position, length and harness `uuid` remain, and the harness's
//! transcript file stays as it was.

use std::fs;

use cucumber::then;
use store::Source;
use store::source::{prefix_digest, unconfirmable};

use crate::World;
use crate::record::{home, last_run, run_core};

/// Every transcript source of the home that holds a purged line.
fn purged_sources(w: &World) -> Result<Vec<Source>, String> {
    let store = store::Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let sources = store.sources().map_err(|e| e.to_string())?;

    Ok(sources
        .into_iter()
        .filter(|s| !s.purged.is_empty())
        .collect())
}

/// Whether `source`'s hash confirms no guess at its purged bytes: it is
/// the SHA-256 of the file's consumed prefix with each purged line
/// zeroed, never that of the prefix as the harness wrote it, each purged
/// line's position and length are recorded, and the file still holds the
/// harness's bytes there.
fn confirms_nothing(store: &store::Store, source: &Source) -> Result<(), String> {
    let name = &source.path;
    let Ok(bytes) = fs::read(name) else {
        let hidden = source.position.prefix == unconfirmable();

        return hidden
            .then_some(())
            .ok_or_else(|| format!("{name} is gone and its hash confirms a guess"));
    };
    let offset = source.position.offset;
    let zeroed = prefix_digest(&bytes, offset, &source.purged);
    let original = prefix_digest(&bytes, offset, &[]);
    if zeroed != Some(source.position.prefix) {
        return Err(format!("{name}: the hash is not that of the zeroed prefix"));
    }
    if original == Some(source.position.prefix) || original.is_none() {
        return Err(format!("{name}: the hash confirms a guess at the prefix"));
    }
    let placed = store.placed(source.id).map_err(|e| e.to_string())?;
    for span in &source.purged {
        let place = placed
            .iter()
            .find(|p| p.place.offset == span.offset && p.place.length == span.length)
            .ok_or_else(|| format!("{name}: a purged line's position or length is gone"))?;
        let end = usize::try_from(span.offset + span.length).unwrap_or(usize::MAX);
        let line = bytes
            .get(span.offset as usize..end)
            .ok_or_else(|| format!("{name}: a purged line's bytes are gone"))?;
        let harness: serde_json::Value = serde_json::from_slice(line)
            .map_err(|e| format!("{name}: a purged transcript line is not JSON: {e}"))?;
        let uuid = harness["uuid"]
            .as_str()
            .ok_or_else(|| format!("{name}: a purged transcript line has no harness uuid"))?;
        if place.place.uuid.as_deref() != Some(uuid) {
            return Err(format!(
                "{name}: the retained uuid is not the harness's own uuid"
            ));
        }
    }

    Ok(())
}

/// Each source with a purged line has a hash that confirms no guess at
/// the purged bytes (REC-17); some source holds a purged line.
pub fn hash_confirms_nothing(w: &mut World) -> Result<(), String> {
    let store = store::Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;
    let sources = purged_sources(w)?;
    if sources.is_empty() {
        return Err("no transcript source holds a purged line".to_owned());
    }
    sources.iter().try_for_each(|s| confirms_nothing(&store, s))
}

/// Each purged source's hash confirms nothing, `parinn ingest` resumes
/// it in its own generation and `parinn verify` exits 0.
pub fn closed(w: &mut World) -> Result<(), String> {
    hash_confirms_nothing(w)?;
    for source in &purged_sources(w)? {
        let before = generations(w, &source.path)?;
        run_core(w, &["ingest", "--all"], None)?;
        succeeded(w, "parinn ingest")?;
        if generations(w, &source.path)? != before {
            return Err(format!("{} did not resume: a new generation", source.path));
        }
    }
    run_core(w, &["verify"], None)?;
    succeeded(w, "parinn verify")?;

    hash_confirms_nothing(w)
}

fn generations(w: &World, path: &str) -> Result<usize, String> {
    let store = store::Store::open(&home(w)?.store()).map_err(|e| e.to_string())?;

    Ok(store.generations(path).map_err(|e| e.to_string())?.len())
}

fn succeeded(w: &mut World, what: &str) -> Result<(), String> {
    let out = last_run(w)?;
    if out.status.success() {
        return Ok(());
    }

    Err(format!(
        "{what} exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

#[then(
    regex = r#"^the transcript source's consumed-prefix hash is recomputed over the prefix with the bytes of each purged transcript line zeroed, so it confirms no guess at the purged bytes, only each purged line's position, length and the harness's own `uuid` remain, only ParleyInn's own record of the source changes, never the harness's transcript file, and "parinn ingest" resumes the source and "parinn verify" exits 0$"#
)]
fn security_residue_closed(w: &mut World) -> Result<(), String> {
    closed(w)
}

#[then(
    regex = r#"^the consumed-prefix hash of each transcript source with a purged line is recomputed with the bytes of each purged transcript line zeroed, so it confirms no guess at the purged bytes, only each purged line's position, length and the harness's own `uuid` remain, only ParleyInn's own record of the source changes, never the harness's transcript file, and "parinn ingest" resumes that source$"#
)]
fn administration_residue_closed(w: &mut World) -> Result<(), String> {
    closed(w)
}
