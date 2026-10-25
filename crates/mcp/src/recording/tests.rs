use super::*;
use serde_json::json;

#[test]
fn every_text_in_the_arguments_and_every_name_is_redacted() {
    let redact = |t: &str| t.replace("secret", "[R]");
    let arguments = json!({
        "query": "a secret",
        "any_of": ["secret", 3, {"secret": "x secret"}],
        "k": 5,
        "flag": true,
        "none": null,
    });

    let out = redacted(&arguments, &redact);

    assert_eq!(
        out,
        json!({
            "query": "a [R]",
            "any_of": ["[R]", 3, {"[R]": "x [R]"}],
            "k": 5,
            "flag": true,
            "none": null,
        })
    );
}
