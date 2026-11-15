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

    let found = sibling_uuids(&dir.path().to_string_lossy(), &own.to_string_lossy());

    assert_eq!(found, BTreeSet::from(["u1".to_owned()]));
    assert!(sibling_uuids(&dir.path().join("none").to_string_lossy(), "").is_empty());
}
