use std::fs;
use std::path::{Path, PathBuf};

use super::*;
use crate::home::Home;
use crate::node_identity::IdentitySource;
use key::random::OsEntropy;
use record::event::{EventKind, Provenance};
use testkit::TempDir;

/// A zlib stream of `data` in stored blocks.
fn zlib(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut rest = data;
    loop {
        let (chunk, tail) = rest.split_at(rest.len().min(65_535));
        out.push(u8::from(tail.is_empty()));
        out.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        out.extend_from_slice(&(!(chunk.len() as u16)).to_le_bytes());
        out.extend_from_slice(chunk);
        rest = tail;
        if rest.is_empty() {
            break;
        }
    }
    let (mut a, mut b) = (1_u32, 0_u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

/// The body of a commit of the empty tree with `parents` and `message`.
fn body(parents: &[&CommitId], message: &str) -> String {
    let mut body = format!("tree {}\n", "0".repeat(40));
    for parent in parents {
        body.push_str(&format!("parent {}\n", parent.as_str()));
    }
    body.push_str(&format!("\n{message}\n"));
    body
}

/// The id the commit with `parents` and `message` has, stored or not.
fn commit_id(parents: &[&CommitId], message: &str) -> CommitId {
    ::repository::commit_object_id(body(parents, message).as_bytes())
}

/// The messages and ids of two root commits, the lower id first.
fn two_roots() -> [(&'static str, CommitId); 2] {
    let (x, y) = (("x", commit_id(&[], "x")), ("y", commit_id(&[], "y")));
    if x.1 < y.1 { [x, y] } else { [y, x] }
}

/// An empty repository at `dir` whose `HEAD` names `main`.
fn init(dir: &Path) {
    let git = dir.join(".git");
    fs::create_dir_all(git.join("refs/heads")).unwrap();
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
}

/// Stores the commit with `parents` and `message` in the repository at
/// `dir` under its real id, and returns the id.
fn commit(dir: &Path, parents: &[&CommitId], message: &str) -> CommitId {
    let body = body(parents, message);
    let id = ::repository::commit_object_id(body.as_bytes());
    let object = format!("commit {}\0{body}", body.len());
    let (fan, rest) = id.as_str().split_at(2);
    let path = dir.join(".git/objects").join(fan).join(rest);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, zlib(object.as_bytes())).unwrap();
    id
}

/// Points `main` of the repository at `dir` at `tip`.
fn point(dir: &Path, tip: &CommitId) {
    fs::write(
        dir.join(".git/refs/heads/main"),
        format!("{}\n", tip.as_str()),
    )
    .unwrap();
}

/// A repository at `dir` with one root commit named `message`, checked
/// out; its id.
fn rooted(dir: &Path, message: &str) -> CommitId {
    init(dir);
    let root = commit(dir, &[], message);
    point(dir, &root);
    root
}

/// A repository at `dir` with no commit.
fn empty(dir: &Path) {
    init(dir);
}

/// A repository at `dir` that reads as unreadable: it stores its
/// objects in SHA-256.
fn unreadable_repository(dir: &Path) {
    rooted(dir, "x");
    fs::write(
        dir.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();
}

/// Replaces the repository at `dir` with a fresh one.
fn reinit(dir: &Path) {
    fs::remove_dir_all(dir.join(".git")).unwrap();
}

struct Fixture {
    dir: TempDir,
    home: Home,
    store: Store,
    node: Node,
}

fn fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("machine-id"), "first").unwrap();
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let node = Node::open(&home, &mut store, &mut OsEntropy).unwrap();
    Fixture {
        dir,
        home,
        store,
        node,
    }
}

impl Fixture {
    fn path(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn bind(&mut self, directory: &Path) -> Outcome {
        self.bind_below(directory, None)
    }

    fn bind_below(&mut self, directory: &Path, ceiling: Option<&Path>) -> Outcome {
        bind(
            &mut self.store,
            &self.node,
            &mut OsEntropy,
            directory,
            ceiling,
            &|| false,
        )
        .unwrap()
    }

    /// The repository bind events the device seat holds.
    /// Records `content` as a structural bind event.
    fn append_bind(&mut self, content: serde_json::Value) {
        self.append(EventKind::RepositoryBind, Provenance::Structural, content);
    }

    fn append(&mut self, kind: EventKind, provenance: Provenance, content: serde_json::Value) {
        let event = NewEvent {
            kind,
            provenance,
            content,
            commitment_key: record::commitment::CommitmentKey::from_bytes([3; 32]),
            transcript_timestamp: None,
        };
        self.store
            .append(self.node.device_writer(), &[event])
            .unwrap();
    }

    /// Records the widening act that binds by hand, and applies it.
    fn record_rebind(&mut self, rebind: &Rebind) {
        let mut content = rebind.content();
        content["act"] = serde_json::json!("repository_bind");
        self.append(EventKind::PrincipalAct, Provenance::Operator, content);
        apply(&mut self.store, rebind).unwrap();
    }

    fn binds(&self) -> Vec<serde_json::Value> {
        self.store
            .log(self.node.device_writer())
            .unwrap()
            .into_iter()
            .filter(|s| s.link.header.kind == EventKind::RepositoryBind)
            .map(|s| {
                assert_eq!(s.link.header.provenance, Provenance::Structural);
                s.content.unwrap()
            })
            .collect()
    }
}

#[test]
fn a_directory_in_no_repository_binds_nothing() {
    let mut f = fixture();
    let plain = f.path("plain");

    assert_eq!(f.bind(&plain), Outcome::NoRepository);
    assert!(f.binds().is_empty());
}

#[test]
fn the_first_look_binds_the_lowest_root_commit_once_by_a_structural_event() {
    let mut f = fixture();
    let app = f.path("app");
    init(&app);
    let [(low_message, low), (high_message, high)] = two_roots();
    commit(&app, &[], high_message);
    commit(&app, &[], low_message);
    let merge = commit(&app, &[&high, &low], "merge");
    point(&app, &merge);

    assert_eq!(f.bind(&app), Outcome::Bound { provisional: false });
    assert_eq!(f.bind(&app), Outcome::Unchanged);

    let binds = f.binds();
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0]["repository"], low.as_str());
}

