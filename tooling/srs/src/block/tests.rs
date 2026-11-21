use super::*;

fn field(label: &str, value: &str) -> (String, String) {
    (label.to_owned(), value.to_owned())
}

#[test]
fn blocks_reads_a_heading_its_fields_and_its_text() {
    let body = "# Doc\n\nIntro.\n\n## REC-01\n\n- Pri: P0\n- Ver: T\n- Traces: I1, I6\n\n\
                Ingest every event.\nEach one MUST\nread whole.\n\nA second paragraph.\n\n\
                ## Notes\n\nNot a requirement.\n";

    assert_eq!(
        blocks(body),
        [Block {
            id: "REC-01".into(),
            fields: vec![
                field("Pri", "P0"),
                field("Ver", "T"),
                field("Traces", "I1, I6")
            ],
            text: "Ingest every event. Each one MUST read whole. A second paragraph.".into(),
            line: 5,
        }]
    );
}

#[test]
fn blocks_reads_a_field_wrapped_onto_indented_lines_whole() {
    let got = blocks(
        "## ASM-01\n\n- Evidence so far: S1 showed\n  it twice.\n- Verified in: S1\n\nHooks carry ids.\n",
    );

    assert_eq!(
        got[0].fields,
        [
            field("Evidence so far", "S1 showed it twice."),
            field("Verified in", "S1")
        ]
    );
    assert_eq!(got[0].text, "Hooks carry ids.");
}

#[test]
fn blocks_skips_a_heading_inside_a_fence() {
    assert!(blocks("```markdown\n## REC-01\n\n- Pri: P0\n\nText.\n```\n").is_empty());
    assert!(blocks("~~~~\n```\n## REC-01\n~~~~\n").is_empty());
}

#[test]
fn a_heading_that_is_no_requirement_id_ends_the_block_before_it() {
    let got = blocks(
        "## REC-01\n\n- Pri: P0\n\nText.\n\n## REC-1\n\nMore.\n### SEC-02\n\n- Pri: P1\n\nOther.\n",
    );

    assert_eq!(got.len(), 2);
    assert_eq!(got[0].text, "Text.");
    assert_eq!((got[1].id.as_str(), got[1].line), ("SEC-02", 10));
}

#[test]
fn blocks_reads_an_item_without_a_colon_as_its_label_and_no_list_as_no_fields() {
    let got = blocks("## REC-01\n\n- Pri P0\n- Ver:\n\nText.\n## REC-02\n\nText only.\n");

    assert_eq!(got[0].fields, [field("Pri P0", ""), field("Ver", "")]);
    assert!(got[1].fields.is_empty());
    assert_eq!(got[1].text, "Text only.");
}

#[test]
fn blocks_ends_the_field_list_at_its_first_other_line() {
    let got = blocks("## REC-01\n- Pri: P0\nText at once.\n- Ver: T\n");

    assert_eq!(got[0].fields, [field("Pri", "P0")]);
    assert_eq!(got[0].text, "Text at once. - Ver: T");
}

#[test]
fn heading_reads_an_atx_heading_and_nothing_else() {
    assert_eq!(heading("## ENG-28"), Some("ENG-28"));
    assert_eq!(heading("###### x  "), Some("x"));
    assert_eq!(heading("#"), Some(""));
    assert_eq!(heading("####### x"), None);
    assert_eq!(heading("#x"), None);
    assert_eq!(heading("text"), None);
}

#[test]
fn a_table_ends_the_block_before_it() {
    let got = blocks("## REC-01\n\n- Pri: P0\n\nText.\n\n| A |\n|---|\n| 1 |\n\nAfter.\n");

    assert_eq!(got[0].text, "Text.");
    assert_eq!(got.len(), 1);
}

#[test]
fn a_fence_inside_a_block_is_part_of_its_text() {
    let got = blocks("## REC-01\n\n- Pri: P0\n\nText.\n```\n## REC-02\n| A |\n```\nMore.\n");

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].text, "Text. ``` ## REC-02 | A | ``` More.");
}
