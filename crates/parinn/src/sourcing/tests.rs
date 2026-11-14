use crate::tests::{DEV, Process, audited};

/// What an ingest of a one-line transcript found.
fn ingested(p: &Process) -> ingest::Ingested {
    let path = p.transcript("{\"n\":1}\n");

    ingest::ingest_path(
        &administration::home::Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap(),
        std::path::Path::new(&path),
        &redaction::Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap()
}

/// How many audit entries of `p` count `counter`.
fn counted(p: &Process, counter: &str) -> usize {
    audited(p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some(counter))
        .count()
}

#[test]
fn each_finding_of_a_transcript_source_is_audited_once() {
    let p = Process::new();
    let mut found = ingested(&p);
    found.new_generation = true;
    found.conflicts = 2;
    found.owners_chosen = 1;
    found.hold_started = true;
    found.released = 3;

    p.with_context(&DEV, |ctx| super::audit(ctx, &found, &mut Vec::new()));

    assert_eq!(counted(&p, "new_transcript_generation"), 1);
    assert_eq!(counted(&p, "repeat_conflicts"), 2);
    assert_eq!(counted(&p, "repeat_owner_chosen"), 1);
    assert_eq!(counted(&p, "repeat_held"), 1);
    assert_eq!(counted(&p, "repeat_released"), 3);
}

#[test]
fn a_finding_the_audit_log_cannot_take_is_said_on_stderr() {
    let p = Process::new();
    let mut found = ingested(&p);
    found.conflicts = 1;
    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap_or(());
    std::fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);
    let mut stderr = Vec::new();

    p.with_context(&DEV, |ctx| super::audit(ctx, &found, &mut stderr));

    let text = String::from_utf8(stderr).unwrap();
    assert!(
        text.contains("parinn ingest: audit the transcript source: "),
        "{text}"
    );
}

#[test]
fn each_entry_says_its_kind_and_names_the_source() {
    let p = Process::new();
    let mut found = ingested(&p);
    found.new_generation = true;
    found.conflicts = 1;
    found.owners_chosen = 1;
    found.hold_started = true;
    found.released = 1;
    let name = found.name.clone();

    p.with_context(&DEV, |ctx| super::audit(ctx, &found, &mut Vec::new()));

    let said: Vec<(String, String)> = audited(&p)
        .into_iter()
        .filter(|e| e.operation == "ingest" && e.detail.ends_with(&name))
        .filter_map(|e| Some((e.counter?, e.kind)))
        .collect();
    let expected = [
        ("new_transcript_generation", "truncated"),
        ("repeat_conflicts", "rejected"),
        ("repeat_owner_chosen", "coalesced"),
        ("repeat_held", "truncated"),
        ("repeat_released", "timed-out"),
    ];
    for (counter, kind) in expected {
        assert!(
            said.contains(&(counter.to_owned(), kind.to_owned())),
            "{counter} as {kind}: {said:?}"
        );
    }
}
