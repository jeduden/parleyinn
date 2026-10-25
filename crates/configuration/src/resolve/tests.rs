use super::{IgnoredKey, Layers, Reason, Settings, SettingsLayer, resolve, tightens};
use crate::key::key;
use crate::layer::{LayerValues, parse};
use crate::value::Value;

fn values(text: &str) -> LayerValues {
    parse(text).unwrap()
}

fn layers(configuration: &str, repository: &str, managed: &str) -> Layers {
    Layers {
        configuration: values(configuration),
        repository: values(repository),
        managed: values(managed),
    }
}

fn ignored(layer: SettingsLayer, name: &str, reason: Reason) -> IgnoredKey {
    IgnoredKey {
        layer,
        key: key(name).unwrap(),
        reason,
    }
}

#[test]
fn layer_names_read_as_the_model_names_them() {
    assert_eq!(SettingsLayer::Configuration.to_string(), "configuration");
    assert_eq!(
        SettingsLayer::RepositoryConfiguration.as_str(),
        "repository configuration"
    );
    assert_eq!(SettingsLayer::ManagedPolicy.as_str(), "managed policy");
}

#[test]
fn defaults_apply_when_no_layer_sets_a_key() {
    let resolved = resolve(&Layers::default(), &LayerValues::new());
    let s = &resolved.settings;
    assert_eq!(s, &Settings::defaults());
    assert_eq!(s.deployment_mode(), "automation");
    assert!(!s.authenticator_required());
    assert!(!s.risk_acceptance_forbidden());
    assert_eq!(s.get("recall.max_k"), Some(&Value::Integer(50)));
    assert_eq!(s.get("home.id"), None);
    assert_eq!(s.texts("transcript.roots"), ["~/.claude/projects"]);
    assert!(s.texts("redaction.extra_patterns").is_empty());
    assert!(s.texts("recall.max_k").is_empty());
    assert_eq!(s.iter().count(), 17);
    assert!(resolved.ignored.is_empty() && resolved.awaiting_acceptance.is_empty());
}

#[test]
fn deployment_mode_reads_automation_when_unset() {
    assert_eq!(Settings::default().deployment_mode(), "automation");
}

#[test]
fn a_configuration_change_awaiting_no_acceptance_applies() {
    let r = resolve(
        &layers("recall.max_k = 10\nhome.id = 'h'", "", ""),
        &LayerValues::new(),
    );
    assert_eq!(r.settings.get("recall.max_k"), Some(&Value::Integer(10)));
    assert_eq!(r.settings.get("home.id"), Some(&Value::Text("h".into())));
    assert!(r.awaiting_acceptance.is_empty());
}

#[test]
fn a_configuration_change_awaiting_acceptance_stands_as_accepted() {
    let config = "node.deployment_mode = 'interactive'\nflag.enabled = false";
    let r = resolve(&layers(config, "", ""), &LayerValues::new());
    assert_eq!(r.settings.deployment_mode(), "automation");
    assert_eq!(r.settings.get("flag.enabled"), Some(&Value::Bool(true)));
    let awaiting: Vec<&str> = r.awaiting_acceptance.iter().map(|k| k.name()).collect();
    assert_eq!(awaiting, ["flag.enabled", "node.deployment_mode"]);
}

#[test]
fn an_accepted_change_applies() {
    let config = "node.deployment_mode = 'interactive'";
    let r = resolve(&layers(config, "", ""), &values(config));
    assert_eq!(r.settings.deployment_mode(), "interactive");
    assert!(r.awaiting_acceptance.is_empty());
}

#[test]
fn removing_an_accepted_key_awaits_acceptance_too() {
    let accepted = values("node.deployment_mode = 'interactive'\nretention_policy.'*'.web = 9");
    let r = resolve(&Layers::default(), &accepted);
    assert_eq!(r.settings.deployment_mode(), "interactive");
    assert_eq!(
        r.settings.get("retention_policy.*.web"),
        Some(&Value::Integer(9))
    );
    assert_eq!(r.awaiting_acceptance.len(), 2);
}

#[test]
fn a_tightening_change_needs_no_acceptance() {
    let accepted = values("flag.enabled = false");
    let r = resolve(&layers("flag.enabled = true", "", ""), &accepted);
    assert_eq!(r.settings.get("flag.enabled"), Some(&Value::Bool(true)));
    assert!(r.awaiting_acceptance.is_empty());
}

#[test]
fn repository_configuration_only_tightens() {
    let repo = "recall.max_k = 5\npayload.threshold_bytes = 4096\nflag.enabled = true\n\
                restore_block.on_prompt = false\nrecall.default_scope = 'run'";
    let r = resolve(&layers("", repo, ""), &LayerValues::new());
    assert_eq!(r.settings.get("recall.max_k"), Some(&Value::Integer(5)));
    assert_eq!(
        r.settings.get("payload.threshold_bytes"),
        Some(&Value::Integer(4096))
    );
    assert!(r.ignored.is_empty(), "{:?}", r.ignored);
}

