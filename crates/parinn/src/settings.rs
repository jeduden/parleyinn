//! The settings layers as a command reads them (ADM-04): where each is,
//! the settings the last accepted configuration set, read from the
//! record (I10), and the audit entries for a layer that fails or a key
//! that is ignored (I6).

use std::path::PathBuf;

use administration::act::{Recorded, accepted_settings, acts};
use administration::home::Home;
use administration::node::Node;
use audit::entry::{Counter, Kind, Occurrence};
use configuration::key::key;
use configuration::layer::LayerValues;
use configuration::source::{CONFIGURATION, repository_configuration};
use configuration::{IgnoredKey, Loaded, Reason, Resolved, SettingsLayer, Sources, Value, load};
use serde_json::Map;
use store::Store;

use crate::Context;

/// The settings layers as read, and resolved.
#[derive(Debug)]
pub(crate) struct InForce {
    pub loaded: Loaded,
    pub resolved: Resolved,
}

/// Where the settings layers are for `home`, run from the context's
/// working directory.
pub(crate) fn sources(ctx: &Context, home: &Home) -> Sources {
    Sources {
        configuration: home.root().join(CONFIGURATION),
        repository: ctx
            .working_dir
            .as_deref()
            .and_then(repository_configuration),
        managed: ctx.managed_policy.clone(),
        running: ctx.os_user,
    }
}

/// The principal acts this node recorded; none before its first start
/// finished: no store, a store with no seat, or a start still staged.
pub(crate) fn recorded_acts(home: &Home) -> Result<Vec<Recorded>, String> {
    opened_acts(home).map(|(_, acts)| acts)
}

/// `home`'s store, opened when it exists, with the principal acts it
/// holds, as [`recorded_acts`] reads them.
pub(crate) fn opened_acts(home: &Home) -> Result<(Option<Store>, Vec<Recorded>), String> {
    if !home.store().exists() {
        return Ok((None, Vec::new()));
    }
    let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
    let acts = acts_in(home, &store)?;

    Ok((Some(store), acts))
}

/// The principal acts `store`, `home`'s open store, holds; none before
/// the node's first start finished.
pub(crate) fn acts_in(home: &Home, store: &Store) -> Result<Vec<Recorded>, String> {
    if home.staged().exists() || store.seats().map_err(|e| e.to_string())?.is_empty() {
        return Ok(Vec::new());
    }
    let node = Node::find(home, store).map_err(|e| e.to_string())?;

    acts(store, &node).map_err(|e| e.to_string())
}

/// The settings the last accepted configuration set, each still a
/// settings key of a kind it takes; none before the first acceptance.
pub(crate) fn accepted(acts: &[Recorded]) -> LayerValues {
    accepted_settings(acts)
        .into_iter()
        .flatten()
        .filter_map(|(name, value)| Some((key(name)?, Value::from_json(value)?)))
        .collect()
}

/// The settings layers in force for `home`, given its principal acts.
pub(crate) fn in_force(ctx: &Context, home: &Home, acts: &[Recorded]) -> InForce {
    in_force_from(&sources(ctx, home), acts)
}

/// The settings layers in force as read from `sources`, given the
/// principal acts the record holds.
pub(crate) fn in_force_from(sources: &Sources, acts: &[Recorded]) -> InForce {
    in_force_loaded(load(sources), acts)
}

/// The settings layers in force as `loaded` holds them, given the
/// principal acts the record holds.
pub(crate) fn in_force_loaded(loaded: Loaded, acts: &[Recorded]) -> InForce {
    let resolved = configuration::resolve(&loaded.layers, &accepted(acts));

    InForce { loaded, resolved }
}

/// What a configuration acceptance records of `layer`: every key it
/// sets, as JSON, a trust policy key left out.
pub(crate) fn recorded(layer: &LayerValues) -> serde_json::Value {
    let settings: Map<String, serde_json::Value> = layer
        .iter()
        .filter(|(k, _)| k.trust_policy().is_none())
        .map(|(k, v)| (k.name().to_owned(), v.to_json()))
        .collect();

    serde_json::Value::Object(settings)
}

/// The audit entry for each layer that failed and each key ignored,
/// under `operation`.
pub(crate) fn occurrences<'a>(operation: &'static str, f: &'a InForce) -> Vec<Occurrence<'a>> {
    let failed = f.loaded.failures.iter().map(|failure| {
        let (counter, detail) = match failure.layer {
            SettingsLayer::Configuration => (Counter::SettingsLayerInvalid, "configuration failed"),
            SettingsLayer::RepositoryConfiguration => (
                Counter::SettingsLayerInvalid,
                "repository configuration failed",
            ),
            SettingsLayer::ManagedPolicy => (
                Counter::ManagedPolicyInvalid,
                "managed policy is writable by the principal or invalid; \
                 no B1 to B3 component starts and no widening act is taken",
            ),
        };
        Occurrence::Degraded {
            kind: Kind::Rejected,
            counter,
            operation,
            detail,
        }
    });
    let ignored = f.resolved.ignored.iter().map(|i| ignored(operation, i));

    failed.chain(ignored).collect()
}

/// The audit entry for one ignored key.
fn ignored<'a>(operation: &'static str, i: &'a IgnoredKey) -> Occurrence<'a> {
    let (counter, detail, key) = match (i.reason, i.key.trust_policy()) {
        (Reason::TrustPolicy, Some(class)) => (
            Counter::TrustPolicyKeyIgnored,
            match i.layer {
                SettingsLayer::Configuration => "configuration: rejected trust override for",
                SettingsLayer::RepositoryConfiguration => {
                    "repository configuration: rejected trust override for"
                }
                SettingsLayer::ManagedPolicy => "managed policy: rejected trust override for",
            },
            class,
        ),
        (Reason::ManagedPolicyOnly, _) => (
            Counter::ConfigurationKeyIgnored,
            "ignored configuration key only managed policy sets",
            i.key.name(),
        ),
        _ => (
            Counter::RepositoryConfigurationKeyIgnored,
            "rejected repository setting",
            i.key.name(),
        ),
    };

    Occurrence::Keyed {
        kind: Kind::Rejected,
        counter,
        operation,
        detail,
        key,
    }
}

/// Audits every layer that failed and every key ignored, as a command
/// that records goes on (I9); returns what the audit log refused, for
/// the command to say.
pub(crate) fn audit(ctx: &Context, operation: &'static str, f: &InForce) -> Vec<String> {
    occurrences(operation, f)
        .iter()
        .filter_map(|o| ctx.audit(o).err().map(|e| e.to_string()))
        .collect()
}

/// The managed policy path a build reads: the documented one, or in a
/// development build the one `PARINN_TEST_MANAGED_POLICY` names.
#[must_use]
pub fn managed_policy(dev_override: Option<std::ffi::OsString>) -> PathBuf {
    dev_override
        .filter(|p| cfg!(debug_assertions) && !p.is_empty())
        .map_or_else(
            || PathBuf::from(configuration::source::MANAGED_POLICY),
            PathBuf::from,
        )
}

#[cfg(test)]
pub(crate) mod tests;
