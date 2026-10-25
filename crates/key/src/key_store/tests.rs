use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;

use super::*;
use record::id::{RoomId, SeatId};
use testkit::TempDir;

fn store(dir: &TempDir) -> FileKeyStore {
    FileKeyStore::new(dir.path().join("keys"))
}

#[test]
fn entries_are_named_for_what_they_keep() {
    let seat = SeatId::derive(&RoomId::derive(&[1; 32], &[2; 16]), &[3; 32]);

    assert_eq!(Entry::device().name(), "device");
    assert_eq!(Entry::recovery().name(), "recovery");
    assert_eq!(Entry::seat(seat).name(), format!("seat-{seat}"));
    assert_eq!(Entry::staged(&Entry::device()).name(), "staged-device");
    assert_eq!(Entry::staged_seat().name(), "staged-seat");
}

#[test]
fn a_kept_secret_reads_back_from_a_file_only_its_owner_reads() {
    let dir = TempDir::new().unwrap();
    let keys = store(&dir);

    keys.put(&Entry::device(), "00ff").unwrap();

    assert_eq!(keys.get(&Entry::device()).unwrap(), "00ff");
    let path = dir.path().join("keys").join("device.key");
    assert_eq!(keys.path(&Entry::device()), path);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}

#[test]
fn an_entry_is_kept_once_and_never_overwritten() {
    let dir = TempDir::new().unwrap();
    let keys = store(&dir);
    keys.put(&Entry::recovery(), "aa").unwrap();

    let err = keys.put(&Entry::recovery(), "bb").unwrap_err();

    assert!(matches!(err, KeyError::File { .. }), "{err}");
    assert_eq!(keys.get(&Entry::recovery()).unwrap(), "aa");
}

#[test]
fn a_missing_entry_is_missing_and_one_that_cannot_be_read_is_not() {
    let dir = TempDir::new().unwrap();
    let keys = store(&dir);
    fs::create_dir_all(dir.path().join("keys")).unwrap();

    let missing = keys.get(&Entry::device()).unwrap_err();
    assert!(missing.is_missing(), "{missing}");
    assert_eq!(missing.to_string(), "no key in the key store entry device");

    // A directory where the file should be: it cannot be read, and is
    // not missing.
    fs::create_dir(keys.path(&Entry::device())).unwrap();
    let unreadable = keys.get(&Entry::device()).unwrap_err();
    assert!(!unreadable.is_missing(), "{unreadable}");
}

#[cfg(unix)]
#[test]
fn an_entry_others_can_read_is_refused_and_is_not_missing() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let keys = store(&dir);
    keys.put(&Entry::device(), "00").unwrap();
    let path = keys.path(&Entry::device());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    let err = keys.get(&Entry::device()).unwrap_err();

    assert!(matches!(err, KeyError::Permissions { .. }), "{err}");
    assert!(!err.is_missing());
}

#[test]
fn removing_an_entry_removes_it_and_a_missing_one_is_no_failure() {
    let dir = TempDir::new().unwrap();
    let keys = store(&dir);
    keys.put(&Entry::device(), "00").unwrap();

    keys.remove(&Entry::device()).unwrap();
    keys.remove(&Entry::device()).unwrap();

    assert!(keys.get(&Entry::device()).unwrap_err().is_missing());
    fs::create_dir(keys.path(&Entry::device())).unwrap();
    assert!(keys.remove(&Entry::device()).is_err());
}

#[test]
fn each_key_store_says_where_it_keeps_keys_and_whether_an_agent_could_read_them() {
    let dir = TempDir::new().unwrap();
    assert_eq!(store(&dir).kind(), KeyStoreKind::File);
    assert_eq!(KeyStoreKind::File.as_str(), "file");
    assert_eq!(KeyStoreKind::Keychain.as_str(), "keychain");
    assert_eq!(KeyStoreKind::Dpapi.as_str(), "dpapi");
    assert!(KeyStoreKind::File.readable_by_agents());
    assert!(KeyStoreKind::Dpapi.readable_by_agents());
    assert!(!KeyStoreKind::Keychain.readable_by_agents());
    assert_eq!(
        KeyStoreKind::File.describe(),
        "files only this OS user can read; an unsandboxed agent of the same OS user could read them"
    );
    assert_eq!(KeyStoreKind::Keychain.describe(), "the macOS Keychain");
    assert_eq!(
        KeyStoreKind::Dpapi.describe(),
        "files DPAPI protects for this OS user; an unsandboxed agent of the same OS user could read them"
    );
}

