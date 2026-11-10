//! The run log: what `claude --output-format stream-json` wrote, read
//! as the review's run reads an agent's log. Each model call once (a
//! call's blocks share one message id), its usage by model, the main
//! thread's text blocks in order, which hold a specialist's findings as
//! it stated them, the files written, the main thread's reads, the
//! result's answer, cost, turns and duration, and the model the run
//! resolved to.

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

/// One `Read` call the main thread made: the path, the 1-based line
/// it started at and the lines it asked for, as the call named them,
/// whether a result that is no error answered it, and whether its
/// offset or limit was given as no whole number. Only an answered read
/// with a good range gave the reviewer lines: one the tool refused, or
/// whose result never came, as when the run was killed at its deadline,
/// gave it none.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadCall {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    /// Whether a tool result that is no error came for it.
    pub answered: bool,
    /// Whether its offset or limit is there but neither a whole number
    /// nor a string of one, so which lines it read is not known.
    pub bad_range: bool,
}

/// A run, read from its log.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunLog {
    /// Whether a result came, and whether it was a success.
    pub answered: bool,
    /// Whether a result came at all, a success or not.
    pub resulted: bool,
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
    /// Every `Read` call the main thread made, in order.
    pub reads: Vec<ReadCall>,
    /// The work directory the run started in, as its first line names
    /// it; empty when the log names none.
    pub cwd: String,
    /// Every path a `Grep` or `Glob` call of the main thread named, in
    /// order. A search that names none covers the whole work directory
    /// and is not listed.
    pub searches: Vec<String>,
}

impl RunLog {
    /// Why the run was charged: it reported a cost, or, with no cost in
    /// it, as a run killed at its deadline or crashed before its result
    /// has none, it made model calls that used tokens; none when it cost
    /// nothing. The turns a result counts are no charge: a call the API
    /// refused counts one and costs nothing.
    #[must_use]
    pub fn charged(&self) -> Option<String> {
        if self.cost > 0.0 {
            return Some(format!("it cost {:.4} USD", self.cost));
        }
        let calls: u64 = self
            .usage
            .values()
            .filter(|u| u.fresh + u.cache_write + u.cache_read + u.output > 0)
            .map(|u| u.calls)
            .sum();
        match calls {
            0 => None,
            1 => Some("it made 1 model call".to_owned()),
            n => Some(format!("it made {n} model calls")),
        }
    }

    /// Whether the run ran, as the review and the grader both decide
    /// it: it ended with a successful result that answered something,
    /// or it was [`charged`](Self::charged), so its log holds a model
    /// call. A run that did not is one the review may start again.
    #[must_use]
    pub fn ran(&self) -> bool {
        self.succeeded() || self.charged().is_some()
    }

    /// Whether the run ended with a successful result that answered
    /// something; a run that ran but did not was cut off, by its turn
    /// budget, its deadline or its cost cap, or answered nothing.
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.answered && !self.answer.trim().is_empty()
    }
}

/// What the reader carries between lines.
#[derive(Default)]
struct Reading {
    log: RunLog,
    /// The message ids seen, so a call's blocks count once.
    seen: BTreeSet<String>,
    /// Each main-thread read's place in `reads`, by its tool use id.
    reads: BTreeMap<String, usize>,
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
            Some("user") => reading.user(&value),
            Some("result") => reading.result(&value),
            Some("system") if reading.log.cwd.is_empty() => {
                reading.log.cwd = value["cwd"].as_str().unwrap_or_default().to_owned();
            }
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
                Some("tool_use") => self.tool_use(block, main),
                Some("text") if main => {
                    if let Some(text) = block["text"].as_str() {
                        self.log.texts.push(text.to_owned());
                    }
                }
                _ => {}
            }
        }
    }

    /// One tool call: a write's path, a main-thread read, or the path a
    /// main-thread search names.
    fn tool_use(&mut self, block: &Value, main: bool) {
        let input = &block["input"];
        if main
            && matches!(block["name"].as_str(), Some("Grep" | "Glob"))
            && let Some(path) = input["path"].as_str()
        {
            self.log.searches.push(path.to_owned());
        }
        let Some(path) = input["file_path"].as_str() else {
            return;
        };
        match block["name"].as_str() {
            Some("Write" | "Edit") => self.log.writes.push(path.to_owned()),
            Some("Read") if main => {
                let (offset, limit) = (number(&input["offset"]), number(&input["limit"]));
                let id = block["id"].as_str().unwrap_or_default().to_owned();
                self.reads.insert(id, self.log.reads.len());
                self.log.reads.push(ReadCall {
                    path: path.to_owned(),
                    bad_range: offset.is_err() || limit.is_err(),
                    offset: offset.unwrap_or_default(),
                    limit: limit.unwrap_or_default(),
                    answered: false,
                });
            }
            _ => {}
        }
    }

    /// The tools' results: a main-thread read is answered when the
    /// last result for it is no error.
    fn user(&mut self, value: &Value) {
        for block in value["message"]["content"].as_array().into_iter().flatten() {
            if let Some(at) = block["tool_use_id"]
                .as_str()
                .and_then(|id| self.reads.get(id))
                && let Some(read) = self.log.reads.get_mut(*at)
            {
                read.answered = block["is_error"].as_bool() != Some(true);
            }
        }
    }

    /// The result: the answer, the cost, the turns and each model's
    /// usage as the run summed it.
    fn result(&mut self, value: &Value) {
        let log = &mut self.log;
        let is_error = value["is_error"].as_bool() == Some(true);
        log.resulted = true;
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

/// A `Read`'s offset or limit: none when it is absent or null, the
/// number when it is a whole number or a string of one, else an error,
/// as the lines such a read gave are not known.
fn number(value: &Value) -> Result<Option<usize>, ()> {
    let n = match value {
        Value::Null => return Ok(None),
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    };
    n.map(|n| Some(usize::try_from(n).unwrap_or(usize::MAX)))
        .ok_or(())
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
