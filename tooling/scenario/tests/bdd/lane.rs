//! The steps of SRS §5.2, rooms and lanes (LANE), that phase 11 of plan
//! 2609292012 carries. `@LANE-02`: a repository's identity is a
//! parentless commit of its default branch, bound once by a structural
//! event, independent of the local path; a change in what a directory
//! resolves to is audited and rebinds only on the widening principal act
//! `parinn repository bind`.
//!
//! The bind is recorded when the hook handler's `SessionStart`
//! observation reaches the record, as `parinn ingest` or an MCP server
//! starts: reading a clone's git data cannot stay within the handler's
//! hook budget (NFR-02), and the observation is the first hook event.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use cucumber::{given, then, when};
use serde_json::{Value, json};

use crate::World;
use crate::live::Harness;
use crate::record::{home, last_run, run_core, run_core_at_terminal, set_env};

/// What the repository steps track: the clone, and the identity git
/// names for it.
#[derive(Debug, Default)]
pub struct LaneSteps {
    clone: Option<PathBuf>,
    root: Option<String>,
}

fn lane(w: &mut World) -> &mut LaneSteps {
    w.section::<LaneSteps>()
}

/// Runs `git` in `dir` as a person with no configuration of their own.
fn git(w: &World, dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = testkit::isolate(&mut Command::new("git"), &w.home)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=a",
            "-c",
            "user.email=a@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Makes a repository at `dir`, with a root commit when `commits`.
fn make(w: &World, dir: &Path, commits: bool) -> Result<Option<String>, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    git(w, dir, &["init", "-q"])?;
    if !commits {
        return Ok(None);
    }
    fs::write(dir.join("a.txt"), "a").map_err(|e| e.to_string())?;
    git(w, dir, &["add", "."])?;
    git(w, dir, &["commit", "-q", "-m", "root"])?;
    git(w, dir, &["rev-list", "--max-parents=0", "HEAD"]).map(Some)
}

#[given(
    regex = r"^a repository clone that (contains the bound parentless commit of the default branch|has no commit)$"
)]
fn clone_that(w: &mut World, how: String) -> Result<(), String> {
    let dir = w.home.join("work").join("app");
    let root = make(w, &dir, how.starts_with("contains"))?;
    // The node reads its identity from a file of the scenario's, so a step
    // can move it to another machine.
    let identity = w.home.join("machine-id");
    fs::write(&identity, "first\n").map_err(|e| e.to_string())?;
    set_env(
        w,
        "PARINN_TEST_NODE_IDENTITY",
        &identity.display().to_string(),
    );
    let steps = lane(w);
    steps.clone = Some(dir);
    steps.root = root;
    Ok(())
}

/// How many session-start looks the node has marked done.
fn looks_done(w: &World) -> Result<i64, String> {
    let store = home(w)?.store();
    if !store.exists() {
        return Ok(0);
    }
    let conn = rusqlite::Connection::open(store).map_err(|e| e.to_string())?;
    // The marks' table is made by the first look.
    Ok(conn
        .query_row(
            "SELECT count(*) FROM repository_worktree_marks WHERE mark LIKE 'looked:%'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0))
}

/// The first hook event for `dir` under a stand-in harness: its
/// `SessionStart` hook runs, its MCP server takes the session binding
/// and, once it holds it, looks at the clone (SEC-18).
fn first_hook_event(w: &mut World, dir: &Path, session: &str) -> Result<(), String> {
    let path = w.home.join(format!("{session}.jsonl"));
    let input = json!({
        "session_id": session,
        "transcript_path": path,
        "cwd": dir,
        "hook_event_name": "SessionStart",
        "source": "startup",
    });
    let before = looks_done(w)?;
    let mut harness = Harness::start(w, false, &[])?;
    harness.hook("SessionStart", &input)?;
    harness.start_mcp()?;
    // The server holds the binding once it has taken it, and says so.
    let params = json!({"name": "mcp_server_get", "arguments": {}});
    let mut held = false;
    let mut looked = false;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && !(held && looked) {
        let reply = harness.request("tools/call", params.clone())?;
        let state: Value = reply["result"]["content"][0]["text"]
            .as_str()
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or(Value::Null);
        held = state["session_binding"] == "held";
        looked = held && looks_done(w)? > before;
        if !(held && looked) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    harness.stop()?;
    if held && looked {
        return Ok(());
    }
    Err("the MCP server did not take the session binding and look at the clone".to_owned())
}

fn clone_dir(w: &mut World) -> Result<PathBuf, String> {
    lane(w)
        .clone
        .clone()
        .ok_or_else(|| "no step made a clone".to_owned())
}

#[when(regex = r"^the first hook event for the clone arrives$")]
fn first_event(w: &mut World) -> Result<(), String> {
    let dir = clone_dir(w)?;
    first_hook_event(w, &dir, "lane-02-first")
}

/// What `parinn repository show` says of `dir`.
fn shown(w: &mut World, dir: &Path) -> Result<Value, String> {
    run_core(
        w,
        &["repository", "show", &dir.display().to_string(), "--json"],
        None,
    )?;
    let out = last_run(w)?;
    serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "parinn repository show: {e}: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[then(
    regex = r"^the repository identity is (the bound identity|a provisional, node-local identity)$"
)]
fn identity_is(w: &mut World, which: String) -> Result<(), String> {
    let dir = clone_dir(w)?;
    let v = shown(w, &dir)?;
    let root = lane(w).root.clone();
    let wanted = if which == "the bound identity" {
        json!(root)
    } else {
        Value::Null
    };
    if v["status"] != "bound" || v["repository"] != wanted {
        return Err(format!("the directory shows {v}, wanting {which}"));
    }
    if root.is_none() {
        let id = provisional_id(w)?;
        if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("the provisional id is not 32 hex digits: {id}"));
        }
    }
    Ok(())
}