#[test]
fn a_second_clone_of_a_bound_repository_is_recorded_as_another_worktree_of_that_identity() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    let root = rooted(&one, "root");
    rooted(&two, "root");
    let child = commit(&two, &[&root], "child");
    point(&two, &child);

    assert_eq!(f.bind(&one), Outcome::Bound { provisional: false });
    assert_eq!(f.bind(&two), Outcome::Bound { provisional: false });

    let binds = f.binds();
    assert_eq!(binds.len(), 2);
    assert_eq!(binds[0]["repository"], binds[1]["repository"]);
    assert_ne!(binds[0]["worktree_digest"], binds[1]["worktree_digest"]);
}

#[test]
fn a_clone_holding_a_bound_commit_takes_that_identity() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    let [(low_message, low), (high_message, high)] = two_roots();
    // The first clone holds only the higher root, which becomes bound.
    init(&one);
    commit(&one, &[], high_message);
    point(&one, &high);
    // A lower root joined the second clone's history; the bound commit stays.
    init(&two);
    commit(&two, &[], low_message);
    commit(&two, &[], high_message);
    let merge = commit(&two, &[&low, &high], "merge");
    point(&two, &merge);

    f.bind(&one);
    assert_eq!(f.bind(&two), Outcome::Bound { provisional: false });

    let binds = f.binds();
    assert_eq!(binds.len(), 2);
    assert!(binds.iter().all(|b| b["repository"] == high.as_str()));
}

