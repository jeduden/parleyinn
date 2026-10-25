use std::collections::BTreeSet;

use key::KeyError;
use key::random::{Entropy, OsEntropy};
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use serde_json::json;
use store::Store;
use testkit::TempDir;

use super::*;
use crate::home::Home;
use crate::node::Node;

struct Started {
    _dir: TempDir,
    home: Home,
    store: Store,
    node: Node,
}

fn started() -> Started {
    let dir = TempDir::new().unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    Started {
        _dir: dir,
        home,
        store,
        node,
    }
}

fn all() -> BTreeSet<ResidualRisk> {
    RESIDUAL_RISKS.into_iter().collect()
}

fn gate() -> Gate {
    Gate {
        terminal: true,
        ..Gate::default()
    }
}

struct Dry;

impl Entropy for Dry {
    fn fill(&mut self, _buf: &mut [u8]) -> Result<(), KeyError> {
        Err(KeyError::Entropy("no entropy".to_owned()))
    }
}

#[test]
fn classes_have_their_names() {
    assert_eq!(Class::Cut.as_str(), "cut");
    assert_eq!(Class::Neutral.as_str(), "neutral");
    assert_eq!(Class::Widening.as_str(), "widening");
}

#[test]
fn each_act_has_its_class_and_an_unnamed_one_is_widening() {
    assert_eq!(class_of("counter_ack"), Class::Neutral);
    assert_eq!(class_of("risk_acceptance_withdraw"), Class::Cut);
    assert_eq!(class_of("configuration_accept"), Class::Widening);
    assert_eq!(class_of("risk_acceptance_record"), Class::Widening);
    assert_eq!(class_of("named_by_no_requirement"), Class::Widening);
    assert_eq!(Act::CounterAck { through: None }.class(), Class::Neutral);
    assert_eq!(Act::RiskAcceptanceWithdraw.class(), Class::Cut);
}

#[test]
fn residual_risks_have_ids_texts_and_parse_back() {
    for risk in RESIDUAL_RISKS {
        assert_eq!(ResidualRisk::parse(risk.id()), Some(risk));
        assert!(
            risk.to_string()
                .starts_with(&format!("{}: the agent ", risk.id()))
        );
        assert!(!risk.text().is_empty());
    }
    assert_eq!(ResidualRisk::parse("R9"), None);
}

#[test]
fn an_acts_content_names_it_its_class_and_its_principal_surface() {
    assert_eq!(
        Act::CounterAck {
            through: Some((7, Digest::ZERO))
        }
        .content(),
        json!({
            "act": "counter_ack",
            "class": "neutral",
            "principal_surface": "cli",
            "audit_entry": 7,
            "audit_hash": Digest::ZERO.to_hex(),
        })
    );
    assert_eq!(
        Act::CounterAck { through: None }.content(),
        json!({"act": "counter_ack", "class": "neutral", "principal_surface": "cli", "audit_entry": null})
    );
    let digest = Digest::of(&[b"config"]);
    assert_eq!(
        Act::ConfigurationAccept {
            digest,
            settings: json!({"recall.max_k": 10}),
        }
        .content(),
        json!({
            "act": "configuration_accept", "class": "widening", "principal_surface": "cli",
            "digest": digest.to_hex(), "settings": {"recall.max_k": 10},
        })
    );
    let risks = [ResidualRisk::R1, ResidualRisk::R8].into();
    assert_eq!(
        Act::RiskAcceptanceRecord { risks }.content(),
        json!({
            "act": "risk_acceptance_record", "class": "widening", "principal_surface": "cli",
            "risks": ["R1", "R8"], "forgeable": FORGEABLE,
        })
    );
    assert_eq!(
        Act::RiskAcceptanceWithdraw.content(),
        json!({"act": "risk_acceptance_withdraw", "class": "cut", "principal_surface": "cli"})
    );
}

#[test]
fn record_appends_an_operator_event_to_the_device_seat() {
    let mut s = started();
    let address = record(
        &mut s.store,
        &s.node,
        &Act::CounterAck {
            through: Some((3, Digest::ZERO)),
        },
        &mut OsEntropy,
    )
    .unwrap();
    assert_eq!(address.writer, s.node.device_writer());
    let stored = s.store.event(address).unwrap().unwrap();
    assert_eq!(stored.link.header.kind, EventKind::PrincipalAct);
    assert_eq!(stored.link.header.provenance, Provenance::Operator);
    let acts = acts(&s.store, &s.node).unwrap();
    assert_eq!(acts.len(), 1);
    assert_eq!(acts[0].seq, address.seq);
    assert_eq!(
        (acts[0].name.as_str(), acts[0].class),
        ("counter_ack", Class::Neutral)
    );
    assert_eq!(acknowledged(&acts), Some((3, Digest::ZERO)));
}

