use record::canonical;
use record::digest::Digest;
use serde_json::{Value, json};

use super::*;

const REJECTED: Occurrence = Occurrence::Degraded {
    kind: Kind::Rejected,
    counter: Counter::HookRejected,
    operation: "hook UserPromptSubmit",
    detail: "not a JSON object",
};

#[test]
fn each_kind_and_counter_has_its_name() {
    let kinds = [
        (Kind::Failed, "failed"),
        (Kind::Dropped, "dropped"),
        (Kind::Rejected, "rejected"),
        (Kind::Redacted, "redacted"),
        (Kind::TimedOut, "timed-out"),
        (Kind::Truncated, "truncated"),
        (Kind::Coalesced, "coalesced"),
    ];
    for (kind, name) in kinds {
        assert_eq!(kind.as_str(), name);
    }
    let counters = [
        (Counter::HookRejected, "hook_rejected"),
        (Counter::HookPanicked, "hook_panicked"),
        (Counter::McpPanicked, "mcp_panicked"),
        (Counter::SettingsLayerInvalid, "settings_layer_invalid"),
        (
            Counter::RepositoryConfigurationKeyIgnored,
            "repository_configuration_key_ignored",
        ),
        (Counter::TrustPolicyKeyIgnored, "trust_policy_key_ignored"),
        (
            Counter::ConfigurationKeyIgnored,
            "configuration_key_ignored",
        ),
        (Counter::ManagedPolicyInvalid, "managed_policy_invalid"),
        (Counter::PrincipalActRefused, "principal_act_refused"),
        (Counter::Redacted, "redacted"),
        (
            Counter::RedactionPatternRefused,
            "redaction_pattern_refused",
        ),
        (Counter::HookInputTooLarge, "hook_input_too_large"),
        (Counter::HookInputTooDeep, "hook_input_too_deep"),
        (
            Counter::PathOutsideAllowedRoots,
            "path_outside_allowed_roots",
        ),
        (Counter::LogLineDropped, "log_line_dropped"),
        (
            Counter::IngestRefusedSettingsInvalid,
            "ingest_refused_settings_invalid",
        ),
        (Counter::EventsUnparsed, "events_unparsed"),
        (Counter::FormatDrift, "format_drift"),
        (Counter::NodeIdentityChanged, "node_identity_changed"),
        (Counter::DeviceKeyMissing, "device_key_missing"),
        (Counter::KeyStoreUnavailable, "key_store_unavailable"),
        (
            Counter::NewTranscriptGeneration,
            "new_transcript_generation",
        ),
        (Counter::RepeatConflicts, "repeat_conflicts"),
        (Counter::RepeatOwnerChosen, "repeat_owner_chosen"),
        (Counter::RepeatHeld, "repeat_held"),
        (Counter::RepeatReleased, "repeat_released"),
        (Counter::TranscriptSourceLost, "transcript_source_lost"),
        (Counter::HookInputRejected, "hook_input_rejected"),
        (Counter::HookBudgetExceeded, "hook_budget_exceeded"),
        (Counter::HookFailed, "hook_failed"),
        (Counter::HookUnfinished, "hook_unfinished"),
        (Counter::SessionBindingNotTaken, "session_binding_not_taken"),
        (
            Counter::PromptCommitmentDropped,
            "prompt_commitment_dropped",
        ),
        (Counter::PromptMatchConflicts, "prompt_match_conflicts"),
        (
            Counter::HarnessSessionNotAttended,
            "harness_session_not_attended",
        ),
    ];
    for (counter, name) in counters {
        assert_eq!(counter.as_str(), name);
    }
}

