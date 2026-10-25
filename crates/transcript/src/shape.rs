//! The recorded shapes (REC-26, ENG-17): for each supported harness
//! version, per shape key, the keys every recorded line of that shape
//! carries and those any carries. They are generated from the
//! version's recorded transcripts under `testdata/claude-code`, and
//! each version's file is committed beside this crate and compiled in.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde_json::{Map, Value};

use crate::Line;

/// The harness the recorded shapes describe.
pub const HARNESS: &str = "claude-code";

/// Each supported version with its recorded shapes' text, oldest first.
/// A newly recorded version joins with its generated file.
const SUPPORTED: [(&str, &str); 1] = [(
    "2.1.296",
    include_str!("../shapes/claude-code/2.1.296.json"),
)];

/// A line's shape key: its type, an attachment's followed by its
/// attachment type and a `system` line's by its subtype, each after a
/// dot. A line with no textual type has none.
#[must_use]
pub fn shape_key(line: &Line) -> Option<String> {
    let kind = line.get("type")?.as_str()?;
    let sub = match kind {
        "attachment" => line
            .get("attachment")
            .and_then(|a| a.get("type"))
            .and_then(Value::as_str),
        "system" => line.get("subtype").and_then(Value::as_str),
        _ => None,
    };

    Some(match sub {
        Some(sub) => format!("{kind}.{sub}"),
        None => kind.to_owned(),
    })
}

/// One recorded shape: the keys a line of it needs, and those it may
/// carry, the needed ones among them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shape {
    pub needs: BTreeSet<String>,
    pub allows: BTreeSet<String>,
}

impl Shape {
    /// The first key, in name order, the shape needs and `line` lacks.
    #[must_use]
    pub fn missing(&self, line: &Line) -> Option<&str> {
        self.needs
            .iter()
            .find(|k| !line.contains_key(k.as_str()))
            .map(String::as_str)
    }

    /// The first key of `line`, in name order, beyond those the shape
    /// allows.
    #[must_use]
    pub fn beyond<'a>(&self, line: &'a Line) -> Option<&'a str> {
        let mut keys: Vec<&String> = line.keys().collect();
        keys.sort();
        keys.into_iter()
            .find(|k| !self.allows.contains(k.as_str()))
            .map(String::as_str)
    }
}

/// Why recorded shapes' text was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeError {
    /// The version the text was given as, when it was.
    pub version: String,
    pub reason: &'static str,
}

impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.version.is_empty() {
            write!(f, "recorded shapes: {}", self.reason)
        } else {
            write!(f, "recorded shapes {}: {}", self.version, self.reason)
        }
    }
}

impl std::error::Error for ShapeError {}

/// One harness version's recorded shapes, by shape key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordedShapes {
    pub version: String,
    pub shapes: BTreeMap<String, Shape>,
}

impl RecordedShapes {
    /// The shapes `lines`, recorded from `version`, show; a line with
    /// no shape key adds none.
    #[must_use]
    pub fn generate(version: &str, lines: &[Line]) -> Self {
        let keyed: Vec<(String, Line)> = lines
            .iter()
            .filter_map(|line| Some((shape_key(line)?, line.clone())))
            .collect();

        Self::generate_keyed(version, &keyed)
    }

    /// The shapes `objects`, recorded from `version`, show, each object
    /// under the shape key it is given with: a hook input's under its
    /// hook, say.
    #[must_use]
    pub fn generate_keyed(version: &str, objects: &[(String, Line)]) -> Self {
        let mut shapes: BTreeMap<String, Shape> = BTreeMap::new();
        for (key, line) in objects {
            let key = key.clone();
            let keys: BTreeSet<String> = line.keys().cloned().collect();
            match shapes.get_mut(&key) {
                Some(shape) => {
                    shape.needs.retain(|k| keys.contains(k));
                    shape.allows.extend(keys);
                }
                None => {
                    let shape = Shape {
                        needs: keys.clone(),
                        allows: keys,
                    };
                    shapes.insert(key, shape);
                }
            }
        }

        Self {
            version: version.to_owned(),
            shapes,
        }
    }

    /// The line types it records.
    #[must_use]
    pub fn types(&self) -> BTreeSet<String> {
        self.shapes
            .keys()
            .map(|k| k.split('.').next().unwrap_or(k).to_owned())
            .collect()
    }

