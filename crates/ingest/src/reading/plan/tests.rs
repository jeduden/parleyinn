use super::*;

#[test]
fn the_directory_s_other_transcripts_give_their_uuids() {
    let dir = testkit::TempDir::new().unwrap();
    let own = dir.path().join("own.jsonl");
    std::fs::write(&own, "{\"uuid\":\"mine\"}\n").unwrap();
    std::fs::write(
        dir.path().join("other.jsonl"),
        "{\"uuid\":\"u1\"}\nnot json\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.txt"), "{\"uuid\":\"u2\"}\n").unwrap();
    std::fs::create_dir(dir.path().join("sub.jsonl")).unwrap();

    let served = BTreeSet::from([dir.path().join("other.jsonl")]);

    let found = sibling_uuids(
        &dir.path().to_string_lossy(),
        &own.to_string_lossy(),
        &served,
    );
    let unserved = sibling_uuids(
        &dir.path().to_string_lossy(),
        &own.to_string_lossy(),
        &BTreeSet::new(),
    );

    assert_eq!(found.all, BTreeSet::from(["u1".to_owned()]));
    assert_eq!(found.served, found.all, "a transcript a server serves");
    assert!(unserved.served.is_empty());
    let none = sibling_uuids(
        &dir.path().join("none").to_string_lossy(),
        "",
        &BTreeSet::new(),
    );
    assert!(none.all.is_empty());
}

#[test]
fn a_path_is_compared_with_its_links_resolved_when_they_can_be() {
    let dir = testkit::TempDir::new().unwrap();
    let real = dir.path().join("real.jsonl");
    std::fs::write(&real, "").unwrap();

    assert_eq!(canonical(&real), real.canonicalize().unwrap());
    let missing = dir.path().join("missing.jsonl");
    assert_eq!(canonical(&missing), missing);
}

#[test]
fn dense_json_is_refused_before_value_allocation_but_a_maximum_string_fits() {
    let dense = format!("{{\"items\":[{}]}}", "0,".repeat(40_000) + "0");
    let plain = format!(
        "{{\"text\":\"{}\"}}",
        "x".repeat(read::bounded::LINE_LIMIT - 12)
    );

    assert!(json_heap_bound(&dense) > PARSED_LINE_LIMIT);
    assert!(json_heap_bound(&plain) <= PARSED_LINE_LIMIT);
}
