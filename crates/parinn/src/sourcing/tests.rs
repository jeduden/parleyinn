use crate::tests::{DEV, Process, audited};

/// A transcript of one line, then one over the 4 MiB limit, then one.
fn refused_text() -> String {
    format!("{{\"n\":1}}\n{}\n{{\"n\":2}}\n", "z".repeat((4 << 20) + 1))
}

#[test]
fn a_refused_line_is_audited_and_counted_with_its_source_offset_and_length() {
    let p = Process::new();
    let path = p.transcript(&refused_text());

    let (code, _, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!(code, 0, "{err}");
    let entries = audited(&p);
    let entry = entries
        .iter()
        .find(|e| e.counter.as_deref() == Some("transcript_line_too_large"))
        .unwrap();
    assert_eq!(
        (entry.kind.as_str(), entry.operation.as_str()),
        ("rejected", "ingest")
    );
    assert!(entry.detail.starts_with(audit::entry::LINE_TOO_LARGE));
    assert!(
        entry.detail.ends_with("at offset 8, 4194305 bytes"),
        "{}",
        entry.detail
    );
}

#[test]
fn the_ingest_report_names_the_most_it_held_in_memory() {
    let p = Process::new();
    let path = p.transcript("{\"n\":1}\n");

    let (_, out, _) = p.invoke(&["ingest", "--path", &path, "--json"], &DEV);

    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    let held = report["read_window_high_water"].as_u64().unwrap();
    assert!((8..=64 << 20).contains(&held), "{held}");
}

#[test]
fn harness_identifiers_over_the_memory_ceiling_are_refused_audited_and_counted() {
    let p = Process::new();
    let line = |n| {
        let id = format!("{n}-{}", "x".repeat(400 * 1024));
        format!(
            "{{\"attachment\":{{\"type\":\"credential_org\",\"organizationUuid\":\"{id}\"}}}}\n"
        )
    };
    let path = p.transcript(&(1..=6).map(line).collect::<String>());

    let (code, _, err) = p.invoke(&["ingest", "--path", &path], &DEV);

    assert_eq!(code, 1, "{err}");
    let entries = audited(&p);
    let entry = entries
        .iter()
        .find(|e| e.counter.as_deref() == Some("transcript_source_memory_limit"))
        .unwrap();
    assert_eq!(
        (entry.kind.as_str(), entry.operation.as_str()),
        ("rejected", "ingest")
    );
    assert!(entry.detail.contains("source-memory ceiling"));
}

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
