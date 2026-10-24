//! `parinn repository show|bind` (LANE-02): what identity a directory's
//! repository has, and the widening principal act that binds it by hand
//! or rebinds it after a change.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ::repository::CommitId;
use std::collections::BTreeMap;
use std::path::Path;

use administration::act::Act;
use administration::home::Home;
use administration::node::Node;
use administration::repository::{self, Outcome, Shown};
use audit::entry::{Counter, Kind, Occurrence};
use key::random::OsEntropy;
use serde_json::json;
use store::Store;

use crate::act::{Taking, at_terminal};
use crate::{Context, EXIT_USAGE, failed, written};

/// How long a look at a clone's git data may take (ENG-03).
const WITHIN: Duration = Duration::from_secs(10);

/// How long all the clones an ingest or an MCP server start looks at may
/// take together: the server's start must not wait on a large or hostile
/// clone (I9).
const START_WITHIN: Duration = Duration::from_secs(2);

/// What a command line asked for.
struct Asked {
    directory: PathBuf,
    commit: Option<CommitId>,
    json: bool,
}

/// Reads `args`, the directory against `ctx`'s working directory.
fn parse(args: impl Iterator<Item = String>, bind: bool, ctx: &Context) -> Result<Asked, String> {
    let (mut directory, mut commit, mut json) = (None, None, false);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--commit" if bind => {
                let text = args.next().ok_or("--commit wants a commit id")?;
                commit = Some(
                    CommitId::parse(&text)
                        .ok_or("the commit must be 40 lowercase hexadecimal digits")?,
                );
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag:?}")),
            _ if directory.is_none() => directory = Some(PathBuf::from(arg)),
            _ => return Err("want at most one directory".to_owned()),
        }
    }
    let directory = directory.unwrap_or_default();
    let base = ctx.working_dir.as_deref().unwrap_or_else(|| Path::new(""));
    let directory = base.join(directory);

    Ok(Asked {
        directory,
        commit,
        json,
    })
}

/// A deadline `within` from now.
#[expect(
    clippy::disallowed_methods,
    reason = "a deadline is wall-clock time (ENG-03); no derived artifact reads it (I10)"
)]
fn deadline(within: Duration) -> impl Fn() -> bool {
    let end = Instant::now() + within;

    move || Instant::now() >= end
}

/// Binds the identity of every repository the session bindings an MCP
/// server holds, and no look has marked, showed, once each (LANE-02).
///
/// The bind is recorded here, when an MCP server starts, and by a
/// server's live ingest as it takes a session binding, not in the hook
/// handler (reading git cannot stay within its budget, NFR-02) and not by
/// `parinn ingest` (LANE-02). Only the `cwd` of a session binding an MCP
/// server holds is read (SEC-18): a hook input no server took, which any
/// process able to run the hook handler could forge, steers nothing.
/// A change in a bound identity is audited and counted, a clone that
/// cannot be read counted, each once (I6); a bind the record refuses is
/// counted under its own name once for each observation, and tried again
/// at each start until it lands. Nothing is relinked, and a failure
/// leaves the record as it was (I9).
pub(crate) fn bind_observed(ctx: &Context, home: &Home, store: &mut Store) {
    if let (Ok(node), Ok(held)) = (Node::find(home, store), store.hook_observations()) {
        // One budget for every directory this call looks at (ENG-03, I9).
        let say = |occurrence: &Occurrence| {
            // A failed audit is said in the log: nothing else can say it.
            let _ = ctx.audit(occurrence).map_err(|e| {
                tracing::warn!(error = e.to_string().as_str(), "audit a repository look");
            });
        };
        let ceiling = ctx.home.as_deref().map(Path::new);
        bind_held(&say, ceiling, store, &node, held, &deadline(START_WITHIN));
    }
}

/// The deadline all the looks of one start share.
pub(crate) fn start_deadline() -> impl Fn() -> bool {
    deadline(START_WITHIN)
}

