use std::io::Cursor;

use super::{Entry, check_header, find_offset, read_entry};
use crate::fixture::pack::{
    Failing, PackBuilder, delta, delta_insert, entry_header, ofs_distance, pack_index,
};
use crate::fixture::{Prng, zlib_stored};
use crate::object::{Kind, Object};
use crate::{CommitId, Error};

const MAX: usize = 1 << 20;

fn id(bytes: [u8; 20]) -> CommitId {
    CommitId::from_bytes(&bytes)
}

fn ids(count: usize, seed: u64) -> Vec<[u8; 20]> {
    let mut prng = Prng::new(seed);
    (0..count)
        .map(|_| prng.bytes(20).try_into().unwrap())
        .collect()
}

fn lookup(index: &[u8], bytes: [u8; 20]) -> Result<Option<u64>, Error> {
    find_offset(&mut Cursor::new(index), &id(bytes))
}

fn entry(pack: &[u8], offset: u64) -> Result<Entry, Error> {
    read_entry(&mut Cursor::new(pack), offset, MAX)
}

fn corrupt<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

fn whole(kind: Kind, data: &[u8]) -> Entry {
    Entry::Whole(Object {
        kind,
        data: data.to_vec(),
    })
}

#[test]
fn every_listed_id_is_found_at_its_offset() {
    let mut listed = ids(300, 1);
    listed.extend([
        [0; 20],
        [0xff; 20],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    ]);
    let entries: Vec<_> = listed
        .iter()
        .enumerate()
        .map(|(n, id)| (*id, 12 + n as u64 * 7))
        .collect();
    let index = pack_index(&entries);
    for (id, offset) in &entries {
        assert_eq!(lookup(&index, *id).unwrap(), Some(*offset));
    }
}

#[test]
fn an_id_the_index_does_not_list_is_not_found() {
    let entries: Vec<_> = ids(300, 2).into_iter().map(|id| (id, 12)).collect();
    let index = pack_index(&entries);
    for other in ids(300, 3) {
        assert_eq!(lookup(&index, other).unwrap(), None);
    }
    let mut near = entries[0].0;
    near[19] ^= 1;
    assert_eq!(lookup(&index, near).unwrap(), None);
    assert_eq!(lookup(&index, [0; 20]).unwrap(), None);
    assert_eq!(lookup(&index, [0xff; 20]).unwrap(), None);
}

#[test]
fn an_empty_index_lists_nothing() {
    assert_eq!(lookup(&pack_index(&[]), [7; 20]).unwrap(), None);
}

#[test]
fn an_offset_past_two_gibibytes_comes_from_the_large_table() {
    let entries = [
        ([1; 20], 40),
        ([2; 20], 0x1_2345_6789),
        ([3; 20], 1 << 31),
        ([4; 20], 0xffff_ffff_ffff),
    ];
    let index = pack_index(&entries);
    for (id, offset) in entries {
        assert_eq!(lookup(&index, id).unwrap(), Some(offset));
    }
}

#[test]
fn a_large_offset_beyond_the_large_table_is_corrupt() {
    let entries = [([1; 20], 1 << 31)];
    let mut index = pack_index(&entries);
    let at = 8 + 1024 + 20 + 4;
    index[at..at + 4].copy_from_slice(&0x8000_0005_u32.to_be_bytes());
    assert!(corrupt(lookup(&index, [1; 20])));
}

