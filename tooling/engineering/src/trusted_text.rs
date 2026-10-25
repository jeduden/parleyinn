//! SEC-07, ENG-16, I2: restore content is built from trusted text alone.
//! `TrustedText` is constructed only inside the `restore_block` crate,
//! and there only by the constructors that take qualifying pins,
//! sanitized structural fields and fixed text ParleyInn ships:
//!
//! - `TrustedText::fixed("...")`, whose argument is a string literal;
//! - `TrustedText::pin(...)`, which takes a qualifying pin;
//! - `TrustedText::field(...)`, which takes a sanitized structural field;
//! - `TrustedText::join(...)`, which joins trusted text already built.
//!
//! The check reads every Rust file of every crate under `crates/`, with
//! comments and the contents of string literals blanked, and fails,
//! naming the file and line, on a construction outside the crate and on
//! one inside it that is none of the four. A type position is no
//! construction: `fn f(t: &TrustedText)` and `-> TrustedText` are how a
//! restore builder takes and passes trusted text.

use std::fs;
use std::path::Path;

use crate::Checkout;

/// The one crate that constructs `TrustedText`, by its directory.
pub const RESTORE_BLOCK: &str = "restore_block";

/// The constructors of `TrustedText` the `restore_block` crate may use.
pub const CONSTRUCTORS: [&str; 4] = ["fixed", "pin", "field", "join"];

/// The traits whose implementation for `TrustedText` makes one from
/// something else, and the derives that do.
const BUILDING: [&str; 6] = [
    "From",
    "TryFrom",
    "FromStr",
    "Default",
    "FromIterator",
    "Deserialize",
];

/// The type's name.
const NAME: &str = "TrustedText";

/// Problems: each construction of `TrustedText` the rule refuses.
#[must_use]
pub fn constructions(c: &Checkout) -> Vec<String> {
    let crates = c.root.join("crates");
    let Ok(entries) = fs::read_dir(&crates) else {
        return vec!["list crates: cannot read the crates directory".to_owned()];
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut problems = Vec::new();
    for name in names {
        let mut files = Vec::new();
        collect(&crates.join(&name), &mut files);
        files.sort();
        for file in files {
            let shown = file
                .strip_prefix(&c.root)
                .unwrap_or(&file)
                .display()
                .to_string();
            match fs::read_to_string(&file) {
                Ok(text) => problems.extend(check_file(&shown, &text, name == RESTORE_BLOCK)),
                Err(e) => problems.push(format!("read {shown}: {e}")),
            }
        }
    }

    problems
}

/// Every Rust file under `dir`, build output left out.
fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() && path.file_name().is_some_and(|n| n != "target") {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The problems in one file, `inside` the `restore_block` crate or not.
fn check_file(file: &str, text: &str, inside: bool) -> Vec<String> {
    let code = blank(text);
    let mut problems = Vec::new();
    for (offset, _) in code.match_indices(NAME) {
        if !whole_word(&code, offset) {
            continue;
        }
        let line = code[..offset].matches('\n').count() + 1;
        let Some(how) = construction(&code, offset) else {
            continue;
        };
        let refused = if inside {
            refused_inside(&code, offset, &how)
        } else {
            Some(format!(
                "TrustedText is constructed ({how}) outside the {RESTORE_BLOCK} crate"
            ))
        };
        if let Some(why) = refused {
            problems.push(format!("{file}:{line}: {why}"));
        }
    }

    problems
}

/// Whether the name at `offset` stands alone as an identifier.
fn whole_word(code: &str, offset: usize) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let before = code[..offset].chars().next_back();
    let after = code[offset + NAME.len()..].chars().next();

    !before.is_some_and(word) && !after.is_some_and(word)
}

/// How the mention of the name at `offset` constructs a value, if it
/// does: the form it takes.
fn construction(code: &str, offset: usize) -> Option<String> {
    let rest = code[offset + NAME.len()..].trim_start();
    let before = code[..offset].trim_end();
    let previous = before
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default();
    // After these the name is declared or used as a type, not built.
    let naming = ["->", "struct", "enum", "dyn", "trait", "use", "for"].contains(&previous)
        || previous.starts_with("impl");
    if let Some(path) = rest.strip_prefix("::") {
        let called: String = path
            .trim_start()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        return Some(format!("TrustedText::{called}"));
    }
    if !naming && rest.starts_with('(') {
        return Some("TrustedText(..)".to_owned());
    }
    if !naming && rest.starts_with('{') {
        return Some("a struct literal".to_owned());
    }
    if previous == "for" && builds_by_trait(before) {
        return Some("a trait that builds one".to_owned());
    }
    if previous == "struct" && derives_building(before) {
        return Some("a derive that builds one".to_owned());
    }

    None
}

/// Whether the `impl` the text before `for` ends is one of the traits
/// that build a value from something else.
fn builds_by_trait(before: &str) -> bool {
    let statement = before.rsplit([';', '}', '{']).next().unwrap_or_default();
    statement.contains("impl") && BUILDING.iter().any(|t| has_word(statement, t))
}

/// Whether the attributes just before `struct` derive a building trait.
fn derives_building(before: &str) -> bool {
    let attributes = before
        .rsplit(['}', ';'])
        .next()
        .unwrap_or_default()
        .rsplit_once("struct")
        .map_or("", |(attrs, _)| attrs);

    attributes.contains("derive") && BUILDING.iter().any(|t| has_word(attributes, t))
}

/// Whether `word` occurs in `text` as a whole word.
fn has_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(at, _)| {
        let id = |c: char| c.is_alphanumeric() || c == '_';
        !text[..at].chars().next_back().is_some_and(id)
            && !text[at + word.len()..].chars().next().is_some_and(id)
    })
}

