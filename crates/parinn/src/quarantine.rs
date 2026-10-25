//! `parinn quarantine add|release|list` (SEC-12, RCL-06): holds events
//! out of every path that carries content to an agent, keeping them (I5).
//! `add` is a cut act while no pin, pin version or landmark can be
//! removed from a restore block, and `release` a widening act, each at a
//! terminal (OWN-11, OWN-12). Each is an `operator` event in the device
//! seat that states the selector and the reason, and takes effect at
//! once: recall reads the quarantine set from the record on every call.
//!
//! The selector is one of `--range`, `--run`, `--author`, `--room`,
//! `--writer`, `--provenance`, `--flag` and `--since`. `show`, the
//! forensic view of quarantined content, waits for VIEW-11.

use std::io::Write;

use administration::act::Act;
use administration::quarantine::{QuarantineSet, Selector, device_writers};
use audit::entry::Occurrence;
use record::id::{RoomId, SeatId};
use serde_json::json;

use crate::act::{Taking, at_terminal};
use crate::event::safe;
use crate::purge::resolve;
use crate::{Context, EXIT_USAGE, failed, start_node, written};

/// What a command line asked for.
#[derive(Debug, PartialEq, Eq)]
enum Asked {
    Add { selector: Wanted, reason: String },
    Release { selector: Wanted },
    List { as_json: bool },
}

/// A selector as typed: a range or a writer may be a label that only the
/// store can resolve.
#[derive(Debug, PartialEq, Eq)]
enum Wanted {
    Range(String),
    Writer(String),
    Done(Selector),
}

/// Reads the command line after `quarantine`.
fn parse(mut args: impl Iterator<Item = String>) -> Result<Asked, String> {
    let verb = args.next().ok_or("want add, release or list")?;
    let (mut selector, mut reason, mut as_json) = (None, None, false);
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| args.next().ok_or(format!("{flag} wants a value"));
        let mut choose = |wanted: Result<Wanted, String>| {
            if selector.is_some() {
                return Err("want one selector".to_owned());
            }
            selector = Some(wanted?);
            Ok(())
        };
        match arg.as_str() {
            "--json" => as_json = true,
            "--reason" => reason = Some(value("--reason")?),
            "--range" => choose(Ok(Wanted::Range(value("--range")?)))?,
            "--writer" => choose(Ok(Wanted::Writer(value("--writer")?)))?,
            "--run" => choose(done(Selector::run(&value("--run")?)))?,
            "--author" => choose(
                SeatId::parse(&value("--author")?)
                    .map(|seat| Wanted::Done(Selector::Author(seat)))
                    .map_err(|_| "--author wants a seat id".to_owned()),
            )?,
            "--room" => choose(
                RoomId::parse(&value("--room")?)
                    .map(|room| Wanted::Done(Selector::Room(room)))
                    .map_err(|_| "--room wants a room id".to_owned()),
            )?,
            "--provenance" => choose(done(Selector::provenance(&value("--provenance")?)))?,
            "--flag" => choose(done(Selector::flag(&value("--flag")?)))?,
            "--since" => choose(done(Selector::since(&value("--since")?)))?,
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag:?}")),
            other => return Err(format!("unexpected {other:?}")),
        }
    }
    let none_given = selector.is_none();
    let selector_for =
        |verb: &str| selector.ok_or(format!("{verb} wants a selector, such as --run ID"));
    match verb.as_str() {
        "add" => Ok(Asked::Add {
            selector: selector_for("add")?,
            reason: reason.unwrap_or_default(),
        }),
        "release" if reason.is_none() => Ok(Asked::Release {
            selector: selector_for("release")?,
        }),
        "release" => Err("release states no reason".to_owned()),
        "list" if none_given && reason.is_none() => Ok(Asked::List { as_json }),
        "list" => Err("list takes no selector".to_owned()),
        "show" => Err("show, the forensic view, waits for the room view (VIEW-11)".to_owned()),
        _ => Err("want add, release or list".to_owned()),
    }
}

fn done(selector: Result<Selector, administration::quarantine::Invalid>) -> Result<Wanted, String> {
    selector.map(Wanted::Done).map_err(|e| e.to_string())
}

/// `parinn quarantine`.
pub(crate) fn quarantine(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let asked = match parse(args) {
        Ok(asked) => asked,
        Err(e) => {
            let _ = writeln!(stderr, "parinn quarantine: {e}");
            return EXIT_USAGE;
        }
    };
    match asked {
        Asked::List { as_json } => list(ctx, as_json, stdout, stderr),
        Asked::Add { selector, reason } => change(ctx, true, selector, reason, stdout, stderr),
        Asked::Release { selector } => change(ctx, false, selector, String::new(), stdout, stderr),
    }
}

