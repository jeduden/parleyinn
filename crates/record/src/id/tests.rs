use super::*;

const KEY: [u8; 32] = [7; 32];

#[test]
fn ids_derive_from_keys_under_their_own_labels() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);
    let seat = SeatId::derive(&room, &KEY);

    assert_eq!(
        writer.digest(),
        &Digest::of(&[b"parinn writer id v1\0", &KEY])
    );
    assert_eq!(
        room.digest(),
        &Digest::of(&[b"parinn room id v1\0", &KEY, &[1; 16]])
    );
    assert_eq!(
        seat.digest(),
        &Digest::of(&[b"parinn seat id v1\0", room.digest().as_bytes(), &KEY])
    );
    assert_ne!(writer.digest(), seat.digest());
    assert_ne!(RoomId::derive(&KEY, &[2; 16]), room);
}

#[test]
fn ids_show_and_parse_as_hex() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);
    let seat = SeatId::derive(&room, &KEY);

    assert_eq!(WriterId::parse(&writer.to_string()), Ok(writer));
    assert_eq!(RoomId::parse(&room.to_string()), Ok(room));
    assert_eq!(SeatId::parse(&seat.to_string()), Ok(seat));
    assert_eq!(
        WriterId::parse("x"),
        Err(HexError::Length { want: 64, got: 1 })
    );
    assert_eq!(
        RoomId::parse("x"),
        Err(HexError::Length { want: 64, got: 1 })
    );
    assert_eq!(
        SeatId::parse("x"),
        Err(HexError::Length { want: 64, got: 1 })
    );
    assert_eq!(format!("{writer:?}"), format!("WriterId({writer})"));
    assert_eq!(format!("{room:?}"), format!("RoomId({room})"));
    assert_eq!(format!("{seat:?}"), format!("SeatId({seat})"));
}

#[test]
fn seq_starts_at_1_and_stays_a_safe_integer() {
    assert_eq!(Seq::new(0), None);
    assert_eq!(Seq::new(1).map(Seq::get), Some(1));
    assert_eq!(Seq::new(MAX_SAFE).map(Seq::get), Some(MAX_SAFE));
    assert_eq!(Seq::new(MAX_SAFE + 1), None);
    assert_eq!(Seq::FIRST.get(), 1);
    assert_eq!(Seq::FIRST.next(), Seq::new(2));
    assert_eq!(Seq::new(MAX_SAFE).and_then(Seq::next), None);
    assert_eq!(Seq::FIRST.to_string(), "1");
}

#[test]
fn an_address_shows_in_its_ascii_and_full_forms() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);
    let address = Address::new(writer, Seq::new(48213).unwrap());

    assert_eq!(address.to_string(), format!("{writer}:48213"));
    assert_eq!(
        address.full(&room),
        format!("parinn:room/{room}/w/{writer}/48213")
    );
}

#[test]
fn an_address_parses_from_its_ascii_and_full_forms() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);
    let address = Address::new(writer, Seq::new(9).unwrap());

    assert_eq!(Address::parse(&format!("{writer}:9")), Ok((address, None)));
    assert_eq!(
        Address::parse(&address.full(&room)),
        Ok((address, Some(room)))
    );
}

#[test]
fn a_malformed_address_is_refused_naming_the_forms() {
    let writer = WriterId::derive(&KEY);
    let room = RoomId::derive(&KEY, &[1; 16]);
    for bad in [
        String::new(),
        "nocolon".to_owned(),
        format!("{writer}:0"),
        format!("{writer}:x"),
        format!("{writer}:9007199254740992"),
        "short:1".to_owned(),
        format!("parinn:room/{room}/w/{writer}"),
        format!("parinn:room/{room}/x/{writer}/1"),
        format!("parinn:room/zz/w/{writer}/1"),
        format!("parinn:room/{room}/w/zz/1"),
        format!("parinn:room/{room}/w/{writer}/0"),
    ] {
        assert_eq!(Address::parse(&bad), Err(AddressError::new(&bad)), "{bad}");
    }
}

#[test]
fn an_address_error_names_the_input_shortened_and_the_forms() {
    let long = "x".repeat(200);

    assert_eq!(
        AddressError::new("a:b").to_string(),
        "not an address: \"a:b\"; want <writer id>:<seq> or parinn:room/<room id>/w/<writer id>/<seq>"
    );
    assert!(
        AddressError::new(&long)
            .to_string()
            .contains(&format!("\"{}…\"", "x".repeat(80)))
    );
}
