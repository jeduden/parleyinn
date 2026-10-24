use super::*;
use crate::random::Counted;
use serde_json::json;

#[test]
fn each_kind_of_key_mints_its_own_key_and_shows_its_public_half() {
    let mut e = Counted::default();
    let device = DeviceKey::mint(&mut e).unwrap();
    let seat = SeatKey::mint(&mut e).unwrap();
    let recovery = RecoveryKey::mint(&mut e).unwrap();

    assert_ne!(device.public(), seat.public());
    assert_ne!(seat.public(), recovery.public());
    assert_eq!(
        format!("{device:?}"),
        format!("DeviceKey({})", device.public())
    );
    assert_eq!(format!("{seat:?}"), format!("SeatKey({})", seat.public()));
    assert_eq!(
        format!("{recovery:?}"),
        format!("RecoveryKey({})", recovery.public())
    );
}

#[test]
fn a_key_round_trips_through_its_seed() {
    let key = SeatKey::mint(&mut Counted::default()).unwrap();

    let again = SeatKey::from_seed(&key.seed_hex()).unwrap();

    assert_eq!(again.public(), key.public());
    assert!(matches!(
        SeatKey::from_seed("zz"),
        Err(KeyError::Malformed(_))
    ));
    assert!(
        DeviceKey::from_seed(&DeviceKey::mint(&mut Counted::default()).unwrap().seed_hex()).is_ok()
    );
    assert!(
        RecoveryKey::from_seed(
            &RecoveryKey::mint(&mut Counted::default())
                .unwrap()
                .seed_hex()
        )
        .is_ok()
    );
}

#[test]
fn a_signature_verifies_only_for_its_kind_body_and_key() {
    let mut e = Counted::default();
    let key = DeviceKey::mint(&mut e).unwrap();
    let other = DeviceKey::mint(&mut e).unwrap();
    let body = json!({"a": 1});

    let sig = key.sign(SignatureKind::KeyEvent, &body).unwrap();

    assert!(verify(&key.public(), SignatureKind::KeyEvent, &body, &sig).is_ok());
    for (public, kind, body) in [
        (key.public(), SignatureKind::SeatCertificate, body.clone()),
        (key.public(), SignatureKind::KeyEvent, json!({"a": 2})),
        (other.public(), SignatureKind::KeyEvent, body.clone()),
    ] {
        assert!(matches!(
            verify(&public, kind, &body, &sig),
            Err(KeyError::Signature(k)) if k == kind
        ));
    }
}

#[test]
fn a_body_without_a_canonical_encoding_is_refused() {
    let key = SeatKey::mint(&mut Counted::default()).unwrap();
    let body = json!(0.5);

    assert!(matches!(
        key.sign(SignatureKind::KeyEvent, &body),
        Err(KeyError::Encoding(_))
    ));
    let sig = key.sign(SignatureKind::KeyEvent, &json!(1)).unwrap();
    assert!(matches!(
        verify(&key.public(), SignatureKind::KeyEvent, &body, &sig),
        Err(KeyError::Encoding(_))
    ));
}

#[test]
fn signature_kinds_have_their_names() {
    assert_eq!(SignatureKind::KeyEvent.as_str(), "key_event");
    assert_eq!(SignatureKind::SeatCertificate.as_str(), "seat_certificate");
}

#[test]
fn public_keys_and_signatures_round_trip_through_hex() {
    let key = SeatKey::mint(&mut Counted::default()).unwrap();
    let public = key.public();
    let sig = key.sign(SignatureKind::KeyEvent, &json!(1)).unwrap();

    assert_eq!(PublicKey::from_hex(&public.to_string()).unwrap(), public);
    assert_eq!(format!("{public:?}"), format!("PublicKey({public})"));
    assert_eq!(
        public.fingerprint(),
        record::digest::Digest::of(&[b"parinn key fingerprint v1\0", public.as_bytes()])
    );
    assert_eq!(Signature::from_hex(&sig.to_string()).unwrap(), sig);
    assert_eq!(format!("{sig:?}"), format!("Signature({sig})"));
    assert!(matches!(
        PublicKey::from_hex("zz"),
        Err(KeyError::Malformed(_))
    ));
    assert!(matches!(
        Signature::from_hex("zz"),
        Err(KeyError::Malformed(_))
    ));
    assert!(matches!(
        Signature::from_hex(&"g".repeat(128)),
        Err(KeyError::Malformed(_))
    ));
}

#[test]
fn a_public_key_off_the_curve_verifies_nothing() {
    let key = SeatKey::mint(&mut Counted::default()).unwrap();
    let sig = key.sign(SignatureKind::KeyEvent, &json!(1)).unwrap();
    // The y coordinate 2 has no point on edwards25519.
    let mut bytes = [0u8; 32];
    bytes[0] = 2;

    assert!(matches!(
        verify(
            &PublicKey::from_bytes(bytes),
            SignatureKind::KeyEvent,
            &json!(1),
            &sig
        ),
        Err(KeyError::Signature(SignatureKind::KeyEvent))
    ));
}

#[test]
fn errors_explain_themselves() {
    assert_eq!(
        KeyError::Entropy("x".into()).to_string(),
        "draw random bytes: x"
    );
    assert_eq!(
        KeyError::Signature(SignatureKind::SeatCertificate).to_string(),
        "a seat_certificate signature does not verify"
    );
    assert_eq!(KeyError::Malformed("y".into()).to_string(), "malformed y");
    let enc = record::canonical::Error::Number("0.5".into());
    assert_eq!(KeyError::Encoding(enc.clone()).to_string(), enc.to_string());
    assert!(std::error::Error::source(&KeyError::Encoding(enc)).is_some());
    assert!(std::error::Error::source(&KeyError::Malformed("y".into())).is_none());
}

#[test]
fn a_derived_secret_is_the_keys_own_and_each_label_has_its_own() {
    let key = DeviceKey::mint(&mut Counted::default()).unwrap();
    let again = DeviceKey::from_seed(&key.seed_hex()).unwrap();
    let other = DeviceKey::from_seed(&"22".repeat(32)).unwrap();

    assert_eq!(key.derive_secret(b"a"), again.derive_secret(b"a"));
    assert_ne!(key.derive_secret(b"a"), key.derive_secret(b"b"));
    assert_ne!(key.derive_secret(b"a"), other.derive_secret(b"a"));
    assert_ne!(&key.derive_secret(b"a"), key.public().as_bytes());
}
