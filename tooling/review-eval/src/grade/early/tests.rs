use super::*;

const KEPT: &str = "docs/x.md:3 — blocking — it breaks ENG-01 — lang-rust";
const DROPPED: &str = "dropped — docs/x.md:9 — main has it — lang-rust";
const NONE: [&str; 0] = [];

fn texts(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|t| (*t).to_owned()).collect()
}

#[test]
fn a_weighing_line_the_final_answer_does_not_hold_is_a_slip() {
    let answer = format!("{KEPT}\nI read the whole diff.");
    let early = texts(&[DROPPED, &answer]);

    assert_eq!(
        slips(&early, &answer),
        [format!(
            "stated before its final answer, which does not hold it: {DROPPED}"
        )]
    );
}

#[test]
fn a_final_answer_that_holds_every_line_is_no_slip() {
    let answer = format!("{KEPT}\n{DROPPED}\nI read the whole diff.");

    assert_eq!(slips(&texts(&[&answer]), &answer), NONE);
    assert_eq!(
        slips(&texts(&[&format!("  {KEPT}"), &answer]), &answer),
        NONE
    );
}

#[test]
fn a_line_the_final_answer_holds_indented_is_no_slip() {
    let answer = format!("  {KEPT}\nI read the whole diff.");

    assert_eq!(slips(&texts(&[KEPT]), &answer), NONE);
}

#[test]
fn a_working_note_before_the_final_answer_is_no_slip() {
    let answer = "none — nothing to add — review-code";
    let early = texts(&["docs/x.md:3 — verified — the test takes it", "Reading on."]);

    assert_eq!(slips(&early, answer), NONE);
}

#[test]
fn a_line_stated_twice_is_one_slip() {
    let early = texts(&[DROPPED, DROPPED]);

    assert_eq!(
        slips(&early, "none — nothing to add — review-code").len(),
        1
    );
}

#[test]
fn a_run_with_no_final_answer_has_no_slip_here() {
    assert_eq!(slips(&texts(&[DROPPED]), " \n"), NONE);
}
