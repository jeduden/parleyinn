//! Principal acts at the CLI (OWN-02, OWN-11, OWN-12, OWN-22): each an
//! `operator` event in this node's device seat's writer, in its
//! personal room, naming its class and its principal surface.
//!
//! - Every verb that writes one refuses unless its standard input and
//!   output are a terminal, before any other check (OWN-12).
//! - A widening act is refused while managed policy requires an
//!   authenticator, as presence proofs come later (SEC-22), and while
//!   managed policy fails, since what it would require is unknown.
//! - A widening act but the risk acceptance itself is refused unless
//!   the standing risk acceptance names every residual risk a run on
//!   the node leaves open (OWN-22). No run records a sandbox state yet,
//!   so every run counts as unsandboxed and leaves every risk open.
//!
//! Before PRV-10 ships nothing signs an act: its principal surface
//! tells it from a room act (OWN-02).

use std::collections::BTreeSet;
use std::fmt;

use key::certificate::{SeatCertificate, SeatKind};
use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use record::id::{Address, Seq, WriterId};
use serde_json::{Map, Value, json};
use store::{NewEvent, Store};

use crate::AdminError;
use crate::node::Node;

/// The principal surface every act here is taken at: the CLI at a
/// terminal (OWN-02).
pub const PRINCIPAL_SURFACE: &str = "cli";

/// The risk every risk acceptance names besides the open ones (OWN-22).
pub const FORGEABLE: &str = "an unsandboxed agent can forge this risk acceptance itself";

/// A principal act's class (OWN-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    Cut,
    Neutral,
    Widening,
}

impl Class {
    /// Its name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cut => "cut",
            Self::Neutral => "neutral",
            Self::Widening => "widening",
        }
    }
}

/// The class of the act named `name`: a principal act no requirement
/// names is widening (OWN-11).
#[must_use]
pub fn class_of(name: &str) -> Class {
    match name {
        "counter_ack" => Class::Neutral,
        "risk_acceptance_withdraw" => Class::Cut,
        _ => Class::Widening,
    }
}

/// One of the residual risks of §6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResidualRisk {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

/// Every residual risk, in order.
pub const RESIDUAL_RISKS: [ResidualRisk; 8] = [
    ResidualRisk::R1,
    ResidualRisk::R2,
    ResidualRisk::R3,
    ResidualRisk::R4,
    ResidualRisk::R5,
    ResidualRisk::R6,
    ResidualRisk::R7,
    ResidualRisk::R8,
];

impl ResidualRisk {
    /// Its id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::R1 => "R1",
            Self::R2 => "R2",
            Self::R3 => "R3",
            Self::R4 => "R4",
            Self::R5 => "R5",
            Self::R6 => "R6",
            Self::R7 => "R7",
            Self::R8 => "R8",
        }
    }

    /// What it is, in §6.1's words, shortened.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::R1 => "the agent types into the harness's own prompt to approve itself",
            Self::R2 => {
                "the agent edits the harness's settings or hooks, or replaces ParleyInn's executables"
            }
            Self::R3 => {
                "the agent writes to, deletes or rolls back the home, or signs with its keys"
            }
            Self::R4 => {
                "the agent shows a harmless screen while a widening act waits for confirmation"
            }
            Self::R5 => "the agent reads every room from the store, unenveloped",
            Self::R6 => "the agent uses cut acts against other agents",
            Self::R7 => "the agent confirms widening acts at a terminal",
            Self::R8 => {
                "the agent appends to a harness transcript or runs a hook handler with its own input"
            }
        }
    }

    /// The risk `id` names.
    #[must_use]
    pub fn parse(id: &str) -> Option<Self> {
        RESIDUAL_RISKS.into_iter().find(|r| r.id() == id)
    }
}

impl fmt::Display for ResidualRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.id(), self.text())
    }
}

/// A principal act this node's CLI takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// Acknowledge the counters through the audit entry `through` names
    /// by number and hash, the log's last when the act was taken; none
    /// when the log held no entry (OPS-03). Neutral.
    CounterAck { through: Option<(u64, Digest)> },
    /// Accept the principal's configuration whose text has `digest`,
    /// recording the settings it sets (ADM-04); widening.
    ConfigurationAccept { digest: Digest, settings: Value },
    /// Accept the residual risks `risks` (OWN-22); widening, but exempt
    /// from OWN-22's condition.
    RiskAcceptanceRecord { risks: BTreeSet<ResidualRisk> },
    /// Withdraw the risk acceptance; a cut act.
    RiskAcceptanceWithdraw,
    /// Bind a repository's identity by hand, or rebind it after a change
    /// (LANE-02): `identity` holds the commit it binds to as
    /// `repository`, or a provisional one as `repository_provisional`.
    /// Widening. It names the worktree by a keyed digest, never a path: the
    /// act replicates.
    RepositoryBind { identity: Value },
}

