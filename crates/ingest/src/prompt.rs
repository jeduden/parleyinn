//! Prompt matches (PRV-08): what becomes of the prompt commitment a
//! `UserPromptSubmit` hook handler kept, when ingest reads the first
//! prompt line carrying its `prompt_id`. That line uses it up, whatever
//! its `promptSource`; it matches only when it is a `user` line marked
//! typed whose redacted text commits to the same value under the
//! commitment's key. Every later line carrying the `prompt_id` is a
//! conflict. Nothing here reads a file, a clock or randomness.

use record::commitment::commit;
use serde_json::{Value, json};
use store::PromptCommitment;

/// What became of a prompt commitment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The line is typed `user` input whose redacted text matches.
    Matched,
    /// The line used it up without matching.
    Unmatched,
    /// It was used up already.
    Conflict,
}

impl Outcome {
    /// Its name in a prompt match event.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::Unmatched => "unmatched",
            Self::Conflict => "conflict",
        }
    }
}

/// A prompt line: one carrying a `prompt_id` and a `promptSource`,
/// with the prompt text its content holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptLine {
    pub prompt_id: String,
    /// Marked typed at the harness's input (`typed`, never `sdk`).
    pub typed: bool,
    /// Its prompt text: a text content, or its text blocks joined by
    /// line breaks.
    pub text: Option<String>,
}

impl PromptLine {
    /// The prompt line `line` is, if it is one.
    #[must_use]
    pub fn of(line: &Value) -> Option<Self> {
        let prompt_id = line["promptId"].as_str()?;
        let source = line["promptSource"].as_str()?;
        let content = &line["message"]["content"];
        let text = match content {
            Value::String(text) => Some(text.clone()),
            Value::Array(blocks) if !blocks.is_empty() => blocks
                .iter()
                .map(|b| b["text"].as_str().filter(|_| b["type"] == "text"))
                .collect::<Option<Vec<&str>>>()
                .map(|texts| texts.join("\n")),
            _ => None,
        };

        Some(Self {
            prompt_id: prompt_id.to_owned(),
            typed: source == "typed",
            text,
        })
    }
}

/// What a prompt line does to `commitment`, the one of its `prompt_id`
/// if any, given whether ingest already used it up (`used`), whether
/// the line is classed `user` (`user`), and its text redacted by
/// `redact`. None when no prompt commitment names it.
#[must_use]
pub fn outcome(
    commitment: Option<&PromptCommitment>,
    used: bool,
    user: bool,
    line: &PromptLine,
    redact: &dyn Fn(&str) -> String,
) -> Option<Outcome> {
    let commitment = commitment?;
    if used || commitment.used {
        return Some(Outcome::Conflict);
    }
    let matched = user
        && line.typed
        && line
            .text
            .as_deref()
            .and_then(|text| commit(&commitment.key, &json!(redact(text))).ok())
            .is_some_and(|c| c == commitment.commitment);

    Some(if matched {
        Outcome::Matched
    } else {
        Outcome::Unmatched
    })
}

#[cfg(test)]
mod tests;
