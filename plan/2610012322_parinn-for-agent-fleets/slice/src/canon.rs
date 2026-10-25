//! The canonical encoding of §8.3: RFC 8785 (JCS) over a JSON value.
//!
//! serde_json without `preserve_order` keeps object keys in a BTreeMap,
//! so they come out sorted by their UTF-8 bytes, and it escapes strings
//! the way RFC 8785 does (`\b \t \n \f \r`, other control characters as
//! lowercase `\u00XX`, everything else verbatim). Two limits keep that
//! equal to RFC 8785 here: every key is ASCII, where UTF-8 and UTF-16
//! order agree, and every number is an integer.

use serde_json::Value;

/// The canonical bytes of a value.
pub fn encode(v: &Value) -> Vec<u8> {
    // Serializing a Value to a Vec cannot fail: no I/O, string keys.
    serde_json::to_vec(v).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keys_sort_and_output_is_compact() {
        let v = json!({"seq": 2, "kind": "message", "a": [1, {"z": 0, "b": "x"}]});
        assert_eq!(
            encode(&v),
            br#"{"a":[1,{"b":"x","z":0}],"kind":"message","seq":2}"#.to_vec()
        );
    }

    #[test]
    fn strings_escape_as_rfc_8785_does() {
        let v = json!("\u{8}\t\n\u{c}\r\u{1f}\"\\/\u{7f}é€");
        assert_eq!(
            encode(&v),
            "\"\\b\\t\\n\\f\\r\\u001f\\\"\\\\/\u{7f}é€\""
                .as_bytes()
                .to_vec()
        );
    }
}
