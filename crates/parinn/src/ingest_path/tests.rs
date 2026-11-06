use std::fs;
use std::path::Path;

use administration::home::Home;
use record::digest::Digest;
use store::Store;

use crate::settings::acts_in;
use crate::tests::{DEV, Process, audited, set_mode};
use crate::{EXIT_FAILURE, EXIT_OK};

const SECRET: &str = "key AKIAIOSFODNN7EXAMPLE\n";

/// A process whose standard input and output are no terminal, as a hook
/// handler's or an agent's shell has them.
pub(crate) fn off_terminal() -> Process {
    Process {
        terminal: false,
        ..Process::new()
    }
}

/// A transcript outside every transcript root, named `name`.
fn elsewhere(p: &Process, name: &str) -> std::path::PathBuf {
    let path = p.dir.path().join(name);
    fs::write(&path, SECRET).unwrap();
    path
}

fn named(p: &Process, path: &Path) -> (u8, String, String) {
    p.invoke(&["ingest", "--path", &path.display().to_string()], &DEV)
}

/// The principal acts the home of `p` recorded, as (name, class, content).
fn acts(p: &Process) -> Vec<(String, &'static str, serde_json::Value)> {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();

    acts_in(&home, &store)
        .unwrap()
        .into_iter()
        .map(|a| (a.name, a.class.as_str(), a.content))
        .collect()
}

fn ingested_paths(p: &Process) -> Vec<String> {
    let home = Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap();
    let store = Store::open(&home.store()).unwrap();

    store
        .ingested_runs()
        .unwrap()
        .into_iter()
        .map(|r| r.path)
        .collect()
}

fn refusals(p: &Process) -> Vec<(String, String)> {
    audited(p)
        .into_iter()
        .map(|e| (e.counter.unwrap_or_default(), e.detail))
        .collect()
}

#[test]
fn a_path_the_person_names_at_a_terminal_is_a_widening_act_and_is_ingested() {
    let p = Process::new();
    let path = elsewhere(&p, "elsewhere.jsonl");

    let (code, out, err) = named(&p, &path);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""), "{out}");
    let resolved = fs::canonicalize(&path).unwrap();
    assert_eq!(ingested_paths(&p), [resolved.display().to_string()]);
    let recorded = acts(&p);
    assert_eq!(recorded.len(), 1);
    let (name, class, content) = &recorded[0];
    assert_eq!((name.as_str(), *class), ("ingest_path", "widening"));
    let digest = Digest::of(&[resolved.to_string_lossy().as_bytes()]);
    assert_eq!(content["path_digest"], digest.to_hex());
    assert!(
        !content.to_string().contains("elsewhere.jsonl"),
        "{content}"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), SECRET);
}

#[cfg(unix)]
#[test]
fn a_link_the_person_names_is_resolved_to_its_target() {
    let p = Process::new();
    let target = elsewhere(&p, "elsewhere.jsonl");
    let link = p.dir.path().join("link.jsonl");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let (code, _, err) = named(&p, &link);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    let resolved = fs::canonicalize(&target).unwrap();
    assert_eq!(ingested_paths(&p), [resolved.display().to_string()]);
    let digest = Digest::of(&[resolved.to_string_lossy().as_bytes()]);
    assert_eq!(acts(&p)[0].2["path_digest"], digest.to_hex());
}

#[test]
fn a_directory_the_person_names_is_refused_audited_and_takes_no_act() {
    let p = Process::new();
    let dir = p.dir.path().join("dir");
    fs::create_dir_all(&dir).unwrap();

    let (code, out, err) = named(&p, &dir);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert_eq!(err, "parinn ingest: the named path is not a regular file\n");
    assert_eq!(
        refusals(&p),
        [(
            "principal_act_refused".to_owned(),
            "refused: named path is not a regular file".to_owned()
        )]
    );
    assert!(!p.parinn_home().join("store.db").exists());
}

#[test]
fn a_path_named_without_a_terminal_stays_outside_the_allowed_roots() {
    let p = off_terminal();
    let path = elsewhere(&p, "elsewhere.jsonl");

    let (code, out, err) = named(&p, &path);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert_eq!(err, "parinn ingest: path outside allowed roots\n");
    assert_eq!(
        refusals(&p),
        [(
            "path_outside_allowed_roots".to_owned(),
            "path outside allowed roots".to_owned()
        )]
    );
    assert!(!p.parinn_home().join("store.db").exists());
}

#[test]
fn nothing_in_the_home_and_nothing_missing_is_a_named_transcript() {
    let p = Process::new();
    p.configure("");
    let key = p.parinn_home().join("config.toml");
    let missing = p.dir.path().join("missing.jsonl");

    for path in [key, missing] {
        let (code, _, err) = named(&p, &path);
        assert_eq!(
            (code, err.as_str()),
            (EXIT_FAILURE, "parinn ingest: path outside allowed roots\n")
        );
    }
    assert!(acts(&p).is_empty());
}

#[test]
fn a_path_within_the_roots_takes_no_act() {
    let p = Process::new();

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);

    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert!(acts(&p).is_empty());
}

#[test]
fn a_named_path_after_a_run_waits_for_the_risk_acceptance_like_any_widening_act() {
    let p = Process::new();
    assert_eq!(
        p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV)
            .0,
        0
    );
    let path = elsewhere(&p, "elsewhere.jsonl");

    let (code, _, err) = named(&p, &path);

    assert_eq!(code, EXIT_FAILURE);
    assert!(err.contains("OWN-22"), "{err}");
    assert_eq!(
        refusals(&p).last().unwrap(),
        &(
            "principal_act_refused".to_owned(),
            "refused: residual risks open with no risk acceptance".to_owned()
        )
    );
    assert_eq!(ingested_paths(&p).len(), 1);

    assert_eq!(p.invoke(&["risk-acceptance", "record"], &DEV).0, EXIT_OK);
    let (code, _, err) = named(&p, &path);
    assert_eq!((code, err.as_str()), (EXIT_OK, ""));
    assert_eq!(ingested_paths(&p).len(), 2);
}

#[test]
fn a_refusal_the_audit_log_cannot_take_is_still_said() {
    let p = Process::new();
    let dir = p.dir.path().join("dir");
    fs::create_dir_all(&dir).unwrap();
    named(&p, &dir);
    let audit_dir = p.parinn_home().join("audit");
    let _ = fs::remove_dir_all(&audit_dir);
    fs::write(&audit_dir, "").unwrap();
    set_mode(&audit_dir, 0o600);

    let (code, _, err) = named(&p, &dir);

    assert_eq!(code, EXIT_FAILURE);
    assert!(
        err.contains("parinn ingest: audit a refused named path: "),
        "{err}"
    );
    assert!(
        err.ends_with("parinn ingest: the named path is not a regular file\n"),
        "{err}"
    );
}

#[test]
fn an_act_that_cannot_be_taken_leaves_the_path_unread() {
    let p = Process::new();
    let path = elsewhere(&p, "elsewhere.jsonl");
    p.configure("");
    let store = p.parinn_home().join("store.db");
    fs::create_dir_all(&store).unwrap();

    let (code, out, err) = named(&p, &path);

    assert_eq!((code, out.as_str()), (EXIT_FAILURE, ""));
    assert!(err.starts_with("parinn ingest: "), "{err}");
    assert_eq!(err.matches("parinn ingest: ").count(), 1, "{err}");
}
