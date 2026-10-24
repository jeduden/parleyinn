//! A repository's identity (LANE-02, I1, I6, I8): independent of the
//! local path and the same on every node holding a clone, a parentless
//! commit reachable from the repository's default branch, the lowest id
//! when there are several, found inside the core (no program start, no
//! socket) and bound once, at the first hook event that shows it, by a
//! structural event of the device seat that replicates (never an
//! `operator` one). The event names the worktree by a keyed digest of its
//! root path, as does each later event that binds another worktree to an
//! identity or a commit to a worktree that held a provisional one; a
//! node rebuilds which identity each of its worktrees is bound to from
//! its writer logs alone (I10).
//!
//! - A node whose clone contains a commit already bound uses that
//!   identity. One whose clone lacks it mints none: a widening principal
//!   act binds it (`parinn repository bind`).
//! - A repository with no commit, or a shallow clone with none, gets a
//!   provisional, node-local identity, and binds later without
//!   rewriting the record.
//! - A change in the identity a directory resolves to is detected and
//!   reported, never relinks a room's branches, and rebinds only on the
//!   widening act.
//! - The identity serves only to identify branches and relate a room's
//!   branches and commits, never to scope ParleyInn state. What this
//!   node keeps of a worktree and its identity is hashed under a secret
//!   derived from its device key, so no local name reveals either off it.

use std::collections::BTreeMap;
use std::path::Path;

use key::key_event::KeyEventKind;
use key::random::{Entropy, draw};
use record::commitment::CommitmentKey;
use record::digest::Digest;
use record::event::{EventKind, Provenance};
use repository::{CommitId, Deadline};
use serde_json::{Value, json};
use store::{NewEvent, Store};

use crate::AdminError;
use crate::node::Node;

/// What a directory's repository resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    /// Named by its lowest root commit.
    Bound(CommitId),
    /// Node-local, minted by this node until a commit binds it.
    Provisional(String),
}

/// What one look at a directory came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The directory lies in no repository.
    NoRepository,
    /// The directory's identity is the one it last resolved to.
    Unchanged,
    /// The directory was bound now; whether to a provisional identity.
    Bound { provisional: bool },
    /// The directory was bound now, after the node's device key was
    /// replaced (a node identity change, a rotation or a key minted
    /// anew): its kept names went with the old key, so a bind of the same
    /// worktree is indistinguishable from a first one. It keeps the same
    /// identity, so it is no rebind: audited and counted apart, and not a
    /// widening act.
    Rebound,
    /// The identity the directory resolves to changed since it was bound
    /// or last reported; nothing was relinked.
    Changed,
    /// It changed, and the change was reported already.
    StillChanged,
    /// The clone's git data could not be read.
    Unreadable,
    /// It could not be read, and that was reported already.
    StillUnreadable,
    /// The look ran out of its time; nothing was kept, so the next look
    /// tries again.
    TimedOut,
}

/// What the clone of a directory says its identity is now: a bound
/// commit, or none yet.
enum Current {
    Bound(CommitId),
    Provisional,
}

/// The label a local name is hashed under.
const LABEL: &[u8] = b"parinn repository v1\0";

/// What a provisional identity's table value starts with; a bound one's
/// is a digest, which never does.
const PROVISIONAL: &str = "provisional:";

/// The secret this node hashes its local repository names under, derived
/// from its device key, so it rebuilds with the key set (I10). It is never
/// logged, exported, backed up or placed in a segment; public so a test can
/// look for it.
#[must_use]
pub fn name_secret(node: &Node) -> [u8; 32] {
    node.device().derive_secret(b"repository names")
}