#[test]
fn an_index_of_another_version_is_unsupported() {
    let mut index = pack_index(&[([1; 20], 12)]);
    index[7] = 3;
    assert!(matches!(
        lookup(&index, [1; 20]),
        Err(Error::Unsupported(_))
    ));
    let version_one = vec![0; 2000];
    assert!(matches!(
        lookup(&version_one, [1; 20]),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn an_index_cut_anywhere_is_corrupt() {
    let index = pack_index(&[([1; 20], 12), ([2; 20], 40)]);
    for cut in 0..index.len() {
        assert!(corrupt(lookup(&index[..cut], [2; 20])), "cut at {cut}");
    }
}

#[test]
fn an_index_that_counts_more_objects_than_it_holds_is_corrupt() {
    let mut index = pack_index(&[([1; 20], 12)]);
    let last = 8 + 255 * 4;
    index[last..last + 4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(corrupt(lookup(&index, [1; 20])));
}

#[test]
fn a_fanout_that_runs_backwards_is_corrupt() {
    let mut index = pack_index(&[([0x10; 20], 12), ([0x20; 20], 40)]);
    let at = 8 + 0x20 * 4;
    index[at..at + 4].copy_from_slice(&0_u32.to_be_bytes());
    assert!(corrupt(lookup(&index, [0x20; 20])));
}

#[test]
fn an_index_that_cannot_be_read_is_an_io_error() {
    let result = find_offset(&mut Failing, &id([1; 20]));
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn random_and_corrupted_indexes_never_panic() {
    let valid = pack_index(&[([1; 20], 12), ([2; 20], 1 << 31), ([0x80; 20], 99)]);
    let mut prng = Prng::new(11);
    for round in 0..3000 {
        let bytes = if round % 3 == 0 {
            let len = prng.below(1200);
            prng.bytes(len)
        } else {
            prng.corrupt(&valid)
        };
        for probe in [[1; 20], [2; 20], [0x80; 20], [0; 20]] {
            let _ = lookup(&bytes, probe);
        }
    }
}

#[test]
fn a_pack_header_of_version_two_or_three_passes() {
    for version in [2_u8, 3] {
        let mut bytes = b"PACK\0\0\0".to_vec();
        bytes.extend([version, 0, 0, 0, 0]);
        assert!(check_header(&mut Cursor::new(bytes)).is_ok());
    }
}

#[test]
fn a_pack_header_that_is_not_one_is_refused() {
    let mut bytes = b"PACK\0\0\0\x04\0\0\0\0".to_vec();
    assert!(matches!(
        check_header(&mut Cursor::new(bytes.clone())),
        Err(Error::Unsupported(_))
    ));
    bytes[0] = b'X';
    assert!(corrupt(check_header(&mut Cursor::new(bytes))));
    assert!(corrupt(check_header(&mut Cursor::new(b"PACK".to_vec()))));
    assert!(matches!(check_header(&mut Failing), Err(Error::Io { .. })));
}

#[test]
fn a_whole_object_of_each_kind_is_read_back() {
    let mut pack = PackBuilder::new();
    let offsets: Vec<u64> = (1..=4)
        .map(|kind| pack.whole([kind; 20], kind, format!("data {kind}").as_bytes()))
        .collect();
    let (bytes, _) = pack.finish();
    let kinds = [Kind::Commit, Kind::Tree, Kind::Blob, Kind::Tag];
    for (n, offset) in offsets.iter().enumerate() {
        let expected = whole(kinds[n], format!("data {}", n + 1).as_bytes());
        assert_eq!(entry(&bytes, *offset).unwrap(), expected);
    }
}

#[test]
fn a_size_in_several_header_bytes_is_read() {
    for size in [15, 16, 300, 5000, 70_000] {
        let data = vec![b'x'; size];
        let mut pack = PackBuilder::new();
        let offset = pack.whole([1; 20], 3, &data);
        let (bytes, _) = pack.finish();
        assert_eq!(entry(&bytes, offset).unwrap(), whole(Kind::Blob, &data));
    }
}

#[test]
fn an_offset_delta_names_its_base_by_distance() {
    for filler in [0, 100, 200, 20_000, 300_000] {
        let mut pack = PackBuilder::new();
        let base = pack.whole([1; 20], 3, b"base");
        pack.whole([2; 20], 3, &vec![b'f'; filler]);
        let d = delta(4, 1, &[delta_insert(b"x")]);
        let at = pack.offset_delta([3; 20], base, &d);
        let (bytes, _) = pack.finish();
        let expected = Entry::OffsetDelta { base, delta: d };
        assert_eq!(entry(&bytes, at).unwrap(), expected, "{filler}");
    }
}

#[test]
fn a_reference_delta_names_its_base_by_id() {
    let mut pack = PackBuilder::new();
    let d = delta(4, 1, &[delta_insert(b"x")]);
    let at = pack.reference_delta([3; 20], [9; 20], &d);
    let (bytes, _) = pack.finish();
    let expected = Entry::ReferenceDelta {
        base: id([9; 20]),
        delta: d,
    };
    assert_eq!(entry(&bytes, at).unwrap(), expected);
}

#[test]
fn an_entry_of_a_type_git_does_not_define_is_corrupt() {
    for kind in [0, 5] {
        let bytes = [entry_header(kind, 3), zlib_stored(b"abc")].concat();
        assert!(corrupt(entry(&bytes, 0)), "{kind}");
    }
}

#[test]
fn an_entry_whose_data_is_not_its_declared_size_is_not_read() {
    let short = [entry_header(3, 5), zlib_stored(b"abcd")].concat();
    assert!(corrupt(entry(&short, 0)));
    let long = [entry_header(3, 3), zlib_stored(b"abcd")].concat();
    assert!(entry(&long, 0).is_err());
}

#[test]
fn an_entry_declaring_more_than_the_maximum_is_refused_unread() {
    let bytes = entry_header(3, 5000);
    let result = read_entry(&mut Cursor::new(bytes.clone()), 0, 4999);
    assert!(matches!(result, Err(Error::TooLarge(_))));
    let bytes = [entry_header(6, 5000), ofs_distance(1)].concat();
    let result = read_entry(&mut Cursor::new(bytes), 0, 4999);
    assert!(matches!(result, Err(Error::TooLarge(_))));
}

#[test]
fn an_entry_header_that_never_ends_is_corrupt() {
    let bytes = [vec![0x3f], vec![0xff; 12], vec![0x01]].concat();
    assert!(corrupt(entry(&bytes, 0)));
}

#[test]
fn an_offset_delta_pointing_outside_the_pack_is_corrupt() {
    let payload = zlib_stored(b"d");
    for distance in [ofs_distance(0), ofs_distance(1000), vec![0xff; 12]] {
        let bytes = [vec![0; 20], entry_header(6, 1), distance, payload.clone()].concat();
        assert!(corrupt(entry(&bytes, 20)));
    }
}

#[test]
fn an_entry_cut_short_is_corrupt() {
    let mut pack = PackBuilder::new();
    let a = pack.whole([1; 20], 3, b"hello world");
    let b = pack.offset_delta([2; 20], a, &delta(11, 1, &[delta_insert(b"x")]));
    let c = pack.reference_delta([3; 20], [4; 20], b"d");
    let (bytes, _) = pack.finish();
    for start in [a, b, c] {
        let from = start as usize;
        let end = bytes.len() - 20;
        let stop = (from..=end).find(|&s| entry(&bytes[..s], start).is_ok());
        assert!(stop.is_some_and(|s| s > from), "{start}");
        for cut in from..stop.unwrap() {
            assert!(corrupt(entry(&bytes[..cut], start)), "{start} cut at {cut}");
        }
    }
    assert!(corrupt(entry(&bytes, bytes.len() as u64 + 100)));
}

#[test]
fn an_entry_that_cannot_be_read_is_an_io_error() {
    let result = read_entry(&mut Failing, 0, MAX);
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn random_and_corrupted_packs_never_panic() {
    let mut pack = PackBuilder::new();
    let a = pack.whole([1; 20], 1, b"tree 0123\nparent 4567\n\nmessage");
    let b = pack.offset_delta([2; 20], a, &delta(31, 1, &[delta_insert(b"x")]));
    let c = pack.reference_delta([3; 20], [1; 20], b"d");
    let (valid, _) = pack.finish();
    let mut prng = Prng::new(21);
    for round in 0..3000 {
        let bytes = if round % 3 == 0 {
            let len = prng.below(200);
            prng.bytes(len)
        } else {
            prng.corrupt(&valid)
        };
        for start in [a, b, c, 0, 5] {
            let _ = read_entry(&mut Cursor::new(&bytes), start, 1 << 12);
        }
        let _ = check_header(&mut Cursor::new(&bytes));
    }
}
