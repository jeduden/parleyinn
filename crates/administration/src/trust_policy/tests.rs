use super::*;

fn input(provenance: &Provenance, kind: EventKind) -> Input<'_> {
    Input {
        provenance,
        kind,
        origin: Origin::Witnessed,
        device_seat: true,
        launcher_signed: false,
        interactive: false,
        prompt_matched: false,
        attended: false,
    }
}

fn level(provenance: Provenance, kind: EventKind) -> TrustLevel {
    trust_level(&input(&provenance, kind))
}

#[test]
fn an_operator_event_of_a_witnessed_writer_is_trusted() {
    assert_eq!(
        level(Provenance::Operator, EventKind::PrincipalAct),
        TrustLevel::Trusted
    );
}

#[test]
fn the_structural_events_the_core_records_itself_are_trusted() {
    for kind in [
        EventKind::KeyEvent,
        EventKind::CreateRoom,
        EventKind::CertificateRecord,
        EventKind::NodeIdentityChange,
    ] {
        assert_eq!(level(Provenance::Structural, kind), TrustLevel::Trusted);
    }
}

#[test]
fn a_hook_observation_and_the_structural_events_capture_derives_are_untrusted() {
    for kind in [
        EventKind::HookObservation,
        EventKind::PromptMatch,
        EventKind::RepeatReference,
        EventKind::RepeatOwner,
    ] {
        assert_eq!(level(Provenance::Structural, kind), TrustLevel::Untrusted);
    }
}

#[test]
fn no_event_of_another_origin_is_trusted() {
    for origin in [Origin::Ingested, Origin::Bundle, Origin::Peer] {
        let mut i = input(&Provenance::Operator, EventKind::PrincipalAct);
        i.origin = origin;
        assert_eq!(trust_level(&i), TrustLevel::Untrusted);
    }
}

#[test]
fn every_model_reproducible_or_third_party_class_is_untrusted() {
    for provenance in [
        Provenance::Assistant,
        Provenance::ToolCall,
        Provenance::ToolResult("Bash".to_owned()),
        Provenance::Web,
        Provenance::Mcp("github".to_owned()),
        Provenance::File,
        Provenance::SubagentResult,
        Provenance::HarnessText,
        Provenance::Post,
        Provenance::Summary,
        Provenance::Unparsed,
    ] {
        assert_eq!(
            level(provenance, EventKind::TranscriptLine),
            TrustLevel::Untrusted
        );
    }
}

#[test]
fn harness_meta_is_trusted_only_when_the_launcher_signed_it() {
    let mut i = input(&Provenance::HarnessMeta, EventKind::TranscriptLine);
    assert_eq!(trust_level(&i), TrustLevel::Untrusted);
    i.launcher_signed = true;
    assert_eq!(trust_level(&i), TrustLevel::Trusted);
}

#[test]
fn a_user_event_is_trusted_only_when_every_condition_holds() {
    let mut i = input(&Provenance::User, EventKind::TranscriptLine);
    i.interactive = true;
    i.prompt_matched = true;
    i.attended = true;
    assert!(user_trusted(&i, false));
    assert!(!user_trusted(&i, true));
    for spoil in [
        |i: &mut Input<'_>| i.interactive = false,
        |i: &mut Input<'_>| i.prompt_matched = false,
        |i: &mut Input<'_>| i.attended = false,
    ] {
        let mut j = i;
        spoil(&mut j);
        assert!(!user_trusted(&j, false));
    }
}

#[test]
fn under_m1_no_user_event_is_trusted_whatever_the_caller_says() {
    let mut i = input(&Provenance::User, EventKind::TranscriptLine);
    i.interactive = true;
    i.prompt_matched = true;
    i.attended = true;

    assert_eq!(trust_level(&i), TrustLevel::Untrusted);
}

#[test]
fn an_operator_or_structural_event_outside_a_device_seat_is_untrusted() {
    for (provenance, kind) in [
        (Provenance::Operator, EventKind::PrincipalAct),
        (Provenance::Structural, EventKind::KeyEvent),
    ] {
        let mut i = input(&provenance, kind);
        i.device_seat = false;
        assert_eq!(trust_level(&i), TrustLevel::Untrusted);
    }
}

#[test]
fn the_levels_names_are_fixed() {
    assert_eq!(TrustLevel::Trusted.as_str(), "trusted");
    assert_eq!(TrustLevel::Untrusted.as_str(), "untrusted");
}

fn address(n: u8) -> Address {
    Address::new(
        record::id::WriterId::derive(&[n; 32]),
        record::id::Seq::new(1).unwrap(),
    )
}

#[test]
fn an_artifact_records_its_sources_and_is_untrusted_if_any_source_is() {
    let both = [
        (address(1), TrustLevel::Trusted),
        (address(2), TrustLevel::Untrusted),
    ];

    let taint = Taint::of(&both, false);

    assert_eq!(taint.derives_from, [address(1), address(2)]);
    assert_eq!(taint.trust, TrustLevel::Untrusted);
}

#[test]
fn an_artifact_of_trusted_sources_alone_is_trusted() {
    let taint = Taint::of(&[(address(1), TrustLevel::Trusted)], false);

    assert_eq!(taint.trust, TrustLevel::Trusted);
}

#[test]
fn the_free_text_of_a_cut_or_neutral_act_taints_an_artifact_of_trusted_sources() {
    let taint = Taint::of(&[(address(1), TrustLevel::Trusted)], true);

    assert_eq!(taint.trust, TrustLevel::Untrusted);
}
