//! What a benchmark's core runs over, made before each sample and
//! outside its time: for ingestion (NFR-04), a fresh home holding one
//! transcript under its transcript roots. `HOME` and `PARINN_HOME` both
//! point inside it, so a sample never reaches the machine's own home,
//! ParleyInn home or Claude Code configuration (ENG-14).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A recorded Claude Code 2.1.296 transcript whose lines all fit their
/// recorded shapes: the lines a synthetic transcript repeats.
const RECORDED: &str = include_str!(
    "../../../testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/686aa14f-b535-45bc-9ed0-5400ea84bec7.jsonl"
);

/// The recall requests the search benchmark feeds the core: it
/// initializes, then asks `event_search` for a word nearly every line
/// holds, so the search ranks the newest 2,000 matches (NFR-03).
pub const SEARCH_REQUESTS: &str = concat!(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
    "\n",
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"event_search","arguments":{"query":"message","scope":"room","k":50}}}"#,
    "\n",
);

/// A transcript of `lines` lines: the recorded transcript's, repeated,
/// each copy's `uuid`s made its own, so no line repeats another's.
#[must_use]
pub fn transcript(lines: usize) -> String {
    let recorded: Vec<&str> = RECORDED.lines().collect();
    let mut out = String::new();
    for n in 0..lines {
        let copy = n / recorded.len().max(1);
        let line = recorded.get(n % recorded.len().max(1)).unwrap_or(&"");
        out.push_str(&line.replace("\"uuid\":\"", &format!("\"uuid\":\"c{copy}-")));
        out.push('\n');
    }

    out
}

/// A fresh home for one sample, removed when dropped.
#[derive(Debug)]
pub struct Home {
    root: PathBuf,
}

/// Counts the homes this process made, so no two share a name.
static MADE: AtomicU64 = AtomicU64::new(0);

impl Home {
    /// A fresh home under `base`, readable only by its owner, holding
    /// `transcript` under its default transcript root.
    ///
    /// # Errors
    ///
    /// Fails when the home or the transcript cannot be written.
    pub fn with_transcript(base: &Path, transcript: &str) -> io::Result<Self> {
        let n = MADE.fetch_add(1, Ordering::Relaxed);
        let root = base.join(format!("parinn-benchmark-{}-{n}", std::process::id()));
        let projects = root.join(".claude").join("projects").join("-w-bench");
        fs::create_dir_all(&projects)?;
        let home = Self { root };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&home.root, fs::Permissions::from_mode(0o700))?;
        }
        fs::write(projects.join("bench.jsonl"), transcript)?;

        Ok(home)
    }

    /// The hook input of `hook` in Claude Code 2.1.296's recorded shape,
    /// naming this home's transcript.
    #[must_use]
    pub fn hook_input(&self, hook: &str) -> String {
        let transcript = self.root.join(".claude/projects/-w-bench/bench.jsonl");
        let own = match hook {
            "SessionStart" => r#","source":"startup""#,
            "UserPromptSubmit" => {
                r#","prompt_id":"b4d185a7-444a-48c7-b7b3-4f5da463e538","permission_mode":"default","prompt":"Reply with just the word A1.""#
            }
            _ => "",
        };

        format!(
            r#"{{"session_id":"bench","transcript_path":{},"cwd":"/w/bench","hook_event_name":{}{own}}}"#,
            json_string(&transcript.to_string_lossy()),
            json_string(hook),
        )
    }

    /// Whether this home's audit log holds an entry counting `counter`.
    ///
    /// # Errors
    ///
    /// Fails when the audit directory exists and cannot be read.
    pub fn counted(&self, counter: &str) -> io::Result<bool> {
        let Ok(listing) = fs::read_dir(self.root.join(".parinn").join("audit")) else {
            return Ok(false);
        };
        let wanted = format!("\"{counter}\"");
        for file in listing {
            if fs::read_to_string(file?.path()).is_ok_and(|text| text.contains(&wanted)) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// The variables the core starts with: `HOME` and `PARINN_HOME`, both
    /// inside this home.
    #[must_use]
    pub fn env(&self) -> [(&'static str, PathBuf); 2] {
        [
            ("HOME", self.root.clone()),
            ("PARINN_HOME", self.root.join(".parinn")),
        ]
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// `text` as a JSON string.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');

    out
}

#[cfg(test)]
mod tests;