impl Act {
    /// Its name in the record.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::CounterAck { .. } => "counter_ack",
            Self::ConfigurationAccept { .. } => "configuration_accept",
            Self::RiskAcceptanceRecord { .. } => "risk_acceptance_record",
            Self::RiskAcceptanceWithdraw => "risk_acceptance_withdraw",
            Self::RepositoryBind { .. } => "repository_bind",
        }
    }

    /// Its class (OWN-11).
    #[must_use]
    pub fn class(&self) -> Class {
        class_of(self.name())
    }

    /// Its event's content: its name, class and principal surface, and
    /// what it acts with. None of these acts acts on a room.
    #[must_use]
    pub fn content(&self) -> Value {
        let mut content = Map::new();
        content.insert("act".into(), json!(self.name()));
        content.insert("class".into(), json!(self.class().as_str()));
        content.insert("principal_surface".into(), json!(PRINCIPAL_SURFACE));
        match self {
            Self::CounterAck { through } => {
                content.insert("audit_entry".into(), json!(through.map(|(n, _)| n)));
                if let Some((_, hash)) = through {
                    content.insert("audit_hash".into(), json!(hash.to_hex()));
                }
            }
            Self::ConfigurationAccept { digest, settings } => {
                content.insert("digest".into(), json!(digest.to_hex()));
                content.insert("settings".into(), settings.clone());
            }
            Self::RiskAcceptanceRecord { risks } => {
                let ids: Vec<&str> = risks.iter().map(|r| r.id()).collect();
                content.insert("risks".into(), json!(ids));
                content.insert("forgeable".into(), json!(FORGEABLE));
            }
            Self::RiskAcceptanceWithdraw => {}
            Self::RepositoryBind { identity } => {
                if let Value::Object(fields) = identity {
                    content.extend(fields.clone());
                }
            }
        }

        Value::Object(content)
    }
}

/// A principal act as the record holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    pub seq: Seq,
    pub name: String,
    pub class: Class,
    pub content: Value,
}

/// The principal acts in `node`'s device seat's writer, in seq order.
///
/// # Errors
///
/// Fails when the store does.
pub fn acts(store: &Store, node: &Node) -> Result<Vec<Recorded>, AdminError> {
    let log = store.log(node.device_writer())?;

    Ok(log
        .into_iter()
        .filter(|s| s.link.header.kind == EventKind::PrincipalAct)
        .filter_map(|s| {
            let content = s.content?;
            let name = content.get("act")?.as_str()?.to_owned();
            Some(Recorded {
                seq: s.link.header.address.seq,
                class: class_of(&name),
                name,
                content,
            })
        })
        .collect())
}

/// The last act named `name` among `acts`.
fn last<'a>(acts: &'a [Recorded], name: &str) -> Option<&'a Recorded> {
    acts.iter().rev().find(|a| a.name == name)
}

/// The audit entry the counters were last acknowledged through, by
/// number and hash; none when they never were, or when the log held no
/// entry then.
#[must_use]
pub fn acknowledged(acts: &[Recorded]) -> Option<(u64, Digest)> {
    let content = &last(acts, "counter_ack")?.content;
    let number = content.get("audit_entry")?.as_u64()?;
    let hash = Digest::from_hex(content.get("audit_hash")?.as_str()?).ok()?;

    Some((number, hash))
}

/// The settings the last accepted configuration set, as recorded.
#[must_use]
pub fn accepted_settings(acts: &[Recorded]) -> Option<&Map<String, Value>> {
    last(acts, "configuration_accept")?
        .content
        .get("settings")?
        .as_object()
}

/// The residual risks the standing risk acceptance names: the last one
/// recorded, unless a withdrawal followed it.
#[must_use]
pub fn standing_risk_acceptance(acts: &[Recorded]) -> Option<BTreeSet<ResidualRisk>> {
    let recorded = last(acts, "risk_acceptance_record")?;
    let withdrawn = last(acts, "risk_acceptance_withdraw").is_some_and(|w| w.seq > recorded.seq);
    if withdrawn {
        return None;
    }

    Some(
        recorded
            .content
            .get("risks")?
            .as_array()?
            .iter()
            .filter_map(|r| ResidualRisk::parse(r.as_str()?))
            .collect(),
    )
}

/// The residual risks the runs on the node leave open, and those runs,
/// each named by the writer of the seat that records it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenRisks {
    pub risks: BTreeSet<ResidualRisk>,
    pub runs: Vec<WriterId>,
}

/// The residual risks open on the node. No run records a sandbox state
/// yet (OQ-40), so each run counts as unsandboxed and leaves every risk
/// open (OWN-22); a run is recorded by its run seat, an ingest seat
/// included.
///
/// # Errors
///
/// Fails when the store does or a seat's certificate does not read.
pub fn open_residual_risks(store: &Store) -> Result<OpenRisks, AdminError> {
    let mut runs = Vec::new();
    for seat in store.seats()? {
        if SeatCertificate::from_value(&seat.certificate)?.seat_kind == SeatKind::Run {
            runs.push(seat.writer);
        }
    }
    let risks = if runs.is_empty() {
        BTreeSet::new()
    } else {
        RESIDUAL_RISKS.into_iter().collect()
    };

    Ok(OpenRisks { risks, runs })
}

