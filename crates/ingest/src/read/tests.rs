use super::*;
use testkit::TempDir;

#[test]
fn every_terminated_line_is_consumable_with_the_bytes_it_spans() {
    let got = lines("a\n\nbc\n");

    assert_eq!(
        got,
        [
            Line {
                text: "a",
                offset: 0,
                end: 2
            },
            Line {
                text: "",
                offset: 2,
                end: 3
            },
            Line {
                text: "bc",
                offset: 3,
                end: 6
            },
        ]
    );
    assert!(lines("").is_empty());
}

#[test]
fn a_last_line_without_its_newline_waits_unless_it_parses() {
    assert_eq!(lines("a\n{\"type\": \"assist").len(), 1);
    let parsed = lines("a\n{\"x\":1}");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[1].text, "{\"x\":1}");
    assert_eq!(parsed[1].end, 9);
}

#[test]
fn a_regular_file_is_read_whole() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, "x\n").unwrap();

    assert_eq!(transcript(&path).unwrap(), b"x\n");
    assert_eq!(
        transcript(&dir.path().join("none")).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn a_directory_or_link_is_refused_before_it_is_opened() {
    let dir = TempDir::new().unwrap();

    let err = transcript(dir.path()).unwrap_err();
    assert_eq!(err.to_string(), "not a regular file");

    #[cfg(unix)]
    {
        let target = dir.path().join("t.jsonl");
        std::fs::write(&target, "x\n").unwrap();
        let link = dir.path().join("link.jsonl");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert_eq!(
            transcript(&link).unwrap_err().to_string(),
            "not a regular file"
        );
    }
}

/// Makes a FIFO at `path` with `mkfifo`, which std cannot, waiting for
/// it up to a deadline (ENG-03).
#[cfg(unix)]
#[expect(
    clippy::disallowed_types,
    reason = "a test makes a FIFO with mkfifo; the core starts no program"
)]
fn mkfifo(path: &Path) {
    let mut child = std::process::Command::new("mkfifo")
        .arg(path)
        .spawn()
        .unwrap();
    let deadline = std::time::Duration::from_secs(10);
    let mut waited = std::time::Duration::ZERO;
    while child.try_wait().unwrap().is_none() {
        assert!(waited < deadline, "mkfifo did not exit");
        std::thread::sleep(std::time::Duration::from_millis(10));
        waited += std::time::Duration::from_millis(10);
    }
    assert!(path.exists());
}

#[cfg(unix)]
#[test]
fn a_fifo_is_refused_without_blocking() {
    let dir = TempDir::new().unwrap();
    let fifo = dir.path().join("fifo.jsonl");
    mkfifo(&fifo);

    assert_eq!(
        transcript(&fifo).unwrap_err().to_string(),
        "not a regular file"
    );
    // Opened anyway, as a swap after the look would open it, it does not
    // wait for a writer.
    assert!(open(&fifo).is_ok());
}

#[test]
fn a_file_swapped_between_the_look_and_the_open_is_refused() {
    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::write(&a, "x").unwrap();
    std::fs::write(&b, "yy").unwrap();
    let looked = std::fs::symlink_metadata(&a).unwrap();

    let err = same_file(&looked, &std::fs::metadata(&b).unwrap()).unwrap_err();

    assert_eq!(err.to_string(), "the file changed while it was opened");
    assert!(same_file(&looked, &std::fs::metadata(&a).unwrap()).is_ok());
    let not_a_file = std::fs::metadata(dir.path()).unwrap();
    assert!(same_file(&not_a_file, &not_a_file).is_err());
}
