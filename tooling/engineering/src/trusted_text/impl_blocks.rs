//! The impl blocks of `TrustedText`: the traits, methods and fields the rule holds to account.

use super::*;

/// A problem at its offset in the code.
pub(crate) type Found = (usize, String);

/// What the `impl` blocks for `TrustedText` in `code` do that the rule
/// refuses, each at its offset, and the braces of every such block: an
/// implementation of a trait off the allow-list, a method that changes
/// one or reads its field where it could change it, and a value built
/// through `Self` anywhere but the four constructors of the inherent
/// `impl`.
pub(crate) fn impl_problems(code: &str) -> (Vec<Found>, Vec<(usize, usize)>) {
    let (mut out, mut blocks) = (Vec::new(), Vec::new());
    for (at, _) in code.match_indices("impl") {
        if !word_at(code, at, "impl") {
            continue;
        }
        let Some(open) = code[at..].find(['{', ';']).map(|i| at + i) else {
            continue;
        };
        if code[open..].starts_with(';') {
            continue;
        }
        let (implemented, target) = impl_header(&code[at + "impl".len()..open]);
        if last_segment(target) != NAME {
            continue;
        }
        let close = matching(code, open);
        blocks.push((open, close));
        if let Some(name) = implemented
            .filter(|t| !trait_allowed(t, &code[open..close]))
            .map(last_segment)
            .filter(|name| !BUILDING.contains(name))
        {
            out.push((
                at,
                format!("TrustedText implements the trait {name}, but only {ALLOWED_TRAITS_SHOWN} may be, so none adds text to it"),
            ));
        }
        for item in methods(code, open, close) {
            out.extend(method_problems(code, &item, implemented.is_none()));
        }
    }

    (out, blocks)
}

/// Whether `TrustedText` may implement the trait `implemented`, whose
/// `impl` block is `block`.
pub(crate) fn trait_allowed(implemented: &str, block: &str) -> bool {
    let name = last_segment(implemented);
    let flat: String = block.split_whitespace().collect();
    match name {
        "AsRef" => {
            let arguments: String = implemented.split_whitespace().collect();
            arguments.ends_with("AsRef<str>")
        }
        "Deref" => flat.contains("typeTarget=str;"),
        _ => ALLOWED_TRAITS.contains(&name),
    }
}

/// What the `restore_block` crate does outside the `impl` blocks of
/// `TrustedText`, whose braces `blocks` holds, that could change one: a
/// tuple field read, or a `ref mut` binding.
pub(crate) fn outside_blocks(code: &str, blocks: &[(usize, usize)]) -> Vec<(usize, String)> {
    let outside = |at: usize| {
        !blocks
            .iter()
            .any(|(open, close)| (*open..*close).contains(&at))
    };
    let mut out: Vec<(usize, String)> = tuple_fields(code, 0, code.len())
        .into_iter()
        .filter(|at| outside(*at))
        .map(|at| (at, format!("the {RESTORE_BLOCK} crate reads a field outside TrustedText's impl, where nothing proves it cannot change trusted text")))
        .collect();
    out.extend(
        ref_muts(code, 0, code.len())
            .into_iter()
            .filter(|at| outside(*at))
            .map(|at| (at, format!("the {RESTORE_BLOCK} crate makes a ref mut binding outside TrustedText's impl, which could change trusted text"))),
    );

    out
}

/// The offsets of every tuple field read, such as `t.0`, in
/// `code[from..to]`; a number such as `1.0` is none.
pub(crate) fn tuple_fields(code: &str, from: usize, to: usize) -> Vec<usize> {
    code[from..to]
        .match_indices('.')
        .map(|(at, _)| from + at)
        .filter(|at| {
            let digit = code[at + 1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit());
            let before = &code[..*at];
            let token = before
                .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()
                .unwrap_or_default();
            let after_value = before.ends_with([')', ']'])
                || (!token.is_empty() && !token.starts_with(|c: char| c.is_ascii_digit()));
            digit && after_value
        })
        .collect()
}

/// The offsets of every `ref mut` in `code[from..to]`.
pub(crate) fn ref_muts(code: &str, from: usize, to: usize) -> Vec<usize> {
    code[from..to]
        .match_indices("ref")
        .map(|(at, _)| from + at)
        .filter(|at| word_at(code, *at, "ref") && code[at + 3..].trim_start().starts_with("mut"))
        .collect()
}

