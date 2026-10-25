use std::fs;

use testkit::TempDir;

use super::{Repo, check_config};
use crate::Error;

fn unsupported_format(text: &str) -> bool {
    matches!(check_config(text), Err(Error::UnsupportedObjectFormat))
}

#[test]
fn a_sha1_repository_passes() {
    for text in [
        "",
        "[core]\n\trepositoryformatversion = 0\n\tbare = false\n",
        "[extensions]\n\tobjectformat = sha1\n",
        "[extensions]\nobjectFormat = SHA1\n",
    ] {
        assert!(check_config(text).is_ok(), "{text:?}");
    }
}

#[test]
fn a_sha256_repository_is_unsupported() {
    for text in [
        "[extensions]\n\tobjectformat = sha256\n",
        "[extensions]\n\tobjectFormat = SHA256\n",
        "[core]\n\tbare = true\n[extensions]\n\tobjectformat=sha256\n",
        "[extensions]\n\tobjectformat = \"sha256\"\n",
        "[extensions] objectformat = sha256\n",
        "[ Extensions ]\n  objectformat   =   sha256   \n",
        "[extensions]\n\tobjectformat = sha256 # the new hash\n",
        "[extensions]\n\tobjectformat = sha3\n",
    ] {
        assert!(unsupported_format(text), "{text:?}");
    }
}

#[test]
fn an_object_format_outside_the_extensions_section_is_not_the_repositorys() {
    for text in [
        "[core]\n\tobjectformat = sha256\n",
        "[remote \"o\"]\n\tobjectformat = sha256\n",
        "# [extensions]\n[core]\n\tobjectformat = sha256\n",
        "[extensions]\n[core]\n\tobjectformat = sha256\n",
        "; objectformat = sha256\n",
    ] {
        assert!(check_config(text).is_ok(), "{text:?}");
    }
}

#[test]
fn reftable_storage_is_unsupported() {
    let result = check_config("[extensions]\n\trefstorage = reftable\n");
    assert!(matches!(result, Err(Error::Unsupported(_))));
    assert!(check_config("[extensions]\n\trefstorage = files\n").is_ok());
}

#[test]
fn a_repository_is_opened_where_it_is() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config"), "[core]\n\tbare = false\n").unwrap();
    let repo = Repo::open(dir.path()).unwrap();
    assert_eq!(repo.head_dir, dir.path());
    assert_eq!(repo.common_dir, dir.path());
    assert_eq!(repo.objects_dir(), dir.path().join("objects"));
}

#[test]
fn a_repository_without_a_configuration_is_opened() {
    let dir = TempDir::new().unwrap();
    assert!(Repo::open(dir.path()).is_ok());
}

#[test]
fn a_directory_that_is_not_there_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    let result = Repo::open(&dir.path().join("none"));
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn a_sha256_configuration_stops_the_open() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("config"),
        "[extensions]\nobjectformat = sha256\n",
    )
    .unwrap();
    assert!(matches!(
        Repo::open(dir.path()),
        Err(Error::UnsupportedObjectFormat)
    ));
}

#[test]
fn a_configuration_too_large_to_be_one_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config"), vec![b'#'; 2 << 20]).unwrap();
    assert!(matches!(Repo::open(dir.path()), Err(Error::TooLarge(_))));
}

#[test]
fn a_linked_worktree_shares_the_repositorys_objects_and_configuration() {
    let dir = TempDir::new().unwrap();
    let common = dir.path().join("repo.git");
    let linked = common.join("worktrees").join("w");
    fs::create_dir_all(&linked).unwrap();
    fs::write(linked.join("commondir"), "../..\n").unwrap();
    fs::write(
        common.join("config"),
        "[extensions]\nobjectformat = sha256\n",
    )
    .unwrap();
    assert!(matches!(
        Repo::open(&linked),
        Err(Error::UnsupportedObjectFormat)
    ));
    fs::write(common.join("config"), "").unwrap();
    let repo = Repo::open(&linked).unwrap();
    assert_eq!(repo.head_dir, linked);
    let common = fs::canonicalize(&common).unwrap();
    assert_eq!(repo.common_dir, common);
    assert_eq!(repo.objects_dir(), common.join("objects"));
}

fn unsupported<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Unsupported(_)))
}

#[test]
fn a_common_directory_that_is_not_above_the_git_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    let other = dir.path().join("other");
    let linked = dir.path().join("linked");
    fs::create_dir_all(&other).unwrap();
    fs::create_dir_all(linked.join("below")).unwrap();
    for target in [
        format!("{}\n", other.display()),
        "../other\n".to_owned(),
        "below\n".to_owned(),
        "/etc\n".to_owned(),
    ] {
        fs::write(linked.join("commondir"), &target).unwrap();
        assert!(unsupported(Repo::open(&linked)), "{target:?}");
    }
}

#[test]
fn a_common_directory_that_is_the_git_directory_or_above_it_is_read() {
    let dir = TempDir::new().unwrap();
    let real = fs::canonicalize(dir.path()).unwrap();
    let linked = real.join("main/worktrees/w");
    fs::create_dir_all(&linked).unwrap();
    for (target, expected) in [
        (".", linked.clone()),
        ("", linked.clone()),
        ("..", real.join("main/worktrees")),
        ("../..", real.join("main")),
        ("../../..", real.clone()),
    ] {
        fs::write(linked.join("commondir"), format!("{target}\n")).unwrap();
        assert_eq!(
            Repo::open(&linked).unwrap().common_dir,
            expected,
            "{target}"
        );
    }
}

#[test]
fn a_common_directory_that_is_not_there_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("commondir"), "../gone\n").unwrap();
    assert!(matches!(Repo::open(dir.path()), Err(Error::Io { .. })));
}

#[test]
fn a_common_directory_that_is_a_file_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("commondir"), "file\n").unwrap();
    fs::write(dir.path().join("file"), "x").unwrap();
    assert!(unsupported(Repo::open(dir.path())));
}
