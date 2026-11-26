use super::*;
use testkit::TempDir;

const REQ_TABLE: &str = "# Doc

| ID | Pri | Requirement | Ver | Traces |
|---|---|---|---|---|
| REC-01 | P0 | Ingest. | T | I1 |
| REC-02 | P1 | Link \\| parent. | T | I1, I10 |
| REC-03 | P2 | Index. | T | — |
";

fn doc() -> &'static Path {
    Path::new("doc.md")
}

#[test]
fn parse_refuses_a_requirement_table() {
    let err = parse(doc(), REQ_TABLE).unwrap_err();

    assert_eq!(
        err.to_string(),
        "srs: doc.md:3: a requirement table: write each requirement as a block under a heading that is its id"
    );
}

#[test]
fn parse_refuses_assumption_and_non_functional_tables() {
    let err = |body: &str| parse(doc(), body).unwrap_err().to_string();

    assert!(
        err(
            "| ID | Assumption | Verified in |\n|---|---|---|\n| ASM-01 | Hooks carry ids | S1 |\n"
        )
        .starts_with("srs: doc.md:1: a requirement table"),
    );
    assert!(
        err("text\n\n| ID | Category | Requirement | Ver |\n|---|---|---|---|\n")
            .starts_with("srs: doc.md:3: a requirement table"),
    );
}

#[test]
fn parse_passes_other_tables_by() {
    let body = "| ID | Constraint |\n|---|---|\n| CON-01 | Go. |\n\n| # | Requirement |\n|---|---|\n| X | y |\n";

    assert!(parse(doc(), body).unwrap().is_empty());
}

#[test]
fn parse_passes_a_quoted_requirement_table_by() {
    let body = "```md\n| ID | Pri | Requirement | Ver |\n|---|---|---|---|\n| REC-01 | P0 | x | T |\n```\n";

    assert!(parse(doc(), body).unwrap().is_empty());
}

#[test]
fn parse_accepts_the_lane_families() {
    let body: String = ["LANE-01", "VIEW-19", "OWN-22", "PEER-11"]
        .iter()
        .map(|id| format!("## {id}\n\n- Pri: P1\n\nText.\n\n"))
        .collect();

    let ids: Vec<String> = parse(doc(), &body)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();

    assert_eq!(ids, ["LANE-01", "VIEW-19", "OWN-22", "PEER-11"]);
}

#[test]
fn load_reads_every_file_and_rejects_duplicates() {
    let dir = TempDir::new().unwrap();
    let write = |name: &str, body: &str| fs::write(dir.path().join(name), body).unwrap();
    write("a.md", REQ_BLOCKS);
    write("b.md", "## SEC-01\n\n- Pri: P0\n\nx\n");
    write("notes.txt", "## SEC-01\n\n- Pri: P0\n\nx\n");

    assert_eq!(load(dir.path()).unwrap().len(), 4);

    write("c.md", "# C\n\n## SEC-01\n\n- Pri: P0\n\nagain\n");
    let err = load(dir.path()).unwrap_err().to_string();
    assert!(err.contains("c.md:3: SEC-01 already defined at"), "{err}");
}

#[test]
fn load_reports_parse_list_and_read_errors() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.md"), REQ_TABLE).unwrap();
    assert!(
        load(dir.path())
            .unwrap_err()
            .to_string()
            .contains("a.md:3: a requirement table")
    );

    let err = load(&dir.path().join("missing")).unwrap_err();
    assert!(err.to_string().starts_with("srs: list "), "{err}");
    assert!(std::error::Error::source(&err).is_some());

    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("dir.md")).unwrap();
    let err = load(dir.path()).unwrap_err();
    assert!(err.to_string().starts_with("srs: read "), "{err}");
    assert!(std::error::Error::source(&err).is_some());
    assert!(std::error::Error::source(&Error::Malformed(String::new())).is_none());
}

#[test]
fn ids_and_invariants_have_their_shapes() {
    for id in ["REC-01", "ASM-04", "LANE-12"] {
        assert!(is_requirement_id(id), "{id}");
    }
    for id in ["REC-1", "REC-001", "XYZ-01", "REC01", "REC-0a", "REC-0-1"] {
        assert!(!is_requirement_id(id), "{id}");
    }
    for inv in ["I1", "I9", "I10"] {
        assert!(is_invariant(inv), "{inv}");
    }
    for inv in ["I0", "I11", "I", "J1", "I1x"] {
        assert!(!is_invariant(inv), "{inv}");
    }
}

