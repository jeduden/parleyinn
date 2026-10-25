//! How the settings layers combine (ADM-04): defaults first, then the
//! principal's configuration, then the repository configuration, then
//! managed policy, which overrides all three.
//!
//! - A change of the principal's configuration that awaits acceptance
//!   stands as the accepted configuration had it until a widening
//!   principal act accepts it.
//! - A repository configuration key is ignored unless it only tightens
//!   (SEC-11).
//! - A trust policy key is ignored in every layer (PRV-05).
//!
//! Resolving reads no clock and no file: the same layers and accepted
//! settings always give the same settings (I10).

use std::collections::BTreeMap;
use std::fmt;

use crate::key::{Repository, SettingsKey, Spec, fixed};
use crate::layer::LayerValues;
use crate::value::Value;

/// The three settings layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsLayer {
    Configuration,
    RepositoryConfiguration,
    ManagedPolicy,
}

impl SettingsLayer {
    /// Its name in a message.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Configuration => "configuration",
            Self::RepositoryConfiguration => "repository configuration",
            Self::ManagedPolicy => "managed policy",
        }
    }
}

impl fmt::Display for SettingsLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What each settings layer sets; an absent layer sets nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layers {
    pub configuration: LayerValues,
    pub repository: LayerValues,
    pub managed: LayerValues,
}

/// Why a key was ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// A repository configuration key that does more than tighten.
    Loosens,
    /// A key that would change the trust policy.
    TrustPolicy,
    /// A key only managed policy may set, in another layer.
    ManagedPolicyOnly,
}

/// A key a layer sets that changes nothing, audited and counted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoredKey {
    pub layer: SettingsLayer,
    pub key: SettingsKey,
    pub reason: Reason,
}

/// The settings in force: each key's value, a key unset left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    values: BTreeMap<SettingsKey, Value>,
}

impl Settings {
    /// The defaults alone.
    #[must_use]
    pub fn defaults() -> Self {
        let values = fixed()
            .filter_map(|k| k.default_value().map(|v| (k, v)))
            .collect();

        Self { values }
    }

    /// The value of the key named `name`, if it is set.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values
            .iter()
            .find(|(k, _)| k.name() == name)
            .map(|(_, v)| v)
    }

    /// The deployment mode (PRV-04).
    #[must_use]
    pub fn deployment_mode(&self) -> &str {
        match self.get("node.deployment_mode") {
            Some(Value::Text(mode)) => mode,
            _ => "automation",
        }
    }

    /// Whether widening acts need an authenticator: the principal's
    /// configuration or managed policy requires one (OWN-11, SEC-22).
    #[must_use]
    pub fn authenticator_required(&self) -> bool {
        self.get("authenticator.required") == Some(&Value::Bool(true))
    }

    /// Whether managed policy forbids risk acceptance (OWN-22, SEC-22).
    #[must_use]
    pub fn risk_acceptance_forbidden(&self) -> bool {
        self.get("risk_acceptance.forbidden") == Some(&Value::Bool(true))
    }

    /// The list the key named `name` holds; empty when it holds none.
    #[must_use]
    pub fn texts(&self, name: &str) -> &[String] {
        match self.get(name) {
            Some(Value::Texts(items)) => items,
            _ => &[],
        }
    }

    /// Every key set, in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&SettingsKey, &Value)> {
        self.values.iter()
    }

    fn set(&mut self, key: &SettingsKey, value: Option<Value>) {
        match value {
            Some(v) => self.values.insert(key.clone(), v),
            None => self.values.remove(key),
        };
    }

    fn value(&self, key: &SettingsKey) -> Option<&Value> {
        self.values.get(key)
    }
}

/// The settings layers resolved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    pub settings: Settings,
    /// The keys ignored, by layer.
    pub ignored: Vec<IgnoredKey>,
    /// The principal's configuration's keys whose change awaits
    /// acceptance, standing as accepted meanwhile.
    pub awaiting_acceptance: Vec<SettingsKey>,
}

