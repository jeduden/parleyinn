use super::*;
use record::id::Seq;

fn writer(n: u8) -> WriterId {
    WriterId::derive(&[n; 32])
}

fn candidate(writer: WriterId, seq: u64, relevance: f64) -> Candidate {
    Candidate {
        address: Address::new(writer, Seq::new(seq).unwrap()),
        relevance,
    }
}

#[test]
fn a_hit_of_a_writer_with_few_matches_outranks_the_repeats_of_one_with_many() {
    let (long, short) = (writer(1), writer(2));
    let mut found: Vec<Candidate> = (1..=5).map(|n| candidate(long, n, 4.0)).collect();
    found.push(candidate(short, 1, 3.0));

    let hits = rank(&found, 10);

    let order: Vec<(u8, u64)> = hits
        .iter()
        .map(|h| {
            (
                if h.address.writer == long { 1 } else { 2 },
                h.address.seq.get(),
            )
        })
        .collect();
    assert_eq!(order, [(1, 1), (2, 1), (1, 2), (1, 3), (1, 4), (1, 5)]);
    assert!((hits[0].score - 4.0).abs() < 1e-9);
    assert!((hits[1].score - 3.0).abs() < 1e-9);
    assert!((hits[2].score - 4.0 / 1.5).abs() < 1e-9);
}

#[test]
fn ties_go_to_the_smaller_address_whatever_the_order_of_the_matches() {
    let found = [
        candidate(writer(2), 1, 1.0),
        candidate(writer(1), 2, 1.0),
        candidate(writer(1), 1, 1.0),
    ];
    let mut reversed = found;
    reversed.reverse();

    assert_eq!(rank(&found, 10), rank(&reversed, 10));
    let smallest = found.iter().map(|c| c.address).min();
    assert_eq!(Some(rank(&found, 1)[0].address), smallest);
}

#[test]
fn at_most_k_hits_return_and_never_more_than_fifty() {
    let found: Vec<Candidate> = (1..=80).map(|n| candidate(writer(1), n, 1.0)).collect();

    assert_eq!(rank(&found, 3).len(), 3);
    assert_eq!(rank(&found, 500).len(), MAX_HITS);
    assert!(rank(&found, 0).is_empty());
    assert!(rank(&[], 10).is_empty());
}
