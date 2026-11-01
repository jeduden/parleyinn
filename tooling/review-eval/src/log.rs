//! The run log: what `claude --output-format stream-json` wrote, read
//! as the review's run reads an agent's log. Each model call once (a
//! call's blocks share one message id), its usage by model, the main
//! thread's text blocks in order, which hold a specialist's findings as
//! it stated them, the files written, the result's answer, cost, turns
//! and duration, and the model the run resolved to.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The result subtype of a run that answered.
pub const SUCCESS: &str = "success";

/// One model's usage over a run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    pub calls: u64,
    pub fresh: u64,
    pub cache_write: u64,
    pub cache_read: u64,
    pub output: u64,
    pub cost: f64,
}

/// A run, read from its log.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunLog {
    /// Whether a result came, and whether it was a success.
    pub answered: bool,
    /// The result's subtype, or `none` without a result.
    pub subtype: String,
    /// The final answer's text.
    pub answer: String,
    pub cost: f64,
    pub turns: usize,
    pub duration_ms: u64,
    /// The model the main thread's first call ran on.
    pub resolved_model: String,
    pub usage: BTreeMap<String, Usage>,
    /// Every text block the main thread wrote, in order: what a
    /// specialist stated as it went, its report's lines among them.
    pub texts: Vec<String>,
    /// Every path a `Write` or `Edit` call named, in order.
    pub writes: Vec<String>,
}

/// What the reader carries between lines.
#[derive(Default)]
struct Reading {
    log: RunLog,
    /// The message ids seen, so a call's blocks count once.
    seen: BTreeSet<String>,
}

/// Reads a log of one JSON object per line; lines that are not JSON
/// objects are skipped, as a log cut short leaves one.
#[must_use]
pub fn parse(text: &str) -> RunLog {
    let mut reading = Reading {
        log: RunLog {
            subtype: "none".to_owned(),
            ..RunLog::default()
        },
        ..Reading::default()
    };
    for value in text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
    {
        match value["type"].as_str() {
            Some("assistant") => reading.assistant(&value),
            Some("result") => reading.result(&value),
            _ => {}
        }
    }

    reading.log
}

impl Reading {
    /// One model call: its usage once, the main thread's text, and the
    /// tools it used.
    fn assistant(&mut self, value: &Value) {
        let main = value["parent_tool_use_id"].is_null();
        let message = &value["message"];
        if let Some(id) = message["id"].as_str()
            && self.seen.insert(id.to_owned())
        {
            let model = message["model"].as_str().unwrap_or("unknown");
            if main && self.log.resolved_model.is_empty() {
                self.log.resolved_model = model.to_owned();
            }
            add_usage(
                self.log.usage.entry(model.to_owned()).or_default(),
                &message["usage"],
            );
        }
        for block in message["content"].as_array().into_iter().flatten() {
            match block["type"].as_str() {
                Some("tool_use") => self.tool_use(block),
                Some("text") if main => {
                    if let Some(text) = block["text"].as_str() {
                        self.log.texts.push(text.to_owned());
                    }
                }
                _ => {}
            }
        }
    }

    /// One tool call: a write's path.
    fn tool_use(&mut self, block: &Value) {
        if matches!(block["name"].as_str(), Some("Write" | "Edit"))
            && let Some(path) = block["input"]["file_path"].as_str()
        {
            self.log.writes.push(path.to_owned());
        }
    }

    /// The result: the answer, the cost, the turns and each model's
    /// usage as the run summed it.
    fn result(&mut self, value: &Value) {
        let log = &mut self.log;
        let is_error = value["is_error"].as_bool() == Some(true);
        log.answer = value["result"].as_str().unwrap_or_default().to_owned();
        log.subtype = match value["subtype"].as_str() {
            // The harness reports its own refusal, such as a usage limit,
            // as a successful result that is an error: name the refusal.
            Some(SUCCESS) if is_error => {
                format!("error: {}", log.answer.lines().next().unwrap_or_default())
            }
            Some(subtype) => subtype.to_owned(),
            None => "unknown".to_owned(),
        };
        log.answered = log.subtype == SUCCESS;
        log.cost = value["total_cost_usd"].as_f64().unwrap_or_default();
        log.turns =
            usize::try_from(value["num_turns"].as_u64().unwrap_or_default()).unwrap_or(usize::MAX);
        log.duration_ms = value["duration_ms"].as_u64().unwrap_or_default();
        add_model_usage(&mut log.usage, &value["modelUsage"]);
    }
}

/// Adds one call's `usage` to `into`.
fn add_usage(into: &mut Usage, usage: &Value) {
    let n = |key: &str| usage[key].as_u64().unwrap_or_default();
    into.calls += 1;
    into.fresh += n("input_tokens");
    into.cache_write += n("cache_creation_input_tokens");
    into.cache_read += n("cache_read_input_tokens");
    into.output += n("output_tokens");
}

/// Takes each model's cost from the result's `modelUsage`, and the
/// usage of a model the log's calls did not show, such as a model a
/// subagent ran on whose calls the log did not forward.
fn add_model_usage(into: &mut BTreeMap<String, Usage>, by_model: &Value) {
    for (model, usage) in by_model.as_object().into_iter().flatten() {
        let entry = into.entry(model.clone()).or_default();
        entry.cost = usage["costUSD"].as_f64().unwrap_or_default();
        if entry.calls == 0 {
            let n = |key: &str| usage[key].as_u64().unwrap_or_default();
            entry.fresh = n("inputTokens");
            entry.cache_write = n("cacheCreationInputTokens");
            entry.cache_read = n("cacheReadInputTokens");
            entry.output = n("outputTokens");
        }
    }
}

#[cfg(test)]
mod tests;
