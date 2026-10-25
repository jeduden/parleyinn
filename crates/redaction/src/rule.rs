//! The maintained redaction rule set (SEC-08): one named pattern per
//! common secret format, versioned so the record can say which rules
//! a redaction ran under. A change to a pattern, or a rule added or
//! taken away, raises [`REDACTION_RULE_SET_VERSION`].

use regex::{Regex, RegexBuilder};

/// The version of [`RULE_SET`].
pub const REDACTION_RULE_SET_VERSION: u32 = 1;

/// The most a pattern may compile to, in bytes: `regex`'s own default,
/// stated so a principal's redaction pattern can never raise it.
const SIZE_LIMIT: usize = 10 * (1 << 20);

/// The rule set: each rule's name, which its marker carries, and its
/// pattern. Earlier rules win where two match the same text.
pub const RULE_SET: [(&str, &str); 9] = [
    (
        "private_key",
        r"-----BEGIN[A-Z ]{0,40}PRIVATE KEY-----(?s:.*?)(?:-----END[A-Z ]{0,40}PRIVATE KEY-----|\z)",
    ),
    ("aws_access_key", r"\b(?:AKIA|ASIA|ABIA|ACCA)[0-9A-Z]{16}\b"),
    (
        "github_token",
        r"\b(?:gh[pousr]_[A-Za-z0-9]{36,255}|github_pat_[A-Za-z0-9_]{22,255})\b",
    ),
    ("anthropic_api_key", r"\bsk-ant-[A-Za-z0-9_\-]{20,}"),
    (
        "openai_api_key",
        r"\bsk-(?:proj|svcacct|admin)-[A-Za-z0-9_\-]{20,}|\bsk-[A-Za-z0-9]{20}T3BlbkFJ[A-Za-z0-9]{20}\b",
    ),
    ("slack_token", r"\bxox[abposr]-[A-Za-z0-9\-]{10,}"),
    ("google_api_key", r"\bAIza[0-9A-Za-z_\-]{35}"),
    ("stripe_key", r"\b(?:sk|rk)_(?:live|test)_[0-9A-Za-z]{16,}"),
    (
        "jwt",
        r"\beyJ[A-Za-z0-9_\-]{8,}\.eyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}",
    ),
];

/// One named redaction rule: what it matches is replaced by
/// `[REDACTED:<name>]`.
#[derive(Debug, Clone)]
pub struct RedactionRule {
    name: String,
    regex: Regex,
}

impl RedactionRule {
    /// The rule `name` over `pattern`, within the size limit.
    ///
    /// # Errors
    ///
    /// Fails when `pattern` does not compile or exceeds the limit.
    pub fn new(name: &str, pattern: &str) -> Result<Self, regex::Error> {
        let regex = RegexBuilder::new(pattern)
            .size_limit(SIZE_LIMIT)
            .dfa_size_limit(SIZE_LIMIT)
            .build()?;

        Ok(Self {
            name: name.to_owned(),
            regex,
        })
    }

    /// The rule matching `literal` in any case, wherever it appears.
    ///
    /// # Errors
    ///
    /// Fails when the escaped literal exceeds the size limit.
    pub fn literal(name: &str, literal: &str) -> Result<Self, regex::Error> {
        Self::new(name, &format!("(?i){}", regex::escape(literal)))
    }

    /// Its name, which its marker carries.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The byte ranges it matches in `text`, empty matches left out.
    pub(crate) fn spans<'a>(&'a self, text: &'a str) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.regex
            .find_iter(text)
            .filter(|m| !m.is_empty())
            .map(|m| (m.start(), m.end()))
    }
}

/// The rule set, compiled.
///
/// # Errors
///
/// Fails when a rule does not compile, which its unit test rules out.
pub fn rule_set() -> Result<Vec<RedactionRule>, regex::Error> {
    RULE_SET
        .iter()
        .map(|(name, pattern)| RedactionRule::new(name, pattern))
        .collect()
}

#[cfg(test)]
mod tests;