#[test]
fn a_repository_with_no_commit_gets_a_provisional_identity_and_binds_later() {
    let mut f = fixture();
    let blank = f.path("empty");
    empty(&blank);

    assert_eq!(f.bind(&blank), Outcome::Bound { provisional: true });
    assert_eq!(f.bind(&blank), Outcome::Unchanged);
    let binds = f.binds();
    assert_eq!(binds.len(), 1);
    assert_eq!(
        binds[0]["repository_provisional"].as_str().unwrap().len(),
        32
    );

    // The first commit binds it, without a change and without rewriting
    // the provisional bind the record holds.
    let root = commit(&blank, &[], "first");
    point(&blank, &root);
    assert_eq!(f.bind(&blank), Outcome::Bound { provisional: false });
    assert_eq!(f.bind(&blank), Outcome::Unchanged);
    let binds = f.binds();
    assert_eq!(binds.len(), 2);
    assert!(binds[0]["repository_provisional"].is_string());
    // The commit's bind names the provisional identity it replaces, for
    // the same worktree.
    assert_eq!(binds[1]["repository"], root.as_str());
    assert_eq!(
        binds[1]["replaces_provisional"],
        binds[0]["repository_provisional"]
    );
    assert_eq!(binds[1]["worktree_digest"], binds[0]["worktree_digest"]);
}

#[test]
fn a_provisional_directory_whose_clone_gains_a_bound_commit_joins_that_identity_naming_what_it_replaces()
 {
    let mut f = fixture();
    let (one, blank) = (f.path("one"), f.path("blank"));
    let root = rooted(&one, "x");
    f.bind(&one);
    empty(&blank);
    f.bind(&blank);
    // The empty clone fetches the commit this node already bound.
    commit(&blank, &[], "x");
    point(&blank, &root);

    assert_eq!(f.bind(&blank), Outcome::Bound { provisional: false });

    let binds = f.binds();
    assert_eq!(binds.len(), 3);
    assert_eq!(binds[2]["repository"], root.as_str());
    assert_eq!(
        binds[2]["replaces_provisional"],
        binds[1]["repository_provisional"]
    );
}

#[test]
fn a_look_that_has_run_out_of_time_does_not_go_on_to_the_commits_this_node_bound() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    rooted(&one, "x");
    f.bind(&one);
    rooted(&two, "x");

    let outcome = bind(&mut f.store, &f.node, &mut OsEntropy, &two, None, &|| true).unwrap();

    assert_eq!(outcome, Outcome::TimedOut);
}

#[test]
fn a_change_in_the_identity_a_directory_resolves_to_is_reported_once_and_relinks_nothing() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "a");
    f.bind(&app);
    reinit(&app);
    rooted(&app, "b");

    assert_eq!(f.bind(&app), Outcome::Changed);
    assert_eq!(f.bind(&app), Outcome::StillChanged);
    assert_eq!(f.bind(&app), Outcome::StillChanged);
    assert_eq!(f.binds().len(), 1);

    // A second change is reported again, once.
    reinit(&app);
    rooted(&app, "c");
    assert_eq!(f.bind(&app), Outcome::Changed);
    assert_eq!(f.bind(&app), Outcome::StillChanged);
}

#[test]
fn a_clone_whose_git_data_cannot_be_read_is_unreadable() {
    let mut f = fixture();
    let app = f.path("app");
    unreadable_repository(&app);

    assert_eq!(f.bind(&app), Outcome::Unreadable);
    assert_eq!(f.bind(&app), Outcome::StillUnreadable);
    assert!(f.binds().is_empty());
}

#[test]
fn a_clone_with_a_commit_filed_under_the_wrong_id_is_unreadable() {
    let mut f = fixture();
    let app = f.path("app");
    let root = rooted(&app, "a");
    // A planted object: another commit's bytes under the root's id.
    let other = commit(&app, &[], "planted");
    let objects = app.join(".git/objects");
    let (fan, rest) = root.as_str().split_at(2);
    let (ofan, orest) = other.as_str().split_at(2);
    fs::copy(objects.join(ofan).join(orest), objects.join(fan).join(rest)).unwrap();

    assert_eq!(f.bind(&app), Outcome::Unreadable);
    assert!(f.binds().is_empty());
}