/// Why a change after building is refused.
pub(crate) const CHANGED: &str =
    "though trusted text is built whole by its four constructors and never added to";

/// Whether `word` stands alone at `at` in `code`.
pub(crate) fn word_at(code: &str, at: usize, word: &str) -> bool {
    let id = |c: char| c.is_alphanumeric() || c == '_';
    !code[..at].chars().next_back().is_some_and(id)
        && !code[at + word.len()..].chars().next().is_some_and(id)
}

/// The trait an `impl` header implements, if it names one, and the type
/// it implements it for; the header is the text between `impl` and the
/// block.
pub(crate) fn impl_header(header: &str) -> (Option<&str>, &str) {
    let header = skip_generics(header.trim_start());
    let mut depth = 0i32;
    for (i, c) in header.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            _ if depth == 0 && header[i..].starts_with("for") && word_at(header, i, "for") => {
                return (Some(header[..i].trim()), header[i + 3..].trim());
            }
            _ => {}
        }
    }

    (None, header.trim())
}

/// `text` past the generic parameters it starts with, if any.
pub(crate) fn skip_generics(text: &str) -> &str {
    if !text.starts_with('<') {
        return text;
    }
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return &text[i + 1..];
                }
            }
            _ => {}
        }
    }

    ""
}

/// The last segment of the path `path` names, its generic arguments and
/// a `where` clause left off.
pub(crate) fn last_segment(path: &str) -> &str {
    let path = path.split(['<', ' ', '\n']).next().unwrap_or_default();
    path.rsplit("::").next().unwrap_or_default().trim()
}

/// The offset of the brace that closes the one at `open`, or the end.
pub(crate) fn matching(code: &str, open: usize) -> usize {
    let mut depth = 0;
    for (i, c) in code[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + i;
                }
            }
            _ => {}
        }
    }

    code.len()
}

/// How a method takes its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Receiver {
    /// No `self`: an associated function.
    None,
    /// `&self`: it can read, never change.
    Shared,
    /// `self` by value, not `mut`.
    Value,
    /// `&mut self`, `mut self` or `self: &mut ..`.
    Mut,
}

/// One method of an `impl` block.
pub(crate) struct Method {
    at: usize,
    name: String,
    receiver: Receiver,
    /// Where its parameters begin.
    params: usize,
    /// Its body's braces, when it has one.
    body: Option<(usize, usize)>,
}

/// The methods the `impl` block whose braces are at `open` and `close`
/// holds, at its top level.
pub(crate) fn methods(code: &str, open: usize, close: usize) -> Vec<Method> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut i = open;
    while i < close {
        let c = code[i..].chars().next().unwrap_or(' ');
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            'f' if depth == 1 && code[i..].starts_with("fn") && word_at(code, i, "fn") => {
                let method = method_at(code, i, close);
                // Past the body's closing brace, which closes no level
                // of the impl block.
                i = method.body.map_or(i + 2, |(_, end)| end + 1);
                out.push(method);
                continue;
            }
            _ => {}
        }
        i += c.len_utf8();
    }

    out
}

/// The method whose `fn` is at `at`, its block ending by `close`.
pub(crate) fn method_at(code: &str, at: usize, close: usize) -> Method {
    let rest = code[at + 2..close].trim_start();
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let params = code[at..close].find('(').map_or(close, |p| at + p);
    let end = code[params..close].find(')').map_or(close, |e| params + e);
    let first: String = code[(params + 1).min(end)..end]
        .split(',')
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let body = code[at..close]
        .find(['{', ';'])
        .map(|b| at + b)
        .filter(|b| code[*b..].starts_with('{'))
        .map(|b| (b, matching(code, b)));

    Method {
        at,
        name,
        receiver: receiver_of(&first),
        params,
        body,
    }
}

/// How the first parameter `first` takes the value.
pub(crate) fn receiver_of(first: &str) -> Receiver {
    let binding = first.split(':').next().unwrap_or_default();
    if !has_word(binding, "self") {
        return Receiver::None;
    }
    if has_word(first, "mut") {
        Receiver::Mut
    } else if first.starts_with('&') {
        Receiver::Shared
    } else {
        Receiver::Value
    }
}

