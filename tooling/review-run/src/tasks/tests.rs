use super::*;
use crate::parts::Part;
use crate::plan::parse_dispatch;

fn plan(dispatch: &str) -> Plan {
    Plan {
        class: "standard".to_owned(),
        touched: 1,
        dispatch: parse_dispatch(dispatch).unwrap(),
    }
}

fn part(name: &str, index: usize) -> Unit {
    Unit {
        name: name.to_owned(),
        kind: review_eval::component::Kind::Specialist,
        part: Some(Part {
            index,
            count: 2,
            diff: String::new(),
            note: String::new(),
        }),
    }
}

#[test]
fn each_generalist_waits_for_its_own_specialists_alone() {
    let p = plan(
        r#"{"review-code": ["lang-rust", "test-engineer-unit"], "review-spec": ["lang-markdown"]}"#,
    );
    let units = vec![
        Unit::whole("lang-markdown".to_owned()),
        part("lang-rust", 1),
        part("lang-rust", 2),
    ];
    let generalists = vec![
        Unit::whole("review-code".to_owned()),
        Unit::whole("review-spec".to_owned()),
    ];

    let g = graph(&p, units, generalists);

    let kinds: Vec<String> = g
        .tasks
        .iter()
        .map(|t| match t {
            Task::Unit(u) => u.label(),
            Task::Merge(name) => format!("merge {name}"),
            Task::Generalist(u) => u.label(),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "lang-markdown",
            "lang-rust part 1 of 2",
            "lang-rust part 2 of 2",
            "merge lang-rust",
            "review-code",
            "review-spec"
        ]
    );
    // test-engineer-unit was kept from an earlier run: nothing to wait for.
    assert_eq!(
        g.waits,
        [vec![], vec![], vec![], vec![1, 2], vec![3], vec![0]]
    );
    let last: Vec<(&str, usize)> = g.last.iter().map(|(n, a)| (n.as_str(), *a)).collect();
    assert_eq!(
        last,
        [
            ("lang-markdown", 0),
            ("lang-rust", 3),
            ("review-code", 4),
            ("review-spec", 5)
        ]
    );
}

#[test]
fn the_breaker_counts_a_run_that_failed_and_not_one_that_only_left_lines_unread() {
    let run = |failure: Option<&str>| {
        AgentRun::of(
            "lang-rust".to_owned(),
            review_eval::component::Kind::Specialist,
            failure.map(str::to_owned),
            review_eval::log::RunLog::default(),
            1,
        )
    };
    let unread = AgentRun {
        partial: vec![(None, "lines 1-9 unread".to_owned())],
        ..run(None)
    };
    let stated_nothing = AgentRun {
        stated_nothing: vec![(None, "error_max_turns".to_owned())],
        ..run(None)
    };
    let unreadable = AgentRun {
        unreadable: vec![reading::ReportLine {
            agent: "lang-rust".to_owned(),
            part: None,
            line: 1,
            text: "a.rs:9 — not a form".to_owned(),
            why: "no form".to_owned(),
        }],
        ..run(None)
    };

    assert!(!tips(&run(None)));
    assert!(!tips(&unread));
    assert!(tips(&run(Some("claude: not found"))));
    assert!(tips(&stated_nothing));
    assert!(tips(&unreadable));
}