/// The selector `wanted` names, its range or writer resolved in the store.
fn selector_of(taking: &mut Taking, wanted: Wanted) -> Result<Selector, String> {
    match wanted {
        Wanted::Done(selector) => Ok(selector),
        Wanted::Range(text) => {
            let (writer, first, last) = resolve(&mut taking.store, &taking.node, &text)?;

            Ok(Selector::Range {
                writer,
                first,
                last,
            })
        }
        Wanted::Writer(text) => {
            let (writer, _, _) = resolve(&mut taking.store, &taking.node, &format!("{text}:1"))?;

            Ok(Selector::Writer(writer))
        }
    }
}

/// A quarantine act that did not happen.
enum Failure {
    /// A check already reported it, with this exit code.
    Code(u8),
    /// Something failed; the message says what.
    Plain(String),
}

fn plain(e: impl std::fmt::Display) -> Failure {
    Failure::Plain(e.to_string())
}

/// `quarantine add` and `quarantine release`.
fn change(
    ctx: &Context,
    add: bool,
    wanted: Wanted,
    reason: String,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let op = if add {
        "quarantine add"
    } else {
        "quarantine release"
    };
    if let Err(code) = at_terminal(ctx, op, stderr) {
        return code;
    }
    match changed(ctx, op, add, wanted, reason, stderr) {
        Ok(line) => written(writeln!(stdout, "{line}"), stderr),
        Err(Failure::Code(code)) => code,
        Err(Failure::Plain(why)) => failed(op, &why, stderr),
    }
}

/// Takes the act and says what it did.
fn changed(
    ctx: &Context,
    op: &'static str,
    add: bool,
    wanted: Wanted,
    reason: String,
    stderr: &mut dyn Write,
) -> Result<String, Failure> {
    let mut taking = Taking::open(ctx).map_err(plain)?;
    let selector = selector_of(&mut taking, wanted).map_err(plain)?;
    if !add && !holds(&taking, &selector) {
        return Err(plain("no quarantine holds that selector"));
    }
    let chosen = selector.clone();
    let act = if add {
        Act::QuarantineAdd {
            selector: chosen,
            reason,
        }
    } else {
        Act::QuarantineRelease { selector: chosen }
    };
    let address = taking.take(ctx, op, &act, stderr).map_err(Failure::Code)?;
    let detail = if add {
        "quarantine added"
    } else {
        "quarantine released"
    };
    let noted = Occurrence::PrincipalAct {
        operation: op,
        detail,
    };
    ctx.audit(&noted).map_err(unnoted)?;
    let said = if add { "quarantined" } else { "released" };

    Ok(format!("{said} {}; act {address}", selector.describe()))
}

/// The failure to note a recorded act in the audit log.
fn unnoted(e: impl std::fmt::Display) -> Failure {
    plain(format!(
        "the act is recorded, but the audit log took no note of it: {e}"
    ))
}

/// Whether the quarantine set `taking` holds has `selector` in force.
fn holds(taking: &Taking, selector: &Selector) -> bool {
    let devices = device_writers(&taking.store, taking.node.personal_room()).unwrap_or_default();

    QuarantineSet::load(&taking.store, &devices)
        .is_ok_and(|set| set.active().iter().any(|a| a.selector == *selector))
}

/// The quarantine set the home holds, starting the node when it never
/// started.
fn load_set(ctx: &Context, stderr: &mut dyn Write) -> Result<QuarantineSet, String> {
    let home = ctx.home().map_err(|e| e.to_string())?;
    let store = start_node(ctx, &home, stderr)?;
    let node = administration::node::Node::find(&home, &store).map_err(|e| e.to_string())?;
    let devices = device_writers(&store, node.personal_room()).map_err(|e| e.to_string())?;

    QuarantineSet::load(&store, &devices).map_err(|e| e.to_string())
}

/// `quarantine list`: the selectors in force, with the seq of the act
/// that added each and its stated reason.
fn list(ctx: &Context, as_json: bool, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let set = match load_set(ctx, stderr) {
        Ok(set) => set,
        Err(e) => return failed("quarantine list", &e, stderr),
    };
    let text = if as_json {
        let held: Vec<_> = set
            .active()
            .iter()
            .map(|a| json!({"selector": a.selector.to_value(), "act": a.act.get(), "reason": a.reason}))
            .collect();
        json!({"quarantines": held}).to_string()
    } else if set.is_empty() {
        "nothing is quarantined".to_owned()
    } else {
        set.active()
            .iter()
            .map(|a| {
                format!(
                    "{}; act {}; reason: {}",
                    a.selector.describe(),
                    a.act,
                    safe(&a.reason)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    written(writeln!(stdout, "{text}"), stderr)
}

#[cfg(test)]
mod tests;
