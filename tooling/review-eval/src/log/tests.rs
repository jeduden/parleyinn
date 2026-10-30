use super::*;

const LOG: &str = r#"{"type":"system","subtype":"init","model":"claude-haiku-5-5"}
{"type":"assistant","message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":5},"content":[{"type":"text","text":"reading"}]}}
{"type":"assistant","message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":5},"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"review/bundle.md"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"..."}]}}
{"type":"assistant","message":{"id":"m2","model":"claude-haiku-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":1100,"output_tokens":50},"content":[{"type":"tool_use","id":"t2","name":"Write","input":{"file_path":"review/reports/lang-rust.md","content":"outcome: sound"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","content":"ok"}]}}
{"type":"assistant","message":{"id":"m3","model":"claude-haiku-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":1200,"output_tokens":8},"content":[{"type":"text","text":"outcome: sound"}]}}
{"type":"result","subtype":"success","is_error":false,"result":"outcome: sound\nreview/reports/lang-rust.md","total_cost_usd":0.01,"num_turns":3,"duration_ms":4200,"modelUsage":{"claude-haiku-5-5":{"inputTokens":12,"outputTokens":63,"cacheReadInputTokens":3300,"cacheCreationInputTokens":100,"costUSD":0.01}}}
"#;

#[test]
fn a_log_reads_its_result_usage_and_writes() {
    let log = parse(LOG);

    assert!(log.answered);
    assert_eq!(log.subtype, "success");
    assert_eq!(log.answer, "outcome: sound\nreview/reports/lang-rust.md");
    assert!((log.cost - 0.01).abs() < 1e-9);
    assert_eq!(log.turns, 3);
    assert_eq!(log.duration_ms, 4200);
    assert_eq!(log.resolved_model, "claude-haiku-5-5");
    assert_eq!(log.writes, vec!["review/reports/lang-rust.md"]);
    assert_eq!(log.first_write_turn, Some(2));
}

#[test]
fn a_log_counts_each_call_once_with_its_usage() {
    let log = parse(LOG);

    let usage = &log.usage["claude-haiku-5-5"];
    // m1's two blocks count once.
    assert_eq!(usage.calls, 3);
    assert_eq!(usage.fresh, 12);
    assert_eq!(usage.cache_write, 100);
    assert_eq!(usage.cache_read, 3300);
    assert_eq!(usage.output, 63);
    assert!((usage.cost - 0.01).abs() < 1e-9);
}

#[test]
fn a_subagent_s_calls_count_their_usage_and_writes_but_not_the_main_thread_s() {
    let text = r#"{"type":"assistant","message":{"id":"m1","model":"claude-sonnet-5-5","usage":{"input_tokens":1},"content":[{"type":"tool_use","id":"a1","name":"Agent","input":{"subagent_type":"lang-rust"}},{"type":"tool_use","id":"a2","name":"Agent","input":{"subagent_type":"lang-toml"}},{"type":"tool_use","id":"a3","name":"Task","input":{"subagent_type":"review-code"}}]}}
{"type":"assistant","parent_tool_use_id":"a1","message":{"id":"s1","model":"claude-haiku-5-5","usage":{"input_tokens":3,"output_tokens":4},"content":[{"type":"tool_use","id":"w1","name":"Write","input":{"file_path":"review/reports/lang-rust.md"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"a1","content":"outcome: sound"},{"type":"tool_result","tool_use_id":"a2","is_error":true,"content":"failed"},{"type":"tool_result","tool_use_id":"a3","content":[{"type":"text","text":"fine"}]}]}}
{"type":"result","subtype":"success","result":"{}","total_cost_usd":0.5,"num_turns":2,"duration_ms":10,"modelUsage":{"claude-sonnet-5-5":{"costUSD":0.4},"claude-haiku-5-5":{"costUSD":0.1},"claude-opus-5-5":{"inputTokens":7,"outputTokens":8,"cacheReadInputTokens":9,"cacheCreationInputTokens":10,"costUSD":0.0}}}
"#;

    let log = parse(text);

    assert_eq!(log.resolved_model, "claude-sonnet-5-5");
    // A subagent's write is not the main thread's first write.
    assert_eq!(log.writes, vec!["review/reports/lang-rust.md"]);
    assert_eq!(log.first_write_turn, None);
    assert_eq!(log.usage["claude-haiku-5-5"].calls, 1);
    assert!((log.usage["claude-haiku-5-5"].cost - 0.1).abs() < 1e-9);
    // A model the calls did not show takes its usage from the result.
    let opus = &log.usage["claude-opus-5-5"];
    assert_eq!((opus.calls, opus.fresh, opus.output), (0, 7, 8));
    assert_eq!((opus.cache_read, opus.cache_write), (9, 10));
}

#[test]
fn a_result_that_is_an_error_names_the_harness_s_refusal() {
    let refused = r#"{"type":"result","subtype":"success","is_error":true,"result":"You've hit your weekly limit · resets 10am (UTC)\nmore","total_cost_usd":0,"num_turns":1}"#;

    let log = parse(refused);

    assert!(!log.answered);
    assert_eq!(
        log.subtype,
        "error: You've hit your weekly limit · resets 10am (UTC)"
    );
}

#[test]
fn a_log_without_a_result_did_not_answer() {
    let log = parse("not json\n{\"type\":\"system\"}\n");

    assert!(!log.answered);
    assert_eq!(log.subtype, "none");
    assert_eq!(log.answer, "");
    assert_eq!(log.turns, 0);
    assert_eq!(log.resolved_model, "");
}

#[test]
fn a_result_that_errs_did_not_answer() {
    let cut = r#"{"type":"result","subtype":"error_max_turns","is_error":true,"result":"","num_turns":20}"#;
    let log = parse(cut);

    assert!(!log.answered);
    assert_eq!(log.subtype, "error_max_turns");
    assert_eq!(log.turns, 20);

    let unknown = parse(r#"{"type":"result","is_error":false}"#);
    assert_eq!(unknown.subtype, "unknown");
    assert!(!unknown.answered);
}

#[test]
fn a_write_without_a_path_is_no_write() {
    let log = parse(
        r#"{"type":"assistant","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Write","input":{}}]}}"#,
    );

    assert!(log.writes.is_empty());
    assert_eq!(log.first_write_turn, None);
}

#[test]
fn a_call_without_a_model_or_usage_counts_as_unknown() {
    let log = parse(r#"{"type":"assistant","message":{"id":"m1","content":[]}}"#);

    assert_eq!(log.resolved_model, "unknown");
    assert_eq!(log.usage["unknown"].calls, 1);
    assert_eq!(log.usage["unknown"].fresh, 0);
}
