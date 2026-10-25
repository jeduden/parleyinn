//! The crate against repositories the real `git` builds (LANE-02): loose
//! objects, packs with offset and reference deltas, packed references,
//! two roots, shallow and shared clones, linked worktrees, a separate
//! git directory, and corrupted copies. The answer is compared with
//! `git rev-list --max-parents=0`, the lowest id of several.
//!
//! The tests run `git` to build their repositories; the crate under
//! test starts no program. A machine without `git` skips them, saying
//! so. Every start is isolated to a temporary home (ENG-14).

#![expect(
    clippy::disallowed_types,
    reason = "a test builds its repositories with the git binary; the crate under test starts no program"
)]
#![allow(
    clippy::unwrap_used,
    reason = "a test's helpers unwrap: a failure is the assertion"
)]
#![allow(
    clippy::panic,
    reason = "a git start that fails fails the test with git's message"
)]

use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use repository::{CommitId, Error, RepositoryIdentity, git_dir, has_commit, identity, root_commit};
use testkit::TempDir;

/// How long one `git` start may take.
const START: Duration = Duration::from_secs(60);

/// A temporary home, and the `git` that runs in it.
struct World {
    home: TempDir,
    clock: Cell<u64>,
}

impl World {
    /// A world, or `None` (saying why) on a machine without `git`.
    fn new() -> Option<Self> {
        let world = Self {
            home: TempDir::new().unwrap(),
            clock: Cell::new(1_700_000_000),
        };
        fs::create_dir_all(world.home.path().join(".out")).unwrap();
        match world.run(world.home.path(), &["--version"]) {
            Ok(_) => Some(world),
            Err(e) => {
                eprintln!("skipped: no usable git binary ({e})");
                None
            }
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.home.path().join(name)
    }

    /// Runs `git args` in `dir`, isolated, and returns its output.
    fn run(&self, dir: &Path, args: &[&str]) -> Result<String, String> {
        let tick = self.clock.get();
        self.clock.set(tick + 1);
        let out = self.home.path().join(".out").join(format!("{tick}.out"));
        let err = self.home.path().join(".out").join(format!("{tick}.err"));
        let date = format!("{tick} +0000");
        let mut command = Command::new("git");
        testkit::isolate(&mut command, self.home.path());
        command
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "A")
            .env("GIT_AUTHOR_EMAIL", "a@example.com")
            .env("GIT_COMMITTER_NAME", "A")
            .env("GIT_COMMITTER_EMAIL", "a@example.com")
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date)
            .args(["-c", "gc.auto=0", "-c", "maintenance.auto=false"])
            .args(["-c", "protocol.file.allow=always"])
            .args(args)
            .stdout(File::create(&out).unwrap())
            .stderr(File::create(&err).unwrap());
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        let mut waited = Duration::ZERO;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if waited >= START {
                let _ = child.kill();
                return Err(format!("git {args:?} did not finish"));
            }
            std::thread::sleep(Duration::from_millis(5));
            waited += Duration::from_millis(5);
        };
        if status.success() {
            Ok(fs::read_to_string(out).unwrap())
        } else {
            let message = fs::read_to_string(err).unwrap_or_default();
            Err(format!("git {args:?} failed: {message}"))
        }
    }

    fn git(&self, dir: &Path, args: &[&str]) -> String {
        self.run(dir, args).unwrap_or_else(|e| panic!("{e}"))
    }

    /// A new repository whose first branch is `main`.
    fn init(&self, name: &str) -> PathBuf {
        let dir = self.path(name);
        fs::create_dir_all(&dir).unwrap();
        self.git(&dir, &["init", "-q", "-b", "main"]);
        dir
    }

    /// A commit changing `file` to `text`.
    fn commit(&self, repo: &Path, file: &str, text: &str, message: &str) {
        fs::write(repo.join(file), text).unwrap();
        self.git(repo, &["add", "-A"]);
        self.git(repo, &["commit", "-q", "-m", message]);
    }

    /// A history of `count` commits whose files and messages resemble
    /// each other, so that a repack stores them as deltas.
    fn history(&self, repo: &Path, count: usize) {
        let body: String = (0..400)
            .map(|n| format!("line {n} of the file\n"))
            .collect();
        let note: String = (0..60)
            .map(|n| format!("note {n} about the change\n"))
            .collect();
        for n in 0..count {
            let text = format!("{body}changed in step {n}\n");
            self.commit(repo, "data.txt", &text, &format!("step {n}\n\n{note}"));
        }
    }

    /// The lowest id among the parentless commits behind `rev`.
    fn expected_root(&self, repo: &Path, rev: &str) -> Option<CommitId> {
        let out = self.run(repo, &["rev-list", "--max-parents=0", rev]).ok()?;
        out.lines().min().and_then(CommitId::parse)
    }

    /// The root git itself would name for the default branch.
    fn expected_default_root(&self, repo: &Path) -> Option<CommitId> {
        let remote = self.run(
            repo,
            &["rev-parse", "--verify", "-q", "refs/remotes/origin/HEAD"],
        );
        let rev = if remote.is_ok() {
            "refs/remotes/origin/HEAD"
        } else {
            "HEAD"
        };
        self.expected_root(repo, rev)
    }

    fn all_commits(&self, repo: &Path) -> Vec<CommitId> {
        let out = self.git(repo, &["rev-list", "--all"]);
        out.lines().filter_map(CommitId::parse).collect()
    }

    /// Asserts the crate names the root git does, for the git directory
    /// of `repo`, and returns it.
    fn check(&self, repo: &Path) -> Option<CommitId> {
        let found = root_commit(&repo.join(".git"), &|| false).unwrap();
        assert_eq!(found, self.expected_default_root(repo));
        found
    }
}

