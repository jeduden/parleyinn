use review_eval::component::Kind;
use review_eval::log::{self, RunLog};

use super::*;

fn run(name: &str, log: RunLog) -> AgentRun {
    AgentRun {
        name: name.to_owned(),
        kind: Kind::Specialist,
        failure: None,
        log,
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

    let u = usage(&runs);

    assert_eq!(
        u[0],
        AgentUsage {
            agent: "review-code".to_owned(),
            calls: 2,
            fresh: 11,
            cache_write: 22,
            cache_read: 33,
            output: 44,
            cost_usd: 0.5,
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
        "cost_usd",
    ] {
        assert!(json.get(key).is_some(), "{key}");
    }
}
