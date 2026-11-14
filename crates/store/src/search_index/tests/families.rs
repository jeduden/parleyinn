use super::*;

/// A store whose one writer holds a Bash result, a WebFetch result and
/// a web event, seqs 1 to 3, the first at `early` when given.
fn classes(dir: &TempDir, early: Option<&str>) -> Store {
    let class = |n, tool: &str, time| {
        line(
            n,
            "deploy result",
            Provenance::parse(&format!("tool_result:{tool}")).unwrap(),
            time,
        )
    };
    with(
        dir,
        &[
            class(1, "Bash", early),
            class(2, "WebFetch", None),
            line(3, "deploy web", Provenance::Web, None),
        ],
    )
}

/// The seqs `search` finds in `store`, rising.
fn found(store: &Store, search: &Search) -> Vec<u64> {
    let mut found = seqs(&store.search(search).unwrap());
    found.sort_unstable();
    found
}

#[test]
fn the_kind_filter_matches_the_provenance_class_family() {
    let dir = TempDir::new().unwrap();
    let store = classes(&dir, None);
    let query = compiled("deploy");
    let with = |provenance_families: &[String]| {
        found(
            &store,
            &Search {
                provenance_families,
                ..base(&query)
            },
        )
    };

    assert_eq!(with(&["tool_result".to_owned()]), [1, 2]);
    assert_eq!(with(&["tool_result:Bash".to_owned()]), [1]);
    assert_eq!(with(&["web".to_owned()]), [3]);
    assert_eq!(
        with(&["web".to_owned(), "tool_result".to_owned()]),
        [1, 2, 3]
    );
    // A family is the whole text before the colon, and a class the
    // whole text: neither matches as a prefix.
    assert!(with(&["tool".to_owned()]).is_empty());
    assert!(with(&["tool_result:B".to_owned()]).is_empty());
    assert!(with(&["we".to_owned()]).is_empty());
}

#[test]
fn the_families_named_narrow_a_search_together_with_its_other_filters() {
    let dir = TempDir::new().unwrap();
    let store = classes(&dir, Some("2026-03-01T10:00:00Z"));
    let query = compiled("deploy");
    let families = ["web".to_owned(), "tool_result".to_owned()];
    let filtered = Search {
        provenance_families: &families,
        ..base(&query)
    };

    // Each family is one more choice within the other filters, never a
    // way around them: only the Bash result has a time in the range.
    let timed = Search {
        since: Some("2026-01-01"),
        ..filtered
    };
    assert_eq!(found(&store, &timed), [1]);
    let elsewhere = [RoomId::derive(&[9; 32], &[9; 16])];
    let roomed = Search {
        rooms: &elsewhere,
        ..filtered
    };
    assert!(found(&store, &roomed).is_empty());
}