#[test]
fn a_keychain_account_is_named_per_home() {
    let a = account(Path::new("/home/a/.parinn"), &Entry::device());
    let b = account(Path::new("/home/b/.parinn"), &Entry::device());

    assert_ne!(a, b);
    assert!(a.ends_with("/device"), "{a}");
    assert_eq!(a.len(), "home-".len() + 32 + "/device".len());
    assert_eq!(a, account(Path::new("/home/a/.parinn"), &Entry::device()));
}

/// A keychain in memory, standing in for the macOS Keychain.
#[derive(Default)]
struct Memory {
    items: RefCell<BTreeMap<String, String>>,
    locked: bool,
}

impl Keychain for Memory {
    fn get(&self, account: &str) -> Result<Option<String>, String> {
        if self.locked {
            return Err("the keychain is locked".into());
        }
        Ok(self.items.borrow().get(account).cloned())
    }

    fn add(&self, account: &str, secret: &str) -> Result<bool, String> {
        if self.locked {
            return Err("the keychain is locked".into());
        }
        let mut items = self.items.borrow_mut();
        if items.contains_key(account) {
            return Ok(false);
        }
        items.insert(account.to_owned(), secret.to_owned());
        Ok(true)
    }

    fn delete(&self, account: &str) -> Result<(), String> {
        if self.locked {
            return Err("the keychain is locked".into());
        }
        self.items.borrow_mut().remove(account);
        Ok(())
    }
}

#[test]
fn a_keychain_store_keeps_each_entry_once_under_the_homes_account() {
    let home = Path::new("/home/a/.parinn");
    let keys = KeychainKeyStore::new(Memory::default(), home);

    keys.put(&Entry::device(), "00ff").unwrap();

    assert_eq!(keys.kind(), KeyStoreKind::Keychain);
    assert_eq!(keys.get(&Entry::device()).unwrap(), "00ff");
    assert!(
        keys.keychain
            .items
            .borrow()
            .contains_key(&account(home, &Entry::device()))
    );
    let again = keys.put(&Entry::device(), "11").unwrap_err();
    assert_eq!(
        again.to_string(),
        "the key store entry device already holds a key"
    );
    assert!(keys.get(&Entry::recovery()).unwrap_err().is_missing());
    keys.remove(&Entry::device()).unwrap();
    assert!(keys.get(&Entry::device()).unwrap_err().is_missing());
}

#[test]
fn a_locked_keychain_is_unavailable_not_missing() {
    let keys = KeychainKeyStore::new(
        Memory {
            locked: true,
            ..Memory::default()
        },
        Path::new("/h"),
    );

    for err in [
        keys.get(&Entry::device()).unwrap_err(),
        keys.put(&Entry::device(), "00").unwrap_err(),
        keys.remove(&Entry::device()).unwrap_err(),
    ] {
        assert!(!err.is_missing(), "{err}");
        assert_eq!(
            err.to_string(),
            "the key store entry device is unavailable: the keychain is locked"
        );
    }
}

/// A protector standing in for DPAPI: it flips every bit, and fails on
/// a blob that starts with a zero byte once flipped.
struct Flip;

impl Protect for Flip {
    fn protect(&self, plain: &[u8]) -> Result<Vec<u8>, String> {
        Ok(plain.iter().map(|b| !b).collect())
    }

    fn unprotect(&self, blob: &[u8]) -> Result<Vec<u8>, String> {
        match blob.first() {
            Some(0xff) => Err("the blob does not unprotect".into()),
            _ => Ok(blob.iter().map(|b| !b).collect()),
        }
    }
}