#[test]
fn repository_configuration_that_loosens_is_ignored() {
    let repo = "node.deployment_mode = 'interactive'\nrecall.max_k = 100\n\
                restore_block.on_prompt = true\nrecall.default_scope = 'room'\nflag.enabled = false";
    let r = resolve(&layers("", repo, ""), &LayerValues::new());
    assert_eq!(r.settings, Settings::defaults());
    let layer = SettingsLayer::RepositoryConfiguration;
    assert_eq!(
        r.ignored,
        [
            ignored(layer, "flag.enabled", Reason::Loosens),
            ignored(layer, "node.deployment_mode", Reason::Loosens),
            ignored(layer, "recall.default_scope", Reason::Loosens),
            ignored(layer, "recall.max_k", Reason::Loosens),
            ignored(layer, "restore_block.on_prompt", Reason::Loosens),
        ]
    );
}

#[test]
fn repository_configuration_lowers_from_what_the_configuration_set() {
    let r = resolve(
        &layers("recall.max_k = 10", "recall.max_k = 20", ""),
        &LayerValues::new(),
    );
    assert_eq!(r.settings.get("recall.max_k"), Some(&Value::Integer(10)));
    assert_eq!(r.ignored.len(), 1);
}

#[test]
fn managed_policy_overrides_every_layer_and_needs_no_acceptance() {
    let r = resolve(
        &layers(
            "node.deployment_mode = 'interactive'\nrecall.max_k = 10",
            "recall.max_k = 5",
            "node.deployment_mode = 'automation'\nrecall.max_k = 100\nauthenticator.required = true",
        ),
        &LayerValues::new(),
    );
    assert_eq!(r.settings.deployment_mode(), "automation");
    assert_eq!(r.settings.get("recall.max_k"), Some(&Value::Integer(100)));
    assert!(r.settings.authenticator_required());
    assert!(r.awaiting_acceptance.is_empty());
}

#[test]
fn managed_policy_sets_a_mode_with_no_acceptance() {
    let r = resolve(
        &layers(
            "",
            "",
            "node.deployment_mode = 'interactive'\nrisk_acceptance.forbidden = true",
        ),
        &LayerValues::new(),
    );
    assert_eq!(r.settings.deployment_mode(), "interactive");
    assert!(r.settings.risk_acceptance_forbidden());
}

#[test]
fn every_layer_has_its_trust_policy_keys_ignored() {
    let trust = "trust_policy.web = 'trusted'";
    let r = resolve(&layers(trust, trust, trust), &values(trust));
    assert_eq!(r.settings, Settings::defaults());
    assert_eq!(
        r.ignored,
        [
            ignored(
                SettingsLayer::Configuration,
                "trust_policy.web",
                Reason::TrustPolicy
            ),
            ignored(
                SettingsLayer::RepositoryConfiguration,
                "trust_policy.web",
                Reason::TrustPolicy
            ),
            ignored(
                SettingsLayer::ManagedPolicy,
                "trust_policy.web",
                Reason::TrustPolicy
            ),
        ]
    );
}

#[test]
fn tightens_follows_each_repository_rule() {
    let spec = |name: &str| key(name).unwrap().spec();
    let int = Value::Integer;
    assert!(!tightens(spec("home.id"), None, &Value::Text("x".into())));
    assert!(tightens(spec("recall.max_k"), Some(&int(50)), &int(50)));
    assert!(!tightens(spec("recall.max_k"), None, &int(5)));
    assert!(!tightens(
        spec("recall.max_k"),
        Some(&int(50)),
        &Value::Bool(true)
    ));
    assert!(tightens(spec("kernel.enabled"), None, &Value::Bool(false)));
    assert!(!tightens(spec("kernel.enabled"), None, &Value::Bool(true)));
    assert!(!tightens(
        spec("authenticator.required"),
        Some(&Value::Bool(false)),
        &Value::Bool(true)
    ));
}

#[test]
fn the_principal_s_configuration_may_require_an_authenticator_but_a_repository_may_not() {
    let r = resolve(
        &layers("authenticator.required = true", "", ""),
        &LayerValues::new(),
    );
    assert!(r.settings.authenticator_required());
    assert!(r.ignored.is_empty());

    let r = resolve(
        &layers("", "authenticator.required = true", ""),
        &LayerValues::new(),
    );
    assert!(!r.settings.authenticator_required());
    assert_eq!(
        r.ignored,
        [ignored(
            SettingsLayer::RepositoryConfiguration,
            "authenticator.required",
            Reason::Loosens
        )]
    );
}

#[test]
fn only_managed_policy_may_forbid_risk_acceptance() {
    let forbid = "risk_acceptance.forbidden = true";
    let r = resolve(&layers(forbid, forbid, ""), &values(forbid));
    assert!(!r.settings.risk_acceptance_forbidden());
    assert!(r.awaiting_acceptance.is_empty());
    assert_eq!(
        r.ignored,
        [
            ignored(
                SettingsLayer::Configuration,
                "risk_acceptance.forbidden",
                Reason::ManagedPolicyOnly
            ),
            ignored(
                SettingsLayer::RepositoryConfiguration,
                "risk_acceptance.forbidden",
                Reason::Loosens
            ),
        ]
    );
    let r = resolve(&layers("", "", forbid), &LayerValues::new());
    assert!(r.settings.risk_acceptance_forbidden());
    assert!(r.ignored.is_empty());
}

#[test]
fn a_key_the_configuration_drops_falls_back_to_its_default_when_no_acceptance_holds_it() {
    let accepted = values("home.id = 'h'\nrecall.max_k = 10");
    let r = resolve(&Layers::default(), &accepted);
    assert_eq!(r.settings.get("home.id"), None);
    assert_eq!(r.settings.get("recall.max_k"), Some(&Value::Integer(50)));
    assert!(r.awaiting_acceptance.is_empty());
}