/// The provisional id the record's one bind carries.
fn provisional_id(w: &World) -> Result<String, String> {
    let binds = events(w, "repository_bind")?;
    match binds.as_slice() {
        [(_, c)] => c["repository_provisional"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("the bind holds no provisional id: {c}")),
        _ => Err(format!("wanting one bind, the record holds {binds:?}")),
    }
}

/// The events of `kind` the record holds, with their provenance and
/// content.
fn events(w: &World, kind: &str) -> Result<Vec<(String, Value)>, String> {
    let conn = rusqlite::Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT provenance, content FROM events WHERE kind = ?1 ORDER BY writer, seq")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([kind], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    rows.map(|row| {
        let (provenance, content) = row.map_err(|e| e.to_string())?;
        Ok((
            provenance,
            serde_json::from_str(&content).map_err(|e| e.to_string())?,
        ))
    })
    .collect()
}

#[then(
    regex = r#"^a repository's first bind is a structural event ParleyInn records when the first hook's observation reaches the record, never an "operator" event, and it names the worktree by a keyed digest of its root path$"#
)]
fn first_bind_is_structural(w: &mut World) -> Result<(), String> {
    let binds = events(w, "repository_bind")?;
    if binds.len() != 1 || binds[0].0 != "structural" {
        return Err(format!(
            "the record holds {binds:?}, wanting one structural bind"
        ));
    }
    let digest = binds[0].1["worktree_digest"].as_str().unwrap_or_default();
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("the bind names no keyed digest: {:?}", binds[0].1));
    }
    let acts = events(w, "principal_act")?;
    match acts.iter().find(|(_, c)| c["act"] == "repository_bind") {
        Some(act) => Err(format!("an operator act bound it: {act:?}")),
        None => Ok(()),
    }
}

#[then(
    regex = r"^no identity is minted from the local path, and a further worktree of the identity is recorded by a bind event of its own$"
)]
fn no_identity_from_the_path(w: &mut World) -> Result<(), String> {
    // A second clone at another path names the same identity and binds
    // nothing more; a provisional one's id holds no path.
    let first = clone_dir(w)?;
    let root = lane(w).root.clone();
    if root.is_some() {
        let second = w.home.join("elsewhere").join("copy");
        fs::create_dir_all(second.parent().ok_or("no parent")?).map_err(|e| e.to_string())?;
        git(
            w,
            &w.home.clone(),
            &[
                "clone",
                "-q",
                &first.display().to_string(),
                &second.display().to_string(),
            ],
        )?;
        first_hook_event(w, &second, "lane-02-second")?;
        let v = shown(w, &second)?;
        // The copy is another worktree of the same identity: one more
        // bind, naming that identity and a worktree of its own.
        let binds = events(w, "repository_bind")?;
        let digests: Vec<_> = binds
            .iter()
            .map(|(_, c)| c["worktree_digest"].clone())
            .collect();
        let same = binds
            .iter()
            .all(|(_, c)| json!(c["repository"]) == json!(root));
        if v["repository"] != json!(root) || binds.len() != 2 || !same || digests[0] == digests[1] {
            return Err(format!("the copy at another path shows {v}: {binds:?}"));
        }
    } else {
        let first_id = provisional_id(w)?;
        let second = w.home.join("elsewhere").join("empty");
        fs::create_dir_all(&second).map_err(|e| e.to_string())?;
        git(w, &second, &["init", "-q"])?;
        first_hook_event(w, &second, "lane-02-empty")?;
        let ids: Vec<String> = events(w, "repository_bind")?
            .iter()
            .filter_map(|(_, c)| c["repository_provisional"].as_str().map(str::to_owned))
            .collect();
        if ids.len() != 2 || ids[0] == ids[1] || !ids.contains(&first_id) {
            return Err(format!("two empty clones share a provisional id: {ids:?}"));
        }
    }
    for (_, content) in events(w, "repository_bind")? {
        if content.to_string().contains(&first.display().to_string()) {
            return Err(format!("a bind holds the path: {content}"));
        }
    }
    Ok(())
}

