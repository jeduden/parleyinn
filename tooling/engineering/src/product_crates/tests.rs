use std::collections::BTreeMap;

use super::*;
use testkit::TempDir;

/// Every ban, in the table clippy reads it from.
fn every_ban() -> Vec<(&'static str, &'static str)> {
    let types = REACH_BANS.iter().chain(&DETERMINISM_TYPES);
    types
        .map(|b| (TYPES, *b))
        .chain(
            DETERMINISM_METHODS
                .iter()
                .chain(&DEADLINE_METHODS)
                .map(|b| (METHODS, *b)),
        )
        .collect()
}

/// A clippy.toml carrying every ban.
fn full() -> String {
    let mut body = String::from("disallowed-types = [\n");
    for b in REACH_BANS.iter().chain(&DETERMINISM_TYPES) {
        body.push_str(&format!("  {{ path = \"{b}\", reason = \"r\" }},\n"));
    }
    body.push_str("]\n\ndisallowed-methods = [\n");
    for b in DETERMINISM_METHODS.iter().chain(&DEADLINE_METHODS) {
        body.push_str(&format!("  {{ path = \"{b}\", reason = \"r\" }},\n"));
    }
    body.push_str("]\n");
    body
}

/// The set of `paths`.
fn set(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(|p| (*p).to_owned()).collect()
}