#[test]
fn no_name_this_node_keeps_reveals_the_directory_or_the_identity() {
    let mut f = fixture();
    let app = f.path("app");
    let root = rooted(&app, "a");
    f.bind(&app);

    let conn = rusqlite::Connection::open(f.dir.path().join(".parinn/store.db")).unwrap();
    let mut statement = conn
        .prepare("SELECT worktree, identity FROM repository_worktrees")
        .unwrap();
    let rows: Vec<(String, String)> = statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert_eq!(rows.len(), 1);
    for (worktree, identity) in rows {
        assert!(!identity.contains(root.as_str()) && !worktree.contains("app"));
        assert_eq!((worktree.len(), identity.len()), (64, 64));
    }
}

impl Fixture {
    fn show(&mut self, directory: &Path) -> Shown {
        self.show_below(directory, None)
    }

    fn show_below(&mut self, directory: &Path, ceiling: Option<&Path>) -> Shown {
        show(Some((&self.store, &self.node)), directory, ceiling, &|| {
            false
        })
        .unwrap()
    }

    fn plan(&mut self, directory: &Path, commit: Option<&CommitId>) -> Result<Rebind, AdminError> {
        self.plan_below(directory, None, commit)
    }

    fn plan_below(
        &mut self,
        directory: &Path,
        ceiling: Option<&Path>,
        commit: Option<&CommitId>,
    ) -> Result<Rebind, AdminError> {
        plan(
            &mut self.store,
            &self.node,
            &mut OsEntropy,
            directory,
            ceiling,
            commit.cloned(),
            &|| false,
        )
    }
}

#[test]
fn show_says_what_a_directory_is_and_binds_nothing() {
    let mut f = fixture();
    let (plain, app) = (f.path("plain"), f.path("app"));
    let a = rooted(&app, "a");

    assert_eq!(f.show(&plain), Shown::NoRepository);
    assert_eq!(f.show(&app), Shown::Unbound(Some(a.clone())));
    assert!(f.binds().is_empty());
    f.bind(&app);
    assert_eq!(f.show(&app), Shown::Bound(Some(a)));
    reinit(&app);
    let b = rooted(&app, "b");
    assert_eq!(f.show(&app), Shown::Changed(Some(b)));
}

#[test]
fn show_names_a_provisional_directory_without_a_commit_and_an_unreadable_one() {
    let mut f = fixture();
    let (blank, bad) = (f.path("empty"), f.path("bad"));
    empty(&blank);
    unreadable_repository(&bad);

    assert_eq!(f.show(&blank), Shown::Unbound(None));
    assert_eq!(f.show(&bad), Shown::Unreadable);
}

#[test]
fn a_widening_bind_makes_the_directory_resolve_to_the_identity_it_names() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "a");
    f.bind(&app);
    reinit(&app);
    let b = rooted(&app, "b");
    assert_eq!(f.bind(&app), Outcome::Changed);

    let rebind = f.plan(&app, None).unwrap();
    assert_eq!(rebind.commit, Some(b));
    // Nothing moves until the act is recorded and the plan applied.
    assert_eq!(f.bind(&app), Outcome::StillChanged);
    apply(&mut f.store, &rebind).unwrap();

    assert_eq!(f.bind(&app), Outcome::Unchanged);
}

#[test]
fn a_widening_bind_may_name_a_commit_the_clone_lacks() {
    let mut f = fixture();
    let shallow = f.path("shallow");
    rooted(&shallow, "b");
    let absent = commit_id(&[], "absent");

    let rebind = f.plan(&shallow, Some(&absent)).unwrap();

    assert_eq!(rebind.commit, Some(absent));
}

#[test]
fn a_widening_bind_of_a_clone_with_no_commit_is_provisional() {
    let mut f = fixture();
    let blank = f.path("empty");
    empty(&blank);

    assert_eq!(f.plan(&blank, None).unwrap().commit, None);
}