/// What the rule refuses in `method`, of the inherent `impl` or not.
pub(crate) fn method_problems(code: &str, method: &Method, inherent: bool) -> Vec<(usize, String)> {
    let name = &method.name;
    if method.receiver == Receiver::Mut {
        return vec![(
            method.at,
            format!("TrustedText is changed by the method {name}, which takes mut self, {CHANGED}"),
        )];
    }
    let mut out = Vec::new();
    if inherent && CHANGING_METHODS.contains(&name.as_str()) {
        out.push((
            method.at,
            format!("TrustedText is changed by the method {name}, {CHANGED}"),
        ));
    }
    let end = method.body.map_or(method.params, |(_, close)| close);
    out.extend(
        mut_self_refs(code, method.params, end)
            .into_iter()
            .map(|at| {
                (
                    at,
                    format!("TrustedText is changed by a &mut Self, {CHANGED}"),
                )
            }),
    );
    let Some((open, close)) = method.body else {
        return out;
    };
    let constructor = inherent && CONSTRUCTORS.contains(&name.as_str());
    if !constructor {
        out.extend(body_changes(code, method, open, close));
    }
    out.extend(self_builds(code, name, constructor, open, close));

    out
}

/// The offsets in `code[from..to]` of each `&mut Self`.
pub(crate) fn mut_self_refs(code: &str, from: usize, to: usize) -> Vec<usize> {
    code[from..to]
        .match_indices("Self")
        .map(|(at, _)| from + at)
        .filter(|at| word_at(code, *at, "Self") && mutable_borrow(code[..*at].trim_end()))
        .collect()
}

/// Whether `before` ends in a mutable borrow: `&mut`, `& mut` or `&'a mut`.
pub(crate) fn mutable_borrow(before: &str) -> bool {
    before
        .strip_suffix("mut")
        .map(str::trim_end)
        .map(strip_lifetime)
        .is_some_and(|b| b.ends_with('&'))
}

/// What a method's body, from `open` to `close`, does that could change
/// the value, outside the four constructors: a `mut` binding, and a
/// field read in a method that takes no `&self`.
pub(crate) fn body_changes(
    code: &str,
    method: &Method,
    open: usize,
    close: usize,
) -> Vec<(usize, String)> {
    let name = &method.name;
    let mut out: Vec<(usize, String)> = code[open..close]
        .match_indices("mut")
        .map(|(at, _)| open + at)
        .filter(|at| word_at(code, *at, "mut") && !mutable_borrow(code[..at + 3].trim_end()))
        .map(|at| {
            (
                at,
                format!("TrustedText is changed by a mut binding in {name}, {CHANGED}"),
            )
        })
        .collect();
    if method.receiver != Receiver::Shared {
        out.extend(tuple_fields(code, open, close).into_iter().map(|at| {
            (at, format!("TrustedText's impl reads its field in {name}, which takes no &self, though only the four constructors and methods taking &self may"))
        }));
    }

    out
}

/// The values `Self` builds in a body from `open` to `close` that the
/// rule refuses: any outside a `constructor`, and a call through
/// `Self::` to anything but the four.
pub(crate) fn self_builds(
    code: &str,
    name: &str,
    constructor: bool,
    open: usize,
    close: usize,
) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (at, _) in code[open..close].match_indices("Self") {
        let at = open + at;
        if !word_at(code, at, "Self") {
            continue;
        }
        let before = code[..at].trim_end();
        if before.ends_with("let") && word_at(code, before.len() - 3, "let") {
            continue;
        }
        let rest = code[at + 4..].trim_start();
        let how = if rest.starts_with('(') {
            format!("Self(..) in {name}")
        } else if rest.starts_with('{') {
            format!("Self {{ .. }} in {name}")
        } else if let Some(path) = rest.strip_prefix("::") {
            let called: String = path
                .trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let after = path.trim_start()[called.len()..].trim_start();
            if !after.starts_with('(') {
                continue;
            }
            if CONSTRUCTORS.contains(&called.as_str()) {
                if called == "fixed" && !fixed_argument_ok(after) {
                    out.push((at, fixed_refusal()));
                }
                continue;
            }
            format!("Self::{called}")
        } else {
            continue;
        };
        if !(constructor && !how.starts_with("Self::")) {
            out.push((at, building_refusal(&how)));
        }
    }

    out
}
