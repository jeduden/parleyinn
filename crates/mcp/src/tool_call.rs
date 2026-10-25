//! The run that made a tool call (PRV-14). A `tools/call` names no run,
//! only its tool-use id (`_meta` `claudecode/toolUseId`, ASM-21); the
//! transcript line carrying that id names the run. The server finds the
//! run only that way, through what its ingest has read, waiting for
//! ingest to read the line at most [`WAIT`]; never from the call's
//! arguments or any other field.

use std::time::Duration;

use serde_json::Value;

/// How long a call waits for ingest to read the line carrying its
/// tool-use id (PRV-14, OQ-54).
pub const WAIT: Duration = Duration::from_secs(2);

/// Where a call's tool-use id is.
const TOOL_USE_ID: &str = "claudecode/toolUseId";

/// What finds runs by tool-use id: the MCP server's live ingest
/// (REC-13).
pub trait Runs {
    /// Whether ingest serves runs yet: whether the server holds a
    /// session binding. Until it does, a call is served under no run.
    fn serving(&self) -> bool;

    /// The runs whose transcript lines ingest has read carrying
    /// `tool_use_id`, waiting while none does, at most `wait`; with how
    /// long it waited.
    fn find(&self, tool_use_id: &str, wait: Duration) -> (Vec<String>, Duration);
}

/// Why a call's run cannot be found, each counted on its own (I6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cause {
    /// The call carries no tool-use id.
    NoToolUseId,
    /// No line ingest read carried its tool-use id within [`WAIT`].
    NotFound,
    /// Lines of two runs carry its tool-use id.
    Ambiguous,
}

impl Cause {
    /// The cause as the log names it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoToolUseId => "no tool-use id",
            Self::NotFound => "no line carries the tool-use id",
            Self::Ambiguous => "lines of two runs carry the tool-use id",
        }
    }

    /// The fixed text the call is refused with.
    #[must_use]
    pub fn refusal(self) -> &'static str {
        match self {
            Self::NoToolUseId => {
                "the call carries no tool-use id, so the run that made it cannot be found"
            }
            Self::NotFound => {
                "no transcript line carried the call's tool-use id within 2 s, so the run that made it cannot be found"
            }
            Self::Ambiguous => {
                "transcript lines of two runs carry the call's tool-use id, so the run that made it cannot be told"
            }
        }
    }
}

/// What finding one call's run came to, for the caller to log and, when
/// refused, to count and audit (PRV-14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The call's tool-use id, when it carries one.
    pub tool_use_id: Option<String>,
    /// The run found, or why none was.
    pub run: Result<String, Cause>,
    /// How long the call waited for ingest.
    pub waited: Duration,
}

/// The run of the call whose params are `params`, found through `runs`.
#[must_use]
pub fn find(runs: &dyn Runs, params: &Value) -> Found {
    let Some(id) = params["_meta"][TOOL_USE_ID]
        .as_str()
        .filter(|id| !id.is_empty())
    else {
        return Found {
            tool_use_id: None,
            run: Err(Cause::NoToolUseId),
            waited: Duration::ZERO,
        };
    };
    let (mut found, waited) = runs.find(id, WAIT);
    found.sort();
    found.dedup();
    let run = match found.len() {
        0 => Err(Cause::NotFound),
        1 => Ok(found.remove(0)),
        _ => Err(Cause::Ambiguous),
    };

    Found {
        tool_use_id: Some(id.to_owned()),
        run,
        waited,
    }
}

#[cfg(test)]
mod tests;
