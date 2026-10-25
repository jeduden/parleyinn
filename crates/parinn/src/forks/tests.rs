use audit::entry::Entry;
use audit::log::Log;
use record::digest::Digest;
use record::id::WriterId;
use testkit::TempDir;

use super::*;

const NOW: u64 = 1_760_000_000;

fn writer(n: u8) -> WriterId {
    WriterId::parse(&Digest::from_bytes([n; 32]).to_hex()).unwrap()
}

fn seq(n: u64) -> record::id::Seq {
    record::id::Seq::new(n).unwrap()
}

fn fork(first: u64, last: u64) -> Fork {
    Fork {
        writer: writer(1),
        first: seq(first),
        last: seq(last),
        held: Digest::of(&[b"held"]),
        offered: Digest::of(&[b"offered"]),
    }
}

/// A home holding `events` events of writer 1, and a copy of that log
/// that disagrees from seq `from` on.
pub(crate) fn home_and_a_disagreeing_copy(
    dir: &TempDir,
    events: u64,
    from: u64,
) -> (Home, Vec<Link>) {
    use record::commitment::CommitmentKey;
    use record::event::{EventKind, Provenance};
    use serde_json::json;
    use store::NewEvent;

    let home = Home::locate(Some(dir.path().join(".parinn").as_os_str()), None).unwrap();
    home.create().unwrap();
    let mut store = Store::open(&home.store()).unwrap();
    let new: Vec<NewEvent> = (1..=events)
        .map(|n| NewEvent {
            kind: EventKind::UnparsedLine,
            provenance: Provenance::Unparsed,
            content: json!({"text": format!("line {n}")}),
            commitment_key: CommitmentKey::from_bytes([n as u8; 32]),
            transcript_timestamp: None,
        })
        .collect();
    store.append(writer(1), &new).unwrap();
    let mut copy: Vec<Link> = store
        .log(writer(1))
        .unwrap()
        .into_iter()
        .map(|s| s.link)
        .collect();
    for link in copy.iter_mut().skip(from as usize - 1) {
        link.hash = Digest::of(&[b"another history", &link.hash.to_hex().into_bytes()]);
    }

    (home, copy)
}

fn entries(home: &Home) -> Vec<Entry> {
    Log::new(home.audit()).entries().unwrap()
}

#[test]
fn a_fork_is_refused_counted_and_audited_naming_the_writer_and_range() {
    let dir = TempDir::new().unwrap();
    let (home, copy) = home_and_a_disagreeing_copy(&dir, 3, 2);

    let audit = |o: &Occurrence| Log::new(home.audit()).append(o, NOW).is_ok();
    let refused = refuse(&home, &copy, "restore", &mut { audit }).unwrap();

    assert_eq!(refused.forks.len(), 1);
    assert_eq!(refused.unaudited, 0);
    let seen = entries(&home);
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].kind, "rejected");
    assert_eq!(seen[0].counter.as_deref(), Some("fork_conflict"));
    assert_eq!(seen[0].operation, "restore");
    assert_eq!(
        seen[0].detail,
        format!(
            "refused a second event at one seq of writer {}:2-3",
            writer(1)
        )
    );
    let held = Store::open(&home.store()).unwrap();
    assert_eq!(held.refused_forks().unwrap(), refused.forks);
}

#[test]
fn a_fork_refused_again_is_not_audited_again() {
    let dir = TempDir::new().unwrap();
    let (home, copy) = home_and_a_disagreeing_copy(&dir, 2, 2);
    let audit = |o: &Occurrence| Log::new(home.audit()).append(o, NOW).is_ok();
    refuse(&home, &copy, "restore", &mut { audit }).unwrap();

    let again = refuse(&home, &copy, "restore", &mut { audit }).unwrap();

    assert_eq!(again.forks, vec![]);
    assert_eq!(entries(&home).len(), 1);
}

#[test]
fn a_fork_whose_entry_the_audit_log_refused_is_counted_for_the_caller_to_say() {
    let dir = TempDir::new().unwrap();
    let (home, copy) = home_and_a_disagreeing_copy(&dir, 2, 1);

    let refused = refuse(&home, &copy, "import", &mut |_| false).unwrap();

    assert_eq!((refused.forks.len(), refused.unaudited), (1, 1));
    let store = Store::open(&home.store()).unwrap();
    assert_eq!(store.refused_forks().unwrap(), refused.forks);
}

#[test]
fn the_same_history_refuses_nothing() {
    let dir = TempDir::new().unwrap();
    let (home, _) = home_and_a_disagreeing_copy(&dir, 2, 3);
    let same = Store::open(&home.store())
        .unwrap()
        .log(writer(1))
        .unwrap()
        .into_iter()
        .map(|s| s.link)
        .collect::<Vec<_>>();

    let refused = refuse(&home, &same, "restore", &mut |_| false).unwrap();

    assert_eq!((refused.forks, refused.unaudited), (vec![], 0));
}

#[test]
fn a_refusal_reads_as_json_and_lines_with_digests_never_content() {
    let forks = [fork(2, 2), fork(3, 5)];

    assert_eq!(
        json(&forks),
        serde_json::json!([
            {
                "writer": writer(1).to_string(),
                "first": 2,
                "last": 2,
                "held": forks[0].held.to_hex(),
                "offered": forks[0].offered.to_hex(),
            },
            {
                "writer": writer(1).to_string(),
                "first": 3,
                "last": 5,
                "held": forks[1].held.to_hex(),
                "offered": forks[1].offered.to_hex(),
            }
        ])
    );
    assert_eq!(
        lines(&forks),
        vec![
            format!("refused: {}", forks[0]),
            format!("refused: {}", forks[1])
        ]
    );
}
