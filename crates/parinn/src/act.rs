//! The verbs that take a principal act (§9.5): `parinn counter ack`,
//! `parinn configuration accept` and `parinn risk-acceptance`. Each
//! refuses without a terminal before any other check (OWN-12), audits
//! and counts every refusal (OWN-02, I6), and records the act in this
//! node's device seat (OWN-02).

use std::collections::BTreeSet;
use std::io::Write;

use administration::act::{
    self, Act, Class, FORGEABLE, Gate, OpenRisks, Recorded, Refusal, ResidualRisk,
    open_residual_risks, permit, standing_risk_acceptance,
};
use administration::home::Home;
use administration::node::Node;
use audit::entry::{Counter, Kind, Occurrence};
use audit::log::Log;
use key::random::OsEntropy;
use record::digest::Digest;
use record::id::Address;
use serde_json::json;
use store::Store;

use crate::settings::{InForce, acts_in, in_force, recorded};
use crate::{Context, EXIT_USAGE, failed, json_flag, written};

/// What an act is taken in: the home, its store and node, the acts it
/// recorded, and the settings in force.
pub(crate) struct Taking {
    pub(crate) home: Home,
    pub(crate) store: Store,
    pub(crate) node: Node,
    acts: Vec<Recorded>,
    settings: InForce,
}

impl Taking {
    /// Opens the home for an act, starting the node when it never has.
    pub(crate) fn open(ctx: &Context) -> Result<Self, String> {
        let home = ctx.home().map_err(|e| e.to_string())?;
        home.create().map_err(|e| e.to_string())?;
        let mut store = Store::open(&home.store()).map_err(|e| e.to_string())?;
        let node = crate::open_node(ctx, &home, &mut store, &mut std::io::sink())?;
        let acts = act::acts(&store, &node).map_err(|e| e.to_string())?;
        let settings = in_force(ctx, &home, &acts);

        Ok(Self {
            home,
            store,
            node,
            acts,
            settings,
        })
    }

    /// The residual risks open on the node.
    fn open_risks(&self) -> Result<OpenRisks, String> {
        open_residual_risks(&self.store).map_err(|e| e.to_string())
    }

    /// What decides whether `act` is permitted here. Only a widening
    /// act reads the open residual risks. The principal's configuration
    /// or managed policy requires an authenticator (OWN-11); only
    /// managed policy forbids risk acceptance (SEC-22).
    fn gate(&self, ctx: &Context, act: &Act) -> Result<Gate, String> {
        let open = if act.class() == Class::Widening {
            self.open_risks()?
        } else {
            OpenRisks::default()
        };
        let settings = &self.settings.resolved.settings;

        Ok(Gate {
            terminal: ctx.terminal,
            authenticator_required: settings.authenticator_required(),
            managed_policy_failed: self.settings.loaded.managed_policy_failed(),
            risk_acceptance_forbidden: settings.risk_acceptance_forbidden(),
            open,
            standing: standing_risk_acceptance(&self.acts),
        })
    }

    /// Takes `act` if the gate permits it: records it, or audits and
    /// reports the refusal.
    pub(crate) fn take(
        &mut self,
        ctx: &Context,
        operation: &'static str,
        act: &Act,
        stderr: &mut dyn Write,
    ) -> Result<Address, u8> {
        let gate = self
            .gate(ctx, act)
            .map_err(|e| failed(operation, &e, stderr))?;
        if let Err(refusal) = permit(act, &gate) {
            return Err(refuse(ctx, operation, &refusal, stderr));
        }
        act::record(&mut self.store, &self.node, act, &mut OsEntropy)
            .map_err(|e| failed(operation, &e, stderr))
    }
}

/// Audits and reports `refusal` of an act under `operation`.
fn refuse(ctx: &Context, operation: &'static str, refusal: &Refusal, stderr: &mut dyn Write) -> u8 {
    refused(
        ctx,
        operation,
        refusal.detail(),
        &refusal.to_string(),
        stderr,
    )
}

/// Audits a refused act with `detail` and reports `why`.
fn refused(
    ctx: &Context,
    operation: &'static str,
    detail: &'static str,
    why: &str,
    stderr: &mut dyn Write,
) -> u8 {
    let occurrence = Occurrence::Degraded {
        kind: Kind::Rejected,
        counter: Counter::PrincipalActRefused,
        operation,
        detail,
    };
    if let Err(e) = ctx.audit(&occurrence) {
        let _ = writeln!(stderr, "parinn {operation}: audit the refusal: {e}");
    }

    failed(operation, &why, stderr)
}

/// Refuses an act unless standard input and output are a terminal, the
/// first check of every verb that writes one (OWN-12).
pub(crate) fn at_terminal(
    ctx: &Context,
    operation: &'static str,
    stderr: &mut dyn Write,
) -> Result<(), u8> {
    if ctx.terminal {
        Ok(())
    } else {
        Err(refuse(ctx, operation, &Refusal::NoTerminal, stderr))
    }
}

