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
        refused_ended: false,
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
fn a_line_over_4_mib_is_stored_unparsed_redacted_whole_its_place_is_recorded_and_ingest_goes_on_past_it()
 {
    let dir = TempDir::new().unwrap();
    let (home, (text, at)) = (home(&dir), refused_text());
    let path = written(&dir, &text);

    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let second = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    let refused = first.too_large.unwrap();
    assert_eq!((refused.offset, refused.length), (at, (4 << 20) + 1));
    assert_eq!(first.too_large_line, Some(3));
    assert_eq!((first.events, second.events, again.events), (3, 1, 0));
    assert_eq!(first.redactions, ["transcript_line_too_large"]);
    assert_eq!((first.unparsed, first.scan_limited), (3, 0));
    assert!(first.resumable && !second.resumable);
    assert_eq!(second.too_large, None);
    let row = source(&home);
    assert_eq!(row.position.offset, text.len() as u64);
    assert_eq!(row.position.lines, 4);
    assert!(row.position.complete());
    // The place is kept with its line break, its bytes zeros in the hash.
    let place = Store::open(&home.store())
        .unwrap()
        .purged_places(row.id)
        .unwrap();
    assert_eq!(
        place
            .iter()
            .map(|p| (p.line, p.offset, p.length))
            .collect::<Vec<_>>(),
        [(3, at, (4 << 20) + 2)]
    );
    assert!(read::open::continues(&path, &row));
    assert_eq!(refused_line(&home, &path).unwrap(), None);
    let texts = placed_texts(&home);
    assert!(
        texts.iter().all(|t| !t.contains("zzzz")),
        "its content is redacted whole"
    );
    assert_eq!(texts.len(), 4);
    assert_eq!(texts[2], "[REDACTED:transcript_line_too_large]");
    // Its event is recoverable by its address, where the line stood.
    let store = Store::open(&home.store()).unwrap();
    let placed = store.placed(row.id).unwrap();
    assert_eq!(placed[2].place.line, 3);
    assert_eq!(placed[2].place.offset, at);
    assert_eq!(placed[2].kind, record::event::EventKind::UnparsedLine);
}

/// The redacted text of each line event of the one source, in order.
fn placed_texts(home: &Home) -> Vec<String> {
    let store = Store::open(&home.store()).unwrap();
    let id = store.sources().unwrap()[0].id;
    store
        .placed(id)
        .unwrap()
        .iter()
        .map(|p| {
            let content = store.event(p.address).unwrap().unwrap().content.unwrap();
            content["text"].as_str().unwrap().to_owned()
        })
        .collect()
}

#[test]
fn a_source_whose_first_line_is_over_the_limit_records_its_place_and_the_lines_after_it() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let big = "z".repeat((4 << 20) + 1);
    let path = written(&dir, &format!("{big}\n{}", line(1, 50)));

    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!(first.events, 1);
    assert_eq!(first.too_large.map(|r| r.offset), Some(0));
    assert_eq!(first.too_large_line, Some(1));
    assert!(first.resumable);
    assert_eq!(source(&home).position.offset, (4 << 20) + 2);
    let second = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    assert_eq!((second.events, second.too_large), (1, None));
    assert_eq!(source(&home).position.lines, 2);
}

#[test]
fn a_refused_line_no_line_follows_ends_the_source_and_asks_for_no_more_read() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text = line(1, 50) + &"z".repeat((4 << 20) + 1) + "\n";
    let path = written(&dir, &text);

    let first = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((first.events, first.too_large_line), (2, Some(2)));
    assert!(!first.resumable);
    assert!(source(&home).position.complete());
}

#[test]
fn every_refused_line_of_a_source_is_passed_in_its_own_read() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let big = "z".repeat((4 << 20) + 1) + "\n";
    let path = written(&dir, &(big.clone() + &line(1, 50) + &big + &line(2, 50)));

    let mut reads = Vec::new();
    loop {
        let read = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
        let more = read.resumable;
        reads.push((read.events, read.too_large_line));
        if !more {
            break;
        }
    }

    assert_eq!(reads, [(1, Some(1)), (2, Some(3)), (1, None)]);
    let row = source(&home);
    assert_eq!((row.position.lines, row.purged.len()), (4, 2));
    assert!(read::open::continues(&path, &row));
}

#[test]
fn a_refused_line_still_being_written_stays_before_the_position_until_its_line_break() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let head = line(1, 50);
    let big = "z".repeat((4 << 20) + 1);
    let path = written(&dir, &(head.clone() + &big));

    let waiting = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert_eq!((waiting.events, waiting.too_large_line), (1, None));
    assert_eq!(waiting.too_large.map(|r| r.offset), Some(head.len() as u64));
    assert!(!waiting.resumable);
    assert_eq!(source(&home).position.offset, head.len() as u64);
    assert_eq!(
        refused_line(&home, &path).unwrap().map(|r| r.length),
        Some((4 << 20) + 1)
    );
    std::fs::write(&path, head + &big + "\n" + &line(2, 50)).unwrap();
    let done = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    assert_eq!((done.events, done.too_large_line), (1, Some(2)));
    assert!(done.resumable);
    let last = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    assert_eq!(last.events, 1);
}