/// Marks every observation of `ids` as failed to bind; whether any was
/// not marked before. A mark that cannot be made counts as a first
/// failure, as the failure is better counted twice than not at all (I6).
fn first_failure(store: &mut Store, ids: &[i64]) -> bool {
    let mut first = false;
    for id in ids {
        let key = failed_mark(*id);
        first |= store.worktree_mark(&key).ok().flatten().is_none();
        let _ = store.set_worktree_mark(&key, "1").map_err(|e| {
            tracing::warn!(error = e.to_string().as_str(), "mark a failed bind");
        });
    }

    first
}

/// The key a failed bind of observation `id` is marked under.
fn failed_mark(id: i64) -> String {
    format!("failed:{id}")
}

/// The key a look at observation `id` is marked under.
fn looked(id: i64) -> String {
    format!("looked:{id}")
}

/// [`bind_observed`]'s work over the observations `held`: each
/// `SessionStart` observation an MCP server took as its session binding
/// and no look has marked, whether or not it has reached the record, is
/// looked at once. Its directory's outcome is
/// reported through `say`, and the observation marked, so a look that ran
/// out of time or found a clone unreadable is not made, or counted,
/// again for it; a new hook event for the directory looks again. A bind
/// the record refused leaves the mark off, to be tried again.
pub(crate) fn bind_held(
    say: &dyn Fn(&Occurrence),
    ceiling: Option<&Path>,
    store: &mut Store,
    node: &Node,
    held: Vec<store::Held>,
    deadline: &dyn ::repository::Deadline,
) {
    let mut by_directory: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for h in held {
        let o = h.observation;
        if o.hook == "SessionStart"
            && h.taken
            && !o.cwd.is_empty()
            && store.worktree_mark(&looked(h.id)).ok().flatten().is_none()
        {
            by_directory.entry(o.cwd).or_default().push(h.id);
        }
    }
    for (directory, ids) in by_directory {
        let outcome = repository::bind(
            store,
            node,
            &mut OsEntropy,
            Path::new(&directory),
            ceiling,
            deadline,
        );
        let occurrence = match outcome {
            Ok(Outcome::Changed) => Some(Occurrence::Degraded {
                kind: Kind::Rejected,
                counter: Counter::RepositoryIdentityChanged,
                operation: "repository bind",
                detail: "repository identity changed: no branch link is relinked until a widening bind",
            }),
            Ok(Outcome::Rebound) => Some(Occurrence::Degraded {
                kind: Kind::Dropped,
                counter: Counter::RepositoryIdentityRebound,
                operation: "repository bind",
                detail: "worktree bound again after node key change, identity bound by structural event",
            }),
            Ok(Outcome::Unreadable) => Some(Occurrence::Degraded {
                kind: Kind::Failed,
                counter: Counter::RepositoryReadFailed,
                operation: "repository bind",
                detail: "repository could not be read: its identity stays as it was",
            }),
            Ok(Outcome::TimedOut) => Some(Occurrence::Degraded {
                kind: Kind::TimedOut,
                counter: Counter::RepositoryReadTimedOut,
                operation: "repository bind",
                detail: "repository look ran out of time: bind it by hand, or the next hook event looks again",
            }),
            Err(e) => {
                // The audit entry is fixed text; the cause goes to the log.
                tracing::warn!(error = e.to_string().as_str(), "bind a repository identity");
                // The look is tried again at each start, but counted once
                // for each observation (I6).
                if first_failure(store, &ids) {
                    say(&Occurrence::Degraded {
                        kind: Kind::Failed,
                        counter: Counter::RepositoryBindFailed,
                        operation: "repository bind",
                        detail: "repository identity could not be bound: the record or the random source failed",
                    });
                }
                continue;
            }
            Ok(_) => None,
        };
        if let Some(occurrence) = &occurrence {
            say(occurrence);
        }
        for id in ids {
            // Unmarked, the observation is looked at again; say why.
            let _ = store.set_worktree_mark(&looked(id), "1").map_err(|e| {
                tracing::warn!(error = e.to_string().as_str(), "mark a repository look");
            });
        }
    }
}