#[test]
fn a_protected_file_store_keeps_only_the_protected_bytes() {
    let dir = TempDir::new().unwrap();
    let keys = ProtectedFileKeyStore::new(Flip, dir.path().join("keys"));

    keys.put(&Entry::device(), "00ff").unwrap();

    assert_eq!(keys.kind(), KeyStoreKind::Dpapi);
    assert_eq!(keys.get(&Entry::device()).unwrap(), "00ff");
    let path = dir.path().join("keys").join("device.dpapi");
    let on_disk = fs::read_to_string(&path).unwrap();
    assert!(!on_disk.contains("00ff"), "{on_disk}");
    assert!(keys.get(&Entry::recovery()).unwrap_err().is_missing());
    keys.remove(&Entry::device()).unwrap();
    assert!(keys.get(&Entry::device()).unwrap_err().is_missing());
}

#[test]
fn a_protected_entry_that_does_not_unprotect_is_unavailable() {
    let dir = TempDir::new().unwrap();
    let keys = ProtectedFileKeyStore::new(Flip, dir.path().join("keys"));
    // "00" flips to "ff", which Flip refuses to unprotect.
    keys.put(&Entry::device(), "\0").unwrap();
    let err = keys.get(&Entry::device()).unwrap_err();
    assert!(!err.is_missing(), "{err}");
    assert_eq!(
        err.to_string(),
        "the key store entry device is unavailable: the blob does not unprotect"
    );

    fs::write(keys.path(&Entry::recovery()), "zz").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            keys.path(&Entry::recovery()),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
    }
    let err = keys.get(&Entry::recovery()).unwrap_err();
    assert!(!err.is_missing(), "{err}");
}

#[test]
fn a_protector_that_fails_keeps_nothing() {
    struct Broken;
    impl Protect for Broken {
        fn protect(&self, _: &[u8]) -> Result<Vec<u8>, String> {
            Err("no DPAPI".into())
        }
        fn unprotect(&self, _: &[u8]) -> Result<Vec<u8>, String> {
            Err("no DPAPI".into())
        }
    }
    let dir = TempDir::new().unwrap();
    let keys = ProtectedFileKeyStore::new(Broken, dir.path().join("keys"));

    let err = keys.put(&Entry::device(), "00").unwrap_err();

    assert_eq!(
        err.to_string(),
        "the key store entry device is unavailable: no DPAPI"
    );
    assert!(!keys.path(&Entry::device()).exists());
}

#[test]
fn the_platform_key_store_is_this_oss() {
    let dir = TempDir::new().unwrap();
    let keys = platform(&dir.path().join("keys"), dir.path());

    #[cfg(target_os = "macos")]
    assert_eq!(keys.kind(), KeyStoreKind::Keychain);
    #[cfg(windows)]
    assert_eq!(keys.kind(), KeyStoreKind::Dpapi);
    #[cfg(not(any(target_os = "macos", windows)))]
    assert_eq!(keys.kind(), KeyStoreKind::File);
}

#[test]
fn only_an_even_run_of_lowercase_hex_digits_unhexes() {
    assert_eq!(unhex_vec("00ff"), Some(vec![0, 0xff]));
    assert_eq!(unhex_vec(""), Some(vec![]));
    assert_eq!(unhex_vec("0"), None);
    assert_eq!(unhex_vec("zz"), None);
    assert_eq!(unhex_vec("éé"), None);
}

#[test]
fn a_key_store_whose_directory_cannot_be_made_keeps_nothing() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("keys"), "").unwrap();
    let keys = store(&dir);

    let err = keys.put(&Entry::device(), "00").unwrap_err();

    assert!(matches!(err, KeyError::File { .. }), "{err}");
    assert!(err.is_unavailable());
}

#[test]
fn only_a_store_that_cannot_be_read_is_unavailable() {
    let missing = KeyError::Missing {
        entry: "device".into(),
    };
    let locked = KeyError::Unavailable {
        entry: "device".into(),
        reason: "locked".into(),
    };
    let loose = KeyError::Permissions {
        path: "/k".into(),
        mode: 0o644,
    };

    assert!(!missing.is_unavailable());
    assert!(locked.is_unavailable());
    assert!(loose.is_unavailable());
    assert!(!KeyError::Malformed("x".into()).is_unavailable());
}