/// Every file under `dir`, with its bytes.
fn files(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            files(&path, out)?;
        } else {
            out.push((path.clone(), fs::read(&path).map_err(|e| e.to_string())?));
        }
    }
    Ok(())
}

#[then(
    regex = r"^no name ParleyInn derives from the identity for its local state reveals anything about the identity off this node$"
)]
fn no_local_name_reveals_it(w: &mut World) -> Result<(), String> {
    let dir = clone_dir(w)?;
    let secrets: Vec<String> = [lane(w).root.clone(), Some(dir.display().to_string())]
        .into_iter()
        .flatten()
        .collect();
    let conn = rusqlite::Connection::open(home(w)?.store()).map_err(|e| e.to_string())?;
    let mut statement = conn
        .prepare("SELECT worktree, identity FROM repository_worktrees")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("the node keeps no name for the directory".to_owned());
    }
    for (directory, identity) in &rows {
        if secrets
            .iter()
            .any(|s| directory.contains(s) || identity.contains(s))
        {
            return Err(format!("a local name reveals it: {directory} {identity}"));
        }
    }
    // Nor does any file of the home but the record (the audit log, the
    // settings, the hook state).
    let mut all = Vec::new();
    files(home(w)?.root(), &mut all)?;
    let root = lane(w).root.clone().unwrap_or_default();
    for (path, bytes) in all {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.starts_with("store.db") || path.to_string_lossy().contains("/payloads/") {
            continue;
        }
        if !root.is_empty() && bytes.windows(root.len()).any(|b| b == root.as_bytes()) {
            return Err(format!("{} holds the identity", path.display()));
        }
    }
    Ok(())
}

/// How many times the change of a repository identity is counted.
fn changed(w: &mut World) -> Result<u64, String> {
    run_core(w, &["status", "--json"], None)?;
    let status: Value = serde_json::from_slice(&last_run(w)?.stdout).map_err(|e| e.to_string())?;
    Ok(status["counters"]["repository_identity_changed"]["total"]
        .as_u64()
        .unwrap_or(0))
}

/// Replaces the repository in `dir` with a new one whose root commit
/// is `message`.
fn swap_repository(w: &World, dir: &Path, message: &str) -> Result<(), String> {
    fs::remove_dir_all(dir.join(".git")).map_err(|e| e.to_string())?;
    fs::write(dir.join("b.txt"), message).map_err(|e| e.to_string())?;
    git(w, dir, &["init", "-q"])?;
    git(w, dir, &["add", "."])?;
    git(w, dir, &["commit", "-q", "-m", message])?;
    Ok(())
}

