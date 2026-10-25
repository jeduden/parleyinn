//! ENG-07, SEC-08: the redactor on hostile transcript lines. The input
//! is redacted as a line and as text, after the harness account
//! identifiers are learnt from it; the redactor must never panic, and
//! no AWS access key may survive a line's redaction.
#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use redaction::Redactor;
use redaction::account::HarnessAccountIdentifiers;

/// The rule set and one principal's pattern, compiled once per process.
fn base() -> Option<&'static Redactor> {
    static BASE: OnceLock<Option<Redactor>> = OnceLock::new();
    BASE.get_or_init(|| {
        Redactor::new(&["ACME-[0-9]{8}".to_owned()])
            .ok()
            .map(|(r, _)| r)
    })
    .as_ref()
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Some(base) = base() else {
        return;
    };
    let mut ids = HarnessAccountIdentifiers::default();
    ids.learn(text);
    let Ok(redactor) = base.with_identifiers(&ids) else {
        return;
    };
    let line = redactor.redact_line(text);
    let again = redactor.redact_text(&line.text);
    assert!(
        again.rules.iter().all(|r| r != "aws_access_key"),
        "an AWS access key survived redaction: {:?}",
        line.text
    );
    let _ = redactor.redact_text(text);
});
