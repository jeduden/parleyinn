//! Provenance from a transcript line's structure (PRV-01, PRV-03,
//! PRV-08). A [`Classifier`] reads one transcript's lines in order and
//! gives each exactly one class from PRV-01's closed set, derived from
//! the line's structural fields alone: its line, attachment and content
//! block types, `promptSource`, `isMeta` and the like, and the tool
//! call a tool result answers. Never from its `renderedRole`, and text
//! inside its content only ever lowers a class to `harness_text`.
//!
//! - `user` only for text typed at the harness's input, in a harness
//!   session whose process ancestry a hook observation records and that
//!   no run's tool call started ([`Session`]).
//! - A tool result takes the class of its call's tool; an assistant
//!   line's tool call is `tool_call`, its text `assistant`.
//! - Every attachment is untrusted: a CLAUDE.md's text `file`, an MCP
//!   server's instructions `mcp:<server>`, any other `harness_text`.
//! - Metadata lines that carry no free text are `harness_meta`; any
//!   other line is `harness_text`.
//!
//! A line that drifts from its recorded shape is `harness_text`, as is
//! a `user` or `harness_meta` line with a key beyond its shape's
//! (REC-26). Nothing here reads a file, a clock or randomness.

use std::collections::{BTreeMap, BTreeSet};

pub mod structural_field;

use record::event::Provenance;
use redaction::Redactor;
use serde_json::Value;
use transcript::shape::Shapes;
use transcript::{Drift, Fit, Line, Parsed};

/// The line types whose lines are harness metadata with no free text.
const METADATA: [&str; 5] = [
    "atis-latch",
    "cost-state",
    "file-history-snapshot",
    "mode",
    "permission-mode",
];

/// The `system` subtypes that are harness metadata with no free text.
const METADATA_SYSTEM: [&str; 1] = ["turn_duration"];

/// The flags that mark a `user` line as other than typed input.
const NOT_TYPED: [&str; 5] = [
    "isMeta",
    "isCompactSummary",
    "isSidechain",
    "isVisibleInTranscriptOnly",
    "repeatsCompactSummary",
];

/// The markers of slash command output the harness writes as a `user`
/// line.
const COMMAND_OUTPUT: [&str; 2] = ["<local-command-stdout>", "<local-command-stderr>"];

/// The longest tool or server name a header carries.
const NAME_LIMIT: usize = 64;

/// What the hook handlers recorded of the harness session a transcript
/// belongs to (PRV-08).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Session {
    /// A structural hook observation records its process ancestry.
    pub ancestry_observed: bool,
    /// Its harness process descends from a run's tool call.
    pub nested: bool,
}

/// One line's class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classified {
    /// A line ParleyInn cannot parse: an `unparsed` event (REC-05), with its
    /// drift when its type is one no supported version records.
    Unparsed(Option<Drift>),
    /// A transcript line of this class, with its drift when it drifted
    /// from its recorded shape (REC-26).
    Line(Provenance, Option<Drift>),
}

/// Classifies one transcript's lines, in order.
#[derive(Debug, Clone)]
pub struct Classifier<'a> {
    shapes: &'a Shapes,
    session: Session,
    redactor: &'a Redactor,
    /// Each tool call's tool, by its tool-use id, as the lines read so
    /// far carry them.
    calls: BTreeMap<String, String>,
}

impl<'a> Classifier<'a> {
    /// A classifier of a harness session's lines against the supported
    /// versions' `shapes`; a tool or server name enters a class only
    /// when `redactor` leaves it unchanged.
    #[must_use]
    pub fn new(shapes: &'a Shapes, session: Session, redactor: &'a Redactor) -> Self {
        Self {
            shapes,
            session,
            redactor,
            calls: BTreeMap::new(),
        }
    }

    /// The class of the next line, `text`.
    pub fn line(&mut self, text: &str) -> Classified {
        match transcript::parse(text, self.shapes) {
            Parsed::Unparsed(drift) => Classified::Unparsed(drift),
            Parsed::Drifted(_, drift) => Classified::Line(Provenance::HarnessText, Some(drift)),
            Parsed::Fits(fit) => self.fits(&fit),
        }
    }

    /// The class of a line that fits its shape: one classed `user` or
    /// `harness_meta` must carry no key beyond it.
    fn fits(&mut self, fit: &Fit<'_>) -> Classified {
        let class = self.class(&fit.line);
        if matches!(class, Provenance::User | Provenance::HarnessMeta)
            && let Some(drift) = fit.beyond()
        {
            return Classified::Line(Provenance::HarnessText, Some(drift));
        }

        Classified::Line(class, None)
    }

    fn class(&mut self, line: &Line) -> Provenance {
        match text(line, "type") {
            "user" => self.user(line),
            "assistant" => self.assistant(line),
            "attachment" => self.attachment(line),
            "system" if METADATA_SYSTEM.contains(&text(line, "subtype")) => Provenance::HarnessMeta,
            kind if METADATA.contains(&kind) => Provenance::HarnessMeta,
            _ => Provenance::HarnessText,
        }
    }

