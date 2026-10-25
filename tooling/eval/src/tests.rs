use serde_json::json;
use testkit::TempDir;

use super::*;
use crate::dataset::Kind;
use crate::lines::Session;

/// A short run: a value planted before one compaction, and a question
/// whose value the run never held.
pub(crate) fn short_run(templates: &Templates) -> Run {
    let mut s = Session::new(templates, "5eed0000-0000-4000-a000-000000000000");
    s.prompt("Attach a debugger to ledger.");
    s.reply("The ledger service listens for the debugger on debug port 41234; attach there.");
    s.compact("Investigated ledger debug port.");
    s.prompt("Is ledger healthy?");
    s.reply("ledger reports healthy; the mailer debug port is unknown.");
    let session = s.id().to_owned();
    let (lines, boundary) = s.into_lines();
    let fact = |component: &str, value: &str| Fact {
        kind: Kind::DebugPort,
        component: component.to_owned(),
        value: value.to_owned(),
        depth: 1,
    };

    Run {
        session,
        lines,
        boundary,
        facts: vec![fact("ledger", "41234"), fact("mailer", "50505")],
    }
}

pub(crate) fn fixture_templates() -> Templates {
    templates(&testkit::repo_root()).unwrap()
}

#[test]
fn the_fixture_s_shapes_are_read_from_the_checkout_and_a_missing_one_is_named() {
    fixture_templates();
    let err = templates(std::path::Path::new("/nonexistent")).unwrap_err();
    assert!(err.starts_with("read /nonexistent/testdata/"), "{err}");
}

#[test]
fn a_planted_value_is_recalled_through_the_tools_and_one_never_held_is_not() {
    let dir = TempDir::new().unwrap();
    let run = short_run(&fixture_templates());

    let evaluated = evaluate(std::slice::from_ref(&run), 4, dir.path()).unwrap();

    assert_eq!(evaluated.unparsed, 0);
    let [found, missing] = evaluated.outcomes.as_slice() else {
        panic!("{:?}", evaluated.outcomes)
    };
    assert_eq!(found.question, "ledger debug port");
    assert_eq!((found.seed, found.depth, found.in_context), (4, 1, false));
    assert!(found.rank.is_some(), "{found:?}");
    assert_eq!(missing.rank, None);
}

#[test]
fn a_value_left_in_the_context_after_the_last_compaction_is_counted_as_held() {
    struct Nothing;
    impl Tools for Nothing {
        fn handle(&mut self, _: &str) -> Option<String> {
            Some(
                json!({"result": {"isError": false, "content": [{"text": "{\"items\":[]}"}]}})
                    .to_string(),
            )
        }
    }
    let run = short_run(&fixture_templates());
    let held = ask(&mut Nothing, 1, "w", &run.facts[0], &run.lines).unwrap();
    assert!(held.in_context);
    assert_eq!(held.rank, None);
}

#[test]
fn a_home_whose_transcript_cannot_be_written_fails_the_evaluation() {
    let dir = TempDir::new().unwrap();
    let gone = dir.path().join("gone");
    let run = short_run(&fixture_templates());
    let err = evaluate(&[run], 1, &gone).unwrap_err();
    assert!(err.starts_with("write "), "{err}");
}
