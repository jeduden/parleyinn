//! The scenario's view of the stand-in harness (`testkit::harness`):
//! it starts one in the scenario's home, with the scenario's
//! environment, and reads the record the steps ask of it.

use std::fs;
use std::path::Path;

use store::Store;

pub use testkit::harness::{Harness, wait_until};
pub use testkit::transcript::{append_lines, clean_line};

use crate::World;
use crate::record::{environment, home};

/// Starts a stand-in harness in the scenario's home, its MCP server at
/// once when `mcp`, with `extra` in its environment beside the
/// scenario's.
pub fn start(w: &mut World, mcp: bool, extra: &[(&str, &str)]) -> Result<Harness, String> {
    let core = testkit::core_executable()?;
    let mut set = environment(w);
    set.extend(
        extra
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
    );
    let env: Vec<(&str, &str)> = set.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    Harness::start(&w.home, &core, mcp, &env)
}

/// The scenario's store, read as it stands.
pub fn store(w: &World) -> Result<Store, String> {
    Store::open(&home(w)?.store()).map_err(|e| e.to_string())
}

/// The key the store keeps the transcript source at `path` by: the
/// path resolved, as ingest reads it.
pub fn source_key(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

/// The line number of each event the record holds of the transcript at
/// `path`, across its generations, in the order each was placed.
pub fn placed_lines(w: &World, path: &Path) -> Result<Vec<u64>, String> {
    let store = store(w)?;
    let mut lines = Vec::new();
    for generation in store
        .generations(&source_key(path))
        .map_err(|e| e.to_string())?
    {
        for placed in store.placed(generation.id).map_err(|e| e.to_string())? {
            lines.push(placed.place.line);
        }
    }
    Ok(lines)
}

/// The number of the last line ingest read of the transcript at `path`.
pub fn lines_read(w: &World, path: &Path) -> Result<u64, String> {
    Ok(store(w)?
        .source(&source_key(path))
        .map_err(|e| e.to_string())?
        .map_or(0, |s| s.position.lines))
}