/// The digest of a worktree's root: keyed under `secret`, so it reveals
/// neither the path nor anything of the repository.
///
/// # Errors
///
/// Fails when the worktree cannot be resolved: a path as spelled would
/// keep one worktree under two names.
fn worktree_digest(secret: &[u8; 32], worktree: &Path) -> Result<String, AdminError> {
    let resolved = std::fs::canonicalize(worktree).map_err(|source| AdminError::Unresolved {
        what: "worktree",
        source,
    })?;

    Ok(Digest::of(&[
        LABEL,
        b"worktree\0",
        secret,
        resolved.to_string_lossy().as_bytes(),
    ])
    .to_hex())
}

/// The name of a directory whose clone could not be read: its worktree
/// is not known, so it is the directory's own.
fn directory_marker(secret: &[u8; 32], directory: &Path) -> Result<String, AdminError> {
    let resolved = std::fs::canonicalize(directory).map_err(|source| AdminError::Unresolved {
        what: "directory",
        source,
    })?;

    Ok(Digest::of(&[
        LABEL,
        b"directory\0",
        secret,
        resolved.to_string_lossy().as_bytes(),
    ])
    .to_hex())
}

/// What this node keeps for a worktree bound to the commit `id`.
fn bound_value(secret: &[u8; 32], id: &CommitId) -> String {
    Digest::of(&[LABEL, b"identity\0", secret, id.as_str().as_bytes()]).to_hex()
}

/// What this node keeps for a worktree bound to the provisional identity
/// `id`.
fn provisional_value(id: &str) -> String {
    format!("{PROVISIONAL}{id}")
}

/// The provisional identity a table value names, if it names one.
fn provisional_in(value: &str) -> Option<&str> {
    value.strip_prefix(PROVISIONAL)
}

/// What a clone shows now, as a change report names it: a bound
/// commit's value, or the bare word for none yet.
fn seen(secret: &[u8; 32], current: &Current) -> String {
    match current {
        Current::Bound(id) => bound_value(secret, id),
        Current::Provisional => "provisional".to_owned(),
    }
}

/// Whether `stored` records a bind: a structural event of the device
/// seat, or the principal act that binds by hand (not any act naming a
/// commit).
fn is_bind(stored: &store::Stored) -> bool {
    match stored.link.header.kind {
        EventKind::RepositoryBind => true,
        EventKind::PrincipalAct => stored
            .content
            .as_ref()
            .is_some_and(|c| c["act"] == "repository_bind"),
        _ => false,
    }
}

/// What the worktrees of this node are bound to, as the record says it:
/// each worktree's digest and its identity's table value, the later bind
/// of a worktree replacing the earlier (I10). The table `parinn rebuild`
/// writes and `parinn verify` checks.
///
/// # Errors
///
/// Fails when the store does.
pub fn derive_worktrees(
    store: &Store,
    node: &Node,
) -> Result<BTreeMap<String, String>, AdminError> {
    let secret = name_secret(node);
    let mut worktrees = BTreeMap::new();
    let log = store.log(node.device_writer())?;
    for content in log
        .iter()
        .filter(|s| is_bind(s))
        .filter_map(|s| s.content.as_ref())
    {
        let value = match (
            content["repository"].as_str().and_then(CommitId::parse),
            content["repository_provisional"].as_str(),
        ) {
            (Some(id), _) => bound_value(&secret, &id),
            (None, Some(id)) => provisional_value(id),
            (None, None) => continue,
        };
        if let Some(worktree) = content["worktree_digest"].as_str() {
            worktrees.insert(worktree.to_owned(), value);
        }
    }

    Ok(worktrees)
}

/// The commits this node has bound, by the structural events of its
/// device seat and by its principal's rebinds, lowest first.
fn bound_commits(store: &Store, node: &Node) -> Result<Vec<CommitId>, AdminError> {
    let mut bound: Vec<CommitId> = store
        .log(node.device_writer())?
        .iter()
        .filter(|s| is_bind(s))
        .filter_map(|s| s.content.as_ref()?["repository"].as_str())
        .filter_map(CommitId::parse)
        .collect();
    bound.sort();
    bound.dedup();

    Ok(bound)
}

