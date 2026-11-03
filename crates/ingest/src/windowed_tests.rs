//! SEC-05 and REC-13: a transcript source is read in windows within the
//! ceilings, a line over 4 MiB is refused and the source left short at it.

use key::random::OsEntropy;
use record::digest::Digest;
use store::Store;
use testkit::TempDir;

use super::*;
use crate::live::{RunSeats, ingest_live, refused_line};

const MIB: usize = 1 << 20;

fn home(dir: &TempDir) -> Home {
    Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap()
}

fn redactor() -> Redactor {
    Redactor::new(&[]).unwrap().0
}

/// A JSON line of `len` bytes with its line break: `n` names it.
fn line(n: usize, len: usize) -> String {
    let head = format!("{{\"n\":{n},\"t\":\"");
    format!("{head}{}\"}}\n", "y".repeat(len - head.len() - 3))
}

/// A harness organization attachment line of exactly `len` bytes.
fn organization_line(n: usize, len: usize) -> String {
    let head =
        format!("{{\"attachment\":{{\"type\":\"credential_org\",\"organizationUuid\":\"{n}-");
    format!("{head}{}\"}}}}\n", "o".repeat(len - head.len() - 4))
}

/// The line count and position of the one source in the store.
fn source(home: &Home) -> store::Source {
    let store = Store::open(&home.store()).unwrap();
    store.sources().unwrap().remove(0)
}

fn written(dir: &TempDir, text: &str) -> std::path::PathBuf {
    let path = dir.path().join("t.jsonl");
    std::fs::write(&path, text).unwrap();
    path
}

/// The whole source `text` as the window an ingest reads.
pub(crate) fn whole(text: &str) -> reading::View<'_> {
    reading::View {
        text,
        base: 0,
        seen: text.len() as u64,
        refused: None,
        resumable: false,
        high_water: text.len(),
    }
}

fn written_as(dir: &TempDir, name: &str, text: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// Three lines, a line over the limit, then one more.
fn refused_text() -> (String, u64) {
    let mut text = line(1, 100) + &line(2, 100);
    let at = text.len() as u64;
    text.push_str(&"z".repeat((4 << 20) + 1));
    text.push('\n');
    text.push_str(&line(3, 100));

    (text, at)
}

#[test]
fn a_line_over_4_mib_is_refused_and_the_source_left_short_at_it() {
    let dir = TempDir::new().unwrap();
    let (home, (text, at)) = (home(&dir), refused_text());
    let path = written(&dir, &text);

    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    let refused = first.too_large.unwrap();
    assert_eq!((refused.offset, refused.length), (at, (4 << 20) + 1));
    assert_eq!((first.events, again.events), (2, 0));
    assert_eq!(again.too_large, first.too_large);
    assert!(!first.resumable);
    let row = source(&home);
    assert_eq!((row.position.offset, row.position.lines), (at, 2));
    assert_eq!(row.position.seen, text.len() as u64);
    assert!(!row.position.complete());
    assert_eq!(refused_line(&home, &path), first.too_large);
}

#[test]
fn a_source_whose_first_line_is_over_the_limit_records_nothing_and_is_no_empty_one() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let path = written(
        &dir,
        &format!("{}\n{}", "z".repeat((4 << 20) + 1), line(1, 50)),
    );

    let got = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(got.events, 0);
    assert_eq!(got.too_large.map(|r| r.offset), Some(0));
    assert_eq!(source(&home).position.offset, 0);
}

#[test]
fn a_source_over_64_mib_is_ingested_across_resumes_at_its_marker() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text: String = (1..=65).map(|n| line(n, MIB)).collect();
    let path = written(&dir, &text);

    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let marker = source(&home);
    let second = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let third = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((first.events, first.resumable), (54, true));
    assert!(first.read_window_high_water <= 64 * MIB);
    assert_eq!(marker.position.lines, 54);
    assert_eq!((second.events, second.resumable), (11, false));
    assert_eq!(third.events, 0);
    let done = source(&home);
    assert_eq!(done.position.lines, 65);
    assert_eq!(done.position.offset, text.len() as u64);
    assert_eq!(done.position.prefix, Digest::of(&[text.as_bytes()]));
    let store = Store::open(&home.store()).unwrap();
    let placed = store.placed(done.id).unwrap();
    let numbers: Vec<u64> = placed.iter().map(|p| p.place.line).collect();
    assert_eq!(numbers, (1..=65).collect::<Vec<_>>());
    assert_eq!(second.generation, first.generation);
    assert!(!second.new_generation);
}

