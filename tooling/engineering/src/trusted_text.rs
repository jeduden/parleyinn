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
//!
//! Inside an `impl` block for the type, `Self(..)` and `Self { .. }`
//! build one only in the bodies of those four constructors of the
//! inherent `impl`, and `Self::name(..)` is held to what
//! `TrustedText::name(..)` would be. A `let` pattern takes one apart and
//! builds nothing.
//!
//! Nothing changes trusted text once built, and these rules fail closed,
//! inside the crate and out:
//!
//! - The only traits implemented for it are those of
//!   [`ALLOWED_TRAITS_SHOWN`]: `Clone`, `Debug`, `Display`, `PartialEq`,
//!   `Eq`, `Hash`, `PartialOrd`, `Ord`, `AsRef<str>` and
//!   `Deref<Target = str>`. Any other, a custom trait included, is
//!   refused.
//! - No method of any `impl` for it takes `&mut self`, `mut self` or
//!   `self: &mut ..`, and no inherent method is named for adding in
//!   place (`push_str`, `append` ...).
//! - `&mut TrustedText`, a `mut` binding of type `TrustedText`, and
//!   `&mut Self` in its `impl` blocks are refused wherever they stand.
//! - In its `impl` blocks, outside the four constructors, no `mut` or
//!   `ref mut` binding is made, and its field is read only by a method
//!   taking `&self`.
//! - Inside the crate, no tuple field is read and no `ref mut` binding
//!   is made outside its `impl` blocks, and the type is a tuple struct
//!   of one private field holding immutable text: `String`, `Box<str>`,
//!   `Arc<str>`, `Rc<str>`, `Cow<'static, str>` or `&'static str`.
//!
//! Three forms hide a construction behind another name and are refused
//! wherever they stand: `use .. TrustedText as X`, `type X = TrustedText`
//! and the qualified path `<TrustedText>::name`. This is a text check,
//! not the type system: it does not stop a construction written through
//! a macro or generated code, and it makes no claim to.

use std::fs;
use std::path::Path;

use crate::Checkout;
use impl_blocks::*;

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

/// The traits `TrustedText` may implement: none of them can add text to
/// it. `AsRef` and `Deref` are allowed only to `str`.
const ALLOWED_TRAITS: [&str; 10] = [
    "Clone",
    "Debug",
    "Display",
    "PartialEq",
    "Eq",
    "Hash",
    "PartialOrd",
    "Ord",
    "AsRef",
    "Deref",
];

/// The allowed traits, as a refusal names them.
pub const ALLOWED_TRAITS_SHOWN: &str = "Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord, AsRef<str> and Deref<Target = str>";

/// The types the one field of `TrustedText` may have: text that cannot
/// change in place.
const FIELD_TYPES: [&str; 6] = [
    "String",
    "Box<str>",
    "Arc<str>",
    "Rc<str>",
    "Cow<'static,str>",
    "&'staticstr",
];

/// The refusal of any other shape.
const SHAPE: &str = "TrustedText must be a tuple struct of one private String, Box<str>, Arc<str>, Rc<str>, Cow<'static, str> or &'static str";

/// The methods whose names say they add to a value in place.
const CHANGING_METHODS: [&str; 6] = [
    "push",
    "push_str",
    "append",
    "extend",
    "insert",
    "write_str",
];

/// The type's name.
const NAME: &str = "TrustedText";

/// The forms that name the type by another route, whose use then hides
/// how a value is built.
const ALIASES: [&str; 3] = ["a `use .. as` alias", "a type alias", "a qualified path"];

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
        if let Some(form) = mutable_mention(&code, offset) {
            problems.push(format!(
                "{file}:{line}: TrustedText is changed by {form}, {CHANGED}"
            ));
            continue;
        }
        if inside && shape_refused(&code, offset) {
            problems.push(format!("{file}:{line}: {SHAPE}"));
            continue;
        }
        let Some(how) = construction(&code, offset) else {
            continue;
        };
        let refused = if inside {
            refused_inside(&code, offset, &how)
        } else if ALIASES.contains(&how.as_str()) {
            Some(format!(
                "TrustedText is named by {how}, which hides how it is built, outside the {RESTORE_BLOCK} crate"
            ))
        } else {
            Some(format!(
                "TrustedText is constructed ({how}) outside the {RESTORE_BLOCK} crate"
            ))
        };
        if let Some(why) = refused {
            problems.push(format!("{file}:{line}: {why}"));
        }
    }
    let (mut found, blocks) = impl_problems(&code);
    if inside {
        found.extend(outside_blocks(&code, &blocks));
    }
    found.sort_by_key(|(offset, _)| *offset);
    // Problems by offset, after those the mentions found.
    for (offset, why) in found {
        let line = code[..offset].matches('\n').count() + 1;
        problems.push(format!("{file}:{line}: {why}"));
    }

    problems
}