/// Why a construction inside the `restore_block` crate is refused, if it
/// is: it is none of the four constructors, or `fixed` takes more than a
/// string literal.
fn refused_inside(code: &str, offset: usize, how: &str) -> Option<String> {
    let constructor = how.strip_prefix("TrustedText::").unwrap_or_default();
    if !CONSTRUCTORS.contains(&constructor) {
        return Some(format!(
            "the {RESTORE_BLOCK} crate builds TrustedText by {how}, not from qualifying pins, sanitized structural fields or fixed text ParleyInn ships"
        ));
    }
    if constructor == "fixed" {
        let after = &code[offset + NAME.len()..];
        let argument = after
            .trim_start()
            .strip_prefix("::")
            .map(str::trim_start)
            .and_then(|p| p.strip_prefix("fixed"))
            .map(str::trim_start)
            .and_then(|p| p.strip_prefix('('))
            .map(str::trim_start)
            .unwrap_or_default();
        let literal =
            argument.starts_with('"') || argument.starts_with("r\"") || argument.starts_with("r#");
        if !(literal || names_a_constant(argument)) {
            return Some(format!(
                "the {RESTORE_BLOCK} crate builds TrustedText::fixed from something but a string literal of fixed text ParleyInn ships"
            ));
        }
    }

    None
}

/// Whether `argument`, the text after a call's opening parenthesis,
/// starts with the name of a constant and ends the call: an identifier
/// in capitals, digits and underscores.
fn names_a_constant(argument: &str) -> bool {
    let name: String = argument
        .chars()
        .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
        .collect();

    name.chars().any(|c| c.is_ascii_uppercase())
        && argument[name.len()..].trim_start().starts_with(')')
}

/// `text` with every comment and the contents of every string and
/// character literal replaced by spaces, line breaks kept, so a match
/// in what it returns is code, at the line it has in `text`.
#[must_use]
pub fn blank(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let rest = &chars[i..];
        if rest.starts_with(&['/', '/']) {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if rest.starts_with(&['/', '*']) {
            i = blank_block_comment(&chars, i, &mut out);
        } else if let Some(hashes) = raw_string_start(rest) {
            i = blank_raw_string(&chars, i, hashes, &mut out);
        } else if chars[i] == '"' {
            i = blank_string(&chars, i, &mut out);
        } else if let Some(len) = char_literal_len(rest) {
            out.push('\'');
            out.extend(std::iter::repeat_n(' ', len - 2));
            out.push('\'');
            i += len;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}

/// Blanks a block comment starting at `i`, nested ones too; returns the
/// index after it.
fn blank_block_comment(chars: &[char], mut i: usize, out: &mut String) -> usize {
    let mut depth = 0;
    while i < chars.len() {
        if chars[i..].starts_with(&['/', '*']) {
            depth += 1;
            out.push_str("  ");
            i += 2;
        } else if chars[i..].starts_with(&['*', '/']) {
            depth -= 1;
            out.push_str("  ");
            i += 2;
            if depth == 0 {
                break;
            }
        } else {
            out.push(if chars[i] == '\n' { '\n' } else { ' ' });
            i += 1;
        }
    }

    i
}

/// The number of hashes of a raw string literal that `rest` starts, if
/// it starts one.
fn raw_string_start(rest: &[char]) -> Option<usize> {
    if rest.first() != Some(&'r') {
        return None;
    }
    let hashes = rest[1..].iter().take_while(|c| **c == '#').count();

    (rest.get(1 + hashes) == Some(&'"')).then_some(hashes)
}

/// Blanks a raw string starting at `i`, keeping its delimiters; returns
/// the index after it.
fn blank_raw_string(chars: &[char], mut i: usize, hashes: usize, out: &mut String) -> usize {
    let open = 2 + hashes;
    out.extend(&chars[i..i + open]);
    i += open;
    let close: Vec<char> = std::iter::once('"')
        .chain(std::iter::repeat_n('#', hashes))
        .collect();
    while i < chars.len() {
        if chars[i..].starts_with(&close) {
            out.extend(&close);
            return i + close.len();
        }
        out.push(if chars[i] == '\n' { '\n' } else { ' ' });
        i += 1;
    }

    i
}

/// Blanks a string literal starting at `i`, keeping its quotes; returns
/// the index after it.
fn blank_string(chars: &[char], mut i: usize, out: &mut String) -> usize {
    out.push('"');
    i += 1;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                out.push(' ');
                i += 1;
                if i < chars.len() {
                    out.push(if chars[i] == '\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            '"' => {
                out.push('"');
                return i + 1;
            }
            c => {
                out.push(if c == '\n' { '\n' } else { ' ' });
                i += 1;
            }
        }
    }

    i
}

/// The length of the character literal `rest` starts, if it starts one
/// and not a lifetime.
fn char_literal_len(rest: &[char]) -> Option<usize> {
    if rest.first() != Some(&'\'') {
        return None;
    }
    match rest.get(1)? {
        '\\' => {
            let end = rest.iter().skip(2).position(|c| *c == '\'')?;
            Some(end + 3)
        }
        _ if rest.get(2) == Some(&'\'') => Some(3),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
