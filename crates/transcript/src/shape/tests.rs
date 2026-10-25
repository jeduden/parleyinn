use serde_json::{Value, json};

use super::*;

fn line(value: &Value) -> Line {
    value.as_object().cloned().unwrap()
}

fn names(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|k| (*k).to_owned()).collect()
}

#[test]
fn a_shape_key_is_the_line_type_with_an_attachment_or_system_subtype() {
    let cases = [
        (json!({"type": "assistant"}), Some("assistant")),
        (
            json!({"type": "attachment", "attachment": {"type": "date"}}),
            Some("attachment.date"),
        ),
        (
            json!({"type": "attachment", "attachment": {"type": 7}}),
            Some("attachment"),
        ),
        (json!({"type": "attachment"}), Some("attachment")),
        (
            json!({"type": "system", "subtype": "turn_duration"}),
            Some("system.turn_duration"),
        ),
        (json!({"type": "system"}), Some("system")),
        (json!({"type": "user", "subtype": "x"}), Some("user")),
        (json!({"type": 3}), None),
        (json!({}), None),
    ];
    for (value, want) in cases {
        assert_eq!(shape_key(&line(&value)).as_deref(), want, "{value}");
    }
}

#[test]
fn generating_keeps_the_keys_every_line_carries_and_those_any_carries() {
    let lines = [
        line(&json!({"type": "assistant", "a": 1, "b": 2})),
        line(&json!({"type": "assistant", "a": 1, "c": 3})),
        line(&json!({"type": "attachment", "attachment": {"type": "date"}, "d": 1})),
        line(&json!({"type": 5})),
    ];
    let recorded = RecordedShapes::generate("9.9.9", &lines);

    assert_eq!(recorded.version, "9.9.9");
    let assistant = &recorded.shapes["assistant"];
    assert_eq!(assistant.needs, names(&["a", "type"]));
    assert_eq!(assistant.allows, names(&["a", "b", "c", "type"]));
    assert_eq!(
        recorded.shapes["attachment.date"].needs,
        names(&["attachment", "d", "type"])
    );
    assert_eq!(recorded.shapes.len(), 2);
    assert_eq!(recorded.types(), names(&["assistant", "attachment"]));
}

#[test]
fn generating_by_given_keys_keeps_each_keys_objects_apart() {
    let keyed = [
        (
            "SessionStart".to_owned(),
            line(&json!({"session_id": "s", "source": "startup"})),
        ),
        (
            "SessionStart".to_owned(),
            line(&json!({"session_id": "s", "source": "fork", "context_tokens": 1})),
        ),
        (
            "UserPromptSubmit".to_owned(),
            line(&json!({"session_id": "s", "prompt": "p"})),
        ),
    ];

    let recorded = RecordedShapes::generate_keyed("9.9.9", &keyed);

    assert_eq!(recorded.version, "9.9.9");
    let start = &recorded.shapes["SessionStart"];
    assert_eq!(start.needs, names(&["session_id", "source"]));
    assert_eq!(
        start.allows,
        names(&["context_tokens", "session_id", "source"])
    );
    assert_eq!(
        recorded.shapes["UserPromptSubmit"].needs,
        names(&["prompt", "session_id"])
    );
}

#[test]
fn a_shape_names_the_first_key_missing_and_the_first_beyond_it() {
    let shape = Shape {
        needs: names(&["a", "type"]),
        allows: names(&["a", "b", "type"]),
    };

    assert_eq!(shape.missing(&line(&json!({"type": "x", "a": 1}))), None);
    assert_eq!(shape.missing(&line(&json!({"type": "x"}))), Some("a"));
    assert_eq!(
        shape.beyond(&line(&json!({"type": "x", "a": 1, "b": 2}))),
        None
    );
    assert_eq!(
        shape.beyond(&line(&json!({"type": "x", "a": 1, "z": 2, "y": 3}))),
        Some("y")
    );
}

#[test]
fn recorded_shapes_round_trip_through_their_text_in_key_order() {
    let lines = [
        line(&json!({"type": "user", "z": 1, "a": 2})),
        line(&json!({"type": "assistant", "m": 1})),
    ];
    let recorded = RecordedShapes::generate("1.2.3", &lines);
    let text = recorded.to_text();

    assert!(text.ends_with("}\n"), "{text}");
    assert!(text.find("\"assistant\"") < text.find("\"user\""), "{text}");
    assert!(text.find("\"a\"") < text.find("\"z\""), "{text}");
    assert_eq!(RecordedShapes::from_text(&text).unwrap(), recorded);
    assert_eq!(RecordedShapes::from_text(&text).unwrap().to_text(), text);
}

#[test]
fn recorded_shapes_refuse_text_of_another_form() {
    let shape = |needs: Value, allows: Value| {
        json!({"harness": HARNESS, "version": "1", "shapes": {"user": {"needs": needs, "allows": allows}}})
            .to_string()
    };
    let cases = [
        ("not json".to_owned(), "not JSON"),
        (
            json!({"harness": "other", "version": "1", "shapes": {}}).to_string(),
            "not this harness's",
        ),
        (
            json!({"harness": HARNESS, "shapes": {}}).to_string(),
            "names no version",
        ),
        (
            json!({"harness": HARNESS, "version": "1"}).to_string(),
            "holds no shapes",
        ),
        (
            json!({"harness": HARNESS, "version": "1", "shapes": {"user": 1}}).to_string(),
            "a shape is not an object",
        ),
        (
            shape(json!("type"), json!(["type"])),
            "a shape's keys are not a list of names",
        ),
        (
            shape(json!(["type"]), json!([1])),
            "a shape's keys are not a list of names",
        ),
        (
            shape(json!(["type"]), json!([])),
            "a shape needs a key it does not allow",
        ),
    ];
    for (text, want) in cases {
        let err = RecordedShapes::from_text(&text).unwrap_err();
        assert_eq!(err.reason, want, "{text}");
        assert!(err.to_string().contains(want));
    }
}

#[test]
fn the_supported_versions_are_the_shapes_recorded_beside_the_crate() {
    assert!(Shapes::from_texts(&SUPPORTED).is_ok());
    let shapes = Shapes::supported();

    let versions: Vec<&str> = shapes.versions().map(|r| r.version.as_str()).collect();
    assert_eq!(versions, ["2.1.296"]);
    assert!(shapes.version("2.1.296").is_some());
    assert!(shapes.version("0.0.1").is_none());
    for kind in ["user", "assistant", "attachment", "system", "last-prompt"] {
        assert!(shapes.knows_type(kind), "{kind}");
    }
    assert!(!shapes.knows_type("future_kind"));
    assert!(
        shapes.version("2.1.296").unwrap().shapes["assistant"]
            .needs
            .contains("message")
    );
}

#[test]
fn a_supported_version_whose_text_does_not_parse_fails_naming_it() {
    let err = Shapes::from_texts(&[("1.0.0", "[]")]).unwrap_err();

    assert_eq!(err.version, "1.0.0");
    assert_eq!(err.to_string(), "recorded shapes 1.0.0: not this harness's");
}

#[test]
fn a_supported_version_whose_text_names_another_version_fails() {
    let text = RecordedShapes::generate("2.0.0", &[]).to_text();
    let err = Shapes::from_texts(&[("1.0.0", &text)]).unwrap_err();

    assert_eq!(
        err.to_string(),
        "recorded shapes 1.0.0: names another version"
    );
}