/// Resolves `layers` over the defaults, given the settings the last
/// accepted configuration held.
#[must_use]
pub fn resolve(layers: &Layers, accepted: &LayerValues) -> Resolved {
    let mut out = Resolved {
        settings: Settings::defaults(),
        ..Resolved::default()
    };
    configuration(&mut out, &layers.configuration, accepted);
    repository(&mut out, &layers.repository);
    managed(&mut out, &layers.managed);

    out
}

/// Applies the principal's configuration, each change that awaits
/// acceptance standing as accepted.
fn configuration(out: &mut Resolved, layer: &LayerValues, accepted: &LayerValues) {
    let ignored = trust_policy(out, SettingsLayer::Configuration, layer);
    out.ignored.extend(
        layer
            .keys()
            .filter(|k| k.managed_policy_only())
            .map(|k| IgnoredKey {
                layer: SettingsLayer::Configuration,
                key: k.clone(),
                reason: Reason::ManagedPolicyOnly,
            }),
    );
    let keys: Vec<&SettingsKey> = layer
        .keys()
        .chain(accepted.keys())
        .filter(|k| !ignored.contains(k) && k.trust_policy().is_none() && !k.managed_policy_only())
        .collect();
    for key in keys {
        let spec = key.spec();
        let set = layer.get(key).cloned().or_else(|| key.default_value());
        let stood = accepted.get(key).cloned().or_else(|| key.default_value());
        if spec
            .acceptance
            .awaits(spec.kind, stood.as_ref(), set.as_ref())
        {
            if !out.awaiting_acceptance.contains(key) {
                out.awaiting_acceptance.push(key.clone());
            }
            out.settings.set(key, stood);
        } else {
            out.settings.set(key, set);
        }
    }
}

/// Applies the repository configuration's keys that only tighten.
fn repository(out: &mut Resolved, layer: &LayerValues) {
    let layer_name = SettingsLayer::RepositoryConfiguration;
    let trust = trust_policy(out, layer_name, layer);
    for (key, value) in layer.iter().filter(|(k, _)| !trust.contains(k)) {
        if tightens(key.spec(), out.settings.value(key), value) {
            out.settings.set(key, Some(value.clone()));
        } else {
            out.ignored.push(IgnoredKey {
                layer: layer_name,
                key: key.clone(),
                reason: Reason::Loosens,
            });
        }
    }
}

/// Applies managed policy, which overrides every other layer.
fn managed(out: &mut Resolved, layer: &LayerValues) {
    let trust = trust_policy(out, SettingsLayer::ManagedPolicy, layer);
    for (key, value) in layer.iter().filter(|(k, _)| !trust.contains(k)) {
        out.settings.set(key, Some(value.clone()));
        out.awaiting_acceptance.retain(|k| k != key);
    }
}

/// Ignores every trust policy key of `layer`, returning them.
fn trust_policy<'a>(
    out: &mut Resolved,
    layer_name: SettingsLayer,
    layer: &'a LayerValues,
) -> Vec<&'a SettingsKey> {
    let keys: Vec<&SettingsKey> = layer
        .keys()
        .filter(|k| k.trust_policy().is_some())
        .collect();
    out.ignored.extend(keys.iter().map(|k| IgnoredKey {
        layer: layer_name,
        key: (*k).clone(),
        reason: Reason::TrustPolicy,
    }));

    keys
}

/// Whether a repository configuration setting `value` over `before`
/// only tightens: its key lets the repository set it that way, and the
/// change awaits no acceptance (SEC-11).
fn tightens(spec: Spec, before: Option<&Value>, value: &Value) -> bool {
    let allowed = match (spec.repository, before, value) {
        (Repository::Any, ..) => true,
        (Repository::OnlyLower, Some(Value::Integer(a)), Value::Integer(b)) => b <= a,
        (Repository::OnlyBool(only), _, Value::Bool(b)) => *b == only,
        (Repository::OnlyChoice(only), _, Value::Text(t)) => t == only,
        _ => false,
    };

    allowed && !spec.acceptance.awaits(spec.kind, before, Some(value))
}

#[cfg(test)]
mod tests;
