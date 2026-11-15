//! The transcripts `parinn ingest` may read (SEC-18, REC-22).
//!
//! A path lies within the transcript roots once resolved, symlinks
//! included. The one exception is a path the principal names on
//! `parinn ingest --path` at a terminal: an explicit widening principal
//! act (OWN-11, OWN-12), allowed once the path resolves to a regular
//! file outside the home, and recorded before anything is read. A path
//! from hook input or a transcript line never has the exception, and
//! neither does one named without a terminal. The act also covers
//! reading its path again while an ingest marker holds lines of it, so
//! a hold is released as REC-04 asks without a second act.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use administration::act::Act;
use allowed_root::{Access, AllowedRoots};
use audit::entry::{Counter, Kind, Occurrence};
use record::digest::Digest;
use store::Store;

use crate::act::Taking;
use crate::settings::acts_in;
use crate::{Context, failed};

const OP: &str = "ingest";

/// Why a candidate is not ingested.
enum Declined {
    /// Said and audited already.
    Said,
    /// To say.
    Why(String),
}

/// The transcripts among `candidates` `ingest` may read, resolved, and
/// how many it may not, each said on `stderr`. `named` says the
/// principal named them on `--path`; a transcript found under the roots
/// is never named.
pub(crate) fn allowed(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    candidates: &[PathBuf],
    named: bool,
    stderr: &mut dyn Write,
) -> (Vec<PathBuf>, usize) {
    let roots = roots(ctx, settings);
    let (mut found, mut refused) = (Vec::new(), 0);
    for path in candidates {
        match transcript(ctx, &roots, path, named, stderr) {
            Ok(resolved) => found.push(resolved),
            Err(declined) => {
                if let Declined::Why(why) = declined {
                    failed(OP, &why, stderr);
                }
                refused += 1;
            }
        }
    }

    (found, refused)
}

/// The transcripts among `held`, an ingest marker's sources, that
/// `ingest` may read again: each within the roots, and each the
/// principal named on `--path` in an act the record holds, whose held
/// lines that act covers until they are released (REC-04). Each other
/// is refused and audited.
pub(crate) fn within(
    ctx: &Context,
    settings: &configuration::resolve::Settings,
    held: &[PathBuf],
    stderr: &mut dyn Write,
) -> Vec<PathBuf> {
    let roots = roots(ctx, settings);
    let named = named_before(ctx);

    held.iter()
        .filter_map(|path| {
            if let Ok(within) = roots.resolve(path, Access::Read) {
                return Some(within.path().to_path_buf());
            }
            let again = outside_home(ctx, path)
                .filter(|resolved| is_file(resolved) && named.contains(&digest(resolved)));
            if again.is_none() {
                outside(ctx, stderr);
            }
            again
        })
        .collect()
}

/// The path digests the `ingest --path` acts in the record name; none
/// when the record cannot be read, so a held path stays refused.
fn named_before(ctx: &Context) -> BTreeSet<String> {
    let recorded = ctx.home().map_err(|e| e.to_string()).and_then(|home| {
        let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
        acts_in(&home, &store)
    });

    recorded
        .unwrap_or_default()
        .into_iter()
        .filter(|act| act.name == "ingest_path")
        .filter_map(|act| Some(act.content.get("path_digest")?.as_str()?.to_owned()))
        .collect()
}

/// The digest an `ingest --path` act carries of the resolved path it
/// names, as text.
fn digest(resolved: &Path) -> String {
    path_digest(resolved).to_hex()
}

/// The digest of `resolved`, a path the principal names.
fn path_digest(resolved: &Path) -> Digest {
    Digest::of(&[resolved.to_string_lossy().as_bytes()])
}

/// The transcript roots, each resolved.
fn roots(ctx: &Context, settings: &configuration::resolve::Settings) -> AllowedRoots {
    let os_home = ctx.home.as_deref().map(Path::new);
    settings
        .texts("transcript.roots")
        .iter()
        .fold(AllowedRoots::default(), |allowed, root| {
            allowed.with(&allowed_root::expand_home(root, os_home), Access::Read)
        })
}

/// `path` resolved: within the roots, or named by the principal at a
/// terminal.
fn transcript(
    ctx: &Context,
    roots: &AllowedRoots,
    path: &Path,
    named: bool,
    stderr: &mut dyn Write,
) -> Result<PathBuf, Declined> {
    if let Ok(within) = roots.resolve(path, Access::Read) {
        return Ok(within.path().to_path_buf());
    }
    let widened = if named && ctx.terminal {
        widen(ctx, path, stderr)
    } else {
        None
    };
    match widened {
        Some(resolved) => resolved,
        None => Err(outside(ctx, stderr)),
    }
}

/// The path the principal named, once the act that widens the roots to
/// it is taken. None when it is no path they may name.
fn widen(ctx: &Context, path: &Path, stderr: &mut dyn Write) -> Option<Result<PathBuf, Declined>> {
    let resolved = outside_home(ctx, path)?;
    if !is_file(&resolved) {
        return Some(Err(not_a_file(ctx, stderr)));
    }

    Some(take(ctx, resolved, stderr))
}

/// `path` resolved, when the principal may name it. None when it does
/// not resolve, or it lies in the home, whose keys and store are no
/// transcript.
fn outside_home(ctx: &Context, path: &Path) -> Option<PathBuf> {
    let resolved = fs::canonicalize(path).ok()?;
    let home = ctx.home().ok()?;
    // A home that is not there yet holds no path, so a root that does
    // not resolve reads as "not in the home": the named path resolved.
    if fs::canonicalize(home.root()).is_ok_and(|root| resolved.starts_with(root)) {
        return None;
    }

    Some(resolved)
}

/// Whether `resolved` is a regular file.
fn is_file(resolved: &Path) -> bool {
    fs::metadata(resolved).is_ok_and(|m| m.is_file())
}

/// Records the act that names `resolved`.
fn take(ctx: &Context, resolved: PathBuf, stderr: &mut dyn Write) -> Result<PathBuf, Declined> {
    let mut taking = Taking::open(ctx).map_err(Declined::Why)?;
    let act = Act::IngestPath {
        path_digest: path_digest(&resolved),
    };
    taking
        .take(ctx, OP, &act, stderr)
        .map_err(|_| Declined::Said)?;

    Ok(resolved)
}

/// Audits and counts a path outside every allowed root, naming none
/// (SEC-18, I6).
fn outside(ctx: &Context, stderr: &mut dyn Write) -> Declined {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Rejected,
        counter: Counter::PathOutsideAllowedRoots,
        operation: OP,
        detail: "path outside allowed roots",
    };
    if let Err(e) = ctx.audit(&occurrence) {
        let _ = writeln!(
            stderr,
            "parinn ingest: audit a path outside allowed roots: {e}"
        );
    }

    Declined::Why("path outside allowed roots".to_owned())
}

/// Audits and counts a named path that is no regular file: a refused
/// principal act, and nothing is read from it (SEC-18, I6).
fn not_a_file(ctx: &Context, stderr: &mut dyn Write) -> Declined {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Rejected,
        counter: Counter::PrincipalActRefused,
        operation: OP,
        detail: "refused: named path is not a regular file",
    };
    if let Err(e) = ctx.audit(&occurrence) {
        let _ = writeln!(stderr, "parinn ingest: audit a refused named path: {e}");
    }

    Declined::Why("the named path is not a regular file".to_owned())
}

#[cfg(test)]
pub(crate) mod tests;