/// What the clone at `git_dir` is now called: a commit this node bound
/// that the clone's default branch reaches as a root, else its lowest
/// root, else nothing yet. A bound commit the branch does not reach, a
/// dangling object or a side fetch leaves, names the clone no more than
/// any other commit does.
fn resolve(
    git_dir: &Path,
    bound: &[CommitId],
    deadline: &dyn Deadline,
) -> Result<Current, repository::Error> {
    let roots = repository::roots(git_dir, deadline)?;
    let chosen = bound.iter().find(|id| roots.contains(id)).or(roots.first());

    Ok(chosen.map_or(Current::Provisional, |id| Current::Bound(id.clone())))
}

/// Reports `directory`'s clone unreadable the first time it is, and
/// never again until it reads.
fn unreadable(
    store: &mut Store,
    secret: &[u8; 32],
    directory: &Path,
) -> Result<Outcome, AdminError> {
    let marker = format!("unreadable:{}", directory_marker(secret, directory)?);
    if store.worktree_mark(&marker)?.as_deref() == Some(UNREADABLE) {
        return Ok(Outcome::StillUnreadable);
    }
    store.set_worktree_mark(&marker, UNREADABLE)?;

    Ok(Outcome::Unreadable)
}

/// What a directory's marker holds while its clone is reported
/// unreadable; no identity value is this short.
const UNREADABLE: &str = "unreadable";

/// Whether this node's device key was replaced since its first: its
/// device seat's log holds a node identity change, a rotation or a key
/// minted anew (REC-24, PRV-12). Its names for worktrees went with the key
/// it replaced.
fn after_key_change(store: &Store, node: &Node) -> Result<bool, AdminError> {
    let replaced = |s: &store::Stored| match s.link.header.kind {
        EventKind::NodeIdentityChange => true,
        EventKind::KeyEvent => s
            .content
            .as_ref()
            .and_then(|c| c["kind"].as_str())
            .and_then(KeyEventKind::parse)
            .is_some_and(|k| matches!(k, KeyEventKind::MintedAnew | KeyEventKind::Rotated)),
        _ => false,
    };

    Ok(store.log(node.device_writer())?.iter().any(replaced))
}

/// A fresh node-local id for a provisional identity.
fn fresh_provisional(entropy: &mut dyn Entropy) -> Result<String, AdminError> {
    Ok(Digest::of(&[&draw::<32>(entropy)?]).to_hex()[..32].to_owned())
}

