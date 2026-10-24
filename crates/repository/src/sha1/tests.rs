use super::{Sha1, object_id};
use crate::fixture::hex;

fn digest(data: &[u8]) -> Vec<u8> {
    let mut sha = Sha1::new();
    sha.update(data);
    sha.finish().to_vec()
}

fn of_a(count: usize) -> Vec<u8> {
    digest(&vec![b'a'; count])
}

#[test]
fn the_standard_vectors_hash_as_published() {
    let cases: [(&[u8], &str); 3] = [
        (b"", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        (b"abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1",
        ),
    ];
    for (data, expected) in cases {
        assert_eq!(digest(data), hex(expected), "{data:?}");
    }
}

#[test]
fn a_million_letters_hash_as_published() {
    assert_eq!(
        of_a(1_000_000),
        hex("34aa973cd4c4daa4f61eeb2bdbad27316534016f")
    );
}

#[test]
fn the_lengths_around_a_block_boundary_hash_as_sha1sum_does() {
    let cases = [
        (55, "c1c8bbdc22796e28c0e15163d20899b65621d65a"),
        (56, "c2db330f6083854c99d4b5bfb6e8f29f201be699"),
        (57, "f08f24908d682555111be7ff6f004e78283d989a"),
        (63, "03f09f5b158a7a8cdad920bddc29b81c18a551f5"),
        (64, "0098ba824b5c16427bd7a1122a5a442a25ec644d"),
        (65, "11655326c708d70319be2610e8a57d9a5b959d3b"),
        (119, "ee971065aaa017e0632a8ca6c77bb3bf8b1dfc56"),
        (120, "f34c1488385346a55709ba056ddd08280dd4c6d6"),
        (128, "ad5b3fdbcb526778c2839d2f151ea753995e26a0"),
    ];
    for (count, expected) in cases {
        assert_eq!(of_a(count), hex(expected), "{count} bytes");
    }
}

#[test]
fn the_pieces_of_an_input_hash_as_the_whole_does() {
    let data: Vec<u8> = (0..=255).cycle().take(1000).collect();
    for size in [1, 3, 63, 64, 65, 200] {
        let mut sha = Sha1::new();
        for piece in data.chunks(size) {
            sha.update(piece);
        }
        assert_eq!(sha.finish().to_vec(), digest(&data), "pieces of {size}");
    }
}

#[test]
fn an_object_is_named_the_way_git_names_it() {
    assert_eq!(
        object_id("blob", b"").as_str(),
        "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
    );
    assert_eq!(
        object_id("blob", b"hello\n").as_str(),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
    assert_eq!(
        object_id("tree", b"").as_str(),
        "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
    );
}