/// What decides whether an act is permitted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gate {
    /// Whether standard input and output are both a terminal.
    pub terminal: bool,
    /// Whether managed policy requires an authenticator for widening
    /// acts (SEC-22).
    pub authenticator_required: bool,
    /// Whether managed policy fails, so what it requires is unknown.
    pub managed_policy_failed: bool,
    /// Whether managed policy forbids risk acceptance.
    pub risk_acceptance_forbidden: bool,
    pub open: OpenRisks,
    pub standing: Option<BTreeSet<ResidualRisk>>,
}

/// Why an act was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Standard input or output is not a terminal (OWN-12).
    NoTerminal,
    /// An authenticator is required, and presence proofs come later
    /// (SEC-22).
    AuthenticatorRequired,
    /// Managed policy fails, so a widening act cannot know what it
    /// requires (ADM-04).
    ManagedPolicyInvalid,
    /// Managed policy forbids risk acceptance (SEC-22).
    RiskAcceptanceForbidden,
    /// Runs leave residual risks open that no standing risk acceptance
    /// names (OWN-22).
    OpenRisks {
        risks: BTreeSet<ResidualRisk>,
        runs: Vec<WriterId>,
    },
}

impl Refusal {
    /// The audit entry's detail.
    #[must_use]
    pub fn detail(&self) -> &'static str {
        match self {
            Self::NoTerminal => "refused: no terminal on standard input and output",
            Self::AuthenticatorRequired => "refused: an authenticator is required",
            Self::ManagedPolicyInvalid => "refused: managed policy is invalid",
            Self::RiskAcceptanceForbidden => "refused: managed policy forbids risk acceptance",
            Self::OpenRisks { .. } => "refused: residual risks open with no risk acceptance",
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTerminal => f.write_str(
                "refused: a principal act needs a terminal on standard input and output (OWN-12)",
            ),
            Self::AuthenticatorRequired => f.write_str(
                "refused: an authenticator is required for widening acts, and presence proofs \
                 are not supported yet (SEC-22)",
            ),
            Self::ManagedPolicyInvalid => f.write_str(
                "refused: managed policy is invalid, so no widening act is taken (ADM-04)",
            ),
            Self::RiskAcceptanceForbidden => {
                f.write_str("refused: managed policy forbids risk acceptance (SEC-22)")
            }
            Self::OpenRisks { risks, runs } => {
                let risks: String = risks.iter().map(|r| format!("  {r}\n")).collect();
                let runs: Vec<String> = runs.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "refused: these residual risks are open and no risk acceptance names them \
                     (OWN-22); record one with parinn risk-acceptance record:\n{risks}\
                     runs leaving them open, as no run records a sandbox state: {}",
                    runs.join(", ")
                )
            }
        }
    }
}

/// Whether `gate` permits `act`.
///
/// # Errors
///
/// Fails with the first refusal that applies, the terminal checked
/// first.
pub fn permit(act: &Act, gate: &Gate) -> Result<(), Refusal> {
    if !gate.terminal {
        return Err(Refusal::NoTerminal);
    }
    if act.class() != Class::Widening {
        return Ok(());
    }
    if gate.managed_policy_failed {
        return Err(Refusal::ManagedPolicyInvalid);
    }
    if gate.authenticator_required {
        return Err(Refusal::AuthenticatorRequired);
    }
    if let Act::RiskAcceptanceRecord { .. } = act {
        if gate.risk_acceptance_forbidden {
            return Err(Refusal::RiskAcceptanceForbidden);
        }
        return Ok(());
    }
    let covered = gate.standing.clone().unwrap_or_default();
    let uncovered: BTreeSet<ResidualRisk> = gate.open.risks.difference(&covered).copied().collect();
    if uncovered.is_empty() {
        Ok(())
    } else {
        Err(Refusal::OpenRisks {
            risks: uncovered,
            runs: gate.open.runs.clone(),
        })
    }
}

/// Records `act` in `node`'s device seat's writer.
///
/// # Errors
///
/// Fails when no commitment key can be drawn or the store fails.
pub fn record(
    store: &mut Store,
    node: &Node,
    act: &Act,
    entropy: &mut dyn Entropy,
) -> Result<Address, AdminError> {
    let event = NewEvent {
        kind: EventKind::PrincipalAct,
        provenance: Provenance::Operator,
        content: act.content(),
        commitment_key: CommitmentKey::from_bytes(draw(entropy)?),
        transcript_timestamp: None,
    };
    let addresses = store.append(node.device_writer(), &[event])?;

    addresses
        .into_iter()
        .next()
        .ok_or(AdminError::Node("the store recorded no principal act"))
}

#[cfg(test)]
mod tests;
