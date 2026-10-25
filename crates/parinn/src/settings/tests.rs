use std::path::PathBuf;

use administration::act::{Class, Recorded};
use audit::entry::Occurrence;
use configuration::layer::parse;
use configuration::source::{MANAGED_POLICY, OsUser};
use record::id::Seq;
use serde_json::json;

use super::*;
use crate::tests::{DEV, Process, audited};
use crate::{EXIT_FAILURE, EXIT_OK, EXIT_USAGE};

fn accept(settings: serde_json::Value) -> Recorded {
    Recorded {
        seq: Seq::FIRST,
        name: "configuration_accept".to_owned(),
        class: Class::Widening,
        content: json!({"act": "configuration_accept", "settings": settings}),
    }
}

/// The details of every audit entry naming `counter`.
fn details(p: &Process, counter: &str) -> Vec<String> {
    audited(p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .map(|e| e.detail)
        .collect()
}

#[test]
fn accepted_reads_the_settings_the_last_acceptance_recorded() {
    assert!(accepted(&[]).is_empty());
    let acts = [accept(json!({
        "recall.max_k": 10,
        "nonsense.key": 1,
        "flag.enabled": {"not": "a value"},
    }))];
    let got = accepted(&acts);
    assert_eq!(got.len(), 1);
    assert_eq!(
        got.get(&key("recall.max_k").unwrap()),
        Some(&Value::Integer(10))
    );
}

#[test]
fn recorded_keeps_every_key_but_a_trust_policy_one() {
    let layer = parse("recall.max_k = 10\ntrust_policy.web = 'trusted'\n").unwrap();
    assert_eq!(recorded(&layer), json!({"recall.max_k": 10}));
}

#[test]
fn managed_policy_is_the_documented_path_unless_a_development_build_overrides_it() {
    assert_eq!(managed_policy(None), PathBuf::from(MANAGED_POLICY));
    assert_eq!(
        managed_policy(Some("".into())),
        PathBuf::from(MANAGED_POLICY)
    );
    assert_eq!(
        managed_policy(Some("/tmp/p.toml".into())),
        PathBuf::from("/tmp/p.toml")
    );
}

#[test]
fn every_failed_layer_and_ignored_key_has_its_occurrence() {
    let p = Process::new();
    p.configure("recal.max_k = 1\n");
    p.repository("node.deployment_mode = 'interactive'\ntrust_policy.web = 'trusted'\n");
    p.manage("trust_policy.\"tool_result:Bash\" = 'trusted'\nnonsense = 1\n");
    let mut q = Process::new();
    q.manage("trust_policy.\"mcp:github\" = 'trusted'\n");
    q.configure("trust_policy.file = 'trusted'\n");
    q.os_user = OsUser::Known(u32::MAX);

    let (code, _, err) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    q.invoke(&["ingest", "--path", &q.transcript("a\n")], &DEV);

    assert_eq!(code, EXIT_FAILURE, "{err}");
    assert!(err.contains("parinn ingest: configuration "), "{err}");
    assert!(
        err.contains("parinn ingest: recording without managed policy "),
        "{err}"
    );
    assert_eq!(
        details(&p, "settings_layer_invalid"),
        ["configuration failed"]
    );
    assert_eq!(details(&p, "managed_policy_invalid").len(), 1);
    assert_eq!(
        details(&p, "repository_configuration_key_ignored"),
        ["rejected repository setting node.deployment_mode"]
    );
    assert_eq!(
        details(&p, "trust_policy_key_ignored"),
        ["repository configuration: rejected trust override for web"]
    );
    assert_eq!(
        details(&q, "trust_policy_key_ignored"),
        [
            "configuration: rejected trust override for file",
            "managed policy: rejected trust override for mcp:github",
        ]
    );
}

#[test]
fn an_invalid_repository_configuration_has_its_own_detail() {
    let p = Process::new();
    p.repository("recall.max_k = 'ten'\n");
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(
        details(&p, "settings_layer_invalid"),
        ["repository configuration failed"]
    );
    let (code, _, err) = p.invoke(&["status"], &DEV);
    assert_eq!(code, EXIT_USAGE);
    assert!(
        err.ends_with(".parinn.toml: key \"recall.max_k\": type error: want an integer\n"),
        "{err}"
    );
}

#[test]
fn occurrences_name_the_operation() {
    let p = Process::new();
    p.repository("recall.max_k = 99\n");
    let home = p_home(&p);
    let f = in_force(&ctx_of(&p), &home, &[]);
    let all = occurrences("hook UserPromptSubmit", &f);
    assert!(matches!(
        all.as_slice(),
        [Occurrence::Keyed {
            operation: "hook UserPromptSubmit",
            key: "recall.max_k",
            ..
        }]
    ));
}

#[test]
fn ingest_says_what_it_cannot_read_or_audit_and_still_records() {
    let p = Process::new();
    p.repository("recall.max_k = 99\n");
    let (code, _, _) = p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    assert_eq!(code, EXIT_OK);
    // The device key gone, the accepted configuration cannot be read.
    std::fs::remove_file(p.parinn_home().join("keys/device.key")).unwrap();
    // An audit log that takes no entry.
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap();
    std::fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);

    let (_, _, err) = p.invoke(&["ingest", "--path", &p.transcript("b\n")], &DEV);

    assert!(
        err.contains("parinn ingest: read the accepted configuration: "),
        "{err}"
    );
    assert!(err.contains("parinn ingest: audit the settings: "), "{err}");
}

