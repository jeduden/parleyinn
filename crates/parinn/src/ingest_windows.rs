//! `parinn ingest` of one transcript source, a window at a time: a source
//! over one bounded processing window is read across resumes at its ingest marker until it is
//! caught up, a line over 4 MiB stops it (SEC-05, REC-13, REC-03).

use std::path::Path;

use key::random::OsEntropy;
use redaction::Redactor;

/// Why the windowed reads of one source stopped.
pub(crate) struct Failure {
    pub(crate) message: String,
    /// The source's parsed structure or harness account identifiers
    /// could not fit under the source-memory ceiling, a refusal the CLI
    /// audits and counts.
    pub(crate) source_memory_limit: bool,
}

/// Ingests the source at `path`, each window a report of its own: the
/// reports made, and why a read failed, if one did. It stops when the
/// source is caught up, at a refused line, or when a window recorded
/// neither recorded nor refused anything, which another read of the
/// same marker would not change.
pub(crate) fn ingest(
    home: &administration::home::Home,
    path: &Path,
    redactor: &Result<Redactor, String>,
    inputs: &ingest::Inputs,
) -> (Vec<ingest::Ingested>, Option<Failure>) {
    let redactor = match redactor {
        Ok(redactor) => redactor,
        Err(why) => {
            return (
                Vec::new(),
                Some(Failure {
                    message: why.clone(),
                    source_memory_limit: false,
                }),
            );
        }
    };
    let mut done = Vec::new();
    loop {
        match ingest::ingest_with(home, path, redactor, inputs, &mut OsEntropy) {
            Ok(read) => {
                // A window made only of refused copies still moved the
                // ingest marker. Resume it so valid later lines are not
                // left unread (REC-04, SEC-05).
                let progressed = read.events > 0 || read.refused > 0;
                let more = read.resumable && progressed && !read.held;
                done.push(read);
                if !more {
                    return (done, None);
                }
            }
            Err(e) => {
                let source_memory_limit = matches!(
                    e,
                    ingest::IngestError::IdentifierLimit(_)
                        | ingest::IngestError::SourceMemoryLimit(_)
                );
                return (
                    done,
                    Some(Failure {
                        message: e.to_string(),
                        source_memory_limit,
                    }),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests;