    /// Its text: pretty-printed JSON, every map in key order, ending in
    /// a newline, so regenerating it reproduces it byte for byte.
    #[must_use]
    pub fn to_text(&self) -> String {
        let names = |keys: &BTreeSet<String>| Value::from(keys.iter().cloned().collect::<Vec<_>>());
        let mut shapes = Map::new();
        for (key, shape) in &self.shapes {
            let mut fields = Map::new();
            fields.insert("allows".to_owned(), names(&shape.allows));
            fields.insert("needs".to_owned(), names(&shape.needs));
            shapes.insert(key.clone(), Value::Object(fields));
        }
        let mut doc = Map::new();
        doc.insert("harness".to_owned(), Value::from(HARNESS));
        doc.insert("shapes".to_owned(), Value::Object(shapes));
        doc.insert("version".to_owned(), Value::from(self.version.as_str()));
        let mut text = serde_json::to_string_pretty(&Value::Object(doc)).unwrap_or_default();
        text.push('\n');

        text
    }

    /// The recorded shapes `text` holds.
    ///
    /// # Errors
    ///
    /// Fails, saying why, on text that is not this harness's recorded
    /// shapes.
    pub fn from_text(text: &str) -> Result<Self, ShapeError> {
        let refuse = |reason| ShapeError {
            version: String::new(),
            reason,
        };
        let doc: Value = serde_json::from_str(text).map_err(|_| refuse("not JSON"))?;
        if doc.get("harness").and_then(Value::as_str) != Some(HARNESS) {
            return Err(refuse("not this harness's"));
        }
        let version = doc
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| refuse("names no version"))?;
        let listed = doc
            .get("shapes")
            .and_then(Value::as_object)
            .ok_or_else(|| refuse("holds no shapes"))?;
        let mut shapes = BTreeMap::new();
        for (key, shape) in listed {
            shapes.insert(key.clone(), shape_from(shape).map_err(refuse)?);
        }

        Ok(Self {
            version: version.to_owned(),
            shapes,
        })
    }
}

/// The shape `value` spells.
fn shape_from(value: &Value) -> Result<Shape, &'static str> {
    let fields = value.as_object().ok_or("a shape is not an object")?;
    let names = |field: &str| -> Result<BTreeSet<String>, &'static str> {
        fields
            .get(field)
            .and_then(Value::as_array)
            .and_then(|keys| {
                keys.iter()
                    .map(|k| k.as_str().map(str::to_owned))
                    .collect::<Option<BTreeSet<String>>>()
            })
            .ok_or("a shape's keys are not a list of names")
    };
    let shape = Shape {
        needs: names("needs")?,
        allows: names("allows")?,
    };
    if !shape.needs.is_subset(&shape.allows) {
        return Err("a shape needs a key it does not allow");
    }

    Ok(shape)
}

/// The recorded shapes of every supported harness version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shapes {
    versions: Vec<RecordedShapes>,
}

impl Shapes {
    /// The supported versions' shapes, as this crate compiles them in.
    /// This crate's unit test proves every one parses; were one not to,
    /// no version would be supported, so every line would be unparsed
    /// and counted as format drift, never classified by its fields.
    #[must_use]
    pub fn supported() -> Self {
        Self::from_texts(&SUPPORTED).unwrap_or_default()
    }

    /// The shapes each `(version, text)` holds.
    ///
    /// # Errors
    ///
    /// Fails, naming the version, when a text does not parse or names
    /// another version.
    pub fn from_texts(texts: &[(&str, &str)]) -> Result<Self, ShapeError> {
        let versions = texts
            .iter()
            .map(|(version, text)| {
                let named = |reason| ShapeError {
                    version: (*version).to_owned(),
                    reason,
                };
                let recorded = RecordedShapes::from_text(text).map_err(|e| named(e.reason))?;
                if recorded.version != *version {
                    return Err(named("names another version"));
                }
                Ok(recorded)
            })
            .collect::<Result<_, _>>()?;

        Ok(Self { versions })
    }

    /// Shapes of these recorded versions, for a test or a tool.
    #[must_use]
    pub fn of(versions: Vec<RecordedShapes>) -> Self {
        Self { versions }
    }

    /// Every supported version's shapes, oldest first.
    pub fn versions(&self) -> impl Iterator<Item = &RecordedShapes> {
        self.versions.iter()
    }

    /// The recorded shapes of `version`, when it is supported.
    #[must_use]
    pub fn version(&self, version: &str) -> Option<&RecordedShapes> {
        self.versions.iter().find(|r| r.version == version)
    }

    /// Whether any supported version records line type `kind`.
    #[must_use]
    pub fn knows_type(&self, kind: &str) -> bool {
        self.versions.iter().any(|r| r.types().contains(kind))
    }
}

#[cfg(test)]
mod tests;
