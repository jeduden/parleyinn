//! ENG-07, REC-13: the transcript parser on hostile transcripts. Each
//! input is a transcript: every line is checked against the recorded
//! shapes and classed, as ingest and the MCP server read it, in a
//! session whose ancestry is observed so the `user` path runs too. The
//! parser must never panic, and every line gets exactly one outcome.
#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use provenance::{Classified, Classifier, Session};
use redaction::Redactor;
use transcript::shape::Shapes;

/// The supported shapes and the rule set, built once per process.
fn parts() -> Option<&'static (Shapes, Redactor)> {
    static PARTS: OnceLock<Option<(Shapes, Redactor)>> = OnceLock::new();
    PARTS
        .get_or_init(|| {
            let (redactor, _) = Redactor::new(&[]).ok()?;
            Some((Shapes::supported(), redactor))
        })
        .as_ref()
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Some((shapes, redactor)) = parts() else {
        return;
    };
    let session = Session {
        ancestry_observed: true,
        nested: false,
    };
    let mut classifier = Classifier::new(shapes, session, redactor);
    for line in text.split('\n') {
        match classifier.line(line) {
            Classified::Unparsed(_) | Classified::Line(_, _) => {}
        }
    }
});
