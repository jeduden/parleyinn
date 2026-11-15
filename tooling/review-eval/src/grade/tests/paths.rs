//! A report line that names a file of the case by a tail of its path.

use super::*;

#[test]
fn a_finding_at_a_tail_of_the_case_s_path_is_the_finding_the_case_wants() {
    let mut case = case(
        "domain-model-harness-facts",
        Expect {
            finding: vec![Expected {
                mentions: vec!["ASM-99".into()],
                ..wanted("docs/domain-model/harness-facts.md")
            }],
            stated: vec![wanted("docs/domain-model/harness-facts.md")],
            ..Expect::default()
        },
    );
    case.edits = vec![crate::case::Edit::Remove {
        file: "docs/domain-model/harness-facts.md".into(),
    }];
    let line =
        "harness-facts.md:19 — if not addressed: it cites ASM-99, which nothing defines — ASM-03";
    let log = stated_log(&[line], "outcome: findings — one id is missing");

    let graded = grade(&component(Kind::Specialist, "x", true), &case, &log);

    assert_eq!(graded.failures, Vec::<String>::new());
}