#[then(
    regex = r#"^a later change in the identity the directory resolves to is audited, rewrites no room's branch links silently, rebinds only on the widening principal act "parinn repository bind", the same act that binds by hand, and scopes no ParleyInn state$"#
)]
fn a_later_change(w: &mut World) -> Result<(), String> {
    let dir = clone_dir(w)?;
    if lane(w).root.is_none() {
        // A provisional identity binds on its own once its clone names a
        // commit: no change, and the record is not rewritten.
        swap_repository(w, &dir, "first root")?;
        first_hook_event(w, &dir, "lane-02-binds-later")?;
        if shown(w, &dir)?["status"] != "bound" || changed(w)? != 0 {
            return Err("a provisional identity did not bind on its own".to_owned());
        }
        // The commit's bind names the provisional identity it replaces,
        // for the same worktree.
        let binds = events(w, "repository_bind")?;
        let linked = binds.iter().any(|(_, later)| {
            later["repository"].is_string()
                && binds.iter().any(|(_, first)| {
                    first["worktree_digest"] == later["worktree_digest"]
                        && first["repository_provisional"].is_string()
                        && later["replaces_provisional"] == first["repository_provisional"]
                })
        });
        if !linked {
            return Err(format!(
                "the later bind does not name what it replaces: {binds:?}"
            ));
        }
    }
    let before = events(w, "repository_bind")?;
    // The directory now holds another repository.
    swap_repository(w, &dir, "another root")?;
    first_hook_event(w, &dir, "lane-02-later")?;
    first_hook_event(w, &dir, "lane-02-later-again")?;
    if changed(w)? != 1 {
        return Err("the change is not counted once".to_owned());
    }
    let audit = audit::log::Log::new(home(w)?.audit())
        .entries()
        .map_err(|e| e.to_string())?;
    if !audit
        .iter()
        .any(|e| e.detail.contains("repository identity changed"))
    {
        return Err("no audit entry records the change".to_owned());
    }
    // M1 holds no room, so no branch link exists to be rewritten: the
    // clauses on branch links and on state hold vacuously.
    if events(w, "repository_bind")? != before || shown(w, &dir)?["status"] != "changed" {
        return Err("the change rebound the directory or relinked it".to_owned());
    }
    run_core_at_terminal(w, &["repository", "bind", &dir.display().to_string()])?;
    if !last_run(w)?.status.success() || shown(w, &dir)?["status"] != "bound" {
        return Err("the widening bind did not rebind the directory".to_owned());
    }
    // Only that act rebinds: away from a terminal the same bind is refused.
    let before_acts = events(w, "principal_act")?.len();
    run_core(w, &["repository", "bind", &dir.display().to_string()], None)?;
    if last_run(w)?.status.success() || events(w, "principal_act")?.len() != before_acts {
        return Err("a bind away from a terminal was not refused".to_owned());
    }
    // The same act binds by hand a directory no hook event has shown.
    let by_hand = w.home.join("work").join("by-hand");
    make(w, &by_hand, true)?;
    if shown(w, &by_hand)?["status"] != "unbound" {
        return Err("a directory no hook showed is bound".to_owned());
    }
    run_core_at_terminal(w, &["repository", "bind", &by_hand.display().to_string()])?;
    if !last_run(w)?.status.success() || shown(w, &by_hand)?["status"] != "bound" {
        return Err("the widening bind did not bind a directory by hand".to_owned());
    }
    let acts = events(w, "principal_act")?;
    if !acts
        .iter()
        .any(|(p, c)| p == "operator" && c["act"] == "repository_bind")
    {
        return Err("the rebind is no operator act".to_owned());
    }
    Ok(())
}

#[then(
    regex = r"^a commit bound later to a worktree that held a provisional identity names the provisional identity it replaces$"
)]
fn replaced_provisional_is_named(w: &mut World) -> Result<(), String> {
    let binds = events(w, "repository_bind")?;
    for (_, later) in binds
        .iter()
        .filter(|(_, c)| c["replaces_provisional"].is_string())
    {
        let named = binds.iter().any(|(_, first)| {
            first["repository_provisional"] == later["replaces_provisional"]
                && first["worktree_digest"] == later["worktree_digest"]
        });
        if !named {
            return Err(format!(
                "a bind names a provisional identity not in the record: {later}"
            ));
        }
    }

    Ok(())
}

// "after a rotated device key or one minted anew each worktree is bound
// again in the same way" stays unbound: no command rotates a device key
// before PRV-12 ships; the unit tests of `administration::repository` cover
// both key events.

#[then(
    regex = r#"^after a node identity change each worktree is bound again, keeping the identity the clone resolves to, audited and counted as "repository_identity_rebound", never by a widening principal act$"#
)]
fn rebound_after_a_node_identity_change(w: &mut World) -> Result<(), String> {
    // The node moves to another machine; its next look binds the worktree
    // again under its new device key.
    let dir = clone_dir(w)?;
    let acts_before = events(w, "principal_act")?.len();
    let binds_before = events(w, "repository_bind")?;
    fs::write(w.home.join("machine-id"), "another-machine\n").map_err(|e| e.to_string())?;
    first_hook_event(w, &dir, "lane-02-moved")?;
    run_core(w, &["status", "--json"], None)?;
    let out = last_run(w)?;
    let status: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    if status["counters"]["repository_identity_rebound"]["total"] != 1 {
        return Err(format!("the rebind was not counted once: {status}"));
    }
    if events(w, "principal_act")?.len() != acts_before {
        return Err("a principal act recorded the new bind".to_owned());
    }
    // The new bind names the identity the clone resolves to now.
    let resolves = git(w, &dir, &["rev-list", "--max-parents=0", "HEAD"]).ok();
    let new: Vec<_> = events(w, "repository_bind")?
        .into_iter()
        .filter(|bind| !binds_before.contains(bind))
        .collect();
    let [(_, newest)] = new.as_slice() else {
        return Err(format!("wanting one new bind, the record gained {new:?}"));
    };
    let kept = match &resolves {
        Some(root) => newest["repository"] == json!(root),
        None => newest["repository_provisional"].is_string(),
    };
    if !kept {
        return Err(format!("the new bind does not keep the identity: {newest}"));
    }

    Ok(())
}