#[test]
fn a_widening_bind_of_a_directory_in_no_repository_is_refused() {
    let mut f = fixture();
    let plain = f.path("plain");

    assert!(f.plan(&plain, None).is_err());
    assert!(f.plan(&plain, Some(&commit_id(&[], "x"))).is_err());
}

#[test]
fn a_repository_in_the_home_directory_is_bound_for_no_directory_below_it() {
    let mut f = fixture();
    let home = f.path("home");
    rooted(&home, "dotfiles");
    let notes = home.join("notes/today");
    fs::create_dir_all(&notes).unwrap();

    assert_eq!(f.bind_below(&notes, Some(&home)), Outcome::NoRepository);
    assert_eq!(f.show_below(&notes, Some(&home)), Shown::NoRepository);
    assert!(f.plan_below(&notes, Some(&home), None).is_err());
    assert!(f.binds().is_empty());
    // A project inside the home directory is still its own repository.
    let project = home.join("project");
    rooted(&project, "project");
    assert_eq!(
        f.bind_below(&project, Some(&home)),
        Outcome::Bound { provisional: false }
    );
}

#[test]
fn a_git_directory_that_is_a_symlink_is_unreadable_to_bind_and_show() {
    let mut f = fixture();
    let app = f.path("app");
    let elsewhere = f.path("elsewhere");
    rooted(&elsewhere, "x");
    std::os::unix::fs::symlink(elsewhere.join(".git"), app.join(".git")).unwrap();

    assert_eq!(f.bind(&app), Outcome::Unreadable);
    assert_eq!(f.show(&app), Shown::Unreadable);
}

#[test]
fn a_rebind_records_the_commit_or_the_provisional_id_it_names() {
    let rebind = |commit: Option<&str>, provisional: Option<&str>| Rebind {
        commit: commit.and_then(CommitId::parse),
        provisional: provisional.map(str::to_owned),
        worktree: "w".to_owned(),
        value: String::new(),
        replaces: None,
    };

    let a = commit_id(&[], "a");
    assert_eq!(
        rebind(Some(a.as_str()), None).content(),
        serde_json::json!({"repository": a.as_str(), "worktree_digest": "w"})
    );
    assert_eq!(
        rebind(None, Some("ab")).content(),
        serde_json::json!({"repository_provisional": "ab", "worktree_digest": "w"})
    );
    assert_eq!(
        rebind(None, None).content(),
        serde_json::json!({"worktree_digest": "w"})
    );
    let replacing = Rebind {
        replaces: Some("ab".to_owned()),
        ..rebind(Some(a.as_str()), None)
    };
    assert_eq!(
        replacing.content(),
        serde_json::json!({
            "repository": a.as_str(),
            "worktree_digest": "w",
            "replaces_provisional": "ab",
        })
    );
}

#[test]
fn a_look_that_runs_out_of_time_is_told_from_a_clone_that_cannot_be_read_and_is_tried_again() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "x");

    let expired = bind(&mut f.store, &f.node, &mut OsEntropy, &app, None, &|| true).unwrap();

    assert_eq!(expired, Outcome::TimedOut);
    assert!(f.binds().is_empty());
    // Nothing was kept, so the next look, with time, binds it.
    assert_eq!(f.bind(&app), Outcome::Bound { provisional: false });
}

#[test]
fn show_names_a_provisional_directory_that_gained_a_commit_as_waiting_to_bind() {
    let mut f = fixture();
    let blank = f.path("blank");
    empty(&blank);
    f.bind(&blank);
    let root = commit(&blank, &[], "first");
    point(&blank, &root);

    assert_eq!(f.show(&blank), Shown::Unbound(Some(root)));
}

#[test]
fn a_bound_commit_the_default_branch_does_not_reach_names_no_clone() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    rooted(&one, "x");
    f.bind(&one);
    // The clone holds the bound repository's root as a dangling object,
    // as a side fetch leaves one, and its own branch does not reach it.
    rooted(&two, "y");
    commit(&two, &[], "x");

    assert_eq!(f.bind(&two), Outcome::Bound { provisional: false });

    assert_eq!(f.binds().len(), 2);
}