/// Looks at `directory` as a hook event shows it: binds its worktree's
/// repository identity if this node never has. The search for the
/// repository stops before `ceiling`, the OS user's home directory, so a
/// dotfiles repository there is never bound for the directories below it
/// (`None` only in tests).
///
/// # Errors
///
/// Fails when the random source or the store fails.
pub fn bind(
    store: &mut Store,
    node: &Node,
    entropy: &mut dyn Entropy,
    directory: &Path,
    ceiling: Option<&Path>,
    deadline: &dyn Deadline,
) -> Result<Outcome, AdminError> {
    let secret = name_secret(node);
    let located = match repository::locate(directory, ceiling) {
        Ok(Some(located)) => located,
        Ok(None) => return Ok(Outcome::NoRepository),
        Err(_) => return unreadable(store, &secret, directory),
    };
    let bound = bound_commits(store, node)?;
    let current = match resolve(&located.git_dir, &bound, deadline) {
        Ok(current) => current,
        Err(repository::Error::Deadline) => return Ok(Outcome::TimedOut),
        Err(_) => return unreadable(store, &secret, directory),
    };
    let worktree = worktree_digest(&secret, &located.worktree)?;
    // A clone that reads has no unreadable report to keep.
    let marker = directory_marker(&secret, directory)?;
    store.forget_worktree_mark(&format!("unreadable:{marker}"))?;
    let kept = store.worktree_identity(&worktree)?;
    let mut replaces = None;
    match (kept.as_deref(), &current) {
        // Still provisional: nothing to bind until a commit appears.
        (Some(kept), Current::Provisional) if provisional_in(kept).is_some() => {
            return Ok(Outcome::Unchanged);
        }
        // A provisional identity binds later, on its own, once its clone
        // names a commit, naming the provisional identity it replaces.
        (Some(kept), Current::Bound(_)) if provisional_in(kept).is_some() => {
            replaces = provisional_in(kept).map(str::to_owned);
        }
        (Some(kept), _) if kept == seen(&secret, &current) => {
            // Back to the identity bound: the next change reports again.
            store.forget_worktree_mark(&format!("reported:{worktree}"))?;
            return Ok(Outcome::Unchanged);
        }
        (Some(_), _) => {
            let (marker, now) = (format!("reported:{worktree}"), seen(&secret, &current));
            if store.worktree_mark(&marker)?.as_deref() == Some(now.as_str()) {
                return Ok(Outcome::StillChanged);
            }
            store.set_worktree_mark(&marker, &now)?;
            return Ok(Outcome::Changed);
        }
        (None, _) => {}
    }
    let (mut content, value) = match &current {
        Current::Bound(id) => (json!({"repository": id.as_str()}), bound_value(&secret, id)),
        Current::Provisional => {
            let id = fresh_provisional(entropy)?;
            (
                json!({"repository_provisional": id}),
                provisional_value(&id),
            )
        }
    };
    content["worktree_digest"] = json!(worktree);
    if let Some(provisional) = replaces {
        content["replaces_provisional"] = json!(provisional);
    }
    let event = NewEvent {
        kind: EventKind::RepositoryBind,
        provenance: Provenance::Structural,
        content,
        commitment_key: CommitmentKey::from_bytes(draw(entropy)?),
        transcript_timestamp: None,
    };
    // The event and the table row land together, and only if no other
    // process bound the worktree since it was looked at.
    let landed = store.bind_worktree(
        node.device_writer(),
        &event,
        &worktree,
        kept.as_deref(),
        &value,
    )?;

    let rebound = landed.is_some() && after_key_change(store, node)?;

    Ok(landed_as(
        landed.is_some(),
        rebound,
        matches!(current, Current::Provisional),
    ))
}

/// What a bind that `landed` or not came to: a bind another process made
/// first leaves the worktree as that one did.
fn landed_as(landed: bool, rebound: bool, provisional: bool) -> Outcome {
    match (landed, rebound) {
        (false, _) => Outcome::Unchanged,
        (true, true) => Outcome::Rebound,
        (true, false) => Outcome::Bound { provisional },
    }
}

/// What `parinn repository show` says of a directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    NoRepository,
    Unreadable,
    /// The clone's identity, which this node has not bound for the
    /// worktree yet.
    Unbound(Option<CommitId>),
    /// The identity the worktree is bound to and still resolves to.
    Bound(Option<CommitId>),
    /// The worktree resolves to another identity than it was bound to.
    Changed(Option<CommitId>),
}

/// What `directory` is, as this node sees it, binding and writing
/// nothing. `node` is none when the node never started: no bind exists
/// then.
///
/// # Errors
///
/// Fails when the store fails.
pub fn show(
    node: Option<(&Store, &Node)>,
    directory: &Path,
    ceiling: Option<&Path>,
    deadline: &dyn Deadline,
) -> Result<Shown, AdminError> {
    let located = match repository::locate(directory, ceiling) {
        Ok(Some(located)) => located,
        Ok(None) => return Ok(Shown::NoRepository),
        Err(_) => return Ok(Shown::Unreadable),
    };
    let bound = match node {
        Some((store, node)) => bound_commits(store, node)?,
        None => Vec::new(),
    };
    let Ok(current) = resolve(&located.git_dir, &bound, deadline) else {
        return Ok(Shown::Unreadable);
    };
    let secret = node.map(|(_, node)| name_secret(node)).unwrap_or_default();
    let kept = match node {
        Some((store, _)) => {
            store.worktree_identity(&worktree_digest(&secret, &located.worktree)?)?
        }
        None => None,
    };
    let commit = match &current {
        Current::Bound(id) => Some(id.clone()),
        Current::Provisional => None,
    };

    Ok(match kept {
        None => Shown::Unbound(commit),
        // Binds on its own at the next look (see `bind`).
        Some(kept) if provisional_in(&kept).is_some() => match current {
            Current::Provisional => Shown::Bound(None),
            Current::Bound(_) => Shown::Unbound(commit),
        },
        Some(kept) if kept == seen(&secret, &current) => Shown::Bound(commit),
        Some(_) => Shown::Changed(commit),
    })
}

