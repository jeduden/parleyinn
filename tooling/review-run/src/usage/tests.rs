use review_eval::component::Kind;
use review_eval::log::{self, RunLog};

use super::*;

fn run(name: &str, log: RunLog) -> AgentRun {
    AgentRun {
        name: name.to_owned(),
        kind: Kind::Specialist,
        failure: None,
        log,
        attempts: 1,
        parts: 1,
        failed_parts: Vec::new(),
        reused: false,
        from_store: false,
        answer_key: None,
        store_failed: false,
    }
}

#[test]
fn each_agent_s_usage_sums_its_models_the_most_cached_first() {
    let text = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"a","model":"claude-haiku-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":2,"cache_read_input_tokens":3,"output_tokens":4}}}
{"type":"assistant","parent_tool_use_id":null,"message":{"id":"b","model":"claude-opus-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":20,"cache_read_input_tokens":30,"output_tokens":40}}}
{"type":"result","subtype":"success","result":"ok","total_cost_usd":0.5}
"#;
    let runs = vec![
        run("lang-rust", RunLog::default()),
        run("review-code", log::parse(text)),
        run("lang-json", RunLog::default()),
    ];

    let u = usage(&runs, None);

    assert_eq!(
        u[0],
        AgentUsage {
            agent: "review-code".to_owned(),
            calls: 2,
            fresh: 11,
            cache_write: 22,
            cache_read: 33,
            output: 44,
            attempts: 1,
            parts: 1,
            cost_usd: 0.5,
            reused: false,
            answer_key: None,
            answer_store: false,
            resumed_from: None,
        }
    );
    assert_eq!(u[1].agent, "lang-json");
    assert_eq!(u[2].agent, "lang-rust");
    assert!((total(&u) - 0.5).abs() < f64::EPSILON);
    let json = serde_json::to_value(&u[0]).unwrap();
    for key in [
        "agent",
        "calls",
        "fresh",
        "cache_write",
        "cache_read",
        "output",
        "attempts",
        "parts",
        "cost_usd",
    ] {
        assert!(json.get(key).is_some(), "{key}");
    }
    // A full run's usage reads as it did before a run could resume.
    for key in ["reused", "resumed_from"] {
        assert!(json.get(key).is_none(), "{key}");
    }
}

#[test]
fn a_specialist_run_in_parts_is_one_entry_counting_its_parts_and_attempts() {
    let mut split = run("test-engineer-unit", RunLog::default());
    split.parts = 5;
    split.attempts = 6;

    let u = usage(&[split], None);

    assert_eq!(u.len(), 1);
    assert_eq!((u[0].parts, u[0].attempts), (5, 6));
}

#[test]
fn a_reused_agent_costs_nothing_and_names_the_run_it_came_from() {
    let text =
        "{\"type\":\"result\",\"subtype\":\"success\",\"result\":\"ok\",\"total_cost_usd\":0.5}\n";
    let mut reused = run("lang-rust", log::parse(text));
    reused.reused = true;
    let runs = vec![reused, run("review-code", log::parse(text))];

    let u = usage(&runs, Some("target/review-local/review"));

    let by = |name: &str| u.iter().find(|e| e.agent == name).unwrap();
    assert!(by("lang-rust").reused);
    assert!(by("lang-rust").cost_usd.abs() < f64::EPSILON);
    assert_eq!(
        by("lang-rust").resumed_from.as_deref(),
        Some("target/review-local/review")
    );
    assert!(!by("review-code").reused);
    assert_eq!(by("review-code").resumed_from, None);
    assert!((total(&u) - 0.5).abs() < f64::EPSILON);
    let json = serde_json::to_value(by("lang-rust")).unwrap();
    assert_eq!(json["reused"], true);
    assert_eq!(json["resumed_from"], "target/review-local/review");
}
