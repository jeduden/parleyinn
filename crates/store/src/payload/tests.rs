use std::sync::{Arc, Mutex};

use record::digest::Digest;
use serde_json::json;
use testkit::TempDir;

use super::*;

fn key(n: u8) -> CommitmentKey {
    CommitmentKey::from_bytes([n; 32])
}

#[test]
fn a_payload_s_name_derives_from_its_commitment_key_alone() {
    let name = name(&key(1));

    assert_eq!(name.len(), 64);
    assert!(
        name.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    assert_eq!(name, super::name(&key(1)));
    assert_ne!(name, super::name(&key(2)), "equal contents get two names");
}

#[test]
fn a_payload_s_name_is_no_digest_of_its_content() {
    let content = json!({"line": 1, "text": "x".repeat(9000)});
    let canonical = record::canonical::encode(&content).unwrap();

    let name = name(&key(1));

    assert_ne!(name, Digest::of(&[canonical.as_bytes()]).to_hex());
    assert_ne!(name, Digest::of(&["x".repeat(9000).as_bytes()]).to_hex());
}

#[test]
fn a_preview_is_the_text_cut_to_512_bytes_at_a_character_boundary() {
    let short = json!({"line": 1, "text": "short"});
    assert_eq!(preview(&short, "{}"), "short");

    let long = json!({"text": "é".repeat(300)});
    let cut = preview(&long, "{}");
    assert_eq!(cut.len(), 512);
    assert_eq!(cut, "é".repeat(256));

    let odd = json!({"text": format!("a{}", "é".repeat(300))});
    let cut = preview(&odd, "{}");
    assert_eq!(cut.len(), 511, "the half character is left out");
}

#[test]
fn a_preview_of_content_without_text_cuts_its_canonical_encoding() {
    let content = json!({"n": 1});
    assert_eq!(preview(&content, "{\"n\":1}"), "{\"n\":1}");
    let long = "y".repeat(600);
    assert_eq!(preview(&json!([1]), &long), "y".repeat(512));
}

/// A disk that records each operation, then does it on the real one.
#[derive(Clone, Default)]
struct Recording(Arc<Mutex<Vec<String>>>);

impl Disk for Recording {
    fn write_synced(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.0
            .lock()
            .unwrap()
            .push(format!("write+sync {}", file(path)));
        OsDisk.write_synced(path, bytes)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.0
            .lock()
            .unwrap()
            .push(format!("rename {} {}", file(from), file(to)));
        OsDisk.rename(from, to)
    }

    fn sync_dir(&self, dir: &Path) -> io::Result<()> {
        self.0.lock().unwrap().push(format!("sync {}", file(dir)));
        OsDisk.sync_dir(dir)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        OsDisk.read(path)
    }
}

fn file(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn a_payload_is_written_to_a_temporary_file_synced_then_renamed_into_place() {
    let dir = TempDir::new().unwrap();
    let ops = Recording::default();
    let payloads = Payloads::new(dir.path().join("payloads"), Box::new(ops.clone()));
    let name = name(&key(1));

    payloads.write(&name, b"content").unwrap();

    assert_eq!(
        *ops.0.lock().unwrap(),
        [
            format!("write+sync .tmp-{name}"),
            format!("rename .tmp-{name} {name}"),
            "sync payloads".to_owned(),
        ]
    );
    assert_eq!(payloads.read(&name).unwrap(), b"content");
    let left: Vec<_> = std::fs::read_dir(dir.path().join("payloads"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, [std::ffi::OsString::from(&name)]);
}

#[cfg(unix)]
#[test]
fn the_payload_store_and_each_payload_are_readable_only_by_their_owner() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let payloads = Payloads::new(dir.path().join("payloads"), Box::new(OsDisk));
    let name = name(&key(1));

    payloads.write(&name, b"content").unwrap();

    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&dir.path().join("payloads")), 0o700);
    assert_eq!(mode(&dir.path().join("payloads").join(&name)), 0o600);
}

#[test]
fn a_temporary_file_a_crash_left_is_written_over() {
    let dir = TempDir::new().unwrap();
    let payloads = Payloads::new(dir.path().join("payloads"), Box::new(OsDisk));
    let name = name(&key(1));
    std::fs::create_dir_all(dir.path().join("payloads")).unwrap();
    std::fs::write(
        dir.path().join("payloads").join(format!(".tmp-{name}")),
        "a crash left this, longer than the payload",
    )
    .unwrap();

    payloads.write(&name, b"content").unwrap();

    assert_eq!(payloads.read(&name).unwrap(), b"content");
}

/// A disk whose every write fails.
struct Failing;

impl Disk for Failing {
    fn write_synced(&self, _: &Path, _: &[u8]) -> io::Result<()> {
        Err(io::Error::other("the disk is full"))
    }
    fn rename(&self, _: &Path, _: &Path) -> io::Result<()> {
        Err(io::Error::other("no rename"))
    }
    fn sync_dir(&self, _: &Path) -> io::Result<()> {
        Err(io::Error::other("no sync"))
    }
    fn read(&self, _: &Path) -> io::Result<Vec<u8>> {
        Err(io::Error::other("no read"))
    }
}

#[test]
fn a_failed_write_says_why_and_leaves_no_payload() {
    let dir = TempDir::new().unwrap();
    let payloads = Payloads::new(dir.path().join("payloads"), Box::new(Failing));
    let name = name(&key(1));

    let err = payloads.write(&name, b"content").unwrap_err();

    assert_eq!(err.to_string(), "the disk is full");
    assert!(!dir.path().join("payloads").join(&name).exists());
    assert!(Failing.rename(dir.path(), dir.path()).is_err());
    assert!(Failing.sync_dir(dir.path()).is_err());
    assert!(Failing.read(dir.path()).is_err());
}

#[test]
fn a_payload_store_that_cannot_be_made_says_why() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("payloads"), "a file in its place").unwrap();
    let payloads = Payloads::new(dir.path().join("payloads"), Box::new(OsDisk));

    assert!(payloads.write(&name(&key(1)), b"content").is_err());
}

#[test]
fn only_a_payload_name_is_read() {
    let dir = TempDir::new().unwrap();
    let payloads = Payloads::new(dir.path().to_path_buf(), Box::new(OsDisk));

    for bad in ["../store.db", "x", &"A".repeat(64), &"0".repeat(63)] {
        let err = payloads.read(bad).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{bad}");
    }
}

#[test]
fn the_payload_store_shows_where_it_is() {
    let payloads = Payloads::new(PathBuf::from("/h/payloads"), Box::new(OsDisk));

    assert!(format!("{payloads:?}").contains("/h/payloads"));
}
