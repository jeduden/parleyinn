use super::*;
use crate::fixture;
use testkit::TempDir;

#[test]
fn read_loads_every_record_with_relative_paths() {
    let dir = TempDir::new().unwrap();
    fixture::tree(dir.path());

    let got = Snapshot::read(dir.path()).unwrap();

    assert_eq!(got.requirements.len(), 3);
    assert_eq!(got.requirements[0].path, Path::new("docs/srs/05.md"));
    assert_eq!(got.scenarios.len(), 2);
    assert_eq!(got.scenarios[0].path, Path::new("features/record.feature"));
    assert_eq!(got.codeowners, "/docs/srs/ @owner\n");
    assert_eq!(got.invariants, "I1 holds.\n");
    assert_eq!(
        got.decisions.keys().collect::<Vec<_>>(),
        ["docs/adr/ADR-2610111200-x.md"]
    );
    assert_eq!(
        got.ids().collect::<Vec<_>>(),
        ["REC-01", "ENG-11", "ENG-29", "REC-01", "ENG-29"]
    );
}

#[test]
fn read_takes_a_tree_without_codeowners_or_records() {
    let dir = TempDir::new().unwrap();
    fixture::tree(dir.path());
    fs::remove_file(dir.path().join(".github/CODEOWNERS")).unwrap();
    fs::remove_dir_all(dir.path().join("docs/adr")).unwrap();

    let got = Snapshot::read(dir.path()).unwrap();

    assert!(got.codeowners.is_empty());
    assert!(got.decisions.is_empty());
}

/// A change that breaks one record.
type Breaks<'a> = &'a dyn Fn(&Path);

#[test]
fn read_fails_on_what_it_cannot_read() {
    let broken = |change: &dyn Fn(&Path)| {
        let dir = TempDir::new().unwrap();
        fixture::tree(dir.path());
        change(dir.path());
        let err = Snapshot::read(dir.path()).unwrap_err();
        let has_source = std::error::Error::source(&err).is_some();
        let root = dir.path().display().to_string();
        (err.to_string().replace(&root, "ROOT"), has_source)
    };
    let cases: [(&str, Breaks, &str); 5] = [
        (
            "no SRS",
            &|r| fs::remove_dir_all(r.join("docs/srs")).unwrap(),
            "review-bundle: srs: list ROOT/docs/srs: ",
        ),
        (
            "unreadable features",
            &|r| fs::write(r.join("features/record.feature"), "not gherkin").unwrap(),
            "review-bundle: scenario: read features: ",
        ),
        (
            "no invariants",
            &|r| fs::remove_file(r.join("docs/srs/invariants.md")).unwrap(),
            "review-bundle: read ROOT/docs/srs/invariants.md: ",
        ),
        (
            "CODEOWNERS a directory",
            &|r| {
                fs::remove_file(r.join(".github/CODEOWNERS")).unwrap();
                fs::create_dir(r.join(".github/CODEOWNERS")).unwrap();
            },
            "review-bundle: read ROOT/.github/CODEOWNERS: ",
        ),
        (
            "a record that is a directory",
            &|r| fs::create_dir(r.join("docs/adr/dir.md")).unwrap(),
            "review-bundle: read ROOT/docs/adr/dir.md: ",
        ),
    ];
    for (name, change, want) in cases {
        let (msg, has_source) = broken(change);
        assert!(msg.starts_with(want), "{name}: {msg}");
        assert!(has_source, "{name}");
    }
}

#[test]
fn read_fails_when_the_records_cannot_be_listed() {
    let dir = TempDir::new().unwrap();
    fixture::tree(dir.path());
    fs::remove_dir_all(dir.path().join("docs/adr")).unwrap();
    fs::write(dir.path().join("docs/adr"), "a file").unwrap();

    let err = Snapshot::read(dir.path()).unwrap_err();

    assert!(err.to_string().contains("docs/adr"), "{err}");
}

#[test]
fn relative_keeps_a_path_outside_the_root() {
    assert_eq!(
        relative(Path::new("/a"), Path::new("/a/b/c")),
        Path::new("b/c")
    );
    assert_eq!(
        relative(Path::new("/a"), Path::new("/x/y")),
        Path::new("/x/y")
    );
}