/// What a widening bind will bind a worktree to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rebind {
    /// The commit it binds to; none for a provisional identity.
    pub commit: Option<CommitId>,
    /// The node-local id of a provisional identity.
    pub provisional: Option<String>,
    worktree: String,
    value: String,
    replaces: Option<String>,
}

impl Rebind {
    /// The identity the act records: a commit, or a provisional id, with
    /// the digest of the worktree it binds, and the provisional identity
    /// a commit replaces.
    #[must_use]
    pub fn content(&self) -> Value {
        let mut content = match (&self.commit, &self.provisional) {
            (Some(id), _) => json!({"repository": id.as_str()}),
            (None, Some(id)) => json!({"repository_provisional": id}),
            (None, None) => json!({}),
        };
        content["worktree_digest"] = json!(self.worktree);
        if let Some(provisional) = &self.replaces {
            content["replaces_provisional"] = json!(provisional);
        }

        content
    }
}

/// The identity a widening bind of `directory`'s worktree will give it:
/// `commit` when the principal names one, else what its clone resolves
/// to. Nothing changes until [`apply`] runs, after the act is recorded.
///
/// # Errors
///
/// Fails when the store or the random source fails, when the directory
/// lies in no repository, or when its git data cannot be read.
pub fn plan(
    store: &mut Store,
    node: &Node,
    entropy: &mut dyn Entropy,
    directory: &Path,
    ceiling: Option<&Path>,
    commit: Option<CommitId>,
    deadline: &dyn Deadline,
) -> Result<Rebind, AdminError> {
    let unreadable = AdminError::RepositoryRead;
    let located = repository::locate(directory, ceiling)
        .map_err(unreadable)?
        .ok_or(AdminError::Repository(
            "the directory lies in no repository",
        ))?;
    let current = match commit {
        Some(id) => Current::Bound(id),
        None => {
            resolve(&located.git_dir, &bound_commits(store, node)?, deadline).map_err(unreadable)?
        }
    };
    let secret = name_secret(node);
    let worktree = worktree_digest(&secret, &located.worktree)?;
    let replaces = match (&current, store.worktree_identity(&worktree)?) {
        (Current::Bound(_), Some(kept)) => provisional_in(&kept).map(str::to_owned),
        _ => None,
    };
    let provisional = match current {
        Current::Provisional => Some(fresh_provisional(entropy)?),
        Current::Bound(_) => None,
    };
    let value = match (&current, &provisional) {
        (Current::Bound(id), _) => bound_value(&secret, id),
        (Current::Provisional, id) => provisional_value(id.as_deref().unwrap_or_default()),
    };

    Ok(Rebind {
        commit: match current {
            Current::Bound(id) => Some(id),
            Current::Provisional => None,
        },
        provisional,
        worktree,
        value,
        replaces,
    })
}

/// Makes `rebind` what its worktree resolves to.
///
/// # Errors
///
/// Fails when the store does.
pub fn apply(store: &mut Store, rebind: &Rebind) -> Result<(), AdminError> {
    store.set_worktree_identity(&rebind.worktree, &rebind.value)?;

    Ok(())
}

#[cfg(test)]
mod tests;