/// Every file under `dir` with its bytes and modification time.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Option<(Vec<u8>, SystemTime)>> {
    let mut seen = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(path) = pending.pop() {
        let meta = fs::symlink_metadata(&path).unwrap();
        if meta.is_dir() {
            pending.extend(fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
            seen.insert(path, None);
        } else {
            let stamp = (fs::read(&path).unwrap(), meta.modified().unwrap());
            seen.insert(path, Some(stamp));
        }
    }
    seen
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn a_repository_with_loose_objects_is_named_by_its_root() {
    let Some(world) = World::new() else { return };
    let repo = world.init("loose");
    world.history(&repo, 6);
    let found = world.check(&repo);
    assert!(found.is_some());
    assert!(
        world
            .run(&repo, &["count-objects", "-v"])
            .unwrap()
            .contains("packs: 0")
    );
}

#[test]
fn a_repository_with_no_commit_is_provisional() {
    let Some(world) = World::new() else { return };
    let repo = world.init("empty");
    assert_eq!(world.check(&repo), None);
    let git = repo.join(".git");
    assert_eq!(
        identity(&git, &|| false).unwrap(),
        RepositoryIdentity::Provisional
    );
}

#[test]
fn a_packed_repository_with_offset_deltas_is_named_by_its_root() {
    let Some(world) = World::new() else { return };
    let repo = world.init("packed");
    world.history(&repo, 40);
    world.git(&repo, &["repack", "-adfq", "--window=250", "--depth=250"]);
    world.git(&repo, &["prune-packed"]);
    let git = repo.join(".git");
    let counts = world.git(&repo, &["count-objects", "-v"]);
    assert!(counts.contains("count: 0"), "{counts}");
    let (all, commits) = deltas(&world, &repo);
    assert!(
        all > 0 && commits > 0,
        "deltas {all}, commits among them {commits}"
    );
    let found = world.check(&repo);
    for id in world.all_commits(&repo) {
        assert!(has_commit(&git, &id).unwrap(), "{}", id.as_str());
    }
    assert!(!has_commit(&git, &CommitId::parse(&"1".repeat(40)).unwrap()).unwrap());
    assert!(found.is_some());
}

#[test]
fn a_packed_repository_with_reference_deltas_is_named_by_its_root() {
    let Some(world) = World::new() else { return };
    let repo = world.init("refdelta");
    world.history(&repo, 40);
    let repack = [
        "-c",
        "repack.useDeltaBaseOffset=false",
        "repack",
        "-adfq",
        "--window=250",
        "--depth=250",
    ];
    world.git(&repo, &repack);
    world.git(&repo, &["prune-packed"]);
    let (all, commits) = deltas(&world, &repo);
    assert!(
        all > 0 && commits > 0,
        "deltas {all}, commits among them {commits}"
    );
    world.check(&repo);
}

/// How many objects of the packs of `repo` are deltas, and how many of
/// those are commits.
fn deltas(world: &World, repo: &Path) -> (usize, usize) {
    let mut files = Vec::new();
    files_under(&repo.join(".git/objects/pack"), &mut files);
    let (mut all, mut commits) = (0, 0);
    for index in files
        .iter()
        .filter(|p| p.extension().is_some_and(|x| x == "idx"))
    {
        let out = world.git(repo, &["verify-pack", "-v", index.to_str().unwrap()]);
        for line in out.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() == 7 && fields[1].len() > 3 && fields[5].parse::<u32>().is_ok() {
                all += 1;
                commits += usize::from(fields[1] == "commit");
            }
        }
    }
    (all, commits)
}

