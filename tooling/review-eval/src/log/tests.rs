use super::*;

const LOG: &str = r#"{"type":"system","subtype":"init","model":"claude-haiku-5-5"}
{"type":"assistant","message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":5},"content":[{"type":"text","text":"reading"}]}}
{"type":"assistant","message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":5},"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"review/bundle.md"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"..."}]}}
{"type":"assistant","message":{"id":"m2","model":"claude-haiku-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":1100,"output_tokens":50},"content":[{"type":"text","text":"a.rs:1 — if not addressed: x — I2"},{"type":"tool_use","id":"t2","name":"Write","input":{"file_path":"review/reports/lang-rust.md","content":"outcome: sound"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","content":"ok"}]}}
{"type":"assistant","message":{"id":"m3","model":"claude-haiku-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":1200,"output_tokens":8},"content":[{"type":"text","text":"outcome: findings — x"},{"type":"text"}]}}
{"type":"result","subtype":"success","is_error":false,"result":"outcome: findings — x","total_cost_usd":0.01,"num_turns":3,"duration_ms":4200,"modelUsage":{"claude-haiku-5-5":{"inputTokens":12,"outputTokens":63,"cacheReadInputTokens":3300,"cacheCreationInputTokens":100,"costUSD":0.01}}}
"#;

#[test]
fn a_log_reads_its_result_usage_texts_and_writes() {
    let log = parse(LOG);

    assert!(log.answered);
    assert_eq!(log.subtype, "success");
    assert_eq!(log.answer, "outcome: findings — x");
    assert!((log.cost - 0.01).abs() < 1e-9);
    assert_eq!(log.turns, 3);
    assert_eq!(log.duration_ms, 4200);
    assert_eq!(log.resolved_model, "claude-haiku-5-5");
    assert_eq!(
        log.texts,
        vec![
            "reading",
            "a.rs:1 — if not addressed: x — I2",
            "outcome: findings — x"
        ]
    );
    assert_eq!(log.writes, vec!["review/reports/lang-rust.md"]);
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
fn a_subagent_s_calls_count_their_usage_and_writes_but_not_the_main_thread_s_text() {
    let text = r#"{"type":"assistant","message":{"id":"m1","model":"claude-sonnet-5-5","usage":{"input_tokens":1},"content":[{"type":"tool_use","id":"a1","name":"Agent","input":{"subagent_type":"lang-rust"}},{"type":"tool_use","id":"a2","name":"Agent","input":{"subagent_type":"lang-toml"}},{"type":"tool_use","id":"a3","name":"Task","input":{"subagent_type":"review-code"}}]}}
{"type":"assistant","parent_tool_use_id":"a1","message":{"id":"s1","model":"claude-haiku-5-5","usage":{"input_tokens":3,"output_tokens":4},"content":[{"type":"text","text":"b.rs:2 — if not addressed: y — I2"},{"type":"tool_use","id":"w1","name":"Write","input":{"file_path":"review/reports/lang-rust.md"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"a1","content":"outcome: sound"},{"type":"tool_result","tool_use_id":"a2","is_error":true,"content":"failed"},{"type":"tool_result","tool_use_id":"a3","content":[{"type":"text","text":"fine"}]}]}}
{"type":"result","subtype":"success","result":"{}","total_cost_usd":0.5,"num_turns":2,"duration_ms":10,"modelUsage":{"claude-sonnet-5-5":{"costUSD":0.4},"claude-haiku-5-5":{"costUSD":0.1},"claude-opus-5-5":{"inputTokens":7,"outputTokens":8,"cacheReadInputTokens":9,"cacheCreationInputTokens":10,"costUSD":0.0}}}
"#;

    let log = parse(text);

    assert_eq!(log.resolved_model, "claude-sonnet-5-5");
    // A subagent's text is not the main thread's: nothing it states
    // reaches the report.
    assert!(log.texts.is_empty());
    assert_eq!(log.writes, vec!["review/reports/lang-rust.md"]);
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
    assert!(!log.resulted);
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
    assert!(log.resulted, "a result came, an error");
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
}

#[test]
fn an_edit_is_a_write_and_a_read_is_none() {
    let log = parse(
        r#"{"type":"assistant","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Edit","input":{"file_path":"src/a.rs"}},{"type":"tool_use","id":"t2","name":"Read","input":{"file_path":"src/b.rs"}}]}}"#,
    );

    assert_eq!(log.writes, vec!["src/a.rs"]);
}

#[test]
fn a_call_without_a_model_or_usage_counts_as_unknown() {
    let log = parse(r#"{"type":"assistant","message":{"id":"m1","content":[]}}"#);

    assert_eq!(log.resolved_model, "unknown");
    assert_eq!(log.usage["unknown"].calls, 1);
    assert_eq!(log.usage["unknown"].fresh, 0);
}

const READS: &str = r#"{"type":"assistant","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"toolu_1","name":"Read","input":{"file_path":"review/diffs/lang-rust.diff","offset":1,"limit":400}},{"type":"tool_use","id":"toolu_2","name":"Read","input":{"file_path":"review/diffs/lang-rust.diff"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"toolu_1","is_error":false,"content":"..."},{"type":"tool_result","tool_use_id":"toolu_2","is_error":true,"content":"File content exceeds maximum allowed size"}]}}
{"type":"assistant","message":{"id":"m2","model":"m","content":[{"type":"tool_use","id":"toolu_3","name":"Read","input":{"file_path":"/w/review/diffs/lang-rust.diff","offset":401}}]}}
{"type":"assistant","parent_tool_use_id":"toolu_9","message":{"id":"s1","model":"m","content":[{"type":"tool_use","id":"toolu_4","name":"Read","input":{"file_path":"src/a.rs"}}]}}
{"type":"user","parent_tool_use_id":"toolu_9","message":{"content":[{"type":"tool_result","tool_use_id":"toolu_4","is_error":true,"content":"no"}]}}
"#;

#[test]
fn a_log_reads_each_main_thread_read_in_order_answered_only_by_a_result_that_is_no_error() {
    let log = parse(READS);

    assert_eq!(
        log.reads,
        vec![
            ReadCall {
                path: "review/diffs/lang-rust.diff".to_owned(),
                offset: Some(1),
                limit: Some(400),
                answered: true,
                bad_range: false,
            },
            ReadCall {
                path: "review/diffs/lang-rust.diff".to_owned(),
                offset: None,
                limit: None,
                answered: false,
                bad_range: false,
            },
            // No result came, as when the run was killed at its
            // deadline: the read gave the reviewer nothing.
            ReadCall {
                path: "/w/review/diffs/lang-rust.diff".to_owned(),
                offset: Some(401),
                limit: None,
                answered: false,
                bad_range: false,
            },
        ]
    );
}

#[test]
fn a_read_s_numbers_read_as_whole_numbers_or_numeric_strings_and_else_are_a_bad_range() {
    let call = |offset: Value, limit: Value| {
        let mut input = serde_json::json!({"file_path": "a"});
        for (key, value) in [("offset", offset), ("limit", limit)] {
            if !value.is_null() || key == "offset" {
                input[key] = value;
            }
        }
        let line = serde_json::json!({"type": "assistant", "message": {"id": "m1",
            "model": "m", "content": [{"type": "tool_use", "id": "t1", "name": "Read",
            "input": input}]}});
        let read = parse(&line.to_string()).reads.remove(0);
        (read.offset, read.limit, read.bad_range)
    };
    let json = |text: &str| serde_json::from_str::<Value>(text).unwrap();

    assert_eq!(call(json("3"), json("4")), (Some(3), Some(4), false));
    assert_eq!(
        call(json(r#""1500""#), json(r#"" 20 ""#)),
        (Some(1500), Some(20), false)
    );
    // An offset given as null is none given.
    assert_eq!(call(Value::Null, Value::Null), (None, None, false));
    for (offset, limit) in [
        ("1.5", "null"),
        ("-1", "null"),
        ("null", r#""many""#),
        ("null", r#""-5""#),
        ("null", "true"),
        ("[1]", "null"),
    ] {
        assert!(call(json(offset), json(limit)).2, "{offset} {limit}");
    }
}

#[test]
fn a_read_without_a_path_is_no_read_and_a_prompt_holds_no_result() {
    let log = parse(
        r#"{"type":"assistant","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Read","input":{}}]}}
{"type":"user","message":{"content":"a prompt"}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","is_error":true}]}}"#,
    );

    assert!(log.reads.is_empty());
}

/// One model call that used tokens, and no result: a run killed at its
/// deadline or crashed after the model was called.
const CALLED: &str = r#"{"type":"assistant","parent_tool_use_id":null,"message":{"id":"m1","model":"claude-haiku-5-5","usage":{"input_tokens":10,"output_tokens":5},"content":[{"type":"text","text":"src/a.rs:3 — if not addressed: a long line hides a bug — I9"}]}}"#;

/// A result that failed before any model call: no cost, one turn
/// counted, no usage.
const FREE: &str = r#"{"type":"result","subtype":"error_during_execution","is_error":true,"result":"","total_cost_usd":0,"num_turns":1}"#;

#[test]
fn a_run_is_charged_by_its_model_usage_or_its_cost_never_by_its_turns() {
    assert_eq!(
        parse(CALLED).charged().as_deref(),
        Some("it made 1 model call")
    );
    assert_eq!(parse(FREE).charged(), None);
    assert_eq!(RunLog::default().charged(), None);
    let costly = RunLog {
        cost: 0.25,
        ..RunLog::default()
    };
    assert_eq!(costly.charged().as_deref(), Some("it cost 0.2500 USD"));
    let two = parse(&format!("{CALLED}\n{}", CALLED.replace("\"m1\"", "\"m2\"")));
    assert_eq!(two.charged().as_deref(), Some("it made 2 model calls"));
    // Tokens a result counts for a model whose calls the log did not
    // show are no call it shows.
    let mut unseen = RunLog::default();
    unseen.usage.insert(
        "m".to_owned(),
        Usage {
            output: 1,
            ..Usage::default()
        },
    );
    assert_eq!(unseen.charged(), None);
}

#[test]
fn a_run_ran_when_it_answered_something_or_was_charged() {
    assert!(parse(CALLED).ran());
    assert!(!parse(FREE).ran());
    let answered = |answer: &str| RunLog {
        answered: true,
        subtype: SUCCESS.to_owned(),
        answer: answer.to_owned(),
        ..RunLog::default()
    };
    assert!(answered("outcome: none — x").ran());
    assert!(answered("outcome: none — x").succeeded());
    assert!(!answered("").succeeded());
    assert!(!parse(CALLED).succeeded());
    // A success that answered nothing and used no model is no run, for
    // the grader as for the review.
    assert!(!answered(" ").ran());
    assert!(
        RunLog {
            cost: 0.1,
            ..RunLog::default()
        }
        .ran()
    );
}
