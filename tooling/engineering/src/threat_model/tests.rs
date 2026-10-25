use super::*;
use testkit::TempDir;

const SECURITY_BODY: &str = "\
### Threat sources

| Threat source | Can do |
| --- | --- |
| First | x |
| Second | y |

### Threats
";

const MODEL_BODY: &str = "\
| Id | Threat source |
| --- | --- |
| S1 | First |
| S2 | Second |

| Row | Component | Boundary | Sources | Controls |
| --- | --- | --- | --- | --- |
| 1 | Hook | B0 | S1, S2 | T1 |
| X1 | Import | none | S9 | |

## Comparison with Zed Delta

| Control | ParleyInn | Zed Delta |
| --- | --- | --- |
| Record signing | chain | none |
| Trust in other principals | kept apart | none |
| Central-service dependence | none | DeltaDB |
| Key custody | OS store |   |

| Release | Reviewed by | Outcome |
| --- | --- | --- |
| 0.0 | an agent | awaiting |
| 0.1 | | |
";

fn row(id: &str) -> Row {
    Row {
        id: id.to_owned(),
        name: String::new(),
        component: String::new(),
        boundary: String::new(),
        reach: String::new(),
        default: String::new(),
        requirements: String::new(),
    }
}

#[test]
fn the_sources_are_the_first_cells_of_the_table_under_the_heading() {
    assert_eq!(listed_sources(SECURITY_BODY), ["First", "Second"]);
    assert_eq!(listed_sources("no heading"), Vec::<String>::new());
    assert_eq!(cells("not a row"), Vec::<String>::new());
}

#[test]
fn the_legend_must_match_the_listed_sources() {
    assert_eq!(
        legend_matches(MODEL_BODY, SECURITY_BODY),
        Vec::<String>::new()
    );
    let problems = legend_matches(MODEL_BODY, &SECURITY_BODY.replace("Second", "Third"));
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("differ from those of"));
}

#[test]
fn each_register_row_needs_a_row_with_a_known_source_and_a_control() {
    assert_eq!(rows_covered(MODEL_BODY, &[row("1")]), Vec::<String>::new());

    let problems = rows_covered(MODEL_BODY, &[row("2"), row("X1")]);

    assert_eq!(
        problems,
        [
            format!("{MODEL} has no row for register row 2"),
            "register row X1 names no known threat source".to_owned(),
            "register row X1 names no control".to_owned(),
        ]
    );
}

#[test]
fn the_comparison_goes_through_each_control_with_both_sides() {
    assert_eq!(
        compares(MODEL_BODY),
        ["the comparison on Key custody leaves a side empty"]
    );
    let bare = compares("nothing");
    assert_eq!(bare.len(), 5);
    assert!(bare[0].contains("no comparison"));
    assert!(bare[1].contains("leaves out Record signing"));
}

#[test]
fn the_minor_release_comes_from_the_manifest_version() {
    assert_eq!(
        minor_release("[package]\nversion = \"0.3.7\"\n").as_deref(),
        Some("0.3")
    );
    assert_eq!(minor_release("[package]\n"), None);
    assert_eq!(minor_release("version = \"7\"\n"), None);
}

#[test]
fn a_release_needs_a_review_row_naming_who_and_what() {
    assert_eq!(reviewed(MODEL_BODY, "0.0"), Vec::<String>::new());
    assert_eq!(
        reviewed(MODEL_BODY, "0.1"),
        ["the review of release 0.1 names no reviewer or outcome"]
    );
    assert_eq!(
        reviewed(MODEL_BODY, "0.2"),
        [format!("{MODEL} records no review of release 0.2")]
    );
}

#[test]
fn the_repository_passes_and_a_checkout_without_the_files_is_named() {
    assert_eq!(
        check(&Checkout::new(&testkit::repo_root())),
        Vec::<String>::new()
    );

    let dir = TempDir::new().unwrap();
    let problems = check(&Checkout::new(dir.path()));
    assert_eq!(problems.len(), 3, "{problems:?}");
}

#[test]
fn a_missing_register_or_version_is_named() {
    let dir = TempDir::new().unwrap();
    let write = |path: &str, body: &str| {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    };
    write(MODEL, MODEL_BODY);
    write(SECURITY, SECURITY_BODY);
    write(VERSION_MANIFEST, "[package]\n");
    let c = Checkout::new(dir.path());
    assert!(
        check(&c)[0].starts_with("read docs/srs/06b"),
        "{:?}",
        check(&c)
    );

    write(
        boundary::REGISTER,
        "| 1 | Hook | core | B0 | r | on | — | P0 | SEC-01 |",
    );
    let problems = check(&c);
    assert!(
        problems.iter().any(|p| p.contains("names no version")),
        "{problems:?}"
    );
}
