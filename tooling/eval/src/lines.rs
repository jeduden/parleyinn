//! Transcript lines in the shapes Claude Code 2.1.296 recorded: each
//! generated line is a recorded line of the committed fixture with its
//! ids, links, time and text replaced, so the transcript parser reads
//! it as it reads a real run.

use serde_json::{Value, json};

/// The recorded transcript the line shapes come from.
pub const FIXTURE: &str = "testdata/claude-code/2.1.296/transcripts/projects/-w-my-app/205beaae-e4b8-4798-a1aa-cdf50c807136.jsonl";

/// One recorded line of each shape a scripted run is made of.
#[derive(Debug, Clone, PartialEq)]
pub struct Templates {
    prompt: Value,
    tool_use: Value,
    tool_result: Value,
    reply: Value,
    boundary: Value,
    summary: Value,
}

/// The shape a recorded line has, when it is one a run is made of.
fn shape(line: &Value) -> Option<&'static str> {
    let content = &line["message"]["content"];
    let first = content
        .get(0)
        .map_or("", |b| b["type"].as_str().unwrap_or(""));
    match (line["type"].as_str()?, line["subtype"].as_str()) {
        ("system", Some("compact_boundary")) => Some("boundary"),
        ("user", _) if line["isCompactSummary"] == true => Some("summary"),
        ("user", _) if line["isMeta"] == true => None,
        ("user", _) if content.is_string() => Some("prompt"),
        ("user", _) if first == "tool_result" => Some("tool_result"),
        ("assistant", _) if first == "tool_use" => Some("tool_use"),
        ("assistant", _) if first == "text" => Some("reply"),
        _ => None,
    }
}

impl Templates {
    /// The first line of each shape in the recorded transcript `text`.
    ///
    /// # Errors
    ///
    /// Fails when a line is no JSON or a shape has no line.
    pub fn from_transcript(text: &str) -> Result<Self, String> {
        let mut found: Vec<(&'static str, Value)> = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let value: Value =
                serde_json::from_str(line).map_err(|e| format!("line {}: {e}", n + 1))?;
            if let Some(s) = shape(&value).filter(|s| found.iter().all(|(f, _)| f != s)) {
                found.push((s, value));
            }
        }
        let mut take = |name: &str| {
            found
                .iter()
                .position(|(s, _)| *s == name)
                .map(|i| found.swap_remove(i).1)
                .ok_or_else(|| format!("the fixture has no {name} line"))
        };

        Ok(Self {
            prompt: take("prompt")?,
            tool_use: take("tool_use")?,
            tool_result: take("tool_result")?,
            reply: take("reply")?,
            boundary: take("boundary")?,
            summary: take("summary")?,
        })
    }
}

/// Writes one run's lines in order, each linked to the one before.
#[derive(Debug)]
pub struct Session<'t> {
    templates: &'t Templates,
    id: String,
    /// The `uuid` of the last line written.
    last: Option<String>,
    /// The prompt the lines being written answer.
    prompt: String,
    /// Seconds since the run's start, one more for every line.
    clock: u64,
    /// A counter that makes every generated id unique within the run.
    serial: u64,
    lines: Vec<String>,
    /// Where the last compaction's boundary line is.
    boundary: usize,
}

impl<'t> Session<'t> {
    /// A run whose harness session id is `id`.
    #[must_use]
    pub fn new(templates: &'t Templates, id: &str) -> Self {
        Self {
            templates,
            id: id.to_owned(),
            last: None,
            prompt: String::new(),
            clock: 0,
            serial: 0,
            lines: Vec::new(),
            boundary: 0,
        }
    }

    /// The run's harness session id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The lines written so far, each one JSON object, and where the
    /// last compaction's boundary line is among them: the run's context
    /// starts there.
    #[must_use]
    pub fn into_lines(self) -> (Vec<String>, usize) {
        (self.lines, self.boundary)
    }

    /// A new id: the run's id with the serial in its last group.
    fn next_id(&mut self) -> String {
        self.serial += 1;
        let head = self.id.get(..24).unwrap_or(&self.id);
        format!("{head}{:012x}", self.serial)
    }

    /// The time of the next line, in the recorded transcript's format.
    fn next_time(&mut self) -> String {
        self.clock += 7;
        let (h, m, s) = (9 + self.clock / 3600, self.clock / 60 % 60, self.clock % 60);
        format!("2026-10-11T{h:02}:{m:02}:{s:02}.000Z")
    }

    /// Fills the fields every line carries and keeps it.
    fn push(&mut self, mut line: Value) -> String {
        let uuid = self.next_id();
        line["uuid"] = json!(uuid);
        line["parentUuid"] = json!(self.last);
        line["timestamp"] = json!(self.next_time());
        line["sessionId"] = json!(self.id);
        if line.get("promptId").is_some() {
            line["promptId"] = json!(self.prompt);
        }
        self.lines.push(line.to_string());
        self.last = Some(uuid.clone());
        uuid
    }

    /// A prompt the user typed.
    pub fn prompt(&mut self, text: &str) {
        self.prompt = self.next_id();
        let mut line = self.templates.prompt.clone();
        line["message"]["content"] = json!(text);
        self.push(line);
    }

    /// A shell command the assistant ran and what it printed.
    pub fn command(&mut self, command: &str, output: &str) {
        let tool = format!("toolu_{}", self.next_id());
        let mut call = self.templates.tool_use.clone();
        let input = json!({"command": command, "description": "Run a command"});
        call["message"]["id"] = json!(format!("msg_{}", self.next_id()));
        call["requestId"] = json!(format!("req_{}", self.next_id()));
        call["message"]["content"] = json!([{
            "type": "tool_use", "id": tool, "name": "Bash", "input": input,
            "caller": {"type": "direct"},
        }]);
        call["wireToolInputs"] = json!({ tool.clone(): input });
        let called = self.push(call);
        let mut result = self.templates.tool_result.clone();
        result["message"]["content"] = json!([{
            "tool_use_id": tool, "type": "tool_result", "content": output, "is_error": false,
        }]);
        result["toolUseResult"]["stdout"] = json!(output);
        result["sourceToolAssistantUUID"] = json!(called);
        self.push(result);
    }

    /// The assistant's text reply.
    pub fn reply(&mut self, text: &str) {
        let mut line = self.templates.reply.clone();
        line["message"]["id"] = json!(format!("msg_{}", self.next_id()));
        line["requestId"] = json!(format!("req_{}", self.next_id()));
        line["message"]["content"] = json!([{"type": "text", "text": text}]);
        self.push(line);
    }

    /// A compaction: the boundary, then the summary that replaces what
    /// came before it in the model's context.
    pub fn compact(&mut self, summary: &str) {
        let before = self.last.clone();
        let mut boundary = self.templates.boundary.clone();
        let kept = json!({"headUuid": before, "anchorUuid": before, "tailUuid": before});
        boundary["compactMetadata"]["preservedSegment"] = kept;
        boundary["compactMetadata"]["preservedMessages"] =
            json!({"anchorUuid": before, "uuids": [before], "allUuids": [before]});
        boundary["logicalParentUuid"] = json!(before);
        self.last = None;
        self.boundary = self.lines.len();
        self.push(boundary);
        self.prompt = self.next_id();
        let mut line = self.templates.summary.clone();
        line["message"]["content"] = json!(format!(
            "This session is being continued from a previous conversation that ran out of \
             context. The summary below covers the earlier portion of the conversation.\n\n\
             Summary:\n{summary}"
        ));
        self.push(line);
    }
}

#[cfg(test)]
mod tests;