/// `parinn repository show|bind [<directory>] [--commit <id>] [--json]`.
pub(crate) fn repository(
    mut args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let verb = args.next();
    let bind = match verb.as_deref() {
        Some("show") => false,
        Some("bind") => true,
        _ => {
            let _ = writeln!(stderr, "parinn repository: want show or bind");
            return EXIT_USAGE;
        }
    };
    let op = if bind {
        "repository bind"
    } else {
        "repository show"
    };
    let asked = match parse(args, bind, ctx) {
        Ok(asked) => asked,
        Err(why) => {
            let _ = writeln!(stderr, "parinn {op}: {why}");
            return EXIT_USAGE;
        }
    };
    if bind {
        binding(ctx, &asked, stdout, stderr)
    } else {
        showing(ctx, &asked, stdout, stderr)
    }
}

/// `parinn repository show`.
fn showing(ctx: &Context, asked: &Asked, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "repository show";
    match shown_line(ctx, asked) {
        Ok(text) => written(writeln!(stdout, "{text}"), stderr),
        Err(e) => failed(OP, &e, stderr),
    }
}

/// What `parinn repository show` prints.
fn shown_line(ctx: &Context, asked: &Asked) -> Result<String, String> {
    // Read-only: a node that never started, or a store that was never
    // made, is none, and nothing is created or written.
    let home = ctx.home().map_err(|e| e.to_string())?;
    let kept = if home.store().exists() {
        let store = Store::open_read_only(&home.store()).map_err(|e| e.to_string())?;
        Node::find(&home, &store).ok().map(|node| (store, node))
    } else {
        None
    };
    let shown = repository::show(
        kept.as_ref().map(|(store, node)| (store, node)),
        &asked.directory,
        ctx.home.as_deref().map(Path::new),
        &deadline(WITHIN),
    )
    .map_err(|e| e.to_string())?;
    let (status, commit) = match shown {
        Shown::NoRepository => ("no repository", None),
        Shown::Unreadable => ("unreadable", None),
        Shown::Unbound(c) => ("unbound", Some(c)),
        Shown::Bound(c) => ("bound", Some(c)),
        Shown::Changed(c) => ("changed", Some(c)),
    };
    // A provisional identity has no commit to name.
    let named = commit.as_ref().map(|c| c.as_ref().map(CommitId::as_str));

    Ok(match (asked.json, named) {
        (true, named) => json!({"status": status, "repository": named.flatten()}).to_string(),
        (false, Some(c)) => format!("{status}: {}", c.unwrap_or("provisional")),
        (false, None) => status.to_owned(),
    })
}

/// `parinn repository bind`: the widening principal act.
fn binding(ctx: &Context, asked: &Asked, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    const OP: &str = "repository bind";
    if let Err(code) = at_terminal(ctx, OP, stderr) {
        return code;
    }
    match bound_line(ctx, asked, stderr) {
        Ok(line) => written(writeln!(stdout, "{line}"), stderr),
        Err(code) => code,
    }
}

/// Takes the act and prints what it bound, or the exit code of the
/// failure it reported.
fn bound_line(ctx: &Context, asked: &Asked, stderr: &mut dyn Write) -> Result<String, u8> {
    const OP: &str = "repository bind";
    let mut taking = Taking::open(ctx).map_err(|e| failed(OP, &e, stderr))?;
    let rebind = repository::plan(
        &mut taking.store,
        &taking.node,
        &mut OsEntropy,
        &asked.directory,
        ctx.home.as_deref().map(Path::new),
        asked.commit.clone(),
        &deadline(WITHIN),
    )
    .map_err(|e| failed(OP, &e, stderr))?;
    let act = Act::RepositoryBind {
        identity: rebind.content(),
    };
    let address = taking.take(ctx, OP, &act, stderr)?;
    repository::apply(&mut taking.store, &rebind).map_err(|e| failed(OP, &e, stderr))?;
    ctx.audit(&Occurrence::PrincipalAct {
        operation: OP,
        detail: "repository identity bound",
    })
    .map_err(|e| {
        let why = format!("the bind is recorded, but the audit log took no note of it: {e}");
        failed(OP, &why, stderr)
    })?;
    let commit = rebind.commit.as_ref().map(CommitId::as_str);

    Ok(if asked.json {
        json!({"repository": commit, "act": address.to_string()}).to_string()
    } else {
        format!("bound: {}", commit.unwrap_or("provisional"))
    })
}

#[cfg(test)]
mod tests;