#[test]
fn record_fails_when_no_commitment_key_can_be_drawn() {
    let mut s = started();
    let got = record(
        &mut s.store,
        &s.node,
        &Act::RiskAcceptanceWithdraw,
        &mut Dry,
    );
    assert!(matches!(got, Err(AdminError::Key(_))), "{got:?}");
}

#[test]
fn the_last_acknowledgement_and_accepted_settings_are_read_from_the_record() {
    let mut s = started();
    let acts0 = acts(&s.store, &s.node).unwrap();
    assert!(acts0.is_empty());
    assert_eq!(acknowledged(&acts0), None);
    assert_eq!(accepted_settings(&acts0), None);
    for act in [
        Act::CounterAck {
            through: Some((1, Digest::ZERO)),
        },
        Act::ConfigurationAccept {
            digest: Digest::ZERO,
            settings: json!({"a": 1}),
        },
        Act::CounterAck {
            through: Some((5, Digest::of(&[b"5"]))),
        },
        Act::ConfigurationAccept {
            digest: Digest::ZERO,
            settings: json!({"b": 2}),
        },
    ] {
        record(&mut s.store, &s.node, &act, &mut OsEntropy).unwrap();
    }
    let acts = acts(&s.store, &s.node).unwrap();
    assert_eq!(acknowledged(&acts), Some((5, Digest::of(&[b"5"]))));
    assert_eq!(accepted_settings(&acts), json!({"b": 2}).as_object());
}

#[test]
fn the_standing_risk_acceptance_is_the_last_unless_withdrawn() {
    let mut s = started();
    let mut take = |act: Act| {
        record(&mut s.store, &s.node, &act, &mut OsEntropy).unwrap();
        standing_risk_acceptance(&acts(&s.store, &s.node).unwrap())
    };
    take(Act::RiskAcceptanceRecord {
        risks: [ResidualRisk::R2].into(),
    });
    assert_eq!(
        take(Act::RiskAcceptanceRecord { risks: all() }),
        Some(all())
    );
    assert_eq!(take(Act::RiskAcceptanceWithdraw), None);
    assert_eq!(
        take(Act::RiskAcceptanceRecord {
            risks: [ResidualRisk::R3].into(),
        }),
        Some([ResidualRisk::R3].into())
    );
    assert_eq!(standing_risk_acceptance(&[]), None);
}

#[test]
fn no_run_leaves_no_risk_open_and_any_run_leaves_every_risk_open() {
    let mut s = started();
    assert_eq!(open_residual_risks(&s.store).unwrap(), OpenRisks::default());
    let seat = s
        .node
        .certify_ingest_seat(&s.home, &mut s.store, &mut OsEntropy)
        .unwrap();
    let open = open_residual_risks(&s.store).unwrap();
    assert_eq!(open.risks, all());
    assert_eq!(open.runs, [seat.writer]);
}

#[test]
fn every_act_refuses_without_a_terminal_first() {
    let no_terminal = Gate {
        authenticator_required: true,
        ..Gate::default()
    };
    for act in [
        Act::CounterAck { through: None },
        Act::RiskAcceptanceWithdraw,
    ] {
        assert_eq!(permit(&act, &no_terminal), Err(Refusal::NoTerminal));
    }
}

#[test]
fn cut_and_neutral_acts_need_nothing_more() {
    let strict = Gate {
        terminal: true,
        authenticator_required: true,
        managed_policy_failed: true,
        risk_acceptance_forbidden: true,
        open: OpenRisks {
            risks: all(),
            runs: Vec::new(),
        },
        standing: None,
    };
    assert_eq!(permit(&Act::CounterAck { through: None }, &strict), Ok(()));
    assert_eq!(permit(&Act::RiskAcceptanceWithdraw, &strict), Ok(()));
}

fn accept() -> Act {
    Act::ConfigurationAccept {
        digest: Digest::ZERO,
        settings: json!({}),
    }
}

