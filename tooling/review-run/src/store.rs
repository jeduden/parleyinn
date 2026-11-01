//! The answer store (ENG-28): a content-addressed directory of accepted
//! answers, `<store>/<key>/`, each holding the run's stream-json log,
//! `log.jsonl`, a specialist's assembled report, `report.md`, and the
//! run's usage entry, `usage.json`, under the agent's answer key. It is
//! a local directory, or a GitHub Actions cache restored into one: no
//! network, no server (I4).
//!
//! An answer is reused only when it is there, its log shows a
//! successful result with an answer, and, for a specialist, its report
//! is the one its log assembles and passes the gate's report check
//! again; for a generalist, its answer reads. Only an accepted answer is
//! stored: never a refused, incomplete, silent or over-budget run. A
//! write goes to a directory of its own and is renamed into place, so a
//! reader sees an answer whole or not at all, and of two writers of one
//! key the first rename wins and the second leaves it as it is. A store
//! read alone, `--answers-read-only`, writes nothing. A stored answer
//! is taken out of circulation by its key, `review-run answers drop`,
//! in the spirit of I5.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use review_eval::component::Kind;
use review_eval::log::{self, RunLog, SUCCESS};
use review_gate::report::assemble;

use crate::key::is_key;
use crate::outcome;
use crate::retry::refusal;
use crate::usage::AgentUsage;

/// A stored answer's log.
pub const LOG: &str = "log.jsonl";
/// A stored specialist's report.
pub const REPORT: &str = "report.md";
/// A stored answer's usage entry.
pub const USAGE_FILE: &str = "usage.json";

/// An answer store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    pub dir: PathBuf,
    /// Whether it is read alone: nothing is written to it.
    pub read_only: bool,
}

/// An answer read from the store.
#[derive(Debug, Clone, PartialEq)]
pub struct Stored {
    /// Its log as stored.
    pub log_text: String,
    /// Its log, read.
    pub log: RunLog,
    /// A specialist's report; none for a generalist's.
    pub report: Option<String>,
}

/// What storing an answer did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Put {
    /// It is stored under its key.
    Stored,
    /// An answer was already stored under its key, and is kept.
    Present,
    /// The store is read alone, so nothing was written.
    ReadOnly,
}

/// Fails unless `key` reads as a key, so no other path is touched.
fn checked(key: &str) -> Result<(), String> {
    if is_key(key) {
        Ok(())
    } else {
        Err(format!("{key:?} is no answer key: 64 lowercase hex digits"))
    }
}

impl Store {
    /// The answer stored under `key`; none when none is.
    ///
    /// # Errors
    ///
    /// Fails when `key` is no key, or an answer is there but does not
    /// read, naming the file.
    pub fn get(&self, key: &str) -> Result<Option<Stored>, String> {
        checked(key)?;
        let at = self.dir.join(key);
        if !at.exists() {
            return Ok(None);
        }
        let path = at.join(LOG);
        let log_text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let path = at.join(REPORT);
        let report = match fs::read_to_string(&path) {
            Ok(text) => Some(text),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("read {}: {e}", path.display())),
        };

        Ok(Some(Stored {
            log: log::parse(&log_text),
            log_text,
            report,
        }))
    }

    /// Stores `log_text`, `report` and `usage` under `key`, atomically:
    /// written beside, then renamed into place.
    ///
    /// # Errors
    ///
    /// Fails when `key` is no key, or the answer cannot be written,
    /// naming the path.
    pub fn put(
        &self,
        key: &str,
        log_text: &str,
        report: Option<&str>,
        usage: &AgentUsage,
    ) -> Result<Put, String> {
        self.put_racing(key, log_text, report, usage, &|| {})
    }

    /// [`Store::put`], calling `between` after the answer is written
    /// beside and before it is renamed into place, where another writer
    /// may rename first.
    pub(crate) fn put_racing(
        &self,
        key: &str,
        log_text: &str,
        report: Option<&str>,
        usage: &AgentUsage,
        between: &dyn Fn(),
    ) -> Result<Put, String> {
        checked(key)?;
        if self.read_only {
            return Ok(Put::ReadOnly);
        }
        let target = self.dir.join(key);
        if target.exists() {
            return Ok(Put::Present);
        }
        let named = |path: &Path, e: io::Error| format!("store {}: {e}", path.display());
        fs::create_dir_all(&self.dir).map_err(|e| named(&self.dir, e))?;
        let tmp = self.dir.join(format!(
            ".{key}.{}.{:?}.tmp",
            std::process::id(),
            std::thread::current().id()
        ));
        let written = write_all(&tmp, log_text, report, usage);
        between();
        let renamed =
            written.and_then(|()| fs::rename(&tmp, &target).map_err(|e| named(&target, e)));
        match renamed {
            Ok(()) => Ok(Put::Stored),
            Err(e) => {
                let _ = fs::remove_dir_all(&tmp);
                if target.join(LOG).exists() {
                    Ok(Put::Present)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Takes the answer under `key` out of the store.
    ///
    /// # Errors
    ///
    /// Fails when `key` is no key, none is stored under it, or it cannot
    /// be removed.
    pub fn drop(&self, key: &str) -> Result<(), String> {
        checked(key)?;
        let at = self.dir.join(key);
        match fs::remove_dir_all(&at) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(format!(
                "no answer is stored under {key} in {}",
                self.dir.display()
            )),
            Err(e) => Err(format!("drop {}: {e}", at.display())),
        }
    }
}

/// Writes an answer's files into the fresh directory `dir`.
fn write_all(
    dir: &Path,
    log_text: &str,
    report: Option<&str>,
    usage: &AgentUsage,
) -> Result<(), String> {
    let named = |path: &Path, e: &dyn std::fmt::Display| format!("store {}: {e}", path.display());
    let _ = fs::remove_dir_all(dir);
    fs::create_dir(dir).map_err(|e| named(dir, &e))?;
    let usage = serde_json::to_string_pretty(usage).map_err(|e| named(dir, &e))?;
    let files = [
        (LOG, Some(log_text)),
        (REPORT, report),
        (USAGE_FILE, Some(&usage)),
    ];
    for (name, text) in files {
        if let Some(text) = text {
            let path = dir.join(name);
            fs::write(&path, text).map_err(|e| named(&path, &e))?;
        }
    }
    Ok(())
}

/// Whether an answer of an agent of `kind`, its log `log` and, for a
/// specialist, its report `report`, may be stored or reused: its log
/// shows a successful result with an answer, and a specialist's report
/// is the one its log assembles and the gate takes, its ids held to
/// `known`; a generalist's answer reads.
///
/// # Errors
///
/// Fails naming why it may not.
pub fn accepted(
    kind: Kind,
    log: &RunLog,
    report: Option<&str>,
    known: &BTreeSet<String>,
) -> Result<(), String> {
    if !log.answered || log.subtype != SUCCESS {
        return Err(format!(
            "its log shows no successful result ({})",
            log.subtype
        ));
    }
    if log.answer.trim().is_empty() {
        return Err("its log answered nothing".to_owned());
    }
    if kind != Kind::Specialist {
        return outcome::unreadable(&log.answer).map_or(Ok(()), |why| Err(why.to_owned()));
    }
    let report = report.ok_or("it holds no report")?;
    if report != assemble(&log.texts, &log.answer, None) {
        return Err("its report is not the one its log assembles".to_owned());
    }
    match refusal(report, &log.answer, known) {
        Some(why) => Err(format!("its report is refused: {why}")),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
