//! ENG-07, SEC-04: the query compiler on hostile caller text. The input
//! is split at U+0001 into the typed text, the `any_of` terms, the
//! `all_of` terms and the phrase. The compiler must never panic, and
//! what it accepts must be the literal terms it was given: at most 16
//! of them, the expression made of quoted phrases and the operators
//! AND, OR and parentheses alone, every quote balanced, so no caller
//! text runs as FTS5 syntax.
#![no_main]

use libfuzzer_sys::fuzz_target;
use search::{MAX_TERMS, Request, compile};

/// What `expression` holds outside its quoted phrases, and whether its
/// quotes balance. A quote inside a phrase is written twice.
fn outside_quotes(expression: &str) -> (String, bool) {
    let mut outside = String::new();
    let mut quoted = false;
    let mut chars = expression.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (c, false) => outside.push(c),
            (_, true) => {}
        }
    }

    (outside, !quoted)
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let mut parts = text.split('\u{1}');
    let typed = parts.next().unwrap_or_default();
    let list = |part: Option<&str>| -> Vec<String> {
        part.map(|p| p.split('\u{2}').map(str::to_owned).collect())
            .unwrap_or_default()
    };
    let any_of = list(parts.next());
    let all_of = list(parts.next());
    let phrase = parts.next();
    let request = Request {
        text: typed,
        any_of: &any_of,
        all_of: &all_of,
        phrase,
    };
    let Ok(compiled) = compile(&request) else {
        return;
    };

    assert!(compiled.terms() <= MAX_TERMS);
    let (outside, balanced) = outside_quotes(compiled.expression());
    assert!(balanced, "unbalanced quotes: {:?}", compiled.expression());
    let operators = outside
        .replace("AND", "")
        .replace("OR", "")
        .replace(['(', ')'], "");
    assert!(
        operators.trim().is_empty(),
        "text outside a quoted phrase: {:?}",
        compiled.expression()
    );
});
