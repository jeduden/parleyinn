use super::*;

#[test]
fn errors_explain_themselves_and_keep_their_cause() {
    let io = AdminError::Io {
        path: PathBuf::from("/h"),
        source: std::io::Error::other("x"),
    };
    assert_eq!(io.to_string(), "/h: x");
    assert!(std::error::Error::source(&io).is_some());
    let key = AdminError::from(key::KeyError::Malformed("k".into()));
    assert_eq!(key.to_string(), "malformed k");
    assert!(std::error::Error::source(&key).is_some());
    let store = AdminError::from(store::StoreError::Newer { found: 9 });
    assert!(store.to_string().contains("version 9"));
    assert!(std::error::Error::source(&store).is_some());
    assert!(std::error::Error::source(&AdminError::NoHome).is_none());
    assert!(std::error::Error::source(&AdminError::Node("n")).is_none());
    let repository = AdminError::Repository("r");
    assert_eq!(repository.to_string(), "r");
    assert!(std::error::Error::source(&repository).is_none());
    let read = AdminError::RepositoryRead(::repository::Error::Deadline);
    assert!(
        read.to_string()
            .starts_with("the repository's git data cannot be read: ")
    );
    assert!(std::error::Error::source(&read).is_some());
    let unresolved = AdminError::Unresolved {
        what: "worktree",
        source: std::io::Error::other("gone"),
    };
    assert_eq!(
        unresolved.to_string(),
        "the worktree cannot be resolved: gone"
    );
    assert!(std::error::Error::source(&unresolved).is_some());
}

#[test]
fn the_home_s_refusals_name_what_they_refuse_and_have_no_cause() {
    let p = || PathBuf::from("/h");
    for e in [
        AdminError::LoosePermissions {
            path: p(),
            mode: 0o755,
        },
        AdminError::OtherOsUser {
            path: p(),
            uid: 1,
            running: 2,
        },
        AdminError::SymbolicLink { path: p() },
        AdminError::HomeIdBinding { path: p() },
        AdminError::HomeIdMismatch,
        AdminError::HomeIdMissing,
    ] {
        assert!(std::error::Error::source(&e).is_none(), "{e}");
    }
    let os = AdminError::OsUser(std::io::Error::other("no pipe"));
    assert_eq!(os.to_string(), "learn the running OS user: no pipe");
    assert!(std::error::Error::source(&os).is_some());
}