/// A checkout with one crate, `core`, holding `clippy` (none when
/// `None`) and the source files `src`.
fn checkout(clippy: Option<&str>, src: &[(&str, &str)]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::write(root.join("clippy.toml"), "allow-unwrap-in-tests = true\n").unwrap();
    fs::create_dir_all(root.join("crates/core/src")).unwrap();
    fs::create_dir_all(root.join("crates/notes")).unwrap();
    fs::write(root.join("crates/core/Cargo.toml"), "[package]\n").unwrap();
    if let Some(body) = clippy {
        fs::write(root.join("crates/core/clippy.toml"), body).unwrap();
    }
    for (file, body) in src {
        let path = root.join("crates/core/src").join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    let c = Checkout::new(root);
    (dir, c)
}

#[test]
fn banned_reads_every_path_by_its_table() {
    let got = banned(
        "x = [\n { path = \"a::B\", reason = \"r\" },\n{path=\"c::d\"}, { path = 3 }\n]\n# path is\n",
    );

    assert_eq!(
        got,
        BTreeMap::from([("x".to_owned(), set(&["a::B", "c::d"]))])
    );
}

#[test]
fn banned_skips_a_commented_ban() {
    let got = banned(
        "x = [\n  # { path = \"a::B\" },\n  { path = \"c::d\", reason = \"a # is no comment\" }, # path = \"e::f\"\n]\n",
    );

    assert_eq!(got, BTreeMap::from([("x".to_owned(), set(&["c::d"]))]));
}

#[test]
fn banned_reads_one_line_arrays_and_nested_values() {
    let got = banned(
        "[table]\nx = [{ path = \"a::B\", reason = \"say \\\"]\\\" [\" }]\ny=[ { path = \"c::d\", tags = [1, 2] } ]\nstray ]\nz = [ { path = \"open\" }\n",
    );

    assert_eq!(
        got,
        BTreeMap::from([
            (String::new(), set(&[])),
            ("x".to_owned(), set(&["a::B"])),
            ("y".to_owned(), set(&["c::d"])),
        ])
    );
}

#[test]
fn missing_bans_names_each_absent_one_and_its_table() {
    assert!(missing_bans(&full()).is_empty());
    let without = full().replace("std::time::Instant::now", "x");

    assert_eq!(
        missing_bans(&without),
        [(METHODS, "std::time::Instant::now")]
    );
    assert_eq!(missing_bans(""), every_ban());
}

#[test]
fn the_ban_classes_make_up_every_ban() {
    assert_eq!(all_bans(), every_ban());
    assert_eq!(
        deadline_bans(),
        DEADLINE_METHODS
            .iter()
            .map(|b| (METHODS, *b))
            .collect::<Vec<_>>()
    );
    for ban in determinism_bans().into_iter().chain(deadline_bans()) {
        assert!(all_bans().contains(&ban), "{ban:?}");
    }
    assert_eq!(
        determinism_bans().len(),
        DETERMINISM_TYPES.len() + DETERMINISM_METHODS.len()
    );
}

#[test]
fn the_deadline_bans_name_the_calls_that_wait_with_no_deadline() {
    for call in [
        "std::sync::mpsc::Receiver::recv",
        "std::thread::JoinHandle::join",
        "std::process::Child::wait",
        "std::fs::File::lock",
        "std::io::stdin",
    ] {
        assert!(DEADLINE_METHODS.contains(&call), "{call}");
    }
    for call in ["std::thread::sleep", "std::sync::Mutex::lock"] {
        assert!(!DEADLINE_METHODS.contains(&call), "{call}");
    }
}

#[test]
fn missing_among_names_only_the_bans_it_is_asked_for() {
    let without = full().replace("std::thread::park", "x");

    assert!(missing_among(&without, &determinism_bans()).is_empty());
    assert_eq!(
        missing_among(&without, &deadline_bans()),
        [(METHODS, "std::thread::park")]
    );
    assert_eq!(missing_among(&without, &all_bans()), missing_bans(&without));
}

#[test]
fn crates_keep_bans_checks_the_class_it_is_given() {
    let (_d, c) = checkout(Some(&full().replace("std::thread::park", "x")), &[]);

    assert!(crates_keep_bans(&c, &determinism_bans()).is_empty());
    assert_eq!(
        crates_keep_bans(&c, &deadline_bans()),
        [
            "crate core: crates/core/clippy.toml does not ban std::thread::park in disallowed-methods"
        ]
    );
    assert_eq!(crates_keep_their_bans(&c).len(), 1);
}

#[test]
fn missing_bans_skips_a_commented_ban() {
    let commented = full().replace(
        "  { path = \"std::net::UdpSocket\"",
        "  # { path = \"std::net::UdpSocket\"",
    );

    assert_eq!(missing_bans(&commented), [(TYPES, "std::net::UdpSocket")]);
}

#[test]
fn missing_bans_wants_each_ban_in_its_own_table() {
    let moved = full()
        .replace(
            "  { path = \"std::collections::HashMap\", reason = \"r\" },\n",
            "",
        )
        .replace(
            "disallowed-methods = [\n",
            "disallowed-methods = [\n  { path = \"std::collections::HashMap\" },\n",
        );

    assert_eq!(missing_bans(&moved), [(TYPES, "std::collections::HashMap")]);
}

#[test]
fn missing_bans_reads_one_line_arrays() {
    let types: Vec<String> = REACH_BANS
        .iter()
        .chain(&DETERMINISM_TYPES)
        .map(|b| format!("{{ path = \"{b}\" }}"))
        .collect();
    let methods: Vec<String> = DETERMINISM_METHODS
        .iter()
        .chain(&DEADLINE_METHODS)
        .map(|b| format!("{{ path = \"{b}\" }}"))
        .collect();
    let body = format!(
        "disallowed-types = [{}]\ndisallowed-methods = [{}] # the clocks\n",
        types.join(", "),
        methods.join(", ")
    );

    assert!(missing_bans(&body).is_empty(), "{body}");
}

#[test]
fn mutable_globals_finds_interior_mutability_and_thread_locals() {
    let body = "\
static NAME: &str = \"parinn\";
static CONFIG: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(Vec::new);
pub static COUNT: std::sync::atomic::AtomicU64 = AtomicU64::new(0);
  pub(crate) static CACHE: Mutex<Vec<u8>>
      = Mutex::new(Vec::new());
static mut RAW: u8 = 0;
thread_local! { static T: RefCell<u8> = RefCell::new(0); }
let staticky = 1;
";

    let got = mutable_globals("src/lib.rs", body);

    assert_eq!(
        got,
        [
            "src/lib.rs:3: mutable global state: pub static COUNT: std::sync::atomic::AtomicU64 = AtomicU64::new(0);",
            "src/lib.rs:4: mutable global state: pub(crate) static CACHE: Mutex<Vec<u8>>",
            "src/lib.rs:6: mutable global state: static mut RAW: u8 = 0;",
            "src/lib.rs:7: mutable global state: thread_local! { static T: RefCell<u8> = RefCell::new(0); }",
        ]
    );
}

#[test]
fn mutable_globals_finds_a_type_on_the_next_line() {
    let body = "\
static NAME:
    &str = \"parinn\";
pub static CACHE:
    std::sync::Mutex<Vec<u8>>
    = std::sync::Mutex::new(Vec::new());
static OPEN: RefCell<u8>
";

    let got = mutable_globals("src/lib.rs", body);

    assert_eq!(
        got,
        [
            "src/lib.rs:3: mutable global state: pub static CACHE:",
            "src/lib.rs:6: mutable global state: static OPEN: RefCell<u8>",
        ]
    );
}

#[test]
fn mutable_globals_finds_restricted_visibility_statics() {
    let body = "\
pub(super) static A: Mutex<u8> = Mutex::new(0);
pub(in crate::store) static B: AtomicU64 = AtomicU64::new(0);
pub(self) static C: Cell<u8> = Cell::new(0);
pub(self) static D: u8 = 0;
pub(broken static E: Mutex<u8>
publish static F: Mutex<u8> = Mutex::new(0);
";

    let got = mutable_globals("src/lib.rs", body);

    assert_eq!(
        got,
        [
            "src/lib.rs:1: mutable global state: pub(super) static A: Mutex<u8> = Mutex::new(0);",
            "src/lib.rs:2: mutable global state: pub(in crate::store) static B: AtomicU64 = AtomicU64::new(0);",
            "src/lib.rs:3: mutable global state: pub(self) static C: Cell<u8> = Cell::new(0);",
        ]
    );
}

#[test]
fn crates_keep_their_bans_reads_each_crate_or_the_workspace() {
    let (_d, c) = checkout(Some(&full()), &[]);
    assert!(crates_keep_their_bans(&c).is_empty());

    let (_d, c) = checkout(Some(&full().replace("std::net::UdpSocket", "x")), &[]);
    assert_eq!(
        crates_keep_their_bans(&c),
        [
            "crate core: crates/core/clippy.toml does not ban std::net::UdpSocket in disallowed-types"
        ]
    );

    let (_d, c) = checkout(None, &[]);
    let got = crates_keep_their_bans(&c);
    assert_eq!(got.len(), every_ban().len());
    assert!(
        got[0].starts_with("crate core: clippy.toml does not ban"),
        "{got:?}"
    );
}

#[test]
fn crates_keep_their_bans_reads_the_file_clippy_reads() {
    let (d, c) = checkout(Some(&full()), &[]);
    fs::write(d.path().join("crates/core/.clippy.toml"), "").unwrap();
    let got = crates_keep_their_bans(&c);
    assert_eq!(got.len(), every_ban().len());
    assert!(
        got[0].starts_with("crate core: crates/core/.clippy.toml does not ban"),
        "{got:?}"
    );

    let (d, c) = checkout(None, &[]);
    fs::write(d.path().join("crates/clippy.toml"), full()).unwrap();
    assert!(crates_keep_their_bans(&c).is_empty());
}

#[test]
fn crates_keep_their_bans_names_what_it_cannot_read() {
    let (d, c) = checkout(None, &[]);
    fs::remove_file(d.path().join("clippy.toml")).unwrap();
    assert!(crates_keep_their_bans(&c)[0].starts_with("read clippy.toml"));

    let empty = TempDir::new().unwrap();
    let got = crates_keep_their_bans(&Checkout::new(empty.path()));
    assert!(got[0].starts_with("list crates"), "{got:?}");
    assert!(no_mutable_globals(&Checkout::new(empty.path()))[0].starts_with("list crates"));
}

#[test]
fn no_mutable_globals_reads_shipped_code_only() {
    let (_d, c) = checkout(
        None,
        &[
            ("lib.rs", "static OK: u8 = 1;\n"),
            (
                "store/log.rs",
                "static SEQ: AtomicU64 = AtomicU64::new(0);\n",
            ),
            ("store/tests.rs", "static T: Mutex<u8> = Mutex::new(0);\n"),
            ("notes.md", "static X: Mutex<u8>\n"),
        ],
    );

    assert_eq!(
        no_mutable_globals(&c),
        [
            "crates/core/src/store/log.rs:1: mutable global state: static SEQ: AtomicU64 = AtomicU64::new(0);"
        ]
    );
}

#[test]
fn no_mutable_globals_names_what_it_cannot_read() {
    let (d, c) = checkout(None, &[("lib.rs", "")]);
    fs::write(d.path().join("crates/core/src/bad.rs"), [0xff, 0xfe]).unwrap();
    assert!(no_mutable_globals(&c)[0].starts_with("read crates/core/src/bad.rs"));

    fs::remove_dir_all(d.path().join("crates/core/src")).unwrap();
    assert!(
        no_mutable_globals(&c)[0].starts_with("list "),
        "{:?}",
        no_mutable_globals(&c)
    );
}
