use serde_json::{Value, json};
use transcript::Line;
use transcript::shape::RecordedShapes;

use super::*;

fn object(value: &Value) -> Line {
    value.as_object().cloned().unwrap()
}

/// Two versions: 1.0.0 records `SessionStart` needing `a`; 2.0.0 needs
/// `a` and `b` and allows `c`.
fn shapes() -> Shapes {
    let one = RecordedShapes::generate_keyed(
        "1.0.0",
        &[("SessionStart".into(), object(&json!({"a": 1})))],
    );
    let two = RecordedShapes::generate_keyed(
        "2.0.0",
        &[
            ("SessionStart".into(), object(&json!({"a": 1, "b": 1}))),
            (
                "SessionStart".into(),
                object(&json!({"a": 1, "b": 1, "c": 1})),
            ),
        ],
    );
    Shapes::of(vec![one, two])
}

#[test]
fn a_hook_input_fitting_any_version_has_no_drift() {
    let shapes = shapes();
    for input in [json!({"a": 1}), json!({"a": 1, "b": 2, "c": 3})] {
        assert_eq!(
            drift_in(&shapes, Hook::SessionStart, &object(&input)),
            None,
            "{input}"
        );
    }
}

#[test]
fn a_hook_input_fitting_none_drifts_from_the_newest_version() {
    let shapes = shapes();
    let missing = drift_in(&shapes, Hook::SessionStart, &object(&json!({"b": 1}))).unwrap();
    assert_eq!(
        (
            missing.version.as_deref(),
            missing.shape.as_str(),
            missing.differed
        ),
        (Some("2.0.0"), "SessionStart", Differed::Missing("a".into()))
    );
    let beyond = drift_in(
        &shapes,
        Hook::SessionStart,
        &object(&json!({"a": 1, "b": 1, "d": 1})),
    )
    .unwrap();
    assert_eq!(beyond.differed, Differed::Beyond("d".into()));
}

#[test]
fn a_hook_no_version_records_has_no_drift() {
    assert_eq!(
        drift_in(&shapes(), Hook::UserPromptSubmit, &object(&json!({}))),
        None
    );
}

#[test]
fn the_supported_hook_input_shapes_parse_and_record_the_installed_hooks() {
    let supported = supported();
    let version = supported.version("2.1.296").unwrap();
    for hook in ["SessionStart", "UserPromptSubmit"] {
        assert!(version.shapes.contains_key(hook), "{hook}");
    }
    assert_eq!(
        Shapes::from_texts(&SUPPORTED).map(|s| s.versions().count()),
        Ok(1)
    );
}