#[test]
fn only_a_repository_bind_act_binds_a_commit_not_another_act_naming_one() {
    let mut f = fixture();
    let root = commit_id(&[], "x");
    let append =
        |f: &mut Fixture, kind: EventKind, provenance: Provenance, content: serde_json::Value| {
            let event = store::NewEvent {
                kind,
                provenance,
                content,
                commitment_key: record::commitment::CommitmentKey::from_bytes([3; 32]),
                transcript_timestamp: None,
            };
            f.store.append(f.node.device_writer(), &[event]).unwrap();
        };
    append(
        &mut f,
        EventKind::PrincipalAct,
        Provenance::Operator,
        serde_json::json!({"act": "counter_ack", "repository": root.as_str()}),
    );
    assert!(bound_commits(&f.store, &f.node).unwrap().is_empty());

    append(
        &mut f,
        EventKind::PrincipalAct,
        Provenance::Operator,
        serde_json::json!({"act": "repository_bind", "repository": root.as_str()}),
    );
    assert_eq!(bound_commits(&f.store, &f.node).unwrap(), vec![root]);
}

#[test]
fn an_unreadable_look_does_not_make_the_next_change_report_a_change_twice() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "a");
    f.bind(&app);
    reinit(&app);
    rooted(&app, "b");
    assert_eq!(f.bind(&app), Outcome::Changed);
    // The clone cannot be read for a while, then reads again, unchanged.
    std::fs::write(
        app.join(".git/config"),
        "[extensions]\n\tobjectformat = sha256\n",
    )
    .unwrap();
    assert_eq!(f.bind(&app), Outcome::Unreadable);
    std::fs::write(app.join(".git/config"), "").unwrap();

    assert_eq!(f.bind(&app), Outcome::StillChanged);
}

#[test]
fn an_unreadable_clone_and_a_change_report_again_after_the_directory_reads_or_returns() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "a");
    f.bind(&app);
    // Unreadable, readable, unreadable again: both are reported.
    let broken = "[extensions]\n\tobjectformat = sha256\n";
    std::fs::write(app.join(".git/config"), broken).unwrap();
    assert_eq!(f.bind(&app), Outcome::Unreadable);
    std::fs::write(app.join(".git/config"), "").unwrap();
    assert_eq!(f.bind(&app), Outcome::Unchanged);
    std::fs::write(app.join(".git/config"), broken).unwrap();
    assert_eq!(f.bind(&app), Outcome::Unreadable);
    std::fs::write(app.join(".git/config"), "").unwrap();
    // Changed to b, back to a, changed to b again: reported each time.
    reinit(&app);
    rooted(&app, "b");
    assert_eq!(f.bind(&app), Outcome::Changed);
    reinit(&app);
    rooted(&app, "a");
    assert_eq!(f.bind(&app), Outcome::Unchanged);
    reinit(&app);
    rooted(&app, "b");

    assert_eq!(f.bind(&app), Outcome::Changed);
}

#[test]
fn a_bound_worktree_whose_clone_lost_every_commit_is_a_change_reported_once() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "a");
    f.bind(&app);
    reinit(&app);
    empty(&app);

    assert_eq!(f.bind(&app), Outcome::Changed);
    assert_eq!(f.bind(&app), Outcome::StillChanged);
}

#[test]
fn the_table_of_worktrees_derives_from_the_record_alone() {
    let mut f = fixture();
    let (one, blank, gone) = (f.path("one"), f.path("blank"), f.path("gone"));
    rooted(&one, "x");
    empty(&blank);
    f.bind(&one);
    f.bind(&blank);
    // A provisional worktree that later gains a commit, and one a
    // widening bind rebinds.
    let root = commit(&blank, &[], "later");
    point(&blank, &root);
    f.bind(&blank);
    rooted(&gone, "a");
    f.bind(&gone);
    reinit(&gone);
    rooted(&gone, "b");
    let rebind = f.plan(&gone, None).unwrap();
    f.record_rebind(&rebind);

    let derived = derive_worktrees(&f.store, &f.node).unwrap();

    assert_eq!(derived, f.store.worktree_identities().unwrap());
    assert_eq!(derived.len(), 3);
}

