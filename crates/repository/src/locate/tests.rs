use std::fs;

use testkit::TempDir;

use super::git_dir;
use crate::Error;

#[test]
fn the_git_directory_of_a_repository_is_its_dot_git() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    assert_eq!(
        git_dir(dir.path(), None).unwrap(),
        Some(dir.path().join(".git"))
    );
}

#[test]
fn a_directory_inside_a_repository_finds_the_one_above() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let deep = dir.path().join("a/b/c");
    fs::create_dir_all(&deep).unwrap();
    assert_eq!(git_dir(&deep, None).unwrap(), Some(dir.path().join(".git")));
}

#[test]
fn the_nearest_repository_wins() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let inner = dir.path().join("vendor/lib");
    fs::create_dir_all(inner.join(".git")).unwrap();
    assert_eq!(git_dir(&inner, None).unwrap(), Some(inner.join(".git")));
}

#[test]
fn a_directory_outside_any_repository_finds_none() {
    let dir = TempDir::new().unwrap();
    let inside = dir.path().join("plain/dir");
    fs::create_dir_all(&inside).unwrap();
    assert_eq!(git_dir(&inside, None).unwrap(), None);
}

#[test]
fn a_path_that_is_not_there_is_looked_for_above() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let missing = dir.path().join("not/yet/made");
    assert_eq!(
        git_dir(&missing, None).unwrap(),
        Some(dir.path().join(".git"))
    );
}

#[test]
fn a_dot_git_file_names_the_git_directory_relative_to_itself() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("main/.git/worktrees/w");
    fs::create_dir_all(&real).unwrap();
    let work = dir.path().join("w");
    fs::create_dir(&work).unwrap();
    fs::write(work.join(".git"), "gitdir: ../main/.git/worktrees/w\n").unwrap();
    assert_eq!(
        git_dir(&work, None).unwrap(),
        Some(work.join("../main/.git/worktrees/w"))
    );
}

#[test]
fn a_dot_git_file_may_name_an_absolute_path() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("super/.git/modules/sub");
    fs::create_dir_all(&real).unwrap();
    let work = dir.path().join("sub");
    fs::create_dir(&work).unwrap();
    fs::write(work.join(".git"), format!("gitdir: {}\r\n", real.display())).unwrap();
    assert_eq!(git_dir(&work, None).unwrap(), Some(real));
}

#[test]
fn a_dot_git_file_that_names_no_directory_is_corrupt() {
    let dir = TempDir::new().unwrap();
    for text in [
        "",
        "not a gitdir line\n",
        "gitdir:\n",
        "gitdir: x/.git/worktrees/missing\n",
        "gitdir: plain/.git/worktrees/w\n",
    ] {
        let work = dir.path().join("work");
        fs::create_dir_all(&work).unwrap();
        fs::create_dir_all(work.join("plain/.git/worktrees")).unwrap();
        fs::write(work.join("plain/.git/worktrees/w"), "a file").unwrap();
        fs::write(work.join(".git"), text).unwrap();
        let result = git_dir(&work, None);
        assert!(matches!(result, Err(Error::Corrupt(_))), "{text:?}");
        fs::remove_dir_all(&work).unwrap();
    }
}

#[test]
fn a_dot_git_file_too_large_to_be_one_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".git"), vec![b'x'; 1 << 20]).unwrap();
    assert!(matches!(git_dir(dir.path(), None), Err(Error::TooLarge(_))));
}

#[cfg(unix)]
#[test]
fn a_dot_git_that_is_a_symbolic_link_is_refused() {
    let dir = TempDir::new().unwrap();
    let outside = dir.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    std::os::unix::fs::symlink(&outside, work.join(".git")).unwrap();
    assert!(matches!(git_dir(&work, None), Err(Error::Unsupported(_))));
}

#[cfg(unix)]
#[test]
fn a_gitdir_that_is_a_symbolic_link_is_refused() {
    let dir = TempDir::new().unwrap();
    let outside = dir.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let link = dir.path().join("main/.git/worktrees/link");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&outside, &link).unwrap();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    fs::write(work.join(".git"), "gitdir: ../main/.git/worktrees/link\n").unwrap();
    assert!(matches!(git_dir(&work, None), Err(Error::Unsupported(_))));
}

#[test]
fn a_path_through_a_file_is_looked_for_above_the_file() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("file"), "x").unwrap();
    let through = dir.path().join("file/below");
    assert_eq!(
        git_dir(&through, None).unwrap(),
        Some(dir.path().join(".git"))
    );
}

#[test]
fn a_gitdir_through_a_file_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("plain"), "a file").unwrap();
    fs::write(dir.path().join(".git"), "gitdir: plain/.git/worktrees/w\n").unwrap();
    assert!(matches!(git_dir(dir.path(), None), Err(Error::Io { .. })));
}

