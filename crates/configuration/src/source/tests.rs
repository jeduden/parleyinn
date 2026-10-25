use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use testkit::TempDir;

use super::{
    Failure, LayerFailure, Loaded, MANAGED_POLICY, MAX_LAYER_BYTES, OsUser, Sources, load, read,
    read_managed, repository_configuration, writable,
};
use crate::key::key;
use crate::resolve::SettingsLayer;
use crate::value::Value;

/// The OS user running the test: a file it makes belongs to it.
fn me(dir: &Path) -> u32 {
    fs::metadata(dir).unwrap().uid()
}

/// Sources under `dir`, with managed policy in `dir/managed` and the
/// running OS user someone else, so the files the test makes pass as
/// another's.
fn sources(dir: &Path) -> Sources {
    fs::create_dir_all(dir.join("managed")).unwrap();
    fs::set_permissions(dir.join("managed"), fs::Permissions::from_mode(0o755)).unwrap();
    Sources {
        configuration: dir.join("config.toml"),
        repository: Some(dir.join(".parinn.toml")),
        managed: dir.join("managed").join("managed_policy.toml"),
        running: OsUser::Known(me(dir) + 1),
    }
}

fn write(path: &PathBuf, text: &str) {
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
}

fn failure(loaded: &Loaded) -> &LayerFailure {
    assert_eq!(loaded.failures.len(), 1, "{:?}", loaded.failures);
    &loaded.failures[0]
}

#[test]
fn the_managed_policy_path_is_the_documented_one() {
    assert_eq!(MANAGED_POLICY, "/etc/parinn/managed_policy.toml");
}

#[test]
fn load_reads_nothing_where_no_layer_exists() {
    let dir = TempDir::new().unwrap();
    let loaded = load(&sources(dir.path()));
    assert!(loaded.failures.is_empty());
    assert!(loaded.layers.configuration.is_empty());
    assert_eq!(loaded.configuration_digest, None);
    assert!(!loaded.principal_layer_failed() && !loaded.managed_policy_failed());
}

#[test]
fn load_reads_each_layer_and_digests_the_configuration() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    write(&s.configuration, "recall.max_k = 10\n");
    write(s.repository.as_ref().unwrap(), "recall.max_k = 5\n");
    write(&s.managed, "authenticator.required = true\n");
    let loaded = load(&s);
    assert!(loaded.failures.is_empty(), "{:?}", loaded.failures);
    let get =
        |values: &crate::layer::LayerValues, name: &str| values.get(&key(name).unwrap()).cloned();
    assert_eq!(
        get(&loaded.layers.configuration, "recall.max_k"),
        Some(Value::Integer(10))
    );
    assert_eq!(
        get(&loaded.layers.repository, "recall.max_k"),
        Some(Value::Integer(5))
    );
    assert_eq!(
        get(&loaded.layers.managed, "authenticator.required"),
        Some(Value::Bool(true))
    );
    assert_eq!(
        loaded.configuration_digest,
        Some(record::digest::Digest::of(&[b"recall.max_k = 10\n"]))
    );
}

#[test]
fn load_reports_an_invalid_configuration_and_keeps_the_other_layers() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    write(&s.configuration, "recal.max_k = 10\n");
    write(&s.managed, "risk_acceptance.forbidden = true\n");
    let loaded = load(&s);
    let f = failure(&loaded);
    assert_eq!(f.layer, SettingsLayer::Configuration);
    assert!(
        f.to_string()
            .ends_with("config.toml: key \"recal.max_k\": unknown key"),
        "{f}"
    );
    assert!(loaded.principal_layer_failed() && !loaded.managed_policy_failed());
    assert!(loaded.configuration_digest.is_some());
    assert_eq!(loaded.layers.managed.len(), 1);
}

#[test]
fn load_reports_an_invalid_repository_configuration() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    write(s.repository.as_ref().unwrap(), "recall.max_k = 'ten'\n");
    let f = load(&s).failures.remove(0);
    assert_eq!(f.layer, SettingsLayer::RepositoryConfiguration);
}

#[test]
fn load_reports_an_unreadable_configuration() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    fs::create_dir(&s.configuration).unwrap();
    let loaded = load(&s);
    let f = failure(&loaded);
    assert!(matches!(f.failure, Failure::Unreadable(_)));
    assert!(std::error::Error::source(f).is_some());
    assert!(f.to_string().starts_with("configuration "), "{f}");
    assert!(f.to_string().contains("config.toml: "), "{f}");
}

#[test]
fn load_reports_an_unreadable_repository_configuration_and_keeps_the_others() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    fs::create_dir(s.repository.as_ref().unwrap()).unwrap();
    write(&s.configuration, "recall.max_k = 10\n");
    let loaded = load(&s);
    let f = failure(&loaded);
    assert_eq!(f.layer, SettingsLayer::RepositoryConfiguration);
    assert!(matches!(f.failure, Failure::Unreadable(_)));
    assert!(loaded.layers.repository.is_empty());
    assert_eq!(loaded.layers.configuration.len(), 1);
}

#[test]
fn load_refuses_managed_policy_the_running_user_owns() {
    let dir = TempDir::new().unwrap();
    let mut s = sources(dir.path());
    write(&s.managed, "authenticator.required = true\n");
    s.running = OsUser::Known(me(dir.path()));
    let loaded = load(&s);
    let f = failure(&loaded);
    assert!(matches!(&f.failure, Failure::Writable(p) if *p == s.managed));
    assert!(loaded.managed_policy_failed() && !loaded.principal_layer_failed());
    assert!(loaded.layers.managed.is_empty());
    assert!(
        f.to_string()
            .contains("is writable by the running OS user, so it is no managed policy")
    );
    assert!(std::error::Error::source(f).is_none());
}

