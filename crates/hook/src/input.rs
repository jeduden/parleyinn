//! The hook input decoder (SEC-05, SEC-16, REC-26). A hook input is
//! held to 1 MiB and depth 64 before it is parsed, must be one JSON
//! object, and each field a handler uses must pass the schema: text,
//! an id a plain id, and `hook_event_name` the hook that runs. Only then
//! are its keys checked against the recorded hook input shapes, where a
//! difference is format drift: kept, counted and audited by the caller,
//! the handler still recording what it can. Nothing here reads a file,
//! a clock or randomness.

use serde_json::{Map, Value};
use transcript::Drift;

use crate::{Failure, HOOK_INPUT_LIMIT, Hook, NESTING_LIMIT, shape};

/// The longest id the schema takes (`session_id`, `prompt_id`).
pub const ID_LIMIT: usize = 128;

/// The fields of a hook input a handler uses, each as the schema took
/// it; a field the input lacks is none.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HookInput {
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    /// How a harness session started (`SessionStart`): `fork` makes it a
    /// fork observation (REC-04).
    pub source: Option<String>,
    pub prompt_id: Option<String>,
    /// The prompt's text (`UserPromptSubmit`), which only a prompt
    /// commitment keeps (PRV-08).
    pub prompt: Option<String>,
}

/// A hook input that passed the schema, with its format drift when its
/// keys differ from every supported version's recorded shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    pub input: HookInput,
    pub drift: Option<Drift>,
}

/// The fields a handler uses that must be text, and whether each must
/// be a plain id.
const FIELDS: [(&str, bool); 6] = [
    ("session_id", true),
    ("transcript_path", false),
    ("cwd", false),
    ("source", false),
    ("prompt_id", true),
    ("prompt", false),
];

/// Decodes `text`, the hook input of `hook`.
///
/// # Errors
///
/// [`Failure::TooLarge`] past 1 MiB and [`Failure::TooDeep`] past depth
/// 64, before it is parsed; [`Failure::Malformed`] for anything but one
/// JSON object; [`Failure::Invalid`] when a field fails the schema.
pub fn decode(hook: Hook, text: &str) -> Result<Decoded, Failure> {
    if text.len() > HOOK_INPUT_LIMIT {
        return Err(Failure::TooLarge);
    }
    if depth(text) > NESTING_LIMIT {
        return Err(Failure::TooDeep);
    }
    let Ok(Value::Object(object)) = serde_json::from_str::<Value>(text) else {
        return Err(Failure::Malformed);
    };
    let input = schema(hook, &object)?;

    Ok(Decoded {
        input,
        drift: shape::drift(hook, &object),
    })
}

/// The fields of `object` a handler uses, each checked against the
/// schema.
fn schema(hook: Hook, object: &Map<String, Value>) -> Result<HookInput, Failure> {
    match object.get("hook_event_name") {
        None => {}
        Some(Value::String(name)) if name == hook.name() => {}
        Some(_) => return Err(Failure::Invalid),
    }
    let mut taken = Vec::with_capacity(FIELDS.len());
    for (name, id) in FIELDS {
        let value = match object.get(name) {
            None => None,
            Some(Value::String(text)) if !id || plain_id(text) => Some(text.clone()),
            Some(_) => return Err(Failure::Invalid),
        };
        taken.push(value);
    }
    let mut taken = taken.into_iter();
    let mut next = || taken.next().flatten();

    Ok(HookInput {
        session_id: next(),
        transcript_path: next(),
        cwd: next(),
        source: next(),
        prompt_id: next(),
        prompt: next(),
    })
}

/// Whether `text` is a plain id: 1 to [`ID_LIMIT`] letters, digits,
/// `.`, `_` and `-`, so it can stand in a structural field.
#[must_use]
pub fn plain_id(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= ID_LIMIT
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
}

/// The deepest `text`'s brackets nest, counting only those outside
/// JSON strings; a stray closing bracket counts nothing.
fn depth(text: &str) -> usize {
    let (mut deepest, mut open) = (0, 0usize);
    let (mut in_string, mut escaped) = (false, false);
    for b in text.bytes() {
        if in_string {
            match (escaped, b) {
                (true, _) => escaped = false,
                (false, b'\\') => escaped = true,
                (false, b'"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' | b'[' => {
                open += 1;
                deepest = deepest.max(open);
            }
            b'}' | b']' => open = open.saturating_sub(1),
            _ => {}
        }
    }

    deepest
}

#[cfg(test)]
mod tests;
