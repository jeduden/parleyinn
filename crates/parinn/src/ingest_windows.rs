//! `parinn ingest` of one transcript source, a window at a time: a source
//! over 64 MiB is read across resumes at its ingest marker until it is
//! caught up, a line over 4 MiB stops it (SEC-05, REC-13, REC-03).

use std::path::Path;

use key::random::OsEntropy;
use redaction::Redactor;

/// Ingests the source at `path`, each window a report of its own: the
/// reports made, and why a read failed, if one did. It stops when the
/// source is caught up, at a refused line, or when a window recorded
/// nothing, which another read of the same marker would not change.
pub(crate) fn ingest(
    home: &administration::home::Home,
    path: &Path,
    redactor: &Redactor,
    inputs: &ingest::Inputs,
) -> (Vec<ingest::Ingested>, Option<String>) {
    let mut done = Vec::new();
    loop {
        match ingest::ingest_with(home, path, redactor, inputs, &mut OsEntropy) {
            Ok(read) => {
                let more = read.resumable && read.events > 0 && !read.held;
                done.push(read);
                if !more {
                    return (done, None);
                }
            }
            Err(e) => return (done, Some(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests;