/// The form a mention of the name at `offset` takes that lets a value
/// change, if it does: `&mut TrustedText`, or a `mut` binding of it. A
/// receiver `self: &mut TrustedText` is its method's to refuse.
fn mutable_mention(code: &str, offset: usize) -> Option<&'static str> {
    let before = unpath(code[..offset].trim_end());
    if let Some(borrow) = before.strip_suffix("mut").filter(|b| {
        !b.chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    }) {
        let borrow = borrow.trim_end();
        let borrow = strip_lifetime(borrow);
        if let Some(binding) = borrow.strip_suffix('&') {
            return (!binding.trim_end().ends_with("self:")).then_some("a &mut TrustedText");
        }
        // Any other `mut` before the name, a `*mut` pointer say, fails
        // closed too.
        return Some("a mut TrustedText");
    }
    let binding = before.strip_suffix(':')?.trim_end();
    let name_start = binding
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let mutable = binding[..name_start].trim_end();
    (mutable.ends_with("mut") && word_at(mutable, mutable.len() - 3, "mut"))
        .then_some("a mut binding of TrustedText")
}

/// `before`, the code before a mention of the name, without the path
/// that qualifies it, such as `restore_block::`.
fn unpath(before: &str) -> &str {
    if before.ends_with("::") {
        before
            .trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == ':')
            .trim_end()
    } else {
        before
    }
}

/// `text` without the lifetime it ends in, such as `'a`.
fn strip_lifetime(text: &str) -> &str {
    let name = text.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_');
    name.strip_suffix('\'').map_or(text, str::trim_end)
}

/// Whether the mention of the name at `offset` defines the type in any
/// shape but a tuple struct of one private field of immutable text.
fn shape_refused(code: &str, offset: usize) -> bool {
    let before = code[..offset].trim_end();
    if !(before.ends_with("struct") && word_at(before, before.len() - 6, "struct")) {
        return false;
    }
    let Some(fields) = code[offset + NAME.len()..].trim_start().strip_prefix('(') else {
        return true;
    };
    let field: String = fields
        .split(')')
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    let field = match field.rsplit_once("::") {
        Some((_, last)) if !field.starts_with('&') => last.to_owned(),
        _ => field,
    };

    !FIELD_TYPES.contains(&field.as_str())
}

/// The refusal of a construction inside the crate by `how`.
fn building_refusal(how: &str) -> String {
    format!(
        "the {RESTORE_BLOCK} crate builds TrustedText by {how}, not from qualifying pins, sanitized structural fields or fixed text ParleyInn ships"
    )
}

/// The refusal of `fixed` taking something but fixed text.
fn fixed_refusal() -> String {
    format!(
        "the {RESTORE_BLOCK} crate builds TrustedText::fixed from something but a string literal of fixed text ParleyInn ships"
    )
}

/// Whether `call`, the text from `fixed`'s opening parenthesis, passes
/// a string literal or a constant.
fn fixed_argument_ok(call: &str) -> bool {
    let argument = call
        .strip_prefix('(')
        .map(str::trim_start)
        .unwrap_or_default();
    let literal =
        argument.starts_with('"') || argument.starts_with("r\"") || argument.starts_with("r#");

    literal || names_a_constant(argument)
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
    if let Some(how) = aliasing(before, rest) {
        return Some(how.to_owned());
    }
    // A path such as `restore_block::TrustedText` reads as its name: the
    // word before the path is the one that says how the name is used.
    let unpathed = if before.ends_with("::") {
        before.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == ':')
    } else {
        before
    };
    let previous = unpathed
        .trim_end()
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default();
    // A `let` pattern takes one apart; it builds nothing.
    if previous == "let" {
        return None;
    }
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

/// The alias form the mention of the name takes, if it does: a path
/// qualified by the type, `use .. as`, or `type X = ..`. `before` is the
/// code before the name, `rest` the code after it.
fn aliasing(before: &str, rest: &str) -> Option<&'static str> {
    let qualified = before.ends_with('<')
        && !before[..before.len() - 1]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':')
        && (rest.starts_with("as ") || rest.strip_prefix('>').is_some_and(|r| r.starts_with("::")));
    if qualified {
        return Some(ALIASES[2]);
    }
    if rest.starts_with("as ") && !before.ends_with('<') {
        return Some(ALIASES[0]);
    }
    let statement = before.rsplit([';', '{', '}']).next().unwrap_or_default();
    let named = statement
        .rsplit_once('=')
        .map(|(_, path)| path.trim())
        .filter(|path| {
            path.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
        });
    (has_word(statement, "type") && named.is_some()).then_some(ALIASES[1])
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
    if ALIASES.contains(&how) {
        return Some(format!(
            "the {RESTORE_BLOCK} crate names TrustedText by {how}, which hides how it is built"
        ));
    }
    let constructor = how.strip_prefix("TrustedText::").unwrap_or_default();
    if !CONSTRUCTORS.contains(&constructor) {
        return Some(building_refusal(how));
    }
    if constructor == "fixed" {
        let call = code[offset + NAME.len()..]
            .trim_start()
            .strip_prefix("::")
            .map(str::trim_start)
            .and_then(|p| p.strip_prefix("fixed"))
            .map(str::trim_start)
            .unwrap_or_default();
        if !fixed_argument_ok(call) {
            return Some(fixed_refusal());
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

mod impl_blocks;
#[cfg(test)]
mod tests;