#[test]
fn a_source_over_64_mib_is_ingested_across_resumes_at_its_marker() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let text: String = (1..=65).map(|n| line(n, MIB)).collect();
    let path = written(&dir, &text);

    let mut reports = Vec::new();
    loop {
        let report = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
        let more = report.resumable;
        reports.push(report);
        if !more {
            break;
        }
    }
    let again = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();

    assert!(reports.len() > 1);
    assert_eq!(reports.iter().map(|r| r.events).sum::<usize>(), 65);
    assert!(reports.iter().all(|r| r.read_window_high_water <= 64 * MIB));
    assert_eq!(again.events, 0);
    let done = source(&home);
    assert_eq!(done.position.lines, 65);
    assert_eq!(done.position.offset, text.len() as u64);
    assert_eq!(done.position.prefix, Digest::of(&[text.as_bytes()]));
    let store = Store::open(&home.store()).unwrap();
    let placed = store.placed(done.id).unwrap();
    let numbers: Vec<u64> = placed.iter().map(|p| p.place.line).collect();
    assert_eq!(numbers, (1..=65).collect::<Vec<_>>());
    assert!(
        reports
            .windows(2)
            .all(|pair| pair[0].generation == pair[1].generation)
    );
    assert!(reports.iter().skip(1).all(|r| !r.new_generation));
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
fn structurally_dense_json_is_refused_before_it_can_exceed_the_parse_reserve() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let dense = format!("{{\"items\":[{}]}}\n", "0,".repeat(40_000) + "0");
    let path = written(&dir, &dense);

    let error = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(error, IngestError::SourceMemoryLimit(p) if p == path));
    assert!(!home.store().exists());
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
        refused_ended: false,
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
fn a_live_ingest_reads_a_big_source_a_window_at_a_time() {
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
        Some(first.events),
        &mut OsEntropy,
    )
    .unwrap();
    let mut reports = vec![first];
    while source(&home).position.lines < 65 {
        reports.push(
            ingest_live(
                &home,
                &path,
                &redactor(),
                &Inputs::default(),
                &mut seats,
                Some(65),
                &mut OsEntropy,
            )
            .unwrap()
            .unwrap(),
        );
    }
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

    assert!(limited.is_none());
    assert_eq!(reports.iter().map(|r| r.events).sum::<usize>(), 65);
    assert!(reports[0].resumable && !reports.last().unwrap().resumable);
    assert!(done.is_none());
    assert_eq!(source(&home).position.lines, 65);
}

#[test]
fn a_live_ingest_names_a_refused_line_records_its_place_and_goes_on_past_it() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let (text, at) = refused_text();
    let path = written(&dir, &text);
    let mut seats = RunSeats::default();
    let mut live = |limit| {
        ingest_live(
            &home,
            &path,
            &redactor(),
            &Inputs::default(),
            &mut seats,
            limit,
            &mut OsEntropy,
        )
        .unwrap()
    };

    // Held short of the refused line by the limit, it is not yet passed.
    let short = live(Some(2)).unwrap();
    let level = live(None).unwrap();
    let rest = live(None).unwrap();
    let none = live(None);

    assert_eq!((short.events, short.too_large_line), (2, None));
    assert_eq!(short.too_large.map(|r| r.offset), Some(at));
    assert_eq!((level.events, level.too_large_line), (1, Some(3)));
    assert_eq!(rest.events, 1);
    assert!(none.is_none());
    assert_eq!(refused_line(&home, &path).unwrap(), None);
    assert_eq!(
        refused_line(&home, &dir.path().join("none.jsonl")).unwrap(),
        None
    );
}

#[test]
fn a_refused_line_of_a_live_source_that_cannot_be_read_is_a_failure_not_none() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let (text, _) = refused_text();
    let path = written(&dir, &text);
    ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap();
    std::fs::remove_file(&path).unwrap();

    let failed = refused_line(&home, &path).unwrap_err();

    assert!(matches!(failed, IngestError::Read { .. }), "{failed}");
}

/// Sixty-five 1 MiB lines, so a source read in more than one window.
fn filler() -> String {
    (1..=65).map(|n| line(n, MIB)).collect()
}

#[test]
fn identifiers_past_the_first_window_of_a_long_source_are_refused_at_the_ceiling() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut text = filler();
    text.extend((1..=10).map(|n| organization_line(n, 400 * 1024)));
    let path = written(&dir, &text);

    let error = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(error, IngestError::IdentifierLimit(p) if p == path));
}

#[test]
fn dense_json_past_the_first_window_of_a_long_source_is_refused() {
    let dir = TempDir::new().unwrap();
    let home = home(&dir);
    let mut text = filler();
    text.push_str(&format!("{{\"items\":[{}]}}\n", "0,".repeat(40_000) + "0"));
    let path = written(&dir, &text);

    let error = ingest_path(&home, &path, &redactor(), &mut OsEntropy).unwrap_err();

    assert!(matches!(error, IngestError::SourceMemoryLimit(p) if p == path));
}

#[test]
fn the_ceiling_errors_say_which_ceiling_the_transcript_passed() {
    let path = std::path::PathBuf::from("t.jsonl");

    assert_eq!(
        IngestError::IdentifierLimit(path.clone()).to_string(),
        "the transcript t.jsonl names too many harness account identifiers"
    );
    assert_eq!(
        IngestError::SourceMemoryLimit(path).to_string(),
        "the transcript t.jsonl has a line whose parsed structure exceeds the source-memory ceiling"
    );
}
