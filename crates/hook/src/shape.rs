//! The hook input shapes (REC-26, ENG-17): for each supported harness
//! version, per hook, the keys every recorded hook input of it carries
//! and those any carries, generated from the version's recorded hook
//! inputs under `testdata/claude-code` and committed beside this crate,
//! in the transcript's recorded shapes' text.

use serde_json::{Map, Value};
use transcript::shape::Shapes;
use transcript::{Differed, Drift};

use crate::Hook;

/// Each supported version with its hook input shapes' text, oldest
/// first. A newly recorded version joins with its generated file.
const SUPPORTED: [(&str, &str); 1] = [(
    "2.1.296",
    include_str!("../shapes/claude-code/2.1.296.json"),
)];

/// The supported versions' hook input shapes, as this crate compiles
/// them in. This crate's unit test proves every one parses; were one
/// not to, no hook input would be checked for drift.
#[must_use]
pub fn supported() -> Shapes {
    Shapes::from_texts(&SUPPORTED).unwrap_or_default()
}

/// The format drift of `object`, a hook input of `hook`: none when its
/// keys fit the shape any supported version records for the hook, or
/// when none records one; else what differs from the newest version's.
#[must_use]
pub fn drift(hook: Hook, object: &Map<String, Value>) -> Option<Drift> {
    drift_in(&supported(), hook, object)
}

/// [`drift`] against `shapes`.
#[must_use]
pub fn drift_in(shapes: &Shapes, hook: Hook, object: &Map<String, Value>) -> Option<Drift> {
    let recorded: Vec<_> = shapes
        .versions()
        .filter_map(|v| v.shapes.get(hook.name()).map(|s| (&v.version, s)))
        .collect();
    if recorded
        .iter()
        .any(|(_, s)| s.missing(object).is_none() && s.beyond(object).is_none())
    {
        return None;
    }
    let (version, shape) = recorded.last()?;
    let differed = match shape.missing(object) {
        Some(key) => Differed::Missing(key.to_owned()),
        None => Differed::Beyond(shape.beyond(object)?.to_owned()),
    };

    Some(Drift {
        version: Some((*version).clone()),
        shape: hook.name().to_owned(),
        differed,
    })
}

#[cfg(test)]
mod tests;
