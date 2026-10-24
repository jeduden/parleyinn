use super::{CommitId, RepositoryIdentity};

const ID: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn a_forty_digit_lowercase_hex_id_parses() {
    let id = CommitId::parse(ID).unwrap();
    assert_eq!(id.as_str(), ID);
}

#[test]
fn text_that_is_not_an_id_does_not_parse() {
    for text in [
        "",
        "abc",
        &ID[1..],
        &format!("{ID}0"),
        &ID.to_uppercase(),
        &ID.replace('0', "g"),
        &format!("{} ", &ID[1..]),
        &format!("é{}", &ID[2..]),
    ] {
        assert_eq!(CommitId::parse(text), None, "{text:?}");
    }
}

#[test]
fn raw_bytes_round_trip_through_the_id() {
    let id = CommitId::parse(ID).unwrap();
    let bytes = id.to_bytes();
    assert_eq!(bytes[0], 0x01);
    assert_eq!(bytes[1], 0x23);
    assert_eq!(bytes[19], 0x67);
    assert_eq!(CommitId::from_bytes(&bytes), id);
}

#[test]
fn from_bytes_spells_each_byte_in_two_lowercase_digits() {
    let id = CommitId::from_bytes(&[0xab; 20]);
    assert_eq!(id.as_str(), "ab".repeat(20));
    assert_eq!(CommitId::from_bytes(&[0; 20]).as_str(), "0".repeat(40));
}

#[test]
fn ids_order_by_their_text() {
    let low = CommitId::parse(&"0".repeat(40)).unwrap();
    let high = CommitId::parse(&"f".repeat(40)).unwrap();
    assert!(low < high);
}

#[test]
fn a_root_commit_binds_the_identity() {
    let id = CommitId::parse(ID).unwrap();
    assert_eq!(
        RepositoryIdentity::from_root(Some(id.clone())),
        RepositoryIdentity::Bound(id)
    );
}

#[test]
fn no_root_commit_is_provisional() {
    assert_eq!(
        RepositoryIdentity::from_root(None),
        RepositoryIdentity::Provisional
    );
}
