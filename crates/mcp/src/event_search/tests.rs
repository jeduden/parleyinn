use super::*;
use serde_json::json;

fn room_args(more: Value) -> Value {
    let mut args = json!({"scope": "room"});
    for (k, v) in more.as_object().unwrap() {
        args[k] = v.clone();
    }
    args
}

fn parsed(more: Value) -> Result<Params, &'static str> {
    parse(&room_args(more))
}

#[test]
fn the_default_scope_needs_a_run_this_server_does_not_serve_yet() {
    assert_eq!(parse(&json!({"query": "deploy"})), Err(NEEDS_RUN));
    assert_eq!(parse(&json!({"scope": "run"})), Err(NEEDS_RUN));
}

#[test]
fn the_rooms_scope_is_not_served_and_an_unknown_one_is_refused() {
    assert_eq!(parse(&json!({"scope": "rooms"})), Err(NO_ROOMS));
    assert_eq!(parse(&json!({"scope": "world"})), Err(BAD_SCOPE));
}

#[test]
fn a_call_with_only_a_query_asks_for_ten_hits_in_any_run_of_the_room() {
    let params = parsed(json!({"query": "deploy"})).unwrap();

    assert_eq!(params.text, "deploy");
    assert_eq!(params.k, 10);
    assert_eq!(params.room, None);
    assert_eq!(params.writer, None);
    assert!(params.provenance.is_empty() && params.kinds.is_empty());
    assert_eq!(params.trust, None);
    assert_eq!((params.since, params.until), (None, None));
}

#[test]
fn k_is_capped_at_fifty_and_must_be_a_whole_number_from_one() {
    let k = |value: Value| parsed(json!({"k": value})).map(|p| p.k);

    assert_eq!(k(json!(50)), Ok(50));
    assert_eq!(k(json!(500)), Ok(50));
    assert_eq!(k(json!(1)), Ok(1));
    assert_eq!(k(json!(u64::MAX)), Ok(50));
    for bad in [json!(0), json!(-3), json!(1.5), json!("9"), json!(true)] {
        assert_eq!(k(bad), Err(BAD_K));
    }
}

#[test]
fn trust_is_trusted_or_untrusted() {
    let trust = |value: Value| parsed(json!({"trust": value})).map(|p| p.trust);

    assert_eq!(trust(json!("trusted")), Ok(Some(TrustLevel::Trusted)));
    assert_eq!(trust(json!("untrusted")), Ok(Some(TrustLevel::Untrusted)));
    assert_eq!(trust(json!("maybe")), Err(BAD_TRUST));
    assert_eq!(trust(json!(1)), Err(BAD_TRUST));
}

#[test]
fn the_room_and_the_run_are_ids() {
    let id = "ab".repeat(32);

    let params = parsed(json!({"room": id, "run": id})).unwrap();

    assert_eq!(params.room.unwrap().to_string(), id);
    assert_eq!(params.writer.unwrap().to_string(), id);
    assert_eq!(parsed(json!({"room": "nope"})), Err(BAD_ROOM));
    assert_eq!(parsed(json!({"run": "nope"})), Err(BAD_RUN));
    assert_eq!(parsed(json!({"run": 3})), Err(BAD_RUN));
}

#[test]
fn the_query_parameters_are_strings_and_lists_of_strings() {
    let params = parsed(json!({
        "query": "a b", "any_of": ["x", "y"], "all_of": ["z"], "phrase": "p q",
    }))
    .unwrap();

    assert_eq!(params.any_of, ["x", "y"]);
    assert_eq!(params.all_of, ["z"]);
    assert_eq!(params.phrase.as_deref(), Some("p q"));
    for bad in [
        json!({"query": 3}),
        json!({"any_of": "x"}),
        json!({"all_of": [1]}),
        json!({"phrase": 7}),
    ] {
        assert_eq!(parsed(bad), Err(BAD_QUERY));
    }
}

#[test]
fn the_filters_are_short_lists_of_short_strings() {
    let params = parsed(json!({"provenance": ["web", "file"], "kind": ["tool_result"]})).unwrap();

    assert_eq!(params.provenance, ["web", "file"]);
    assert_eq!(params.kinds, ["tool_result"]);
    let many: Vec<String> = (0..17).map(|n| n.to_string()).collect();
    for bad in [
        json!({"provenance": "web"}),
        json!({"kind": [1]}),
        json!({"kind": many}),
        json!({"provenance": ["x".repeat(65)]}),
    ] {
        assert_eq!(parsed(bad), Err(BAD_FILTER));
    }
}

#[test]
fn since_and_until_are_a_date_or_a_utc_time() {
    let ok = |text: &str| parsed(json!({"since": text})).map(|p| p.since);

    for good in [
        "2026-01-01",
        "2026-06-30T12:00:00Z",
        "2026-06-30T12:00:00.123Z",
    ] {
        assert_eq!(ok(good), Ok(Some(good.to_owned())));
    }
    for bad in [
        "yesterday",
        "2026-1-1",
        "2026-01-01x",
        "2026-01-01T",
        "2026/01/01",
        "é026-01-01",
        "",
    ] {
        assert_eq!(ok(bad), Err(BAD_TIME), "{bad:?}");
    }
    assert_eq!(parsed(json!({"until": 5})), Err(BAD_TIME));
    assert_eq!(
        parsed(json!({"until": "2026-01-01T0000000000000000000000000"})),
        Err(BAD_TIME)
    );
}

#[test]
fn the_request_and_the_search_carry_the_call_through() {
    let params = parsed(json!({
        "query": "deploy", "phrase": "p q", "provenance": ["web"], "since": "2026-01-01", "k": 3,
    }))
    .unwrap();
    let compiled = search::compile(&params.request()).unwrap();
    let room = RoomId::derive(&[1; 32], &[2; 16]);

    let search = params.search(&compiled, room, DEADLINE);

    assert_eq!(compiled.expression(), r#""deploy" AND "p q""#);
    assert_eq!(
        (search.room, search.k, search.deadline),
        (room, 3, DEADLINE)
    );
    assert_eq!(search.provenance, ["web"]);
    assert_eq!(search.since, Some("2026-01-01"));
}
