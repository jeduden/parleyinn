//! Which run each transcript line belongs to (REC-02, REC-04): a
//! transcript is split by the harness's agent id, a line carrying none
//! belonging to the transcript's own run. Attribution comes from the
//! transcript alone, never a hook input, and never from how much of it
//! has been read, so every way of reading it splits it alike.
//!
//! Claude Code names a harness session's transcript `<session id>.jsonl`
//! and each subagent's `<session id>/subagents/agent-<agent id>.jsonl`
//! (ASM-03): the transcript's own run is its harness session's main run,
//! or that subagent's run.

use std::path::Path;

use serde_json::Value;

/// The run a line belongs to, within its transcript source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RunKey {
    /// The harness's agent id; none for a harness session's main run.
    pub agent_id: Option<String>,
    pub harness_session: String,
}

/// The run a transcript's lines belong to unless they name another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    own: RunKey,
}

impl Defaults {
    /// The defaults of the transcript at `path`: a subagent's run when
    /// its path names one, else its harness session's main run, the
    /// harness session named by the path.
    #[must_use]
    pub fn of(path: &Path) -> Self {
        let name = |p: Option<&Path>| {
            p.and_then(Path::file_name)
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parent = path.parent();
        let subagent = stem
            .strip_prefix("agent-")
            .filter(|id| !id.is_empty() && name(parent) == "subagents");
        let own = match subagent {
            Some(id) => RunKey {
                agent_id: Some(id.to_owned()),
                harness_session: name(parent.and_then(Path::parent)),
            },
            None => RunKey {
                agent_id: None,
                harness_session: stem.clone(),
            },
        };

        Self { own }
    }

    /// The transcript's own run.
    #[must_use]
    pub fn own(&self) -> RunKey {
        self.own.clone()
    }

    /// The run the parsed line `line` belongs to: its agent's, when it
    /// names the harness's agent id, else the transcript's own.
    #[must_use]
    pub fn key(&self, line: &Value) -> RunKey {
        let agent_id = line["agentId"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        match agent_id {
            Some(id) => RunKey {
                agent_id: Some(id),
                harness_session: self.own.harness_session.clone(),
            },
            None => self.own(),
        }
    }
}

#[cfg(test)]
mod tests;
