use super::*;

fn row(cells: &[&str], line: usize) -> Row {
    Row {
        cells: cells.iter().map(|c| (*c).to_owned()).collect(),
        line,
    }
}

#[test]
fn tables_reads_header_and_rows_with_lines() {
    let got = tables("intro\n\n| A | B |\n|---|:--:|\n| 1 | 2 |\n| 3 | 4 |\n\nafter\n");

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].header, ["A", "B"]);
    assert_eq!(got[0].rows, [row(&["1", "2"], 5), row(&["3", "4"], 6)]);
}

#[test]
fn tables_skips_tables_inside_fences() {
    let got =
        tables("```md\n| A |\n|---|\n| 1 |\n```\n~~~\n| B |\n|---|\n~~~\n| C |\n|---|\n| 2 |\n");

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].header, ["C"]);
}

#[test]
fn tables_keeps_a_fence_open_until_its_own_marker() {
    assert!(tables("````\n```\n| A |\n|---|\n````\n").is_empty());
    assert!(tables("```\n~~~\n| A |\n|---|\n```\n").is_empty());
}

#[test]
fn tables_needs_a_delimiter_row() {
    assert!(tables("| A |\n| 1 |\n").is_empty());
    assert!(tables("| A |\n").is_empty());
    assert!(tables("| A |\nnot a row\n").is_empty());
}

#[test]
fn tables_reads_two_tables_in_a_row() {
    let got = tables("| A |\n|---|\n| 1 |\ntext\n| B |\n|---|\n");

    assert_eq!(got.len(), 2);
    assert_eq!(got[1].header, ["B"]);
    assert!(got[1].rows.is_empty());
}

#[test]
fn fence_marker_reads_the_opening_run() {
    assert_eq!(fence_marker("```go"), Some("```"));
    assert_eq!(fence_marker("~~~~"), Some("~~~~"));
    assert_eq!(fence_marker("``inline``"), None);
    assert_eq!(fence_marker(""), None);
    assert_eq!(fence_marker("| a |"), None);
}

#[test]
fn fence_after_opens_and_closes_on_its_own_marker() {
    assert_eq!(fence_after(None, "text"), None);
    assert_eq!(fence_after(None, "  ```rust"), Some("```"));
    assert_eq!(fence_after(Some("```"), "text"), Some("```"));
    assert_eq!(fence_after(Some("```"), "````"), None);
    assert_eq!(fence_after(Some("````"), "```"), Some("````"));
    assert_eq!(fence_after(Some("```"), "~~~"), Some("```"));
}

#[test]
fn opens_table_wants_a_pipe_row_with_a_delimiter_row_under_it() {
    assert!(opens_table("  | A |", Some("|---|")));
    assert!(!opens_table("| A |", Some("| B |")));
    assert!(!opens_table("| A |", None));
    assert!(!opens_table("A |", Some("|---|")));
}

#[test]
fn is_delimiter_row_wants_dashes_in_every_cell() {
    assert!(is_delimiter_row("| --- | :-: |"));
    assert!(!is_delimiter_row("| --- | x |"));
    assert!(!is_delimiter_row("| --- | : |"));
    assert!(!is_delimiter_row("---"));
}

#[test]
fn split_row_unescapes_pipes() {
    assert_eq!(split_row(r"| a | b\|c | d |"), ["a", "b|c", "d"]);
    assert_eq!(split_row(r"| a | x\\|"), ["a", r"x\|"]);
    assert_eq!(split_row("| a | b"), ["a", "b"]);
}