#[test]
fn packed_references_resolve_head() {
    let Some(world) = World::new() else { return };
    let repo = world.init("packedrefs");
    world.history(&repo, 5);
    world.git(&repo, &["pack-refs", "--all", "--prune"]);
    assert!(!repo.join(".git/refs/heads/main").exists());
    assert!(world.check(&repo).is_some());
}

#[test]
fn the_lowest_of_two_roots_names_the_repository() {
    let Some(world) = World::new() else { return };
    for round in 0..8 {
        let repo = world.init(&format!("two-roots-{round}"));
        world.commit(&repo, "a.txt", &format!("first {round}"), "first root");
        world.git(&repo, &["checkout", "-q", "--orphan", "other"]);
        world.git(&repo, &["rm", "-rfq", "."]);
        world.commit(&repo, "b.txt", &format!("second {round}"), "second root");
        world.git(&repo, &["checkout", "-q", "main"]);
        world.git(
            &repo,
            &[
                "merge",
                "-q",
                "--allow-unrelated-histories",
                "other",
                "-m",
                "merge",
            ],
        );
        let roots = world.git(&repo, &["rev-list", "--max-parents=0", "HEAD"]);
        assert_eq!(roots.lines().count(), 2);
        let found = world.check(&repo).unwrap();
        assert!(roots.lines().all(|root| found.as_str() <= root));
        world.git(&repo, &["repack", "-adq"]);
        assert_eq!(world.check(&repo), Some(found));
    }
}

#[test]
fn a_shallow_clone_is_provisional() {
    let Some(world) = World::new() else { return };
    let source = world.init("source");
    world.history(&source, 6);
    let url = format!("file://{}", source.display());
    for depth in ["1", "3"] {
        let clone = world.path(&format!("shallow-{depth}"));
        world.git(
            world.home.path(),
            &[
                "clone",
                "-q",
                "--depth",
                depth,
                &url,
                clone.to_str().unwrap(),
            ],
        );
        assert!(clone.join(".git/shallow").exists());
        let git = clone.join(".git");
        assert_eq!(root_commit(&git, &|| false).unwrap(), None);
        assert_eq!(
            identity(&git, &|| false).unwrap(),
            RepositoryIdentity::Provisional
        );
    }
}

#[test]
fn a_full_clone_has_the_identity_of_its_source() {
    let Some(world) = World::new() else { return };
    let source = world.init("origin-repo");
    world.history(&source, 6);
    let clone = world.path("clone");
    let url = format!("file://{}", source.display());
    world.git(
        world.home.path(),
        &["clone", "-q", &url, clone.to_str().unwrap()],
    );
    let there = root_commit(&source.join(".git"), &|| false).unwrap();
    let here = world.check(&clone);
    assert!(here.is_some());
    assert_eq!(there, here);
}

#[test]
fn the_default_branch_of_the_remote_names_the_repository_not_the_checked_out_one() {
    let Some(world) = World::new() else { return };
    let source = world.init("remote-source");
    world.history(&source, 3);
    let clone = world.path("topic-clone");
    let url = format!("file://{}", source.display());
    world.git(
        world.home.path(),
        &["clone", "-q", &url, clone.to_str().unwrap()],
    );
    world.git(&clone, &["checkout", "-q", "--orphan", "topic"]);
    world.git(&clone, &["rm", "-rfq", "."]);
    world.commit(&clone, "t.txt", "topic", "topic root");
    let topic_root = world.expected_root(&clone, "HEAD");
    let found = world.check(&clone);
    assert_ne!(found, topic_root);
    assert_eq!(
        found,
        world.expected_root(&clone, "refs/remotes/origin/main")
    );
}

#[test]
fn a_linked_worktree_names_the_repository_it_belongs_to() {
    let Some(world) = World::new() else { return };
    let repo = world.init("main-tree");
    world.history(&repo, 4);
    let linked = world.path("linked-tree");
    world.git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "side",
            linked.to_str().unwrap(),
        ],
    );
    assert!(linked.join(".git").is_file());
    let sub = linked.join("deeper");
    fs::create_dir_all(&sub).unwrap();
    let found_dir = git_dir(&sub, None).unwrap().unwrap();
    assert!(
        found_dir.ends_with("worktrees/linked-tree"),
        "{}",
        found_dir.display()
    );
    let expected = world.expected_root(&repo, "HEAD");
    assert_eq!(root_commit(&found_dir, &|| false).unwrap(), expected);
    let tip = world.expected_root(&linked, "HEAD").unwrap();
    assert!(has_commit(&found_dir, &tip).unwrap());
}

