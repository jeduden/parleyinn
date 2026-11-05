use record::id::Seq;
use store::{Erase, Store};
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

    assert!(error.is_none());
    let events: Vec<usize> = reports.iter().map(|r| r.events).collect();
    assert!(events.len() > 1, "the source is read in bounded windows");
    assert_eq!(events.iter().sum::<usize>(), 65);
    assert!(reports[0].resumable && !reports.last().unwrap().resumable);
    assert!(reports.iter().all(|r| r.read_window_high_water <= 64 << 20));
}

#[test]
fn a_line_over_the_limit_is_passed_with_its_place_recorded_and_later_lines_are_ingested() {
    for first in [true, false] {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("t.jsonl");
        let big = format!("{}\n", "z".repeat((4 << 20) + 1));
        let text = if first {
            format!("{big}{}", line(2, 100))
        } else {
            format!("{}{big}{}", line(1, 100), line(2, 100))
        };
        std::fs::write(&path, text).unwrap();

        let (reports, error) = ingest(
            &home(&dir),
            &path,
            &Ok(redactor()),
            &ingest::Inputs::default(),
        );

        assert!(error.is_none());
        let named: Vec<_> = reports.iter().filter_map(|r| r.too_large).collect();
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].offset, if first { 0 } else { 100 });
        let events: usize = reports.iter().map(|r| r.events).sum();
        assert_eq!(events, if first { 1 } else { 2 });
    }
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
    assert!(error.unwrap().message.contains("read the transcript"));
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
    assert_eq!(error.unwrap().message, "bad pattern");
}

#[test]
fn a_refused_only_window_resumes_to_valid_later_lines() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let parent = dir.path().join("parent.jsonl");
    let fork = dir.path().join("fork.jsonl");
    let repeated: String = (1..=20_000)
        .map(|n| format!("{{\"uuid\":\"u{n}\",\"sessionId\":\"parent\"}}\n"))
        .collect();
    std::fs::write(&parent, &repeated).unwrap();
    let (owners, failed) = ingest(&home, &parent, &Ok(redactor()), &ingest::Inputs::default());
    assert!(failed.is_none());
    let writer = owners[0].writer;
    let mut store = Store::open(&home.store()).unwrap();
    let device = administration::node::Node::find(&home, &store)
        .unwrap()
        .device_writer();
    store
        .purge(
            device,
            &[],
            &[Erase {
                writer,
                first: Seq::FIRST,
                last: Seq::new(20_000).unwrap(),
            }],
        )
        .unwrap();
    std::fs::write(
        &fork,
        format!("{repeated}{{\"uuid\":\"valid\",\"sessionId\":\"fork\"}}\n"),
    )
    .unwrap();

    let (reports, failed) = ingest(&home, &fork, &Ok(redactor()), &ingest::Inputs::default());

    assert!(failed.is_none());
    assert!(reports.len() > 1);
    assert_eq!(reports[0].events, 0);
    assert!(reports[0].refused > 0 && reports[0].resumable);
    assert_eq!(reports.iter().map(|r| r.refused).sum::<usize>(), 20_000);
    assert_eq!(reports.iter().map(|r| r.events).sum::<usize>(), 1);
}
