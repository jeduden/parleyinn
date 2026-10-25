use super::*;

#[test]
fn a_running_digest_is_the_digest_of_everything_fed_so_far() {
    let mut running = Running::default();
    assert_eq!(running.digest(), Digest::of(&[]));

    running.update(b"a");
    let partial = running.digest();
    running.update(b"bc");

    assert_eq!(partial, Digest::of(&[b"a"]));
    assert_eq!(running.digest(), Digest::of(&[b"abc"]));
    assert_eq!(
        format!("{:?}", Running::default()),
        format!("Running({})", Digest::of(&[]))
    );
}

#[test]
fn of_hashes_the_concatenation_of_its_parts_with_sha_256() {
    // FIPS 180-2's "abc" vector.
    let want = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    assert_eq!(Digest::of(&[b"abc"]).to_hex(), want);
    assert_eq!(Digest::of(&[b"a", b"", b"bc"]).to_hex(), want);
}

#[test]
fn hex_round_trips_and_shows_in_display_and_debug() {
    let d = Digest::of(&[b"abc"]);

    assert_eq!(Digest::from_hex(&d.to_hex()), Ok(d));
    assert_eq!(d.to_string(), d.to_hex());
    assert_eq!(format!("{d:?}"), format!("Digest({})", d.to_hex()));
    assert_eq!(Digest::from_bytes(*d.as_bytes()), d);
    assert_eq!(Digest::ZERO.to_hex(), "0".repeat(64));
}

#[test]
fn from_hex_refuses_a_wrong_length_or_a_digit_outside_lowercase_hex() {
    assert_eq!(
        Digest::from_hex("ab"),
        Err(HexError::Length { want: 64, got: 2 })
    );
    let upper = "A".repeat(64);
    assert_eq!(Digest::from_hex(&upper), Err(HexError::Digit('A')));
    let mut mixed = "0".repeat(63);
    mixed.push('g');
    assert_eq!(Digest::from_hex(&mixed), Err(HexError::Digit('g')));
    assert_eq!(
        HexError::Length { want: 64, got: 2 }.to_string(),
        "want 64 lowercase hex digits, got 2 characters"
    );
    assert_eq!(
        HexError::Digit('g').to_string(),
        "want lowercase hex digits, got 'g'"
    );
}

#[test]
fn unhex_reads_as_many_bytes_as_its_type_holds() {
    assert_eq!(unhex::<2>("00ff"), Ok([0x00, 0xff]));
    assert_eq!(unhex::<16>(&"ab".repeat(16)), Ok([0xab; 16]));
    assert_eq!(
        unhex::<16>(&"ab".repeat(32)),
        Err(HexError::Length { want: 32, got: 64 })
    );
    assert_eq!(
        HexError::Length { want: 32, got: 1 }.to_string(),
        "want 32 lowercase hex digits, got 1 characters"
    );
}

#[test]
fn hex_writes_lowercase_digits() {
    assert_eq!(hex(&[0x00, 0x0f, 0xa0, 0xff]), "000fa0ff");
}

#[test]
fn hmac_sha256_matches_rfc_4231() {
    // Test case 1: a 20-byte key.
    assert_eq!(
        hmac_sha256(&[0x0b; 20], &[b"Hi There"]).to_hex(),
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
    // Test case 2: a key shorter than the block, the message in parts.
    assert_eq!(
        hmac_sha256(b"Jefe", &[b"what do ya want ", b"for nothing?"]).to_hex(),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    // Test case 6: a key longer than the block is hashed first.
    assert_eq!(
        hmac_sha256(
            &[0xaa; 131],
            &[b"Test Using Larger Than Block-Size Key - Hash Key First"]
        )
        .to_hex(),
        "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
    );
}