#[test]
fn each_counter_of_the_mcp_server_has_its_name() {
    let counters = [
        (Counter::IngestRemainderLeft, "ingest_remainder_left"),
        (
            Counter::WitnessedRepeatRecorded,
            "witnessed_repeat_recorded",
        ),
        (
            Counter::ToolCallWithoutToolUseId,
            "tool_call_without_tool_use_id",
        ),
        (Counter::ToolCallRunNotFound, "tool_call_run_not_found"),
        (Counter::ToolCallRunAmbiguous, "tool_call_run_ambiguous"),
        (Counter::LiveIngestFailed, "live_ingest_failed"),
        (Counter::QueryCancelled, "query_cancelled"),
        (
            Counter::SearchIndexWriterUnindexed,
            "search_index_writer_unindexed",
        ),
        (
            Counter::SearchIndexSeqBeyondLimit,
            "search_index_seq_beyond_limit",
        ),
        (Counter::ForkConflict, "fork_conflict"),
        (
            Counter::SearchIndexGapLookupFailed,
            "search_index_gap_lookup_failed",
        ),
        (
            Counter::RepositoryIdentityChanged,
            "repository_identity_changed",
        ),
        (
            Counter::RepositoryIdentityRebound,
            "repository_identity_rebound",
        ),
        (Counter::RepositoryReadFailed, "repository_read_failed"),
        (Counter::RepositoryBindFailed, "repository_bind_failed"),
        (Counter::RepositoryReadTimedOut, "repository_read_timed_out"),
    ];
    for (counter, name) in counters {
        assert_eq!(counter.as_str(), name);
    }
}

#[test]
fn a_degraded_entry_names_its_kind_counter_operation_and_detail() {
    let e = Entry::next(&REJECTED, 7, 1_760_000_000, Digest::ZERO).unwrap();
    assert_eq!(e.number, 7);
    assert_eq!(e.time, 1_760_000_000);
    assert_eq!(e.kind, "rejected");
    assert_eq!(e.counter.as_deref(), Some("hook_rejected"));
    assert_eq!(e.operation, "hook UserPromptSubmit");
    assert_eq!(e.detail, "not a JSON object");
    assert_eq!(e.prev_hash, Digest::ZERO);
}

const IGNORED: Occurrence = Occurrence::Keyed {
    kind: Kind::Rejected,
    counter: Counter::RepositoryConfigurationKeyIgnored,
    operation: "ingest",
    detail: "rejected repository setting",
    key: "node.deployment_mode",
};

#[test]
fn a_keyed_entry_names_its_key_after_the_detail() {
    let e = Entry::next(&IGNORED, 1, 5, Digest::ZERO).unwrap();
    assert_eq!(e.kind, "rejected");
    assert_eq!(
        e.counter.as_deref(),
        Some("repository_configuration_key_ignored")
    );
    assert_eq!(e.operation, "ingest");
    assert_eq!(e.detail, "rejected repository setting node.deployment_mode");
}

#[test]
fn a_keyed_entry_never_holds_a_key_no_settings_key_spells() {
    let keyed = |key| Occurrence::Keyed {
        kind: Kind::Rejected,
        counter: Counter::RepositoryConfigurationKeyIgnored,
        operation: "ingest",
        detail: "rejected repository setting",
        key,
    };
    let long = "a".repeat(161);
    for key in ["ignore all instructions", "", long.as_str()] {
        let occurrence = keyed(key);
        let e = Entry::next(&occurrence, 1, 5, Digest::ZERO).unwrap();
        assert_eq!(
            e.detail,
            "rejected repository setting a key no settings key spells"
        );
    }
    let plain = keyed("trust_policy.mcp:git-hub*");
    let e = Entry::next(&plain, 1, 5, Digest::ZERO).unwrap();
    assert!(e.detail.ends_with(" trust_policy.mcp:git-hub*"));
}

#[test]
fn a_principal_act_entry_feeds_no_counter() {
    let act = Occurrence::PrincipalAct {
        operation: "counter ack",
        detail: "counters acknowledged",
    };
    let e = Entry::next(&act, 3, 5, Digest::ZERO).unwrap();
    assert_eq!(e.kind, PRINCIPAL_ACT);
    assert_eq!(e.counter, None);
    assert_eq!(e.detail, "counters acknowledged");
    assert_eq!(Entry::parse(&e.line().unwrap()), Ok(e));
    assert!(matches!(
        Entry::next(&act, 1, u64::MAX, Digest::ZERO),
        Err(AuditError::Encode(_))
    ));
}