#[test]
fn a_bind_naming_no_identity_or_no_worktree_derives_nothing() {
    let mut f = fixture();
    for content in [
        serde_json::json!({"worktree_digest": "w"}),
        serde_json::json!({"repository": "a".repeat(40)}),
        serde_json::json!({"repository": "not a commit", "worktree_digest": "w"}),
    ] {
        f.append_bind(content);
    }

    assert!(derive_worktrees(&f.store, &f.node).unwrap().is_empty());
}

#[test]
fn show_names_a_provisional_directory_that_is_still_without_a_commit_as_bound() {
    let mut f = fixture();
    let blank = f.path("blank");
    empty(&blank);
    f.bind(&blank);

    assert_eq!(f.show(&blank), Shown::Bound(None));
}

#[test]
fn a_name_for_a_worktree_or_directory_that_is_gone_keeps_the_io_cause() {
    let gone = Path::new("/no/such/worktree");

    for (error, what) in [
        (worktree_digest(&[0; 32], gone).unwrap_err(), "worktree"),
        (directory_marker(&[0; 32], gone).unwrap_err(), "directory"),
    ] {
        assert!(matches!(&error, AdminError::Unresolved { what: w, .. } if *w == what));
        assert!(std::error::Error::source(&error).is_some());
    }
}

#[test]
fn a_worktree_bound_again_after_a_node_identity_change_is_told_from_a_first_bind() {
    let mut f = fixture();
    let app = f.path("app");
    rooted(&app, "a");
    assert_eq!(f.bind(&app), Outcome::Bound { provisional: false });

    // The node moves to another machine: a new device key, a new seat that
    // begins with the identity change. Its names for worktrees are new.
    fs::write(f.dir.path().join("machine-id"), "second").unwrap();
    f.node = Node::open(&f.home, &mut f.store, &mut OsEntropy).unwrap();

    assert_eq!(f.bind(&app), Outcome::Rebound);
    assert_eq!(f.bind(&app), Outcome::Unchanged);
    assert_eq!(f.binds().len(), 1, "the new seat holds its own bind");
}

#[test]
fn a_bind_another_process_made_first_leaves_the_worktree_as_that_one_did() {
    assert_eq!(landed_as(false, true, true), Outcome::Unchanged);
    assert_eq!(landed_as(true, true, false), Outcome::Rebound);
    assert_eq!(
        landed_as(true, false, true),
        Outcome::Bound { provisional: true }
    );
}

#[test]
fn a_worktree_is_bound_again_after_the_device_key_rotated_or_was_minted_anew() {
    for kind in ["rotated", "minted_anew"] {
        let mut f = fixture();
        let (one, two) = (f.path("one"), f.path("two"));
        rooted(&one, "x");
        rooted(&two, "x");
        assert_eq!(f.bind(&one), Outcome::Bound { provisional: false });
        f.append(
            EventKind::KeyEvent,
            Provenance::Structural,
            serde_json::json!({"kind": kind}),
        );

        assert_eq!(f.bind(&two), Outcome::Rebound, "{kind}");
    }
}

#[test]
fn other_key_events_leave_the_names_of_worktrees_as_they_were() {
    let mut f = fixture();
    let (one, two) = (f.path("one"), f.path("two"));
    rooted(&one, "x");
    rooted(&two, "x");
    f.append(
        EventKind::KeyEvent,
        Provenance::Structural,
        serde_json::json!({"kind": "recovered"}),
    );
    f.append(
        EventKind::KeyEvent,
        Provenance::Structural,
        serde_json::json!({"no_kind": 1}),
    );

    assert_eq!(f.bind(&one), Outcome::Bound { provisional: false });
    assert_eq!(f.bind(&two), Outcome::Bound { provisional: false });
}