    /// A `user` line: typed input, tool results, or the harness's own
    /// text.
    fn user(&self, line: &Line) -> Provenance {
        match line.get("message").and_then(|m| m.get("content")) {
            Some(Value::String(content)) => self.typed(line, &[content.as_str()]),
            Some(Value::Array(blocks)) if !blocks.is_empty() => {
                if blocks.iter().all(|b| text(b, "type") == "tool_result") {
                    self.tool_result(blocks)
                } else if blocks.iter().all(|b| text(b, "type") == "text") {
                    let texts: Vec<&str> = blocks.iter().map(|b| text(b, "text")).collect();
                    self.typed(line, &texts)
                } else {
                    Provenance::HarnessText
                }
            }
            _ => Provenance::HarnessText,
        }
    }

    /// `user` when the line marks its `texts` as typed at the harness's
    /// input in a session whose ancestry is observed and not nested;
    /// else `harness_text`.
    fn typed(&self, line: &Line, texts: &[&str]) -> Provenance {
        let source = matches!(text(line, "promptSource"), "typed" | "sdk");
        let flagged = NOT_TYPED
            .iter()
            .any(|k| line.get(*k) == Some(&Value::Bool(true)));
        let delegated = line.contains_key("agentId");
        let output = texts
            .iter()
            .any(|t| COMMAND_OUTPUT.iter().any(|m| t.contains(m)));
        let session = self.session.ancestry_observed && !self.session.nested;
        if source && !flagged && !delegated && !output && session {
            Provenance::User
        } else {
            Provenance::HarnessText
        }
    }

    /// Tool results all answering calls of one tool take that tool's
    /// class; any other set is `harness_text`.
    fn tool_result(&self, blocks: &[Value]) -> Provenance {
        let tools: Option<BTreeSet<&str>> = blocks
            .iter()
            .map(|b| self.calls.get(text(b, "tool_use_id")).map(String::as_str))
            .collect();
        match tools {
            Some(tools) if tools.len() == 1 => tools
                .first()
                .and_then(|tool| self.tool_class(tool))
                .unwrap_or(Provenance::HarnessText),
            _ => Provenance::HarnessText,
        }
    }

    /// The class of a result of `tool`, when its name is plain.
    fn tool_class(&self, tool: &str) -> Option<Provenance> {
        if !self.plain(tool) {
            return None;
        }
        let server = tool
            .strip_prefix("mcp__")
            .and_then(|rest| rest.split_once("__"))
            .filter(|(server, name)| !server.is_empty() && !name.is_empty());
        if let Some((server, _)) = server {
            return Some(Provenance::Mcp(server.to_owned()));
        }

        Some(match tool {
            "WebFetch" | "WebSearch" => Provenance::Web,
            "Read" => Provenance::File,
            "Agent" | "Task" => Provenance::SubagentResult,
            _ => Provenance::ToolResult(tool.to_owned()),
        })
    }

    /// An assistant line: its tool calls are learnt for the results
    /// that answer them. A refusal the harness wrote in place of a
    /// reply is the harness's text.
    fn assistant(&mut self, line: &Line) -> Provenance {
        if line.get("isApiErrorMessage") == Some(&Value::Bool(true)) {
            return Provenance::HarnessText;
        }
        let blocks = line
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_array);
        let mut called = false;
        for block in blocks.into_iter().flatten() {
            if text(block, "type") != "tool_use" {
                continue;
            }
            called = true;
            let (id, name) = (text(block, "id"), text(block, "name"));
            if !id.is_empty() && !name.is_empty() {
                self.calls.insert(id.to_owned(), name.to_owned());
            }
        }

        if called {
            Provenance::ToolCall
        } else {
            Provenance::Assistant
        }
    }

    /// An attachment line, untrusted whatever its type.
    fn attachment(&self, line: &Line) -> Provenance {
        let attachment = line.get("attachment");
        match attachment.map_or("", |a| text(a, "type")) {
            "instructions" => Provenance::File,
            "mcp_instructions_delta" => {
                let added = attachment
                    .and_then(|a| a.get("addedNames"))
                    .and_then(Value::as_array);
                match added.map(Vec::as_slice) {
                    Some([Value::String(server)]) if self.plain(server) => {
                        Provenance::Mcp(server.clone())
                    }
                    _ => Provenance::HarnessText,
                }
            }
            _ => Provenance::HarnessText,
        }
    }

    /// Whether `name` may enter a header: short, of letters, digits,
    /// spaces, `_`, `.` and `-` alone, not starting or ending in a
    /// space, and left unchanged by redaction.
    fn plain(&self, name: &str) -> bool {
        !name.is_empty()
            && name.len() <= NAME_LIMIT
            && name.trim() == name
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || " _.-".contains(c))
            && self.redactor.redact_text(name).rules.is_empty()
    }
}

/// The string at `key` of `value`, or `""` when it is missing or not a
/// string.
fn text<'v>(value: &'v impl Index, key: &str) -> &'v str {
    value.field(key).and_then(Value::as_str).unwrap_or_default()
}

/// A JSON object or value whose fields [`text`] reads.
trait Index {
    fn field(&self, key: &str) -> Option<&Value>;
}

impl Index for Line {
    fn field(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }
}

impl Index for Value {
    fn field(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }
}

#[cfg(test)]
mod tests;