#[test]
fn split_list_reads_a_comma_separated_cell() {
    assert_eq!(
        split_list(" I1 ,I10", is_invariant, "trace").unwrap(),
        ["I1", "I10"]
    );
    assert_eq!(
        split_list("I1, I11", is_invariant, "trace").unwrap_err(),
        r#"malformed trace "I11""#
    );
}

const REQ_BLOCKS: &str = "# Doc

## REC-01

- Pri: P0
- Ver: T
- Traces: I1

Ingest.

## REC-02

- Pri: P1
- Ver: T
- Traces: I1, I10

Link | parent.

| ID | Constraint |
|---|---|
| CON-01 | Go. |

## REC-03

- Pri: P2
- Ver: T
- Traces: —

Index
whole.
";

#[test]
fn parse_reads_a_block() {
    let got = parse(doc(), REQ_BLOCKS).unwrap();

    assert_eq!(
        got[1],
        Requirement {
            id: "REC-02".into(),
            priority: "P1".into(),
            verify: "T".into(),
            traces: vec!["I1".into(), "I10".into()],
            text: "Link | parent.".into(),
            path: doc().into(),
            line: 11,
        }
    );
    assert_eq!(got[2].text, "Index whole.");
    assert!(got[2].traces.is_empty());
}

#[test]
fn parse_returns_blocks_in_the_documents_order() {
    let got: Vec<(String, usize)> = parse(doc(), REQ_BLOCKS)
        .unwrap()
        .into_iter()
        .map(|r| (r.id, r.line))
        .collect();

    assert_eq!(
        got,
        [
            ("REC-01".into(), 3),
            ("REC-02".into(), 11),
            ("REC-03".into(), 23)
        ]
    );
}

#[test]
fn parse_reads_a_block_of_every_field() {
    let body = "## ASM-01\n\n- Evidence so far: S1\n- Verified in: S1\n- Affects: REC-01\n\n\
                Hooks carry ids.\n\n## NFR-01\n\n- Category: Latency\n- Ver: A\n\nFast.\n";

    let got = parse(doc(), body).unwrap();

    assert_eq!(got[0].text, "Hooks carry ids.");
    assert!(got[0].priority.is_empty() && got[0].verify.is_empty());
    assert_eq!(
        (got[1].verify.as_str(), got[1].text.as_str()),
        ("A", "Fast.")
    );
}

#[test]
fn parse_rejects_malformed_blocks() {
    let err = |body: &str| parse(doc(), body).unwrap_err().to_string();

    assert_eq!(
        err("## REC-01\n\n- Trace: I1\n\nText.\n"),
        r#"srs: doc.md:1: REC-01: unknown field "Trace""#
    );
    assert_eq!(
        err("## REC-01\n\n- Pri: P0\n- Pri: P1\n\nText.\n"),
        r#"srs: doc.md:1: REC-01: field "Pri" twice"#
    );
    assert_eq!(
        err("## REC-01\n\nText.\n"),
        "srs: doc.md:1: REC-01: no fields"
    );
    assert_eq!(
        err("## REC-01\n\n- Pri: P0\n\n## REC-02\n"),
        "srs: doc.md:1: REC-01: no text"
    );
    assert_eq!(
        err("## REC-01\n\n- Traces: I11\n\nText.\n"),
        r#"srs: doc.md:1: REC-01: malformed trace "I11""#
    );
}

#[test]
fn load_rejects_a_block_that_redefines_an_id() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.md"), REQ_BLOCKS).unwrap();
    fs::write(
        dir.path().join("b.md"),
        "## REC-01\n\n- Pri: P0\n\nAgain.\n",
    )
    .unwrap();

    let err = load(dir.path()).unwrap_err().to_string();

    let (a, b) = (dir.path().join("a.md"), dir.path().join("b.md"));
    assert_eq!(
        err,
        format!(
            "srs: {}:1: REC-01 already defined at {}:3",
            b.display(),
            a.display()
        )
    );
}