#[test]
fn load_refuses_managed_policy_in_a_directory_others_may_write() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    write(&s.managed, "");
    let parent = s.managed.parent().unwrap().to_path_buf();
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(matches!(&failure(&load(&s)).failure, Failure::Writable(p) if *p == parent));
}

#[test]
fn load_refuses_managed_policy_that_is_invalid() {
    let dir = TempDir::new().unwrap();
    let s = sources(dir.path());
    write(&s.managed, "nonsense = 1\n");
    let loaded = load(&s);
    assert_eq!(failure(&loaded).layer, SettingsLayer::ManagedPolicy);
    assert!(matches!(failure(&loaded).failure, Failure::Layer(_)));
}

#[test]
fn read_managed_reads_policy_through_the_handle_it_checked() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        read_managed(&dir.path().join("none"), OsUser::Known(0)).unwrap(),
        None
    );
    let file = dir.path().join("p.toml");
    write(&file, "x = 1\n");
    let other = me(dir.path()).wrapping_add(1);

    assert_eq!(
        read_managed(&file, OsUser::Known(other)).unwrap(),
        Some("x = 1\n".to_owned())
    );
    assert!(matches!(
        read_managed(&file, OsUser::Known(me(dir.path()))),
        Err(Failure::Writable(p)) if p == file
    ));
}

#[test]
fn read_managed_refuses_a_symbolic_link() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("target.toml");
    write(&target, "");
    let link = dir.path().join("link.toml");
    symlink(&target, &link).unwrap();
    assert!(matches!(
        read_managed(&link, OsUser::Known(0)),
        Err(Failure::SymbolicLink)
    ));
    let shown = LayerFailure {
        layer: SettingsLayer::ManagedPolicy,
        path: link,
        failure: Failure::SymbolicLink,
    };
    assert!(shown.to_string().ends_with("a symbolic link, refused"));
}

#[test]
fn read_managed_refuses_policy_where_the_os_has_no_modes_to_check() {
    let dir = TempDir::new().unwrap();
    assert!(read_managed(&dir.path().join("none"), OsUser::Unchecked).is_ok());
    let file = dir.path().join("p.toml");
    write(&file, "");
    assert!(matches!(
        read_managed(&file, OsUser::Unchecked),
        Err(Failure::Unverifiable)
    ));
    let shown = LayerFailure {
        layer: SettingsLayer::ManagedPolicy,
        path: file,
        failure: Failure::Unverifiable,
    };
    assert!(
        shown.to_string().ends_with(
            "who may write a file cannot be checked on this OS yet, so no file can be shown to be managed policy"
        ),
        "{shown}"
    );
}

#[test]
fn read_managed_refuses_policy_while_the_running_os_user_is_unknown() {
    let dir = TempDir::new().unwrap();
    assert!(read_managed(&dir.path().join("none"), OsUser::Unknown).is_ok());
    let file = dir.path().join("p.toml");
    write(&file, "");
    assert!(matches!(
        read_managed(&file, OsUser::Unknown),
        Err(Failure::OsUserUnknown)
    ));
    let shown = LayerFailure {
        layer: SettingsLayer::ManagedPolicy,
        path: file,
        failure: Failure::OsUserUnknown,
    };
    assert!(
        shown.to_string().ends_with(
            "the running OS user cannot be learnt, so no file can be shown to be managed policy"
        ),
        "{shown}"
    );
}

#[test]
fn read_managed_reports_an_unreadable_path() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("p.toml");
    write(&file, "");
    let path = file.join("below-a-file");
    assert!(matches!(
        read_managed(&path, OsUser::Known(0)),
        Err(Failure::Unreadable(_))
    ));
}

#[test]
fn writable_reads_owner_and_group_or_other_write() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("f");
    write(&file, "");
    let me = me(dir.path());
    let meta = |mode| {
        fs::set_permissions(&file, fs::Permissions::from_mode(mode)).unwrap();
        fs::metadata(&file).unwrap()
    };
    assert!(writable(&meta(0o644), me));
    assert!(!writable(&meta(0o644), me + 1));
    assert!(writable(&meta(0o664), me + 1));
    assert!(writable(&meta(0o646), me + 1));
}

#[test]
fn read_refuses_a_file_too_large_or_not_text() {
    let dir = TempDir::new().unwrap();
    let big = dir.path().join("big");
    fs::write(&big, vec![b'#'; MAX_LAYER_BYTES as usize + 1]).unwrap();
    assert!(matches!(read(&big), Err(Failure::TooLarge)));
    let binary = dir.path().join("bin");
    fs::write(&binary, [0xff, 0xfe]).unwrap();
    assert!(matches!(read(&binary), Err(Failure::NotText)));
    let say = |failure| {
        LayerFailure {
            layer: SettingsLayer::Configuration,
            path: big.clone(),
            failure,
        }
        .to_string()
    };
    assert!(say(Failure::TooLarge).ends_with("larger than 1048576 bytes"));
    assert!(say(Failure::NotText).ends_with("not UTF-8 text"));
}

#[test]
fn repository_configuration_is_at_the_top_of_the_repository() {
    let dir = TempDir::new().unwrap();
    let top = dir.path().join("repo");
    let deep = top.join("a").join("b");
    fs::create_dir_all(&deep).unwrap();
    assert_eq!(repository_configuration(&deep), None);
    fs::create_dir(top.join(".git")).unwrap();
    assert_eq!(
        repository_configuration(&deep),
        Some(top.join(".parinn.toml"))
    );
    assert_eq!(
        repository_configuration(&top),
        Some(top.join(".parinn.toml"))
    );
}
