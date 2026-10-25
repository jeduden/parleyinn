//! The transcript parser (REC-05, REC-26). Each transcript line is one
//! JSON value. [`parse`] decodes it and checks it against the shape
//! recorded for its harness version ([`shape`]): a known line type, the
//! keys its shape needs, and a version among the supported ones. A line
//! that is not a JSON object with a textual type, or whose type no
//! supported version records, is one ParleyInn cannot parse: stored as an
//! `unparsed` event. A line that fails the check otherwise is format
//! drift, kept whole as untrusted `harness_text` and never classified
//! by its fields. Nothing here reads a file, a clock or randomness.

pub mod shape;

use serde_json::{Map, Value};

use shape::{RecordedShapes, Shape, Shapes, shape_key};

/// A decoded transcript line.
pub type Line = Map<String, Value>;

/// What differed between a line and the shape recorded for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Differed {
    /// No supported version records its line type.
    UnrecordedType,
    /// It names a version no recorded shape has, or a version that is
    /// not text.
    UnsupportedVersion,
    /// Its version records its line type, but not its shape key.
    UnrecordedShape,
    /// It lacks this key, which its shape needs.
    Missing(String),
    /// It carries this key, beyond those its shape allows.
    Beyond(String),
}

/// Format drift (REC-26): a line whose shape differs from the one
/// recorded for its harness version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    /// The version the line names, when it names one as text.
    pub version: Option<String>,
    /// The line's shape key.
    pub shape: String,
    pub differed: Differed,
}

impl Drift {
    /// What an audit entry says of it: the harness, the version, the
    /// shape key and what differed, joined by colons. Each part that
    /// came from the line passes `redact` first, then keeps only
    /// letters, digits, `.`, `_` and `-` (any other character becomes
    /// `_`), at most 40 of them, so no line content beyond a plain name
    /// reaches the audit log.
    #[must_use]
    pub fn describe(&self, redact: &dyn Fn(&str) -> String) -> String {
        let plain = |text: &str| -> String {
            redact(text)
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || "._-".contains(c) {
                        c
                    } else {
                        '_'
                    }
                })
                .take(40)
                .collect()
        };
        let version = self
            .version
            .as_deref()
            .map_or_else(|| "unversioned".to_owned(), plain);
        let differed = match &self.differed {
            Differed::UnrecordedType => "unrecorded-type".to_owned(),
            Differed::UnsupportedVersion => "unsupported-version".to_owned(),
            Differed::UnrecordedShape => "unrecorded-shape".to_owned(),
            Differed::Missing(key) => format!("missing:{}", plain(key)),
            Differed::Beyond(key) => format!("beyond:{}", plain(key)),
        };

        format!(
            "{}:{version}:{}:{differed}",
            shape::HARNESS,
            plain(&self.shape)
        )
    }
}

/// A line that fits its recorded shape.
#[derive(Debug, Clone, PartialEq)]
pub struct Fit<'a> {
    pub line: Line,
    /// The shape it fits.
    pub shape: &'a Shape,
    /// Its shape key.
    pub key: String,
    /// The version it names, when it names one.
    pub version: Option<String>,
}

impl Fit<'_> {
    /// The drift of a line that may carry no key beyond its shape's,
    /// as a line classed `user` or `harness_meta` may not (REC-26),
    /// when it carries one.
    #[must_use]
    pub fn beyond(&self) -> Option<Drift> {
        self.shape.beyond(&self.line).map(|key| Drift {
            version: self.version.clone(),
            shape: self.key.clone(),
            differed: Differed::Beyond(key.to_owned()),
        })
    }
}

/// What [`parse`] made of one line.
#[derive(Debug, Clone, PartialEq)]
pub enum Parsed<'a> {
    /// A line ParleyInn cannot parse (REC-05), with its drift when it is a
    /// JSON object whose type no supported version records.
    Unparsed(Option<Drift>),
    /// Format drift: kept whole, never classified by its fields.
    Drifted(Line, Drift),
    Fits(Fit<'a>),
}

/// Parses one transcript line against the supported versions' `shapes`.
#[must_use]
pub fn parse<'a>(text: &str, shapes: &'a Shapes) -> Parsed<'a> {
    let Ok(Value::Object(line)) = serde_json::from_str::<Value>(text) else {
        return Parsed::Unparsed(None);
    };
    let Some(key) = shape_key(&line) else {
        return Parsed::Unparsed(None);
    };
    let named = line.get("version").map(|v| v.as_str().map(str::to_owned));
    let drift = |differed| Drift {
        version: named.clone().flatten(),
        shape: key.clone(),
        differed,
    };
    if !line
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|kind| shapes.knows_type(kind))
    {
        return Parsed::Unparsed(Some(drift(Differed::UnrecordedType)));
    }
    let candidates: Vec<&RecordedShapes> = match &named {
        None => shapes.versions().collect(),
        Some(Some(version)) => shapes.version(version).into_iter().collect(),
        Some(None) => Vec::new(),
    };
    if candidates.is_empty() {
        return Parsed::Drifted(line, drift(Differed::UnsupportedVersion));
    }
    let recorded: Vec<&Shape> = candidates
        .iter()
        .filter_map(|r| r.shapes.get(&key))
        .collect();
    let Some(first) = recorded.first() else {
        return Parsed::Drifted(line, drift(Differed::UnrecordedShape));
    };
    if let Some(shape) = recorded.iter().find(|s| s.missing(&line).is_none()) {
        return Parsed::Fits(Fit {
            line,
            shape,
            version: named.clone().flatten(),
            key: key.clone(),
        });
    }
    let missing = first.missing(&line).unwrap_or_default().to_owned();

    Parsed::Drifted(line, drift(Differed::Missing(missing)))
}

#[cfg(test)]
mod tests;