#[test]
fn a_widening_act_needs_no_more_than_a_terminal_while_no_risk_is_open() {
    assert_eq!(permit(&accept(), &gate()), Ok(()));
}

#[test]
fn a_widening_act_is_refused_while_managed_policy_fails_or_needs_an_authenticator() {
    let failed = Gate {
        managed_policy_failed: true,
        ..gate()
    };
    assert_eq!(
        permit(&accept(), &failed),
        Err(Refusal::ManagedPolicyInvalid)
    );
    let authenticator = Gate {
        authenticator_required: true,
        ..gate()
    };
    assert_eq!(
        permit(&accept(), &authenticator),
        Err(Refusal::AuthenticatorRequired)
    );
    let record = Act::RiskAcceptanceRecord { risks: all() };
    assert_eq!(
        permit(&record, &authenticator),
        Err(Refusal::AuthenticatorRequired)
    );
}

#[test]
fn a_widening_act_is_refused_while_a_risk_is_open_that_no_acceptance_names() {
    let writer = WriterId::parse(&"ab".repeat(32)).unwrap();
    let open = OpenRisks {
        risks: all(),
        runs: vec![writer],
    };
    let refused = Gate {
        open: open.clone(),
        standing: Some([ResidualRisk::R1].into()),
        ..gate()
    };
    let Err(Refusal::OpenRisks { risks, runs }) = permit(&accept(), &refused) else {
        panic!("want a refusal naming the open risks");
    };
    assert_eq!(risks.len(), 7);
    assert!(!risks.contains(&ResidualRisk::R1));
    assert_eq!(runs, [writer]);
    let accepted = Gate {
        open,
        standing: Some(all()),
        ..gate()
    };
    assert_eq!(permit(&accept(), &accepted), Ok(()));
}

#[test]
fn a_risk_acceptance_is_exempt_from_open_risks_but_not_from_a_forbidding_policy() {
    let open = Gate {
        open: OpenRisks {
            risks: all(),
            runs: Vec::new(),
        },
        ..gate()
    };
    let record = Act::RiskAcceptanceRecord { risks: all() };
    assert_eq!(permit(&record, &open), Ok(()));
    let forbidden = Gate {
        risk_acceptance_forbidden: true,
        ..open
    };
    assert_eq!(
        permit(&record, &forbidden),
        Err(Refusal::RiskAcceptanceForbidden)
    );
}

#[test]
fn each_refusal_says_why_and_has_a_fixed_detail() {
    let writer = WriterId::parse(&"cd".repeat(32)).unwrap();
    let open = Refusal::OpenRisks {
        risks: [ResidualRisk::R7].into(),
        runs: vec![writer],
    };
    let text = open.to_string();
    assert!(
        text.contains("  R7: the agent confirms widening acts at a terminal"),
        "{text}"
    );
    assert!(text.ends_with(&writer.to_string()), "{text}");
    for (refusal, says, detail) in [
        (
            Refusal::NoTerminal,
            "needs a terminal",
            "refused: no terminal on standard input and output",
        ),
        (
            Refusal::AuthenticatorRequired,
            "authenticator is required",
            "refused: an authenticator is required",
        ),
        (
            Refusal::ManagedPolicyInvalid,
            "managed policy is invalid",
            "refused: managed policy is invalid",
        ),
        (
            Refusal::RiskAcceptanceForbidden,
            "forbids risk acceptance",
            "refused: managed policy forbids risk acceptance",
        ),
        (
            open,
            "OWN-22",
            "refused: residual risks open with no risk acceptance",
        ),
    ] {
        assert!(refusal.to_string().contains(says), "{refusal}");
        assert_eq!(refusal.detail(), detail);
    }
}

#[test]
fn a_repository_bind_is_a_widening_act_that_records_the_identity_it_names() {
    let act = Act::RepositoryBind {
        identity: json!({"repository": "1".repeat(40)}),
    };

    assert_eq!(act.name(), "repository_bind");
    assert_eq!(act.class(), Class::Widening);
    assert_eq!(
        act.content(),
        json!({
            "act": "repository_bind", "class": "widening", "principal_surface": "cli",
            "repository": "1".repeat(40),
        })
    );
    let provisional = Act::RepositoryBind {
        identity: json!({"repository_provisional": "ab"}),
    };
    assert_eq!(provisional.content()["repository_provisional"], "ab");
}
