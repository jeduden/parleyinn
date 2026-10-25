use std::collections::BTreeMap;
use std::fs;

use testkit::TempDir;

use super::*;

/// A process table: each pid's parent and names.
struct Table(BTreeMap<u32, (Option<u32>, Vec<&'static str>)>);

impl Processes for Table {
    fn parent(&self, pid: u32) -> Option<u32> {
        self.0.get(&pid).and_then(|(p, _)| *p)
    }

    fn names(&self, pid: u32) -> Vec<String> {
        self.0
            .get(&pid)
            .map(|(_, n)| n.iter().map(|s| (*s).to_owned()).collect())
            .unwrap_or_default()
    }
}

/// The chain `names[0]` (the handler) up to its last, each the parent
/// of the one before, the last's parent the first process.
fn chain(names: &[&'static str]) -> Table {
    let mut table = BTreeMap::new();
    for (i, name) in names.iter().enumerate() {
        let pid = 100 + i as u32;
        let parent = if i + 1 == names.len() { 0 } else { pid + 1 };
        table.insert(pid, (Some(parent), vec![*name]));
    }
    Table(table)
}

#[test]
fn a_handler_below_its_own_harness_alone_is_not_nested() {
    let table = chain(&["parinn", "sh", "claude", "bash", "init"]);

    assert_eq!(
        observe(&table, 100),
        Some(Ancestry {
            harnesses: 1,
            nested: false,
            harness: Some(102),
        })
    );
}

#[test]
fn a_harness_above_the_sessions_own_makes_it_nested() {
    let table = chain(&["parinn", "sh", "claude", "bash", "claude", "zsh"]);

    assert_eq!(
        observe(&table, 100),
        Some(Ancestry {
            harnesses: 2,
            nested: true,
            harness: Some(102),
        }),
        "the nearest harness process is the session's own"
    );
}

#[test]
fn a_handler_no_harness_started_counts_none() {
    assert_eq!(
        observe(&chain(&["parinn", "bash"]), 100),
        Some(Ancestry {
            harnesses: 0,
            nested: false,
            harness: None,
        })
    );
}

#[test]
fn an_unreadable_parent_a_loop_or_a_chain_past_the_limit_is_no_observation() {
    let mut unreadable = chain(&["parinn", "sh"]);
    unreadable.0.insert(101, (None, vec!["sh"]));
    assert_eq!(observe(&unreadable, 100), None);

    let mut looped = chain(&["parinn", "sh"]);
    looped.0.insert(101, (Some(101), vec!["sh"]));
    assert_eq!(
        observe(&looped, 100).map(|a| a.harnesses),
        Some(0),
        "a process its own parent ends the chain"
    );

    let mut cycle = BTreeMap::new();
    cycle.insert(1, (Some(2), vec!["a"]));
    cycle.insert(2, (Some(1), vec!["b"]));
    assert_eq!(observe(&Table(cycle), 1), None);

    let deep: Vec<&'static str> = vec!["x"; DEPTH_LIMIT + 2];
    assert_eq!(observe(&chain(&deep), 100), None);
}

/// Writes a `/proc`-shaped entry for `pid` under `root`.
fn proc_entry(root: &Path, pid: u32, parent: u32, comm: &str, cmdline: &[&str]) {
    let dir = root.join(pid.to_string());
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("stat"),
        format!("{pid} ({comm} with) spaces) S {parent} 1 1 0"),
    )
    .unwrap();
    fs::write(dir.join("comm"), format!("{comm}\n")).unwrap();
    fs::write(dir.join("cmdline"), cmdline.join("\0")).unwrap();
}

#[test]
fn proc_gives_each_parent_and_the_names_a_process_runs_under() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    proc_entry(
        root,
        7,
        5,
        "node",
        &["/usr/bin/node", "/opt/bin/claude", "-p"],
    );
    proc_entry(root, 5, 0, "init", &[]);
    let proc = ProcFs::at(root);

    assert_eq!(proc.parent(7), Some(5));
    assert_eq!(proc.parent(9), None);
    assert_eq!(proc.names(7), ["node", "node", "claude"]);
    assert_eq!(proc.names(9), Vec::<String>::new());
    assert_eq!(
        observe(&proc, 7),
        Some(Ancestry {
            harnesses: 0,
            nested: false,
            harness: None,
        })
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("7", root.join("self")).unwrap();
        assert_eq!(proc.own(), Some(7));
    }
}

#[test]
fn a_stat_line_without_its_fields_gives_no_parent() {
    assert_eq!(parent_of("12 (sh) S 4 1"), Some(4));
    assert_eq!(parent_of("12 (sh"), None);
    assert_eq!(parent_of("12 (sh) S x"), None);
    assert_eq!(base_name("C:\\bin\\claude.exe"), "claude.exe");
    assert_eq!(base_name("claude"), "claude");
}

#[cfg(target_os = "linux")]
#[test]
fn this_process_reads_its_own_ancestry_on_linux() {
    assert!(own().is_some());
    assert!(ProcFs::default().own().is_some());
}
