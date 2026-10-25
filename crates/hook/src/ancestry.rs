//! The process ancestry a hook handler reads inside the core (PRV-08):
//! the processes it descends from, up to the first. The harness starts
//! the handler, so the nearest harness process above it is its own
//! harness session's; any further harness process above that one means
//! a run's tool call started this harness session, which is then
//! nested, and none of its lines is `user`.
//!
//! On Linux the ancestry is read from `/proc`; elsewhere none is read,
//! so no harness session there is known not to be nested, and none of
//! its lines is `user` (PRV-08's rule before an observation).

use std::path::{Path, PathBuf};

/// How far up the ancestry is read before it stops unread.
pub const DEPTH_LIMIT: usize = 64;

/// The names a harness process runs under (Claude Code: `claude`).
const HARNESS_NAMES: [&str; 2] = ["claude", "claude.exe"];

/// What a hook handler can learn of a process.
pub trait Processes {
    /// The process `pid`'s parent, `0` for the first process; none when
    /// it cannot be read.
    fn parent(&self, pid: u32) -> Option<u32>;
    /// The names the process `pid` runs under: its command name and
    /// the base names of its first two arguments.
    fn names(&self, pid: u32) -> Vec<String>;
}

/// What a hook handler, or the MCP server, observed of its process
/// ancestry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ancestry {
    /// How many harness processes are among its ancestors.
    pub harnesses: usize,
    /// More than one: a harness process above the session's own.
    pub nested: bool,
    /// The nearest harness process above it: its own harness session's,
    /// whose session binding the MCP server takes (REC-13).
    pub harness: Option<u32>,
}

/// The ancestry of `pid` as `processes` tells it, read up to the first
/// process; none when a parent cannot be read, or the chain is deeper
/// than [`DEPTH_LIMIT`] or loops.
#[must_use]
pub fn observe(processes: &dyn Processes, pid: u32) -> Option<Ancestry> {
    let mut harnesses = 0;
    let mut nearest = None;
    let mut current = pid;
    for _ in 0..DEPTH_LIMIT {
        let parent = processes.parent(current)?;
        if parent == 0 || parent == current {
            return Some(Ancestry {
                harnesses,
                nested: harnesses > 1,
                harness: nearest,
            });
        }
        if processes.names(parent).iter().any(|n| harness(n)) {
            harnesses += 1;
            nearest = nearest.or(Some(parent));
        }
        current = parent;
    }

    None
}

/// Whether a process running under `name` is a harness process.
fn harness(name: &str) -> bool {
    HARNESS_NAMES.contains(&name)
}

/// The processes `/proc` describes, as Linux gives them.
#[derive(Debug, Clone)]
pub struct ProcFs {
    root: PathBuf,
}

impl Default for ProcFs {
    fn default() -> Self {
        Self::at(Path::new("/proc"))
    }
}

impl ProcFs {
    /// The processes a `/proc`-shaped tree at `root` describes.
    #[must_use]
    pub fn at(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// This process's id, as `/proc/self` names it.
    #[must_use]
    pub fn own(&self) -> Option<u32> {
        let target = std::fs::read_link(self.root.join("self")).ok()?;
        target.to_str()?.parse().ok()
    }

    fn read(&self, pid: u32, file: &str) -> Option<Vec<u8>> {
        std::fs::read(self.root.join(pid.to_string()).join(file)).ok()
    }
}

impl Processes for ProcFs {
    fn parent(&self, pid: u32) -> Option<u32> {
        let stat = self.read(pid, "stat")?;
        parent_of(&String::from_utf8_lossy(&stat))
    }

    fn names(&self, pid: u32) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(comm) = self.read(pid, "comm") {
            names.push(String::from_utf8_lossy(&comm).trim_end().to_owned());
        }
        if let Some(cmdline) = self.read(pid, "cmdline") {
            names.extend(
                cmdline
                    .split(|b| *b == 0)
                    .take(2)
                    .map(|arg| base_name(&String::from_utf8_lossy(arg))),
            );
        }
        names
    }
}

/// The parent id in a `/proc/<pid>/stat` line: the fourth field, the
/// command name in parentheses before it possibly holding spaces.
fn parent_of(stat: &str) -> Option<u32> {
    let after = &stat[stat.rfind(')')? + 1..];
    after.split_whitespace().nth(1)?.parse().ok()
}

/// The last component of `path`.
fn base_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_owned()
}

/// The ancestry of this process, read from `/proc` on Linux; none
/// elsewhere, or when it cannot be read.
#[cfg(target_os = "linux")]
#[must_use]
pub fn own() -> Option<Ancestry> {
    let proc = ProcFs::default();

    observe(&proc, proc.own()?)
}

/// The ancestry of this process: none off Linux, where no `/proc` is
/// read.
#[cfg(not(target_os = "linux"))]
#[must_use]
pub fn own() -> Option<Ancestry> {
    None
}

#[cfg(test)]
mod tests;