#[cfg(unix)]
#[test]
fn a_path_through_a_symbolic_link_loop_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    let again = dir.path().join("again");
    std::os::unix::fs::symlink(&again, &again).unwrap();
    assert!(matches!(
        git_dir(&again.join("below"), None),
        Err(Error::Io { .. })
    ));
}

#[test]
fn the_walk_stops_below_the_ceiling() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let below = dir.path().join("a/b/c");
    fs::create_dir_all(&below).unwrap();
    assert_eq!(git_dir(&below, Some(dir.path())).unwrap(), None);
    assert_eq!(
        git_dir(&below, Some(&dir.path().join("a/b"))).unwrap(),
        None
    );
}

#[test]
fn the_ceiling_itself_is_never_looked_in() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    assert_eq!(git_dir(dir.path(), Some(dir.path())).unwrap(), None);
}

#[test]
fn a_repository_below_the_ceiling_is_found() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("proj/.git")).unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    let sub = dir.path().join("proj/sub");
    fs::create_dir(&sub).unwrap();
    assert_eq!(
        git_dir(&sub, Some(dir.path())).unwrap(),
        Some(dir.path().join("proj/.git"))
    );
}

#[test]
fn a_ceiling_that_is_not_above_the_directory_changes_nothing() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let below = dir.path().join("a");
    fs::create_dir(&below).unwrap();
    let elsewhere = dir.path().join("other");
    assert_eq!(
        git_dir(&below, Some(&elsewhere)).unwrap(),
        Some(dir.path().join(".git"))
    );
    assert_eq!(
        git_dir(&below, Some(&below.join("deeper"))).unwrap(),
        Some(dir.path().join(".git"))
    );
}

#[test]
fn a_dotfiles_repository_in_the_home_directory_is_never_bound() {
    let home = TempDir::new().unwrap();
    fs::create_dir(home.path().join(".git")).unwrap();
    let work = home.path().join("scratch/notes");
    fs::create_dir_all(&work).unwrap();
    assert_eq!(git_dir(&work, Some(home.path())).unwrap(), None);
}

/// Makes a FIFO at `path` with `mkfifo`, which std cannot, waiting for
/// it up to a deadline (ENG-03).
#[cfg(unix)]
#[expect(
    clippy::disallowed_types,
    reason = "a test makes a FIFO with mkfifo; the crate under test starts no program"
)]
fn mkfifo(path: &std::path::Path) {
    let mut child = std::process::Command::new("mkfifo")
        .arg(path)
        .spawn()
        .unwrap();
    let step = std::time::Duration::from_millis(10);
    let mut waited = std::time::Duration::ZERO;
    while child.try_wait().unwrap().is_none() {
        assert!(
            waited < std::time::Duration::from_secs(10),
            "mkfifo did not exit"
        );
        std::thread::sleep(step);
        waited += step;
    }
    assert!(path.exists());
}

#[cfg(unix)]
#[test]
fn a_dot_git_that_is_neither_a_file_nor_a_directory_is_passed_over() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let below = dir.path().join("below");
    fs::create_dir(&below).unwrap();
    mkfifo(&below.join(".git"));
    assert_eq!(
        git_dir(&below, None).unwrap(),
        Some(dir.path().join(".git"))
    );
}

#[test]
fn a_gitdir_that_is_neither_a_worktrees_nor_a_submodules_is_refused() {
    let dir = TempDir::new().unwrap();
    let elsewhere = dir.path().join("private/.git");
    fs::create_dir_all(&elsewhere).unwrap();
    let plain = dir.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();

    for target in [
        elsewhere.display().to_string(),
        "../private/.git".to_owned(),
        "../plain".to_owned(),
    ] {
        fs::write(work.join(".git"), format!("gitdir: {target}\n")).unwrap();
        assert!(
            matches!(git_dir(&work, None), Err(Error::Unsupported(_))),
            "{target}"
        );
    }
}

#[test]
fn only_a_worktrees_or_a_modules_directory_below_a_dot_git_is_followed() {
    let dir = TempDir::new().unwrap();
    let work = dir.path().join("work");
    fs::create_dir(&work).unwrap();
    for below in ["a/.git/worktrees/w", "a/.git/modules/m/n"] {
        let real = dir.path().join(below);
        fs::create_dir_all(&real).unwrap();
        fs::write(work.join(".git"), format!("gitdir: {}\n", real.display())).unwrap();
        assert_eq!(git_dir(&work, None).unwrap(), Some(real));
    }
}

#[test]
fn the_worktree_is_the_directory_holding_dot_git_however_deep_the_search_began() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    let deep = dir.path().join("a/b");
    fs::create_dir_all(&deep).unwrap();

    let found = super::locate(&deep, None).unwrap().unwrap();

    assert_eq!(found.worktree, dir.path());
    assert_eq!(found.git_dir, dir.path().join(".git"));
    assert_eq!(
        super::locate(&dir.path().join("none"), Some(dir.path())).unwrap(),
        None
    );
}
