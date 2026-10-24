use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use testkit::TempDir;

use super::{open_below, open_checked, read_limited, refuse_symlink};
use crate::Error;

fn unsupported<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Unsupported(_)))
}

#[test]
fn a_file_is_read_whole() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f"), b"contents").unwrap();
    let bytes = read_limited(dir.path(), "f", 100).unwrap().unwrap();
    assert_eq!(bytes, b"contents");
}

#[test]
fn a_file_below_directories_is_read() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("a/b")).unwrap();
    fs::write(dir.path().join("a/b/f"), b"deep").unwrap();
    assert_eq!(
        read_limited(dir.path(), "a/b/f", 100).unwrap().unwrap(),
        b"deep"
    );
}

#[test]
fn a_file_of_exactly_the_limit_is_read() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f"), b"12345").unwrap();
    assert_eq!(read_limited(dir.path(), "f", 5).unwrap().unwrap(), b"12345");
}

#[test]
fn a_missing_file_is_none() {
    let dir = TempDir::new().unwrap();
    assert!(read_limited(dir.path(), "none", 5).unwrap().is_none());
    assert!(open_below(dir.path(), "none").unwrap().is_none());
}

#[test]
fn a_missing_directory_on_the_way_is_none() {
    let dir = TempDir::new().unwrap();
    assert!(open_below(dir.path(), "no/such/f").unwrap().is_none());
    fs::create_dir(dir.path().join("a")).unwrap();
    assert!(open_below(dir.path(), "a/no/f").unwrap().is_none());
}

#[test]
fn a_file_past_the_limit_is_too_large() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f"), b"123456").unwrap();
    assert!(matches!(
        read_limited(dir.path(), "f", 5),
        Err(Error::TooLarge(_))
    ));
}

#[test]
fn a_directory_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("d")).unwrap();
    assert!(unsupported(read_limited(dir.path(), "d", 5)));
}

#[test]
fn a_device_is_refused_before_it_is_opened() {
    let result = open_below(Path::new("/dev"), "null");
    assert!(unsupported(result));
}

#[test]
fn a_symbolic_link_is_refused_whatever_it_points_to() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("real"), "x").unwrap();
    symlink(dir.path().join("real"), dir.path().join("to-file")).unwrap();
    symlink(dir.path(), dir.path().join("to-dir")).unwrap();
    symlink(dir.path().join("gone"), dir.path().join("dangling")).unwrap();
    for name in ["to-file", "to-dir", "dangling"] {
        assert!(unsupported(open_below(dir.path(), name)), "{name}");
    }
}

#[test]
fn a_symbolic_link_directory_on_the_way_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("elsewhere")).unwrap();
    fs::write(dir.path().join("elsewhere/f"), "x").unwrap();
    fs::create_dir(dir.path().join("a")).unwrap();
    symlink(dir.path().join("elsewhere"), dir.path().join("a/b")).unwrap();
    assert!(unsupported(open_below(dir.path(), "a/b/f")));
}

#[test]
fn a_path_through_a_file_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("file"), "x").unwrap();
    let result = read_limited(dir.path(), "file/below", 5);
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn a_directory_the_file_system_cannot_say_about_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    let long = "x".repeat(300);
    let result = open_below(dir.path(), &format!("{long}/f"));
    assert!(matches!(result, Err(Error::Io { .. })));
    let result = open_below(dir.path(), &long);
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn a_file_that_cannot_be_opened_is_an_io_error() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("f");
    fs::write(&path, "x").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    // A superuser reads it anyway; the refusal is for everyone else.
    if fs::File::open(&path).is_err() {
        assert!(matches!(open_below(dir.path(), "f"), Err(Error::Io { .. })));
    }
}

#[test]
fn a_symbolic_link_is_what_refuse_symlink_refuses() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("d")).unwrap();
    symlink(dir.path().join("d"), dir.path().join("l")).unwrap();
    assert!(refuse_symlink(&dir.path().join("d")).is_ok());
    assert!(refuse_symlink(&dir.path().join("none")).is_ok());
    assert!(unsupported(refuse_symlink(&dir.path().join("l"))));
    let long = dir.path().join("x".repeat(300));
    assert!(matches!(refuse_symlink(&long), Err(Error::Io { .. })));
}

/// Makes a FIFO at `path` with `mkfifo`, which std cannot, waiting for
/// it up to a deadline (ENG-03).
#[expect(
    clippy::disallowed_types,
    reason = "a test makes a FIFO with mkfifo; the crate under test starts no program"
)]
fn mkfifo(path: &Path) {
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
}

#[test]
fn a_regular_file_is_opened_checked() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f"), "x").unwrap();

    assert!(open_checked(&dir.path().join("f")).is_ok());
}

#[test]
fn a_fifo_swapped_in_after_the_look_is_refused_not_waited_on() {
    let dir = TempDir::new().unwrap();
    mkfifo(&dir.path().join("f"));
    let path = dir.path().join("f");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(open_checked(&path).map(|_| ()));
    });

    let answer = rx.recv_timeout(std::time::Duration::from_secs(5));

    assert!(
        matches!(answer, Ok(Err(Error::Unsupported(_)))),
        "{answer:?}"
    );
}

#[test]
fn a_link_swapped_in_after_the_look_is_refused_not_followed() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("target"), "x").unwrap();
    symlink(dir.path().join("target"), dir.path().join("f")).unwrap();

    assert!(unsupported(open_checked(&dir.path().join("f"))));
}

#[test]
fn a_file_gone_since_the_look_is_an_io_error() {
    let dir = TempDir::new().unwrap();

    assert!(matches!(
        open_checked(&dir.path().join("gone")),
        Err(Error::Io { .. })
    ));
}