#[test]
fn a_separate_git_directory_is_refused_as_a_gitdir_that_leaves_the_clone() {
    let Some(world) = World::new() else { return };
    let work = world.path("separate-work");
    let store = world.path("separate-store");
    fs::create_dir_all(&work).unwrap();
    world.git(
        &work,
        &[
            "init",
            "-q",
            "-b",
            "main",
            "--separate-git-dir",
            store.to_str().unwrap(),
        ],
    );
    world.history(&work, 3);

    // Only a linked worktree's and a submodule's git directory are
    // followed; a `.git` file may not send the reader anywhere else.
    assert!(matches!(
        git_dir(&work, None),
        Err(repository::Error::Unsupported(_))
    ));
}

#[test]
fn a_sha256_repository_is_refused() {
    let Some(world) = World::new() else { return };
    let repo = world.path("sha256");
    fs::create_dir_all(&repo).unwrap();
    if let Err(e) = world.run(
        &repo,
        &["init", "-q", "-b", "main", "--object-format=sha256"],
    ) {
        eprintln!("skipped: git cannot make a sha256 repository ({e})");
        return;
    }
    world.commit(&repo, "a.txt", "sha256", "first");
    let git = repo.join(".git");
    assert!(matches!(
        root_commit(&git, &|| false),
        Err(Error::UnsupportedObjectFormat)
    ));
}

#[test]
fn a_shared_clone_is_unsupported_because_its_objects_are_elsewhere() {
    let Some(world) = World::new() else { return };
    let source = world.init("shared-source");
    world.history(&source, 3);
    let clone = world.path("shared-clone");
    world.git(
        world.home.path(),
        &[
            "clone",
            "-q",
            "--shared",
            source.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    assert!(clone.join(".git/objects/info/alternates").exists());
    let result = root_commit(&clone.join(".git"), &|| false);
    assert!(matches!(result, Err(Error::Unsupported(_))), "{result:?}");
}

#[test]
fn a_deadline_that_has_passed_stops_the_read() {
    let Some(world) = World::new() else { return };
    let repo = world.init("deadline");
    world.history(&repo, 4);
    world.git(&repo, &["pack-refs", "--all", "--prune"]);
    assert!(matches!(
        root_commit(&repo.join(".git"), &|| true),
        Err(Error::Deadline)
    ));
}

#[test]
fn reading_a_repository_changes_nothing_in_it() {
    let Some(world) = World::new() else { return };
    let loose = world.init("readonly-loose");
    world.history(&loose, 5);
    let packed = world.init("readonly-packed");
    world.history(&packed, 20);
    world.git(&packed, &["repack", "-adfq", "--window=250", "--depth=250"]);
    world.git(&packed, &["pack-refs", "--all", "--prune"]);
    for repo in [&loose, &packed] {
        let before = snapshot(repo);
        let git = git_dir(&repo.join("anywhere"), None).unwrap().unwrap();
        root_commit(&git, &|| false).unwrap();
        identity(&git, &|| false).unwrap();
        for id in world.all_commits(repo) {
            has_commit(&git, &id).unwrap();
        }
        assert_eq!(snapshot(repo), before, "{}", repo.display());
    }
}

#[test]
fn corrupted_copies_of_a_repository_never_panic() {
    let Some(world) = World::new() else { return };
    let repo = world.init("corrupt-source");
    world.history(&repo, 12);
    world.git(&repo, &["repack", "-adfq", "--window=250", "--depth=250"]);
    world.git(&repo, &["checkout", "-q", "--orphan", "x"]);
    world.commit(&repo, "loose.txt", "a loose object", "loose commit");
    world.git(&repo, &["checkout", "-q", "main"]);
    let mut files = Vec::new();
    files_under(&repo.join(".git"), &mut files);
    files.retain(|p| {
        let name = p.to_string_lossy();
        name.contains("/objects/") || name.ends_with("/HEAD") || name.contains("/refs/")
    });
    let tip = world.expected_root(&repo, "HEAD").unwrap();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for round in 0..60 {
        let copy = world.path(&format!("corrupt-{round}"));
        copy_tree(&repo.join(".git"), &copy);
        for _ in 0..=(next() % 3) {
            let victim = &files[(next() % files.len() as u64) as usize];
            let target = copy.join(victim.strip_prefix(repo.join(".git")).unwrap());
            let mut bytes = fs::read(&target).unwrap();
            if bytes.is_empty() {
                continue;
            }
            let at = (next() % bytes.len() as u64) as usize;
            match next() % 3 {
                0 => bytes[at] ^= 1 << (next() % 8),
                1 => bytes[at] = next() as u8,
                _ => bytes.truncate(at),
            }
            // Git files its objects and packs read-only.
            fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
            fs::write(&target, bytes).unwrap();
        }
        let _ = root_commit(&copy, &|| false);
        let _ = identity(&copy, &|| false);
        let _ = has_commit(&copy, &tip);
        fs::remove_dir_all(&copy).unwrap();
    }
}
