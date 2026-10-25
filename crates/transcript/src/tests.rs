use serde_json::{Value, json};

use super::*;

fn line(value: &Value) -> Line {
    value.as_object().cloned().unwrap()
}

/// Two recorded versions: 1.0.0 records assistant lines needing
/// `message` and `version`, and the date attachment; 2.0.0 records
/// unversioned `mode` lines.
fn shapes() -> Shapes {
    let one = RecordedShapes::generate(
        "1.0.0",
        &[
            line(&json!({"type": "assistant", "message": {}, "version": "1.0.0", "uuid": "a"})),
            line(&json!({"type": "assistant", "message": {}, "version": "1.0.0"})),
            line(
                &json!({"type": "attachment", "attachment": {"type": "date"}, "version": "1.0.0"}),
            ),
        ],
    );
    let two =
        RecordedShapes::generate("2.0.0", &[line(&json!({"type": "mode", "mode": "normal"}))]);
    Shapes::of(vec![one, two])
}

fn drift(version: Option<&str>, shape: &str, differed: Differed) -> Drift {
    Drift {
        version: version.map(str::to_owned),
        shape: shape.to_owned(),
        differed,
    }
}

#[test]
fn a_line_that_is_no_json_object_with_a_textual_type_is_unparsed() {
    let shapes = shapes();
    for text in ["{not json", "[1, 2]", "\"text\"", "{}", "{\"type\": 3}", ""] {
        assert_eq!(parse(text, &shapes), Parsed::Unparsed(None), "{text}");
    }
}

#[test]
fn a_line_of_a_type_no_supported_version_records_is_unparsed_and_drift() {
    let shapes = shapes();
    let parsed = parse(r#"{"type":"future_kind","version":"1.0.0"}"#, &shapes);

    assert_eq!(
        parsed,
        Parsed::Unparsed(Some(drift(
            Some("1.0.0"),
            "future_kind",
            Differed::UnrecordedType
        )))
    );
}

#[test]
fn a_line_naming_an_unsupported_version_is_drift() {
    let shapes = shapes();
    let cases = [
        (
            json!({"type": "assistant", "message": {}, "version": "9.9.9"}),
            Some("9.9.9"),
        ),
        (
            json!({"type": "assistant", "message": {}, "version": 7}),
            None,
        ),
    ];
    for (value, version) in cases {
        let parsed = parse(&value.to_string(), &shapes);
        assert_eq!(
            parsed,
            Parsed::Drifted(
                line(&value),
                drift(version, "assistant", Differed::UnsupportedVersion)
            )
        );
    }
}

#[test]
fn a_line_whose_shape_its_version_does_not_record_is_drift() {
    let value =
        json!({"type": "attachment", "attachment": {"type": "new_thing"}, "version": "1.0.0"});

    assert_eq!(
        parse(&value.to_string(), &shapes()),
        Parsed::Drifted(
            line(&value),
            drift(
                Some("1.0.0"),
                "attachment.new_thing",
                Differed::UnrecordedShape
            )
        )
    );
}

#[test]
fn a_line_missing_a_key_its_shape_needs_is_drift() {
    let value = json!({"type": "assistant", "version": "1.0.0"});

    assert_eq!(
        parse(&value.to_string(), &shapes()),
        Parsed::Drifted(
            line(&value),
            drift(
                Some("1.0.0"),
                "assistant",
                Differed::Missing("message".into())
            )
        )
    );
}

#[test]
fn an_unversioned_line_fits_any_supported_version_that_records_its_shape() {
    let shapes = shapes();
    let Parsed::Fits(fit) = parse(r#"{"type":"mode","mode":"plan"}"#, &shapes) else {
        panic!("an unversioned mode line fits 2.0.0");
    };
    assert_eq!((fit.key.as_str(), fit.version), ("mode", None));

    // An unversioned assistant line lacks the version 1.0.0 needs.
    let value = json!({"type": "assistant", "message": {}});
    assert_eq!(
        parse(&value.to_string(), &shapes),
        Parsed::Drifted(
            line(&value),
            drift(None, "assistant", Differed::Missing("version".into()))
        )
    );
}

#[test]
fn a_fitting_line_names_its_shape_and_any_key_beyond_it() {
    let shapes = shapes();
    let Parsed::Fits(fit) = parse(
        r#"{"type":"assistant","message":{},"version":"1.0.0","uuid":"b"}"#,
        &shapes,
    ) else {
        panic!("the line fits 1.0.0");
    };
    assert_eq!(fit.key, "assistant");
    assert_eq!(fit.version.as_deref(), Some("1.0.0"));
    assert_eq!(fit.beyond(), None);

    let Parsed::Fits(extra) = parse(
        r#"{"type":"assistant","message":{},"version":"1.0.0","zeta":1}"#,
        &shapes,
    ) else {
        panic!("a key beyond the shape still fits what it needs");
    };
    assert_eq!(
        extra.beyond(),
        Some(drift(
            Some("1.0.0"),
            "assistant",
            Differed::Beyond("zeta".into())
        ))
    );
}

#[test]
fn a_drift_describes_harness_version_shape_and_what_differed_in_plain_names() {
    let keep = |s: &str| s.to_owned();
    let cases = [
        (
            drift(
                Some("2.1.296"),
                "assistant",
                Differed::Missing("message".into()),
            ),
            "claude-code:2.1.296:assistant:missing:message",
        ),
        (
            drift(None, "future_kind", Differed::UnrecordedType),
            "claude-code:unversioned:future_kind:unrecorded-type",
        ),
        (
            drift(Some("9.9.9"), "user", Differed::UnsupportedVersion),
            "claude-code:9.9.9:user:unsupported-version",
        ),
        (
            drift(Some("2.1.296"), "attachment.x y", Differed::UnrecordedShape),
            "claude-code:2.1.296:attachment.x_y:unrecorded-shape",
        ),
        (
            drift(Some("2.1.296"), "user", Differed::Beyond("a\"b/c".into())),
            "claude-code:2.1.296:user:beyond:a_b_c",
        ),
    ];
    for (d, want) in cases {
        assert_eq!(d.describe(&keep), want);
    }

    let long = drift(Some("1"), "user", Differed::Beyond("k".repeat(90)));
    assert_eq!(
        long.describe(&keep),
        format!("claude-code:1:user:beyond:{}", "k".repeat(40))
    );

    let redact = |s: &str| s.replace("AKIAIOSFODNN7EXAMPLE", "[REDACTED:aws_access_key]");
    let secret = drift(
        Some("1"),
        "user",
        Differed::Beyond("AKIAIOSFODNN7EXAMPLE".into()),
    );
    assert_eq!(
        secret.describe(&redact),
        "claude-code:1:user:beyond:_REDACTED_aws_access_key_"
    );
}