/// `parinn counter ack [--json]`: acknowledges the counters through the
/// audit log's last entry, named by number and hash (OPS-03), a neutral
/// principal act. Only once the act is recorded does the audit log note
/// it, so the log never notes an acknowledgement the record lacks.
pub(crate) fn counter(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    const OP: &str = "counter ack";
    if args.next().as_deref() != Some("ack") {
        let _ = writeln!(stderr, "parinn counter: want ack");
        return EXIT_USAGE;
    }
    let Some(as_json) = json_flag(OP, args, stderr) else {
        return EXIT_USAGE;
    };
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    let mut taking = match Taking::open(ctx) {
        Ok(t) => t,
        Err(e) => return failed(OP, &e, stderr),
    };
    let through = match Log::new(taking.home.audit()).entries() {
        Ok(entries) => entries.last().map(|e| (e.number, e.hash)),
        Err(e) => return failed(OP, &e, stderr),
    };
    let address = match taking.take(ctx, OP, &Act::CounterAck { through }, stderr) {
        Ok(a) => a,
        Err(code) => return code,
    };
    let noted = ctx.audit(&Occurrence::PrincipalAct {
        operation: OP,
        detail: "counters acknowledged",
    });
    if let Err(e) = noted {
        let why =
            format!("the acknowledgement is recorded, but the audit log took no note of it: {e}");
        return failed(OP, &why, stderr);
    }
    let line = match (as_json, through) {
        (true, _) => {
            json!({"acknowledged": through.map(|(n, _)| n), "act": address.to_string()}).to_string()
        }
        (false, Some((n, _))) => format!("counters acknowledged through audit entry {n}"),
        (false, None) => "counters acknowledged; the audit log held no entry".to_owned(),
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// `parinn configuration accept [--json]`: accepts the principal's
/// configuration, recording its digest and settings (ADM-04), a
/// widening principal act.
pub(crate) fn configuration(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    const OP: &str = "configuration accept";
    if args.next().as_deref() != Some("accept") {
        let _ = writeln!(stderr, "parinn configuration: want accept");
        return EXIT_USAGE;
    }
    let Some(as_json) = json_flag(OP, args, stderr) else {
        return EXIT_USAGE;
    };
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    let mut taking = match Taking::open(ctx) {
        Ok(t) => t,
        Err(e) => return failed(OP, &e, stderr),
    };
    let (digest, act) = match acceptance(&taking, stderr) {
        Ok(accepting) => accepting,
        Err(code) => return code,
    };
    let address = match taking.take(ctx, OP, &act, stderr) {
        Ok(a) => a,
        Err(code) => return code,
    };
    let applying: Vec<&str> = taking
        .settings
        .resolved
        .awaiting_acceptance
        .iter()
        .map(|k| k.name())
        .collect();
    let line = if as_json {
        json!({"digest": digest.to_hex(), "act": address.to_string(), "applying": applying})
            .to_string()
    } else if applying.is_empty() {
        format!("accepted the configuration with digest {}", digest.to_hex())
    } else {
        format!(
            "accepted the configuration with digest {}; now applying: {}",
            digest.to_hex(),
            applying.join(", ")
        )
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// The acceptance of the principal's configuration as it stands, unless
/// a layer fails (exit 2) or there is none. A key managed policy also
/// sets is accepted and stays overridden by managed policy (ADM-04).
fn acceptance(taking: &Taking, stderr: &mut dyn Write) -> Result<(Digest, Act), u8> {
    const OP: &str = "configuration accept";
    let loaded = &taking.settings.loaded;
    if loaded.principal_layer_failed() {
        return Err(crate::invalid_layers(OP, loaded, stderr));
    }
    let Some(digest) = loaded.configuration_digest else {
        let path = taking
            .home
            .root()
            .join(configuration::source::CONFIGURATION);
        let why = format!("no configuration at {} to accept", path.display());
        return Err(failed(OP, &why, stderr));
    };
    let act = Act::ConfigurationAccept {
        digest,
        settings: recorded(&loaded.layers.configuration),
    };

    Ok((digest, act))
}

/// `parinn risk-acceptance show|record|withdraw [--json]` (OWN-22).
pub(crate) fn risk_acceptance(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let verb = args.next();
    let op = match verb.as_deref() {
        Some("show") => "risk-acceptance show",
        Some("record") => "risk-acceptance record",
        Some("withdraw") => "risk-acceptance withdraw",
        _ => {
            let _ = writeln!(
                stderr,
                "parinn risk-acceptance: want show, record or withdraw"
            );
            return EXIT_USAGE;
        }
    };
    let Some(as_json) = json_flag(op, args, stderr) else {
        return EXIT_USAGE;
    };
    match verb.as_deref() {
        Some("show") => show(ctx, as_json, stdout, stderr),
        Some("record") => record_acceptance(ctx, as_json, stdout, stderr),
        _ => withdraw(ctx, as_json, stdout, stderr),
    }
}

/// The ids of `risks`.
fn ids(risks: &BTreeSet<ResidualRisk>) -> Vec<&'static str> {
    risks.iter().map(|r| r.id()).collect()
}

/// `parinn risk-acceptance show`: the residual risks open on this node,
/// the runs leaving them open, and the standing risk acceptance.
/// Read-only; it shows ids and fixed text, no record content.
fn show(ctx: &Context, as_json: bool, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "risk-acceptance show";
    let shown = ctx.home().map_err(|e| e.to_string()).and_then(|home| {
        if !home.store().exists() {
            return Ok((OpenRisks::default(), None));
        }
        let store = Store::open(&home.store()).map_err(|e| e.to_string())?;
        let acts = acts_in(&home, &store)?;
        let open = open_residual_risks(&store).map_err(|e| e.to_string())?;
        Ok((open, standing_risk_acceptance(&acts)))
    });
    let (open, standing) = match shown {
        Ok(s) => s,
        Err(e) => return failed(OP, &e, stderr),
    };
    let text = if as_json {
        let runs: Vec<String> = open.runs.iter().map(ToString::to_string).collect();
        json!({
            "open": ids(&open.risks),
            "runs": runs,
            "accepted": standing.as_ref().map(ids),
        })
        .to_string()
    } else {
        show_text(&open, standing.as_ref())
    };

    written(writeln!(stdout, "{text}"), stderr)
}

/// What `show` prints at a terminal.
fn show_text(open: &OpenRisks, standing: Option<&BTreeSet<ResidualRisk>>) -> String {
    let mut lines = Vec::new();
    if open.risks.is_empty() {
        lines.push("no residual risk is open on this node".to_owned());
    } else {
        lines.push("residual risks open on this node:".to_owned());
        lines.extend(open.risks.iter().map(|r| format!("  {r}")));
        let runs: Vec<String> = open.runs.iter().map(ToString::to_string).collect();
        lines.push(format!(
            "runs leaving them open, as no run records a sandbox state: {}",
            runs.join(", ")
        ));
    }
    lines.push(match standing {
        None => "no risk acceptance stands".to_owned(),
        Some(r) => format!("the standing risk acceptance names {}", ids(r).join(", ")),
    });

    lines.join("\n")
}

/// `parinn risk-acceptance record`: accepts every residual risk open on
/// the node, naming each and that an unsandboxed agent can forge the
/// acceptance; widening, but exempt from OWN-22's condition.
fn record_acceptance(
    ctx: &Context,
    as_json: bool,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    const OP: &str = "risk-acceptance record";
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    let mut taking = match Taking::open(ctx) {
        Ok(t) => t,
        Err(e) => return failed(OP, &e, stderr),
    };
    let risks = match taking.open_risks() {
        Ok(open) => open.risks,
        Err(e) => return failed(OP, &e, stderr),
    };
    if risks.is_empty() {
        let line = if as_json {
            json!({"accepted": [], "act": null}).to_string()
        } else {
            "no residual risk is open on this node: nothing to accept".to_owned()
        };
        return written(writeln!(stdout, "{line}"), stderr);
    }
    let act = Act::RiskAcceptanceRecord {
        risks: risks.clone(),
    };
    let address = match taking.take(ctx, OP, &act, stderr) {
        Ok(a) => a,
        Err(code) => return code,
    };
    let text = if as_json {
        json!({"accepted": ids(&risks), "act": address.to_string(), "forgeable": FORGEABLE})
            .to_string()
    } else {
        let mut lines = vec!["accepted these residual risks:".to_owned()];
        lines.extend(risks.iter().map(|r| format!("  {r}")));
        lines.push(format!("and that {FORGEABLE}"));
        lines.join("\n")
    };

    written(writeln!(stdout, "{text}"), stderr)
}

/// `parinn risk-acceptance withdraw`: withdraws the standing risk
/// acceptance, a cut act.
fn withdraw(ctx: &Context, as_json: bool, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "risk-acceptance withdraw";
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    let mut taking = match Taking::open(ctx) {
        Ok(t) => t,
        Err(e) => return failed(OP, &e, stderr),
    };
    let address = if standing_risk_acceptance(&taking.acts).is_some() {
        match taking.take(ctx, OP, &Act::RiskAcceptanceWithdraw, stderr) {
            Ok(a) => Some(a),
            Err(code) => return code,
        }
    } else {
        None
    };
    let line = match (as_json, address) {
        (true, a) => json!({"withdrawn": a.is_some(), "act": a.map(|a| a.to_string())}).to_string(),
        (false, Some(_)) => "withdrew the risk acceptance".to_owned(),
        (false, None) => "no risk acceptance stands: nothing to withdraw".to_owned(),
    };

    written(writeln!(stdout, "{line}"), stderr)
}

#[cfg(test)]
mod tests;