#[test]
fn sources_find_the_repository_configuration_from_the_working_directory() {
    let p = Process::new();
    let home = p_home(&p);
    let ctx = ctx_of(&p);
    let s = sources(&ctx, &home);
    assert_eq!(s.configuration, p.parinn_home().join("config.toml"));
    assert_eq!(s.repository, None);
    assert_eq!(s.running, OsUser::Known(u32::MAX));
    p.repository("");
    assert_eq!(
        sources(&ctx, &home).repository,
        Some(p.dir.path().join(".parinn.toml"))
    );
}

fn p_home(p: &Process) -> Home {
    Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap()
}

/// A context for `p`'s directory, as `Process::invoke` makes one.
pub(crate) fn ctx_of(p: &Process) -> Context<'static> {
    let input: &'static dyn Fn() -> std::sync::mpsc::Receiver<std::io::Result<String>> =
        Box::leak(Box::new(|| std::sync::mpsc::channel().1));
    Context {
        build: &DEV,
        parinn_home: Some(p.parinn_home().into_os_string()),
        home: None,
        home_id: None,
        hook_panics: false,
        hook_hangs: false,
        terminal: true,
        working_dir: Some(p.dir.path().to_path_buf()),
        managed_policy: p.dir.path().join("managed").join("managed_policy.toml"),
        node_identity: None,
        os_user: OsUser::Known(u32::MAX),
        input,
        clock: &|| 0,
        attended: false,
        hook_budget: None,
        ancestry: || None,
        live_limit: None,
        query_deadline: None,
    }
}

#[test]
fn recorded_acts_are_none_while_the_first_start_is_unfinished() {
    let p = Process::new();
    let home = p_home(&p);
    home.create().unwrap();
    drop(Store::open(&home.store()).unwrap());
    assert_eq!(recorded_acts(&home), Ok(Vec::new()));
    assert_eq!(p.invoke(&["status"], &DEV).0, EXIT_OK);
    assert_eq!(p.invoke(&["doctor"], &DEV).0, EXIT_OK);

    // The genesis recorded, its device key not yet placed.
    let q = Process::new();
    q.invoke(&["ingest", "--path", &q.transcript("a\n")], &DEV);
    std::fs::remove_file(q.parinn_home().join("keys/device.key")).unwrap();
    std::fs::write(p_home(&q).staged(), "change").unwrap();
    assert_eq!(recorded_acts(&p_home(&q)), Ok(Vec::new()));
}

#[test]
fn recorded_acts_reports_a_store_whose_node_is_gone() {
    let p = Process::new();
    p.invoke(&["ingest", "--path", &p.transcript("a\n")], &DEV);
    std::fs::remove_file(p.parinn_home().join("keys/device.key")).unwrap();
    assert!(recorded_acts(&p_home(&p)).is_err());
    let (code, _, err) = p.invoke(&["status"], &DEV);
    assert_eq!(code, crate::EXIT_FAILURE);
    assert!(err.starts_with("parinn status: "), "{err}");
    let (code, _, _) = p.invoke(&["doctor"], &DEV);
    assert_eq!(code, crate::EXIT_FAILURE);
}
