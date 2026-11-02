use testkit::TempDir;

use super::*;

fn home(dir: &TempDir) -> administration::home::Home {
    administration::home::Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

fn line(n: usize, len: usize) -> String {
    let head = format!("{{\"n\":{n},\"t\":\"");
    format!("{head}{}\"}}\n", "y".repeat(len - head.len() - 3))
}

#[test]
fn a_source_over_64_mib_is_read_window_by_window_until_it_is_caught_up() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    let text: String = (1..=65).map(|n| line(n, 1 << 20)).collect();
    std::fs::write(&path, &text).unwrap();

    let (reports, error) = ingest(
        &home(&dir),
        &path,
        &Ok(redactor()),
        &ingest::Inputs::default(),
    );

    assert_eq!(error, None);
    let events: Vec<usize> = reports.iter().map(|r| r.events).collect();
    assert_eq!(events, [64, 1]);
    assert!(reports[0].resumable && !reports[1].resumable);
    assert!(reports.iter().all(|r| r.read_window_high_water <= 64 << 20));
}

#[test]
fn a_refused_line_stops_the_reads_at_the_one_report_that_names_it() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");
    std::fs::write(
        &path,
        format!(
            "{}{}\n{}",
            line(1, 100),
            "z".repeat((4 << 20) + 1),
            line(2, 100)
        ),
    )
    .unwrap();

    let (reports, error) = ingest(
        &home(&dir),
        &path,
        &Ok(redactor()),
        &ingest::Inputs::default(),
    );

    assert_eq!(error, None);
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].too_large.map(|r| r.offset), Some(100));
    assert_eq!(reports[0].events, 1);
}

#[test]
fn a_source_that_cannot_be_read_is_the_error_of_its_reads() {
    let dir = TempDir::new().unwrap();

    let (reports, error) = ingest(
        &home(&dir),
        &dir.path().join("none.jsonl"),
        &Ok(redactor()),
        &ingest::Inputs::default(),
    );

    assert!(reports.is_empty());
    assert!(error.unwrap().contains("read the transcript"));
}

#[test]
fn a_redactor_that_could_not_be_made_is_the_error_of_every_source() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.jsonl");

    let (reports, error) = ingest(
        &home(&dir),
        &path,
        &Err("bad pattern".to_owned()),
        &ingest::Inputs::default(),
    );

    assert!(reports.is_empty());
    assert_eq!(error.as_deref(), Some("bad pattern"));
}