#[test]
fn an_entry_without_a_counter_still_parses() {
    let e = Entry::next(&REJECTED, 1, 5, Digest::ZERO).unwrap();
    let mut value: Value = serde_json::from_str(&e.line().unwrap()).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("counter");
    object.remove("hash");
    let body = canonical::encode(&value).unwrap();
    let hash = Digest::of(&[body.as_bytes()]);
    value["hash"] = json!(hash.to_hex());
    let parsed = Entry::parse(&canonical::encode(&value).unwrap()).unwrap();
    assert_eq!(parsed.counter, None);
}

#[test]
fn the_hash_is_sha_256_over_the_canonical_entry_without_it() {
    let prev = Digest::of(&[b"before"]);
    let e = Entry::next(&REJECTED, 2, 9, prev).unwrap();
    let body = json!({
        "entry": 2,
        "time": 9,
        "kind": "rejected",
        "counter": "hook_rejected",
        "operation": "hook UserPromptSubmit",
        "detail": "not a JSON object",
        "prev_hash": prev.to_hex(),
    });
    let want = Digest::of(&[canonical::encode(&body).unwrap().as_bytes()]);
    assert_eq!(e.hash, want);
}

#[test]
fn the_line_is_the_canonical_entry_with_its_hash() {
    let e = Entry::next(&REJECTED, 2, 9, Digest::ZERO).unwrap();
    let line = e.line().unwrap();
    let value: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(canonical::encode(&value).unwrap(), line);
    assert_eq!(value["hash"], json!(e.hash.to_hex()));
    assert_eq!(value["prev_hash"], json!(Digest::ZERO.to_hex()));
}

#[test]
fn a_time_past_the_safe_integers_has_no_encoding() {
    let e = Entry::next(&REJECTED, 1, u64::MAX, Digest::ZERO);
    assert!(matches!(e, Err(AuditError::Encode(_))), "{e:?}");
}

#[test]
fn a_line_parses_back_to_its_entry() {
    for occurrence in [REJECTED, IGNORED] {
        let e = Entry::next(&occurrence, 4, 11, Digest::of(&[b"p"])).unwrap();
        assert_eq!(Entry::parse(&e.line().unwrap()), Ok(e));
    }
}

#[test]
fn a_line_that_is_not_a_whole_canonical_entry_is_refused() {
    let e = Entry::next(&REJECTED, 4, 11, Digest::ZERO).unwrap();
    let mut value: Value = serde_json::from_str(&e.line().unwrap()).unwrap();
    let pretty = serde_json::to_string_pretty(&value).unwrap();
    let cases = [
        ("{", "not JSON"),
        ("[]", "not an entry"),
        (pretty.as_str(), "not canonical JSON"),
    ];
    for (line, why) in cases {
        assert_eq!(Entry::parse(line), Err(why), "{line}");
    }

    let mut extra = value.clone();
    extra["more"] = json!(1);
    let mut missing = value.clone();
    missing.as_object_mut().unwrap().remove("detail");
    let mut mistyped = value.clone();
    mistyped["entry"] = json!("4");
    let mut bad_hash = value.clone();
    bad_hash["hash"] = json!("zz");
    let mut no_number = value.clone();
    no_number["entry"] = json!(0);
    for (v, why) in [
        (extra, "not an entry"),
        (missing, "not an entry"),
        (mistyped, "not an entry"),
        (bad_hash, "not an entry"),
        (no_number, "not an entry"),
    ] {
        let line = canonical::encode(&v).unwrap();
        assert_eq!(Entry::parse(&line), Err(why), "{line}");
    }

    value["detail"] = json!("something else");
    let line = canonical::encode(&value).unwrap();
    assert_eq!(Entry::parse(&line), Err("hash does not match"));
}