#[test]
fn identifiers_from_a_long_source_cannot_grow_past_its_memory_ceiling() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut text: String = (1..=10).map(|n| organization_line(n, 400 * 1024)).collect();
    text.extend((1..=64).map(|n| line(n, MIB)));
    let path = written(&dir, &text);

    let error = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(error, IngestError::IdentifierLimit(p) if p == path));
    assert!(
        !home.store().exists(),
        "no source was recorded over the ceiling"
    );
}

#[test]
fn a_big_source_replaced_after_its_first_window_starts_a_new_generation() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text: String = (1..=65).map(|n| line(n, MIB)).collect();
    let path = written(&dir, &text);
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let other: String = (101..=165).map(|n| line(n, MIB)).collect();
    std::fs::write(&path, &other).unwrap();

    let got = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert!(got.new_generation);
    assert_eq!((got.generation, source(&home).generation), (2, 2));
}

#[test]
fn a_window_whose_marker_another_ingest_moved_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text: String = (1..=65).map(|n| line(n, MIB)).collect();
    let path = written(&dir, &text);
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let moved = reading::peek(&home, &path).unwrap().unwrap();
    let opened = read::open::open(&path, Some(&moved)).unwrap();
    // The ingest that began at the marker finds the position elsewhere.
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let body = String::from_utf8(opened.window.bytes).unwrap();
    let view = reading::View {
        text: &body,
        base: opened.window.start,
        seen: opened.seen,
        refused: None,
        resumable: false,
        high_water: body.len(),
    };

    let unmarked = written_as(&dir, "other.jsonl", "{\"n\":1}\n");
    let stale = reading::ingest(
        &home,
        &path,
        &view,
        &redactor(),
        &Inputs::default(),
        &mut reading::Recorder::Cli,
        &mut OsEntropy,
    )
    .unwrap_err();
    let no_row = reading::ingest(
        &home,
        &unmarked,
        &view,
        &redactor(),
        &Inputs::default(),
        &mut reading::Recorder::Cli,
        &mut OsEntropy,
    )
    .unwrap_err();

    assert!(stale.position_moved(), "{stale}");
    assert!(no_row.position_moved(), "{no_row}");
}

#[test]
fn a_live_ingest_reads_a_big_source_a_window_at_a_time_and_stops_at_a_refused_line() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut seats = RunSeats::default();
    let text: String = (1..=65).map(|n| line(n, MIB)).collect();
    let path = written(&dir, &text);

    let first = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        None,
        &mut OsEntropy,
    )
    .unwrap()
    .unwrap();
    let limited = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        Some(54),
        &mut OsEntropy,
    )
    .unwrap();
    let last = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        Some(65),
        &mut OsEntropy,
    )
    .unwrap()
    .unwrap();
    let done = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        None,
        &mut OsEntropy,
    )
    .unwrap();

    assert_eq!((first.events, last.events), (54, 11));
    assert!(limited.is_none());
    assert!(first.resumable && !last.resumable);
    assert!(done.is_none());
    assert_eq!(source(&home).position.lines, 65);
}

#[test]
fn a_live_ingest_stops_at_a_refused_line_and_names_it_each_read() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let (text, at) = refused_text();
    let path = written(&dir, &text);
    let mut seats = RunSeats::default();
    let some = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        None,
        &mut OsEntropy,
    )
    .unwrap()
    .unwrap();
    let none = ingest_live(
        &home,
        &path,
        &redactor(),
        &Inputs::default(),
        &mut seats,
        None,
        &mut OsEntropy,
    )
    .unwrap();

    assert_eq!(some.too_large.map(|r| r.offset), Some(at));
    assert!(none.is_none());
    assert_eq!(refused_line(&home, &path).map(|r| r.offset), Some(at));
    assert_eq!(refused_line(&home, &dir.path().join("none.jsonl")), None);
}
