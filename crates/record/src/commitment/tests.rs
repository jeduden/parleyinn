use super::*;
use serde_json::json;

#[test]
fn commit_is_hmac_sha256_over_the_canonical_content_under_the_commitment_key() {
    let key = CommitmentKey::from_bytes([3; 32]);
    let content = json!({"text": "hello", "line": 1});

    let c = commit(&key, &content).unwrap();

    let canonical = crate::canonical::encode(&content).unwrap();
    assert_eq!(
        c.digest(),
        &hmac_sha256(&[3; 32], &[b"parinn commitment v1\0", canonical.as_bytes()])
    );
}

#[test]
fn a_commitment_binds_the_content_and_hides_it_behind_the_key() {
    let (a, b) = (
        CommitmentKey::from_bytes([1; 32]),
        CommitmentKey::from_bytes([2; 32]),
    );
    let content = json!({"text": "x"});

    assert_ne!(commit(&a, &content), commit(&b, &content));
    assert_ne!(commit(&a, &content), commit(&a, &json!({"text": "y"})));
    assert_eq!(commit(&a, &content), commit(&a, &json!({"text": "x"})));
}

#[test]
fn content_with_a_number_canonical_encoding_refuses_is_refused() {
    let key = CommitmentKey::from_bytes([1; 32]);

    assert!(commit(&key, &json!({"x": 0.5})).is_err());
}

#[test]
fn keys_and_commitments_round_trip_through_hex_and_a_key_never_shows_in_debug() {
    let key = CommitmentKey::from_bytes([0xab; 32]);
    let c = commit(&key, &json!(1)).unwrap();

    assert_eq!(CommitmentKey::from_hex(&key.to_hex()), Ok(key.clone()));
    assert_eq!(key.as_bytes(), &[0xab; 32]);
    assert_eq!(format!("{key:?}"), "CommitmentKey(..)");
    assert_eq!(Commitment::from_hex(&c.to_string()), Ok(c));
    assert_eq!(Commitment::from_digest(*c.digest()), c);
    assert_eq!(format!("{c:?}"), format!("Commitment({c})"));
    assert!(CommitmentKey::from_hex("zz").is_err());
}

#[test]
fn a_commitment_to_an_encoding_equals_the_commitment_to_its_content() {
    let key = CommitmentKey::from_bytes([3; 32]);
    let content = json!({"text": "hello", "line": 1});
    let canonical = crate::canonical::encode(&content).unwrap();

    assert_eq!(
        commit_encoded(&key, &canonical),
        commit(&key, &content).unwrap()
    );
}
