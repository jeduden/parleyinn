//! The conservative model-token estimator of §8.4, calibrated by spike
//! S8 on 646 texts against Claude's own counts (`testdata/model-tokens`).
//! The pin budget, the restore block limit and the output caps of
//! RCL-03 and CMP-06 count this estimate, not the true count.
//!
//! One pass sorts each character into a class and adds that class's
//! weight, in 1/1024 of a model token, and adds the switch weight each
//! time a run of letters and digits switches between lowercase,
//! uppercase and digit, which random identifiers do often and prose
//! seldom. The estimate is the sum rounded up, plus a constant for the
//! tokens that join a text to what surrounds it. Integer arithmetic
//! only: the same text always gives the same estimate, on every
//! platform (I10), and adding characters never lowers it.

/// Character classes, in the order of [`WEIGHTS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// ASCII letters.
    Letter,
    /// ASCII digits.
    Digit,
    /// The ASCII space.
    Space,
    /// Every other ASCII whitespace or control character.
    Break,
    /// ASCII punctuation and symbols.
    Punct,
    /// U+0080 to U+02FF: letters with diacritics.
    Latin,
    /// U+0300 to U+07FF: Greek, Cyrillic, Armenian, Hebrew, Arabic.
    Two,
    /// Three-byte UTF-8 outside the classes below: Indic scripts, Thai.
    Three,
    /// U+2000 to U+2BFF: punctuation, symbols, arrows, math, box lines.
    Symbol,
    /// CJK ideographs, kana, Hangul and fullwidth forms.
    Cjk,
    /// Four-byte UTF-8: emoji, mathematical alphanumerics, rare CJK.
    Four,
}

/// How many classes there are.
pub const CLASSES: usize = 11;

/// Each class's weight in 1/1024 of a model token, fitted on the
/// committed sample and raised by 10%.
pub const WEIGHTS: [u64; CLASSES] = [561, 965, 110, 213, 1143, 2160, 850, 967, 4462, 1577, 4166];

/// The weight of a switch between lowercase, uppercase and digit.
pub const SWITCH: u64 = 861;

/// Model tokens added to every estimate.
pub const BASE: u64 = 6;

/// Sorts one character into its class.
#[must_use]
pub fn class(c: char) -> Class {
    match c {
        'a'..='z' | 'A'..='Z' => Class::Letter,
        '0'..='9' => Class::Digit,
        ' ' => Class::Space,
        '\0'..='\x1f' | '\x7f' => Class::Break,
        '!'..='~' => Class::Punct,
        '\u{80}'..='\u{2ff}' => Class::Latin,
        '\u{300}'..='\u{7ff}' => Class::Two,
        '\u{2000}'..='\u{2bff}' => Class::Symbol,
        '\u{2e80}'..='\u{9fff}'
        | '\u{ac00}'..='\u{d7af}'
        | '\u{f900}'..='\u{faff}'
        | '\u{ff00}'..='\u{ffef}' => Class::Cjk,
        '\u{800}'..='\u{ffff}' => Class::Three,
        _ => Class::Four,
    }
}

/// Lowercase 1, uppercase 2, digit 3, anything else 0.
fn kind(c: char) -> u8 {
    match c {
        'a'..='z' => 1,
        'A'..='Z' => 2,
        '0'..='9' => 3,
        _ => 0,
    }
}

/// How many characters of each class the text holds, and how many
/// switches between lowercase, uppercase and digit.
#[must_use]
pub fn counts(text: &str) -> ([u64; CLASSES], u64) {
    let mut n = [0u64; CLASSES];
    let mut switches = 0;
    let mut previous = 0;
    for c in text.chars() {
        n[class(c) as usize] += 1;
        let k = kind(c);
        if k != 0 && previous != 0 && k != previous {
            switches += 1;
        }
        previous = k;
    }

    (n, switches)
}

/// The estimate from the counts.
#[must_use]
pub fn from_counts(n: &[u64; CLASSES], switches: u64) -> u64 {
    let weighed: u64 = n.iter().zip(&WEIGHTS).map(|(n, w)| n * w).sum();

    (weighed + switches * SWITCH).div_ceil(1024) + BASE
}

/// The conservative model-token estimate of a text.
#[must_use]
pub fn estimate(text: &str) -> u64 {
    let (n, switches) = counts(text);

    from_counts(&n, switches)
}

#[cfg(test)]
mod tests;
