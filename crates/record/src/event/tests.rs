use super::*;

#[test]
fn every_event_kind_round_trips_through_its_name() {
    for (kind, name) in [
        (EventKind::KeyEvent, "key_event"),
        (EventKind::CreateRoom, "create_room"),
        (EventKind::CertificateRecord, "certificate_record"),
        (EventKind::UnparsedLine, "unparsed_line"),
        (EventKind::TranscriptLine, "transcript_line"),
        (EventKind::PrincipalAct, "principal_act"),
        (EventKind::NodeIdentityChange, "node_identity_change"),
        (EventKind::RepeatReference, "repeat_reference"),
        (EventKind::RepeatOwner, "repeat_owner"),
        (EventKind::HookObservation, "hook_observation"),
        (EventKind::PromptMatch, "prompt_match"),
        (EventKind::RepositoryBind, "repository_bind"),
    ] {
        assert_eq!(kind.as_str(), name);
        assert_eq!(EventKind::parse(name), Some(kind));
    }
    assert_eq!(EventKind::parse("message"), None);
}

#[test]
fn every_provenance_class_round_trips_through_its_name() {
    let classes = [
        (Provenance::User, "user"),
        (Provenance::Assistant, "assistant"),
        (Provenance::ToolCall, "tool_call"),
        (Provenance::ToolResult("Bash".into()), "tool_result:Bash"),
        (Provenance::Web, "web"),
        (Provenance::Mcp("github".into()), "mcp:github"),
        (Provenance::File, "file"),
        (Provenance::SubagentResult, "subagent_result"),
        (Provenance::HarnessMeta, "harness_meta"),
        (Provenance::HarnessText, "harness_text"),
        (Provenance::Operator, "operator"),
        (Provenance::Post, "post"),
        (Provenance::Summary, "summary"),
        (Provenance::Structural, "structural"),
        (Provenance::Unparsed, "unparsed"),
    ];
    for (class, name) in classes {
        assert_eq!(class.name(), name);
        assert_eq!(Provenance::parse(name), Some(class));
    }
}

#[test]
fn a_provenance_outside_the_closed_set_is_refused() {
    for bad in ["", "tool_result", "tool_result:", "mcp:", "web:x", "system"] {
        assert_eq!(Provenance::parse(bad), None, "{bad}");
    }
}
