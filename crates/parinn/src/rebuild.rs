//! `parinn rebuild` (PRV-13, I10): derives every derived artifact again
//! from the record and the node's key set alone: the origins the store
//! keeps, the identity it keeps for each worktree and the search index. It reads the record and appends no event.

use std::io::Write;

use administration::node::Node;
use administration::{origin, repository};
use store::{Store, StoreError};

use crate::{Context, failed, json_flag, store_failed, written};

/// `parinn rebuild [--json]`.
pub(crate) fn rebuild(
    args: impl Iterator<Item = String>,
    ctx: &Context,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let Some(as_json) = json_flag("rebuild", args, stderr) else {
        return crate::EXIT_USAGE;
    };
    let (origins, worktrees, indexed) = match counts(ctx) {
        Ok(counts) => counts,
        Err(Failure::Store(e)) => return store_failed("rebuild", &e, stderr),
        Err(Failure::Plain(e)) => return failed("rebuild", &e, stderr),
    };
    let line = if as_json {
        format!("{{\"origins\":{origins},\"worktrees\":{worktrees},\"indexed\":{indexed}}}")
    } else {
        format!(
            "rebuilt: {origins} origins, {worktrees} worktrees, the search index of {indexed} events"
        )
    };

    written(writeln!(stdout, "{line}"), stderr)
}

/// Why a rebuild failed: the store failed, so that a store of a newer
/// version is told from the rest (ADM-05), or something else did.
enum Failure {
    Store(StoreError),
    Plain(String),
}

impl From<StoreError> for Failure {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

/// Derives everything again in `ctx`'s home; how many writers' origins,
/// how many worktrees and how many events it read.
fn counts(ctx: &Context) -> Result<(usize, usize, u64), Failure> {
    let plain = |e: &dyn std::fmt::Display| Failure::Plain(e.to_string());
    let home = ctx.home().map_err(|e| plain(&e))?;
    if !home.store().exists() {
        return Err(Failure::Plain(format!(
            "no store at {}",
            home.store().display()
        )));
    }
    let mut store = Store::open(&home.store())?;
    let node = Node::find(&home, &store).map_err(|e| plain(&e))?;

    derive_again(&mut store, &node)
}

/// Derives the origins, the worktrees' identities and the search index
/// again; how many writers' origins, how many worktrees and how many
/// events it read.
fn derive_again(store: &mut Store, node: &Node) -> Result<(usize, usize, u64), Failure> {
    let derived = origin::derive(store, &node.device().public())?;
    let kept = derived
        .iter()
        .map(|(writer, origin)| (*writer, origin.as_str().to_owned()))
        .collect();
    store.replace_origins(&kept)?;
    let worktrees =
        repository::derive_worktrees(store, node).map_err(|e| Failure::Plain(e.to_string()))?;
    store.replace_worktree_identities(&worktrees)?;
    let indexed = store.rebuild_search_index()?;

    Ok((derived.len(), worktrees.len(), indexed))
}
