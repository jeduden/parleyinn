use super::*;
use crate::random::Counted;
use crate::signing::RecoveryKey;

fn minted() -> (DeviceKey, RecoveryKey, KeyEvent) {
    let mut e = Counted::default();
    let device = DeviceKey::mint(&mut e).unwrap();
    let recovery = RecoveryKey::mint(&mut e).unwrap();
    let event = KeyEvent::minted(&device, &recovery.public()).unwrap();
    (device, recovery, event)
}

#[test]
fn a_minted_key_event_carries_only_its_kind_fingerprints_and_signature() {
    let (device, recovery, event) = minted();

    assert_eq!(event.kind, KeyEventKind::Minted);
    assert_eq!(event.key, device.public().fingerprint());
    assert_eq!(event.recovery, recovery.public().digest());
    assert_eq!(
        recovery.public().digest(),
        record::digest::Digest::of(&[recovery.public().as_bytes()])
    );
    assert_eq!(event.follows, None);
    assert!(event.replaces.is_empty());
    assert_eq!(event.signatures.len(), 1);
    let keys: Vec<String> = event
        .to_value()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(
        keys,
        [
            "follows",
            "key",
            "kind",
            "recovery",
            "replaces",
            "signatures"
        ]
    );
    assert!(event.check_minted(&device.public()).is_ok());
}

#[test]
fn a_minted_key_event_checks_only_against_its_own_key() {
    let (_, recovery, event) = minted();

    assert!(matches!(
        event.check_minted(&recovery.public()),
        Err(KeyError::Malformed(_))
    ));
}

#[test]
fn a_changed_minted_key_event_fails_its_check() {
    let (device, recovery, event) = minted();
    let public = device.public();

    let mut e = event.clone();
    e.recovery = public.fingerprint();
    assert!(matches!(
        e.check_minted(&public),
        Err(KeyError::Signature(_))
    ));
    let mut e = event.clone();
    e.kind = KeyEventKind::Rotated;
    assert!(matches!(
        e.check_minted(&public),
        Err(KeyError::Malformed(_))
    ));
    let mut e = event.clone();
    e.follows = Some(public.fingerprint());
    assert!(matches!(
        e.check_minted(&public),
        Err(KeyError::Malformed(_))
    ));
    let mut e = event.clone();
    e.replaces = vec![recovery.public().fingerprint()];
    assert!(matches!(
        e.check_minted(&public),
        Err(KeyError::Malformed(_))
    ));
    let mut e = event.clone();
    e.signatures.clear();
    assert!(matches!(
        e.check_minted(&public),
        Err(KeyError::Malformed(_))
    ));
}

#[test]
fn a_key_event_round_trips_through_its_value() {
    let (device, _, mut event) = minted();
    assert_eq!(KeyEvent::from_value(&event.to_value()).unwrap(), event);
    event.follows = Some(device.public().fingerprint());
    event.replaces = vec![device.public().fingerprint()];
    assert_eq!(KeyEvent::from_value(&event.to_value()).unwrap(), event);
    assert_eq!(
        event.digest(),
        Digest::of(&[record::canonical::encode(&event.to_value())
            .unwrap()
            .as_bytes()])
    );
}

#[test]
fn a_malformed_key_event_value_is_refused_naming_the_field() {
    let (_, _, event) = minted();
    let good = event.to_value();
    for (field, bad) in [
        ("kind", serde_json::json!("born")),
        ("key", serde_json::json!(1)),
        ("recovery", serde_json::json!("x")),
        ("follows", serde_json::json!("x")),
        ("replaces", serde_json::json!("x")),
        ("replaces", serde_json::json!(["x"])),
        ("signatures", serde_json::json!(null)),
        ("signatures", serde_json::json!(["x"])),
    ] {
        let mut v = good.clone();
        v[field] = bad;
        match KeyEvent::from_value(&v) {
            Err(KeyError::Malformed(what)) => assert!(what.contains(field), "{what}"),
            other => panic!("{field}: {other:?}"),
        }
    }
}

#[test]
fn every_key_event_kind_round_trips_through_its_name() {
    for (kind, name) in [
        (KeyEventKind::Minted, "minted"),
        (KeyEventKind::TokenKeyAdopted, "token_key_adopted"),
        (KeyEventKind::MintedAnew, "minted_anew"),
        (KeyEventKind::Rotated, "rotated"),
        (KeyEventKind::Recovered, "recovered"),
        (KeyEventKind::Revoked, "revoked"),
    ] {
        assert_eq!(kind.as_str(), name);
        assert_eq!(KeyEventKind::parse(name), Some(kind));
    }
    assert_eq!(KeyEventKind::parse("born"), None);
}

fn change() -> NodeIdentityChange {
    let writer = record::id::WriterId::derive(&[4; 32]);
    NodeIdentityChange {
        replaces: Digest::of(&[b"old"]),
        key: Digest::of(&[b"new"]),
        chain_head: SeenHead::new(
            Address::new(writer, Seq::new(3).unwrap()),
            Digest::of(&[b"head"]),
        ),
    }
}

#[test]
fn a_node_identity_change_event_carries_only_both_fingerprints_and_the_chain_head() {
    let event = change();

    let value = event.to_value();

    let keys: Vec<&String> = value.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["chain_head", "key", "replaces"]);
    let head: Vec<&String> = value["chain_head"].as_object().unwrap().keys().collect();
    assert_eq!(head, ["head", "seq", "writer"]);
    assert_eq!(NodeIdentityChange::from_value(&value).unwrap(), event);
}

#[test]
fn a_node_identity_change_event_that_does_not_parse_names_its_field() {
    let good = change().to_value();
    for (field, bad) in [
        ("replaces", json!({"replaces": 1})),
        ("key", json!({"key": "zz"})),
        ("chain_head", json!({"chain_head": {"writer": "x"}})),
        ("chain_head", json!({"chain_head": {"seq": 0}})),
        ("chain_head", json!({"chain_head": {"head": 1}})),
    ] {
        let mut value = good.clone();
        for (k, v) in bad.as_object().unwrap() {
            if k == "chain_head" {
                for (hk, hv) in v.as_object().unwrap() {
                    value["chain_head"][hk] = hv.clone();
                }
            } else {
                value[k] = v.clone();
            }
        }
        let err = NodeIdentityChange::from_value(&value).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("malformed node identity change event: bad {field}")
        );
    }
}
