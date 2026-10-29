use super::*;

fn line(
    component: &str,
    model: &str,
    cases_passed: usize,
    runs_passed: usize,
    runs: usize,
    cost: f64,
) -> Line {
    Line {
        component: component.to_owned(),
        model: model.to_owned(),
        resolved: format!("claude-{model}-5-5"),
        cases_passed,
        cases: 4,
        runs_passed,
        runs,
        cost,
        turns: 5.0,
        seconds: 30.0,
        hash: "h".to_owned(),
    }
}

#[test]
fn a_candidate_stands_by_cases_then_rate_then_cost() {
    let published = vec![
        line("a", "haiku", 3, 9, 12, 1.2),
        line("b", "haiku", 4, 12, 12, 1.2),
        line("c", "haiku", 4, 12, 12, 1.2),
        line("d", "haiku", 4, 12, 12, 1.2),
        line("e", "haiku", 4, 12, 12, 1.2),
        line("f", "haiku", 4, 12, 12, 1.2),
        line("g", "haiku", 4, 12, 12, 0.0),
    ];
    let candidate = vec![
        line("a", "sonnet", 4, 12, 12, 2.0),
        line("b", "sonnet", 3, 9, 12, 0.1),
        line("c", "sonnet", 4, 10, 12, 1.2),
        line("d", "sonnet", 4, 12, 12, 0.6),
        line("e", "sonnet", 4, 12, 12, 2.4),
        line("f", "sonnet", 4, 12, 12, 1.3),
        line("g", "sonnet", 4, 12, 12, 1.0),
        line("h", "sonnet", 2, 4, 6, 1.0),
    ];

    let got = compare(&published, &candidate);

    let standings: Vec<(&str, Standing, &str)> = got
        .iter()
        .map(|c| (c.component.as_str(), c.standing, c.why.as_str()))
        .collect();
    assert_eq!(
        standings,
        vec![
            (
                "a",
                Standing::Better,
                "passes 4 of 4 cases, the published model 3"
            ),
            (
                "b",
                Standing::Worse,
                "passes 3 of 4 cases, the published model 4"
            ),
            (
                "c",
                Standing::Worse,
                "the same cases, but the runs' pass rate fell from 100% to 83%"
            ),
            (
                "d",
                Standing::Better,
                "the same cases at 50% of the cost per run"
            ),
            (
                "e",
                Standing::Worse,
                "the same cases at 200% of the cost per run"
            ),
            (
                "f",
                Standing::Equivalent,
                "the same cases at about the same cost"
            ),
            (
                "g",
                Standing::Equivalent,
                "the same cases at about the same cost"
            ),
            ("h", Standing::New, "no published line"),
        ]
    );
    assert_eq!(counts(&got), [2, 2, 3, 1]);
    assert_eq!(
        got[0].published.as_ref().map(|l| l.model.as_str()),
        Some("haiku")
    );
    assert_eq!(got[7].published, None);
}

#[test]
fn the_comparison_renders_as_markdown() {
    let got = compare(
        &[line("a", "haiku", 3, 9, 12, 1.2)],
        &[line("a", "sonnet", 4, 12, 12, 2.4)],
    );

    let md = render(&got);

    assert!(
        md.starts_with("## Model comparison\n\n1 better, 0 equivalent, 0 worse, 0 new.\n"),
        "{md}"
    );
    assert!(
        md.contains("| a | haiku (claude-haiku-5-5): 3/4 cases, 9/12 runs, $0.10 per run | sonnet (claude-sonnet-5-5): 4/4 cases, 12/12 runs, $0.20 per run | better | passes 4 of 4 cases, the published model 3 |"),
        "{md}"
    );
    let new = render(&compare(&[], &[line("z", "opus", 1, 1, 1, 1.0)]));
    assert!(new.contains("| z | none | opus"), "{new}");
    assert_eq!(Standing::New.word(), "new");
    assert_eq!(Standing::Better.word(), "better");
    assert_eq!(Standing::Equivalent.word(), "equivalent");
    assert_eq!(Standing::Worse.word(), "worse");
}
