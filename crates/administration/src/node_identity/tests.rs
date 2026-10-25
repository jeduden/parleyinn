use super::*;
use testkit::TempDir;

/// A home in `dir` whose node identity is read from `dir/machine-id`.
fn home(dir: &TempDir) -> Home {
    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None)
        .unwrap()
        .with_identity(IdentitySource::Files(vec![dir.path().join("machine-id")]));
    home.create().unwrap();
    home
}

/// A host id that reads `value`.
struct Fixed(Option<&'static str>);

impl platform::identity::HostId for Fixed {
    fn read(&self) -> Option<String> {
        self.0.map(str::to_owned)
    }
}

fn linux() -> IdentitySource {
    IdentitySource::Files(vec![
        PathBuf::from("/etc/machine-id"),
        PathBuf::from("/var/lib/dbus/machine-id"),
    ])
}

#[test]
fn this_os_reads_its_own_source() {
    let source = IdentitySource::platform();

    if cfg!(target_os = "linux") {
        assert_eq!(source, linux());
    } else if cfg!(target_os = "macos") {
        assert_eq!(source, IdentitySource::HardwareUuid);
    } else if cfg!(windows) {
        assert_eq!(source, IdentitySource::MachineGuid);
    } else {
        assert_eq!(source, IdentitySource::Unsupported);
    }
    assert_eq!(IdentitySource::Unsupported.read(), None);
}

#[test]
fn the_hardware_uuid_and_the_machine_guid_read_through_the_host_id() {
    for source in [IdentitySource::HardwareUuid, IdentitySource::MachineGuid] {
        assert_eq!(
            source.read_with(&Fixed(Some("abc"))).as_deref(),
            Some("abc")
        );
        assert_eq!(source.read_with(&Fixed(None)), None);
    }
}

#[test]
fn a_file_source_and_an_unsupported_one_ignore_the_host_id() {
    let files = IdentitySource::Files(vec![PathBuf::from("/nonexistent/id")]);

    assert_eq!(files.read_with(&Fixed(Some("abc"))), None);
    assert_eq!(
        IdentitySource::Unsupported.read_with(&Fixed(Some("abc"))),
        None
    );
}

#[test]
fn a_source_says_what_the_identity_is_checked_against() {
    assert_eq!(linux().describe(), Some("Linux machine id"));
    assert_eq!(
        IdentitySource::HardwareUuid.describe(),
        Some("macOS hardware UUID")
    );
    assert_eq!(
        IdentitySource::MachineGuid.describe(),
        Some("Windows MachineGuid")
    );
    assert_eq!(
        IdentitySource::Files(vec![PathBuf::from("/x")]).describe(),
        Some("a file")
    );
    assert_eq!(IdentitySource::Unsupported.describe(), None);
}

#[test]
fn a_host_source_is_checked_like_a_file_one() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir).with_identity(IdentitySource::HardwareUuid);

    // A first start records whatever the host id reads, or its
    // unavailability, and counts as unchanged.
    assert_eq!(changed(&home).unwrap(), None);
    assert!(home.node_identity().exists());
}

#[test]
fn the_first_file_holding_a_value_names_the_node() {
    let dir = TempDir::new().unwrap();
    let (a, b) = (dir.path().join("a"), dir.path().join("b"));
    let source = IdentitySource::Files(vec![a.clone(), b.clone()]);
    assert_eq!(source.observe(), None);

    fs::write(&b, "beta\n").unwrap();
    assert_eq!(source.read().as_deref(), Some("beta"));
    fs::write(&a, "  \n").unwrap();
    assert_eq!(source.read().as_deref(), Some("beta"));
    fs::write(&a, "alpha\n").unwrap();
    assert_eq!(source.observe().as_deref(), Some("alpha"));
}

#[test]
fn a_digest_keeps_no_value_and_an_unavailable_one_has_its_own() {
    let a = digest(Some("alpha"));

    assert_ne!(a, digest(Some("beta")));
    assert_ne!(a, digest(None));
    assert_eq!(digest(None), digest(None));
    assert!(!a.to_hex().contains("alpha"));
}

#[test]
fn a_home_that_recorded_no_identity_records_it_and_is_unchanged() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    fs::write(dir.path().join("machine-id"), "alpha").unwrap();

    assert_eq!(changed(&home).unwrap(), None);

    let recorded = fs::read_to_string(home.node_identity()).unwrap();
    assert_eq!(recorded, digest(Some("alpha")).to_hex());
    assert_eq!(changed(&home).unwrap(), None);
}

#[test]
fn another_value_or_none_at_all_is_a_change() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    fs::write(dir.path().join("machine-id"), "alpha").unwrap();
    changed(&home).unwrap();

    fs::write(dir.path().join("machine-id"), "beta").unwrap();
    assert_eq!(changed(&home).unwrap(), Some(digest(Some("beta"))));

    fs::remove_file(dir.path().join("machine-id")).unwrap();
    assert_eq!(changed(&home).unwrap(), Some(digest(None)));

    record(&home, digest(None)).unwrap();
    assert_eq!(changed(&home).unwrap(), None);
}

#[test]
fn an_unsupported_source_is_never_checked() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir).with_identity(IdentitySource::Unsupported);

    assert_eq!(changed(&home).unwrap(), None);
    assert!(!home.node_identity().exists());
}

#[test]
fn a_recorded_digest_that_cannot_be_read_or_written_is_returned() {
    let dir = TempDir::new().unwrap();
    let a = home(&dir);
    fs::create_dir(a.node_identity()).unwrap();

    assert!(matches!(changed(&a), Err(AdminError::Io { .. })));
    assert!(matches!(
        record(&a, digest(None)),
        Err(AdminError::Io { .. })
    ));

    let dir = TempDir::new().unwrap();
    let b = home(&dir);
    fs::create_dir(staging(&b.node_identity())).unwrap();
    fs::write(staging(&b.node_identity()).join("x"), "").unwrap();
    assert!(matches!(
        record(&b, digest(None)),
        Err(AdminError::Io { .. })
    ));
}
