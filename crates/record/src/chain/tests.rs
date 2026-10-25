use super::*;
use crate::commitment::Commitment;
use crate::event::{EventKind, Provenance};

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn seq(n: u64) -> Seq {
    Seq::new(n).unwrap()
}

/// Appends one event to `log`, citing `heads`, chained to the log's
/// last event.
fn append(log: &mut Vec<Link>, w: WriterId, heads: Vec<SeenHead>) {
    let prev_hash = log.last().map_or(Digest::ZERO, |l| l.hash);
    let header = EventHeader {
        address: Address::new(w, seq(log.len() as u64 + 1)),
        kind: EventKind::UnparsedLine,
        provenance: Provenance::Unparsed,
        prev_hash,
        heads,
        commitment: Commitment::from_digest(Digest::of(&[&[log.len() as u8]])),
    };
    let hash = header.hash();
    log.push(Link { header, hash });
}

fn head_of(log: &[Link]) -> SeenHead {
    let last = log.last().unwrap();
    SeenHead::new(last.header.address, last.hash)
}

/// Two writers, a then b, taking turns, each citing the other's newest
/// event from its second turn on.
fn two_writers(turns: usize) -> BTreeMap<WriterId, Vec<Link>> {
    let (a, b) = (writer(1), writer(2));
    let (mut la, mut lb) = (Vec::new(), Vec::new());
    for _ in 0..turns {
        let cite_b = lb.last().map(|_| head_of(&lb)).into_iter().collect();
        append(&mut la, a, cite_b);
        let cite_a = vec![head_of(&la)];
        append(&mut lb, b, cite_a);
    }
    BTreeMap::from([(a, la), (b, lb)])
}

#[test]
fn an_intact_writer_checks() {
    let record = two_writers(3);

    for (w, links) in &record {
        assert_eq!(check_writer(*w, links), Ok(()));
    }
    assert_eq!(check_heads(&record), Ok(()));
    assert_eq!(check_writer(writer(9), &[]), Ok(()));
}

#[test]
fn a_changed_header_byte_breaks_the_chain_at_that_event() {
    let mut record = two_writers(3);
    let a = writer(1);
    let log = record.get_mut(&a).unwrap();
    log[1].header.commitment = Commitment::from_digest(Digest::ZERO);

    let err = check_writer(a, log).unwrap_err();

    assert_eq!(err, ChainError::new(Address::new(a, seq(2)), Fault::Hash));
    assert_eq!(
        err.fault.to_string(),
        "the stored hash does not match the event's header"
    );
}

#[test]
fn a_gap_a_foreign_event_or_a_broken_link_is_named() {
    let a = writer(1);
    let base = two_writers(3).remove(&a).unwrap();

    let mut gap = base.clone();
    gap.remove(1);
    assert_eq!(
        check_writer(a, &gap),
        Err(ChainError::new(Address::new(a, seq(3)), Fault::Seq(2)))
    );

    let foreign = two_writers(3).remove(&writer(2)).unwrap();
    assert_eq!(
        check_writer(a, &foreign),
        Err(ChainError::new(
            Address::new(writer(2), seq(1)),
            Fault::Writer
        ))
    );

    let mut relinked = base.clone();
    relinked[2].header.prev_hash = Digest::ZERO;
    relinked[2].hash = relinked[2].header.hash();
    assert_eq!(
        check_writer(a, &relinked),
        Err(ChainError::new(Address::new(a, seq(3)), Fault::PrevHash))
    );
}

#[test]
fn a_cited_head_must_name_an_event_of_another_writer_by_its_hash() {
    let (a, b) = (writer(1), writer(2));

    let mut wrong = two_writers(2);
    wrong.get_mut(&b).unwrap()[0].header.heads[0].head = Digest::ZERO;
    let cited = Address::new(a, seq(1));
    assert_eq!(
        check_heads(&wrong),
        Err(ChainError::new(Address::new(b, seq(1)), Fault::Head(cited)))
    );

    let mut missing = two_writers(2);
    missing.get_mut(&a).unwrap().clear();
    assert_eq!(
        check_heads(&missing),
        Err(ChainError::new(Address::new(b, seq(1)), Fault::Head(cited)))
    );

    let mut own = two_writers(2);
    own.get_mut(&a).unwrap()[1].header.heads[0].address.writer = a;
    assert_eq!(
        check_heads(&own),
        Err(ChainError::new(
            Address::new(a, seq(2)),
            Fault::Head(Address::new(a, seq(1)))
        ))
    );
}

#[test]
fn a_cited_head_never_moves_backward_along_a_writer() {
    let (a, b) = (writer(1), writer(2));
    let mut record = two_writers(3);
    let older = SeenHead::new(Address::new(b, seq(1)), record[&b][0].hash);
    record.get_mut(&a).unwrap()[2].header.heads = vec![older];

    assert_eq!(
        check_heads(&record),
        Err(ChainError::new(
            Address::new(a, seq(3)),
            Fault::HeadBackward(Address::new(b, seq(1)))
        ))
    );
}

#[test]
fn causal_order_follows_each_chain_and_the_heads_it_cites() {
    let (a, b) = (writer(1), writer(2));
    let record = two_writers(2);

    let want = vec![
        Address::new(a, seq(1)),
        Address::new(b, seq(1)),
        Address::new(a, seq(2)),
        Address::new(b, seq(2)),
    ];
    assert_eq!(causal_order(&record), Ok(want));
}

#[test]
fn causal_order_breaks_ties_by_writer_id_then_seq() {
    let (a, b) = (writer(1), writer(2));
    let (mut la, mut lb) = (Vec::new(), Vec::new());
    append(&mut la, a, vec![]);
    append(&mut lb, b, vec![]);
    let record = BTreeMap::from([(a, la), (b, lb)]);
    let (first, second) = if a < b { (a, b) } else { (b, a) };

    assert_eq!(
        causal_order(&record),
        Ok(vec![
            Address::new(first, seq(1)),
            Address::new(second, seq(1))
        ])
    );
}

#[test]
fn heads_that_cite_each_other_in_a_cycle_have_no_causal_order() {
    let (a, b) = (writer(1), writer(2));
    let mut record = two_writers(1);
    let b1 = SeenHead::new(Address::new(b, seq(1)), record[&b][0].hash);
    record.get_mut(&a).unwrap()[0].header.heads = vec![b1];

    assert_eq!(
        causal_order(&record),
        Err(ChainError::new(
            Address::new(a.min(b), seq(1)),
            Fault::Cycle
        ))
    );
}

#[test]
fn faults_explain_themselves() {
    let cited = Address::new(writer(2), seq(4));
    for (fault, want) in [
        (
            Fault::Seq(3),
            "want seq 3 here: the writer's log has a gap".to_owned(),
        ),
        (
            Fault::Writer,
            "the event belongs to another writer".to_owned(),
        ),
        (
            Fault::PrevHash,
            "prev_hash does not name the writer's previous event".to_owned(),
        ),
        (
            Fault::Hash,
            "the stored hash does not match the event's header".to_owned(),
        ),
        (
            Fault::Head(cited),
            format!("it cites {cited}, which no event of another writer matches"),
        ),
        (
            Fault::HeadBackward(cited),
            format!("it cites {cited}, at or behind a head its writer cited before"),
        ),
        (Fault::Cycle, "its cited heads form a cycle".to_owned()),
    ] {
        assert_eq!(fault.to_string(), want);
    }
    let err = ChainError::new(cited, Fault::Writer);
    assert_eq!(
        err.to_string(),
        format!("{cited}: the event belongs to another writer")
    );
}
