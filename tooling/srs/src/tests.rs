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
fn parse_reads_requirement_rows() {
    let got = parse(doc(), REQ_TABLE).unwrap();

    assert_eq!(got.len(), 3);
    assert_eq!(
        got[1],
        Requirement {
            id: "REC-02".into(),
            priority: "P1".into(),
            verify: "T".into(),
            traces: vec!["I1".into(), "I10".into()],
            text: "Link | parent.".into(),
            path: doc().into(),
            line: 6,
        }
    );
    assert!(got[2].traces.is_empty());
}

#[test]
fn parse_reads_assumption_and_non_functional_tables() {
    let body = "| ID | Assumption | Verified in |\n|---|---|---|\n| ASM-01 | Hooks carry ids | S1 |\n\n\
                    | ID | Category | Requirement | Ver |\n|---|---|---|---|\n| NFR-01 | Latency | Fast. | A |\n";

    let got = parse(doc(), body).unwrap();

    assert_eq!(got.len(), 2);
    assert_eq!(got[0].text, "Hooks carry ids");
    assert!(got[0].priority.is_empty());
    assert_eq!(got[1].verify, "A");
    assert!(got[1].priority.is_empty());
}

#[test]
fn parse_passes_other_tables_by() {
    let body = "| ID | Constraint |\n|---|---|\n| CON-01 | Go. |\n\n| # | Requirement |\n|---|---|\n| X | y |\n";

    assert!(parse(doc(), body).unwrap().is_empty());
}

#[test]
fn parse_rejects_malformed_ids_and_traces() {
    let err = parse(doc(), "| ID | Requirement |\n|---|---|\n| rec-1 | x |\n").unwrap_err();
    assert_eq!(
        err.to_string(),
        r#"srs: doc.md:3: malformed requirement id "rec-1""#
    );

    let err = parse(
        doc(),
        "| ID | Requirement | Traces |\n|---|---|---|\n| REC-01 | x | I11 |\n",
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        r#"srs: doc.md:3: REC-01: malformed trace "I11""#
    );
}

#[test]
fn parse_tolerates_short_rows() {
    let got = parse(
        doc(),
        "| ID | Requirement | Traces |\n|---|---|---|\n| REC-01 |\n",
    )
    .unwrap();

    assert_eq!(got.len(), 1);
    assert!(got[0].text.is_empty());
}

#[test]
fn parse_accepts_the_lane_families() {
    let body = "| ID | Pri | Requirement | Ver | Traces |\n|---|---|---|---|---|\n\
                    | LANE-01 | P0 | Lanes. | T | I1 |\n| VIEW-19 | P2 | Views. | T | I6 |\n\
                    | OWN-22 | P1 | Owner acts. | T | I2 |\n| PEER-11 | P2 | Peers. | T | I5 |\n";

    let ids: Vec<String> = parse(doc(), body)
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
    write("a.md", REQ_TABLE);
    write("b.md", "| ID | Requirement |\n|---|---|\n| SEC-01 | x |\n");
    write(
        "notes.txt",
        "| ID | Requirement |\n|---|---|\n| SEC-01 | x |\n",
    );

    assert_eq!(load(dir.path()).unwrap().len(), 4);

    write(
        "c.md",
        "| ID | Requirement |\n|---|---|\n| SEC-01 | again |\n",
    );
    let err = load(dir.path()).unwrap_err().to_string();
    assert!(err.contains("c.md:3: SEC-01 already defined at"), "{err}");
}

#[test]
fn load_reports_parse_list_and_read_errors() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.md"),
        "| ID | Requirement |\n|---|---|\n| X |\n",
    )
    .unwrap();
    assert!(
        load(dir.path())
            .unwrap_err()
            .to_string()
            .contains("malformed requirement id")
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

## REC-02

- Pri: P1
- Ver: T
- Traces: I1, I10

Link | parent.

| ID | Pri | Requirement | Ver | Traces |
|---|---|---|---|---|
| REC-03 | P2 | Index. | T | — |

## REC-04

- Pri: P0
- Ver: T
- Traces: —

Recall
whole.
";

#[test]
fn parse_reads_a_block_as_the_row_it_replaces() {
    let rows = parse(doc(), REQ_TABLE).unwrap();
    let got = parse(doc(), REQ_BLOCKS).unwrap();

    assert_eq!(
        got[0],
        Requirement {
            line: 3,
            ..rows[1].clone()
        }
    );
}

#[test]
fn parse_returns_rows_and_blocks_in_the_documents_order() {
    let got: Vec<(String, usize)> = parse(doc(), REQ_BLOCKS)
        .unwrap()
        .into_iter()
        .map(|r| (r.id, r.line))
        .collect();

    assert_eq!(
        got,
        [
            ("REC-02".into(), 3),
            ("REC-03".into(), 13),
            ("REC-04".into(), 15)
        ]
    );
}

#[test]
fn parse_reads_a_block_of_any_table_s_fields() {
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
fn load_rejects_a_block_that_redefines_a_row() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.md"), REQ_TABLE).unwrap();
    fs::write(
        dir.path().join("b.md"),
        "## REC-01\n\n- Pri: P0\n\nAgain.\n",
    )
    .unwrap();

    let err = load(dir.path()).unwrap_err().to_string();

    assert!(err.contains("b.md:1: REC-01 already defined at"), "{err}");
}
