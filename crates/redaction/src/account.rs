//! The harness account identifiers (SEC-08): the email address and the
//! organization id a harness adds to its transcripts. They are learnt
//! from the harness's own fields, a `session_context` attachment's
//! `userEmail` and a `credential_org` attachment's `organizationUuid`,
//! and redacted wherever they appear after, so no generic email
//! pattern removes a person's address from content that merely
//! mentions one.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::rule::RedactionRule;

/// The rule name the harness account's email address is redacted under.
pub const ACCOUNT_EMAIL: &str = "account_email";
/// The rule name the harness account's organization id is redacted
/// under.
pub const ORGANIZATION_ID: &str = "organization_id";

/// The harness account identifiers learnt so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HarnessAccountIdentifiers {
    emails: BTreeSet<String>,
    organizations: BTreeSet<String>,
}

impl HarnessAccountIdentifiers {
    /// Learns the identifiers a transcript line's harness fields carry;
    /// a line that is no JSON, or carries none, teaches nothing.
    pub fn learn(&mut self, line: &str) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            return;
        };
        let attachment = &value["attachment"];
        match attachment["type"].as_str() {
            Some("session_context") => {
                if let Some(text) = attachment["context"]["userEmail"].as_str() {
                    self.emails.extend(emails_in(text));
                }
            }
            Some("credential_org") => {
                if let Some(id) = attachment["organizationUuid"].as_str()
                    && !id.is_empty()
                {
                    self.organizations.insert(id.to_owned());
                }
            }
            _ => {}
        }
    }

    /// Whether none is learnt.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.emails.is_empty() && self.organizations.is_empty()
    }

    /// One literal rule per identifier.
    ///
    /// # Errors
    ///
    /// Fails when an identifier is too long to compile.
    pub fn rules(&self) -> Result<Vec<RedactionRule>, regex::Error> {
        let emails = self.emails.iter().map(|e| (ACCOUNT_EMAIL, e));
        let organizations = self.organizations.iter().map(|o| (ORGANIZATION_ID, o));

        emails
            .chain(organizations)
            .map(|(name, id)| RedactionRule::literal(name, id))
            .collect()
    }
}

/// The email addresses in `text`: each word, its surrounding
/// punctuation trimmed, that holds one `@` with a dot after it.
fn emails_in(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| is_email(w))
        .map(str::to_owned)
}

/// Whether `word` reads as an email address.
fn is_email(word: &str) -> bool {
    let mut parts = word.split('@');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(local), Some(domain), None) => {
            !local.is_empty()
                && domain
                    .split_once('.')
                    .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
