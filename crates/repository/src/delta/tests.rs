use super::apply;
use crate::Error;
use crate::fixture::Prng;
use crate::fixture::pack::{delta, delta_copy, delta_insert, delta_varint};

const BASE: &[u8] = b"0123456789abcdefghij";
const MAX: usize = 1 << 16;

fn corrupt(result: Result<Vec<u8>, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

#[test]
fn copies_and_inserts_build_the_result() {
    let ops = [delta_copy(10, 5), delta_insert(b"-new-"), delta_copy(0, 3)];
    let d = delta(BASE.len(), 13, &ops);
    assert_eq!(apply(BASE, &d, MAX).unwrap(), b"abcde-new-012");
}

#[test]
fn a_delta_of_nothing_makes_an_empty_result() {
    assert_eq!(apply(b"", &delta(0, 0, &[]), MAX).unwrap(), b"");
}

#[test]
fn a_copy_may_name_its_offset_in_several_bytes() {
    let base: Vec<u8> = (0..70_000_u32).map(|i| (i % 253) as u8).collect();
    let d = delta(base.len(), 4, &[delta_copy(0x1_0203, 4)]);
    assert_eq!(apply(&base, &d, MAX).unwrap(), base[0x1_0203..0x1_0207]);
}

#[test]
fn a_copy_of_size_zero_copies_65536_bytes() {
    let base = vec![7_u8; 0x10000];
    let d = delta(base.len(), 0x10000, &[delta_copy(0, 0)]);
    assert_eq!(apply(&base, &d, 1 << 20).unwrap(), base);
}

#[test]
fn a_copy_may_name_its_size_in_several_bytes() {
    let base: Vec<u8> = vec![9; 300];
    let d = delta(base.len(), 258, &[delta_copy(1, 258)]);
    assert_eq!(apply(&base, &d, MAX).unwrap(), vec![9; 258]);
}

#[test]
fn a_base_of_the_wrong_size_is_corrupt() {
    let d = delta(BASE.len() + 1, 1, &[delta_insert(b"x")]);
    assert!(corrupt(apply(BASE, &d, MAX)));
}

#[test]
fn a_result_of_the_wrong_size_is_corrupt() {
    let short = delta(BASE.len(), 5, &[delta_insert(b"xyz")]);
    assert!(corrupt(apply(BASE, &short, MAX)));
    let long = delta(BASE.len(), 2, &[delta_insert(b"xyz")]);
    assert!(corrupt(apply(BASE, &long, MAX)));
}

#[test]
fn a_copy_outside_the_base_is_corrupt() {
    for (offset, size) in [(18, 5), (21, 1), (0, 21), (0x1_0000, 1)] {
        let d = delta(BASE.len(), size.max(1), &[delta_copy(offset, size)]);
        assert!(corrupt(apply(BASE, &d, MAX)), "{offset} {size}");
    }
}

#[test]
fn the_reserved_command_is_corrupt() {
    let d = [delta_varint(BASE.len()), delta_varint(1), vec![0]].concat();
    assert!(corrupt(apply(BASE, &d, MAX)));
}

#[test]
fn a_delta_cut_short_is_corrupt() {
    let ops = [delta_copy(10, 5), delta_insert(b"-new-")];
    let d = delta(BASE.len(), 10, &ops);
    for cut in 0..d.len() {
        assert!(corrupt(apply(BASE, &d[..cut], MAX)), "cut at {cut}");
    }
    assert!(apply(BASE, &d, MAX).is_ok());
}

#[test]
fn a_size_that_never_ends_is_corrupt() {
    let d = vec![0x80; 20];
    assert!(corrupt(apply(BASE, &d, MAX)));
}

#[test]
fn a_result_past_the_maximum_is_refused_before_it_is_built() {
    let d = delta(BASE.len(), 1 << 40, &[delta_copy(0, 5)]);
    assert!(matches!(apply(BASE, &d, MAX), Err(Error::TooLarge(_))));
    let d = delta(BASE.len(), 6, &[delta_insert(b"abcdef")]);
    assert!(matches!(apply(BASE, &d, 5), Err(Error::TooLarge(_))));
    assert!(apply(BASE, &d, 6).is_ok());
}

#[test]
fn random_deltas_never_panic() {
    let mut prng = Prng::new(0xde17a);
    let valid = delta(
        BASE.len(),
        13,
        &[delta_copy(10, 5), delta_insert(b"-new-"), delta_copy(0, 3)],
    );
    for round in 0..4000 {
        let bytes = if round % 2 == 0 {
            prng.corrupt(&valid)
        } else {
            let len = prng.below(40);
            prng.bytes(len)
        };
        if let Ok(out) = apply(BASE, &bytes, 1 << 12) {
            assert!(out.len() <= 1 << 12);
        }
    }
}
