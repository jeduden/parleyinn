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
    assert_eq!(report["read_window_high_water"], 8);
}
