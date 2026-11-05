use crate::tests::{DEV, Process, audited};

#[test]
fn a_sibling_transcript_an_ingest_could_not_read_is_audited_and_counted() {
    let p = Process::new();
    let path = p.transcript("{\"n\":1}\n");
    let mut ingested = ingest::ingest_path(
        &administration::home::Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap(),
        std::path::Path::new(&path),
        &redaction::Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap();
    ingested.unread_sources = vec![ingest::UnreadSource {
        name: "sibling".to_owned(),
        why: "not a regular file".to_owned(),
    }];

    p.with_context(&DEV, |ctx| super::audit(ctx, &ingested, &mut Vec::new()));

    let unread = audited(&p)
        .into_iter()
        .filter(|e| e.counter.as_deref() == Some("ingest_source_unread"))
        .count();
    assert_eq!(unread, 1);
}

#[test]
fn a_sibling_transcript_the_audit_log_cannot_take_is_said_on_stderr() {
    let p = Process::new();
    let path = p.transcript("{\"n\":1}\n");
    let mut ingested = ingest::ingest_path(
        &administration::home::Home::locate(Some(p.parinn_home().as_os_str()), None).unwrap(),
        std::path::Path::new(&path),
        &redaction::Redactor::new(&[]).unwrap().0,
        &mut key::random::OsEntropy,
    )
    .unwrap();
    ingested.unread_sources = vec![ingest::UnreadSource {
        name: "sibling".to_owned(),
        why: "not a regular file".to_owned(),
    }];

    let audit_dir = p.parinn_home().join("audit");
    std::fs::remove_dir_all(&audit_dir).unwrap_or(());
    std::fs::write(&audit_dir, "").unwrap();
    crate::tests::set_mode(&audit_dir, 0o600);
    let mut stderr = Vec::new();

    p.with_context(&DEV, |ctx| super::audit(ctx, &ingested, &mut stderr));

    let text = String::from_utf8(stderr).unwrap();
    assert!(
        text.contains("parinn ingest: audit the transcript source: "),
        "{text}"
    );
}
