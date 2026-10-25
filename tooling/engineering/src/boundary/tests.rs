use super::*;
use crate::review_workflow::parse;
use testkit::TempDir;

const TABLE: &str = "\
| #   | Component | Component | Boundary | Reach | Default | Disable | Pri | Requirements |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1   | Hook handlers | core | B0 | stdin | on install | — | P0 | SEC-01 |
| 3   | Kernel worker | core | B0 | parent | on | — | P1 | CMP-05 |
| 8   | Room-view | room-view component | B1 | loopback | off | B1 | P1 | SEC-20 |
| 11  | Phone, before B2 | room-view component | B1 | the principal's own tunnel, outside ParleyInn, to it | off | B1 | P1 | OWN-16 |
| 23  | The harness's traffic | — | outside | model | — | — | — | I4 |
| X1  | `parinn import https://…` | rejected | — | a fetch | — | — | — | — |
Not a row | here | too
";

fn table() -> Vec<Row> {
    parse_rows(TABLE)
}

#[test]
fn rows_are_the_numbered_table_rows_and_nothing_else() {
    let rows = table();

    assert_eq!(
        rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        ["1", "3", "8", "11", "23", "X1"]
    );
    assert_eq!(rows[0].component, "core");
    assert_eq!(rows[0].boundary, "B0");
    assert_eq!(rows[0].default, "on install");
    assert_eq!(rows[0].requirements, "SEC-01");
    assert_eq!(rows[0].name, "Hook handlers");
    assert!(
        parse_rows("| 1 | short |")
            .iter()
            .all(|r| r.boundary.is_empty())
    );
    assert!(parse_rows("| X | not a number |").is_empty());
}

#[test]
fn the_real_register_is_read_and_a_missing_or_empty_one_is_named() {
    let real = register(&Checkout::new(&testkit::repo_root())).unwrap();
    assert!(real.len() > 28);

    let dir = TempDir::new().unwrap();
    let c = Checkout::new(dir.path());
    assert!(register(&c).unwrap_err().starts_with("read docs/srs"));
    std::fs::create_dir_all(dir.path().join("docs/srs")).unwrap();
    std::fs::write(dir.path().join(REGISTER), "no table").unwrap();
    assert_eq!(
        register(&c).unwrap_err(),
        format!("{REGISTER} holds no row")
    );
}

#[test]
fn each_row_names_one_boundary_and_each_component_sits_in_one() {
    assert_eq!(assigned_once(&table()), Vec::<String>::new());

    let mut rows = table();
    rows[0].boundary = "B0, B1".to_owned();
    rows[1].boundary = "B2".to_owned();
    rows.push(rows[2].clone());
    let problems = assigned_once(&rows);

    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(problems[0].contains("row 1 names \"B0, B1\""));
    assert!(problems.iter().any(|p| p.contains("row 8 appears twice")));
    assert!(
        problems
            .iter()
            .any(|p| p.contains("the core sits in B0, B1 in row 1")),
        "{problems:?}"
    );
}

#[test]
fn every_row_beyond_the_core_is_off_by_default() {
    assert_eq!(off_until_on(&table()), Vec::<String>::new());

    let mut rows = table();
    rows[2].default = "on".to_owned();

    assert_eq!(
        off_until_on(&rows),
        ["row 8 is B1 and \"on\" by default, not off"]
    );
}

#[test]
fn the_tunnel_has_a_row_of_its_own_that_is_off_and_governed() {
    assert_eq!(tunnel_has_its_own_row(&table()), Vec::<String>::new());
    assert_eq!(
        tunnel_has_its_own_row(&table()[..2]),
        ["no row names the principal's own tunnel, outside ParleyInn"]
    );

    let mut rows = table();
    rows[3].default = "on".to_owned();
    rows[3].requirements = String::new();
    assert_eq!(
        tunnel_has_its_own_row(&rows),
        [
            "row 11 is B1 and \"on\" by default, not off",
            "row 11 names no requirement"
        ]
    );
    rows[3].boundary = "outside".to_owned();
    assert!(tunnel_has_its_own_row(&rows).contains(&"row 11 is in no boundary".to_owned()));
}

fn checkout_with(bins: &[&str]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for name in bins {
        let krate = dir.path().join("crates").join(name);
        std::fs::create_dir_all(krate.join("src")).unwrap();
        std::fs::write(krate.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(krate.join("src/main.rs"), "fn main() {}").unwrap();
    }
    let lib = dir.path().join("crates/library");
    std::fs::create_dir_all(lib.join("src")).unwrap();
    std::fs::write(lib.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(lib.join("src/lib.rs"), "").unwrap();
    let declared = dir.path().join("crates/declared");
    std::fs::create_dir_all(&declared).unwrap();
    std::fs::write(declared.join("Cargo.toml"), "[[bin]]\nname = \"x\"\n").unwrap();
    let c = Checkout::new(dir.path());

    (dir, c)
}

#[test]
fn executables_are_the_crates_with_a_main_or_a_bin_target() {
    let (_dir, c) = checkout_with(&["parinn"]);

    assert_eq!(executables(&c).unwrap(), ["declared", "parinn"]);
    assert!(executables(&Checkout::new(std::path::Path::new("/nonexistent"))).is_err());
}

#[test]
fn a_process_the_register_does_not_list_is_named() {
    let rows = table();
    let found = |names: &[&str]| names.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();

    assert_eq!(
        unlisted_processes(&found(&["parinn"]), &rows),
        Vec::<String>::new()
    );
    assert_eq!(
        unlisted_processes(&found(&["rogue"]), &rows),
        ["the process rogue runs as no component the register lists"]
    );
    assert_eq!(
        unlisted_processes(&found(&["parinn"]), &rows[2..]),
        ["the process parinn runs as the core, which no register row names"]
    );
}

fn flow() -> Workflow {
    parse("jobs:\n  test:\n    steps:\n      - run: cargo test\n")
}

#[test]
fn the_core_needs_its_reach_evidence_and_an_unknown_process_has_none() {
    let (_dir, c) = checkout_with(&["parinn"]);
    let found = vec!["parinn".to_owned()];

    let problems = evidence(&c, &flow(), &found);
    assert!(
        problems.iter().any(|p| p.contains("clippy.toml")),
        "{problems:?}"
    );
    assert_eq!(
        evidence(&c, &flow(), &["rogue".to_owned()]),
        ["no reach evidence is defined for the process rogue"]
    );
}

#[test]
fn the_check_reports_a_missing_register_and_the_repository_passes_it() {
    let (_dir, c) = checkout_with(&["parinn"]);
    assert!(check(&c, &flow())[0].starts_with("read docs/srs"));
    std::fs::create_dir_all(_dir.path().join("docs/srs")).unwrap();
    std::fs::write(_dir.path().join(REGISTER), TABLE).unwrap();
    let problems = check(&c, &flow());
    assert!(
        problems.iter().any(|p| p.contains("the process declared")),
        "{problems:?}"
    );

    let real = Checkout::new(&testkit::repo_root());
    let ci = parse(&real.read(static_analysis::CI_WORKFLOW).unwrap());
    assert_eq!(check(&real, &ci), Vec::<String>::new());
}

#[test]
fn a_checkout_without_crates_fails_the_check_naming_the_listing() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("docs/srs")).unwrap();
    std::fs::write(dir.path().join(REGISTER), TABLE).unwrap();

    let problems = check(&Checkout::new(dir.path()), &flow());

    assert_eq!(problems.len(), 1);
    assert!(problems[0].starts_with("list crates"), "{problems:?}");
}
