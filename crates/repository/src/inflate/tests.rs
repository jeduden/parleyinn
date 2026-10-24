use super::{Bits, Huffman, adler32, zlib_inflate};
use crate::Error;
use crate::fixture::{Prng, hex, zlib_stored};

/// `zlib.compress(b"hello", 0)`: one stored block.
const STORED: &str = "7801010500faff68656c6c6f062c0215";
/// `zlib.compress(b"hello")`: one fixed-code block of literals.
const FIXED: &str = "789ccb48cdc9c90700062c0215";
/// `zlib.compress(b"abcabcabcabcabcabc")`: a fixed-code block with a match.
const MATCH: &str = "789c4b4c4a4e444500417c06e5";
/// `zlib.compress(b"")`.
const EMPTY: &str = "789c030000000001";
/// `zlib.compress(b"\0" * 100000, 9)`: 120 bytes that inflate to 100000.
const ZEROS: &str = "78daedc13101000000c2a0f54f6d0d0fa000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080570386af0001";
/// A dynamic-code block whose code lengths use repeat codes 16, 17
/// and 18 (`zlib.compress(DYNAMIC_PLAIN, 9)`).
const DYNAMIC: &str = "78da95906b6ec0300883775563301c21f4f4735a69ffd74835e161be96896128b2203650d5c98054ec62f90da19d6d961ad688517423b31193448b520e8219c80e3a1cd68c6d2b4059af1922ef9e09a23cd8cebadd3ed3e8a94c779382810a83428e4bf62ed85455691ccdd059135ccc568229230622c2bc6a09e9914c844517971c83fbfbbc44f02aa39631cb1eae76289b2fc5c83fc246133926e61de861cf1dfdf9f7739e677d8ecf7376f74f8ea35bb872f69cbdd763ddf3363d5ff40ebe16bb6fc3573a9fef9ddffd055987a4e4";
const DYNAMIC_PLAIN: &str = "6364616863626662646561666367616165656764636261666665636765636565636761666167656567676365666761656767626268666267676164646761626864636167666366666468616263646261646762636861626863656868626662656261636662666263676563616264666264656862636165626864676662636164676164646867616768656464626562636366616168636561686165616468646867636666656167626366656564656563666868636468676662676463626167666461636466616661626162626263656566676666616466656164646162666561666262686663636865676766636765646466616865646263646563686165646362686663676266646763686761676868666465616262636862646865636163656464666768636768676564640000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000787a7a797a797a787a787a7a78797979787a7a78797979787a78787a7a7a797a78787a7a7a78797878797a7a7a7978787879797879787a7a787a787979787978787a787a7a7a797a797a79797a7a7978787a7a787a787978787a7a797a79797a78787979";

const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

const LIMIT: usize = 1 << 20;

fn inflate(stream: &[u8]) -> Result<Vec<u8>, Error> {
    zlib_inflate(stream, LIMIT)
}

fn corrupt(result: Result<Vec<u8>, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

/// Writes a deflate stream bit by bit, least significant bit first.
#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    used: u32,
}

impl BitWriter {
    fn bit(&mut self, bit: u32) {
        if self.used.is_multiple_of(8) {
            self.bytes.push(0);
        }
        if bit != 0 {
            *self.bytes.last_mut().unwrap() |= 1 << (self.used % 8);
        }
        self.used += 1;
    }

    /// A number, least significant bit first.
    fn bits(&mut self, value: u32, count: u32) -> &mut Self {
        (0..count).for_each(|i| self.bit(value >> i & 1));
        self
    }

    /// A Huffman code, most significant bit first.
    fn code(&mut self, value: u32, count: u32) -> &mut Self {
        (0..count).for_each(|i| self.bit(value >> (count - 1 - i) & 1));
        self
    }

    /// A symbol of the fixed literal and length code.
    fn fixed_lit(&mut self, sym: u32) -> &mut Self {
        match sym {
            0..=143 => self.code(0x30 + sym, 8),
            144..=255 => self.code(0x190 + sym - 144, 9),
            256..=279 => self.code(sym - 256, 7),
            _ => self.code(0xc0 + sym - 280, 8),
        }
    }

    fn fixed_dist(&mut self, sym: u32) -> &mut Self {
        self.code(sym, 5)
    }

    /// The header of a final dynamic block, with the code length code
    /// lengths given by symbol.
    fn dynamic_header(&mut self, sizes: (u32, u32, u32), lengths: &[(usize, u32)]) -> &mut Self {
        self.bits(1, 1).bits(2, 2);
        self.bits(sizes.0, 5).bits(sizes.1, 5).bits(sizes.2, 4);
        for &sym in CODE_LENGTH_ORDER.iter().take(sizes.2 as usize + 4) {
            let length = lengths.iter().find(|l| l.0 == sym).map_or(0, |l| l.1);
            self.bits(length, 3);
        }
        self
    }
}

/// A zlib stream around `deflate`, checksummed for `plain`.
fn zlib(deflate: &BitWriter, plain: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    out.extend(&deflate.bytes);
    out.extend(adler32(plain).to_be_bytes());
    out
}

#[test]
fn a_stored_block_inflates() {
    assert_eq!(inflate(&hex(STORED)).unwrap(), b"hello");
}

#[test]
fn a_fixed_code_block_inflates() {
    assert_eq!(inflate(&hex(FIXED)).unwrap(), b"hello");
}

#[test]
fn a_match_copies_from_earlier_output() {
    assert_eq!(inflate(&hex(MATCH)).unwrap(), b"abc".repeat(6));
}

#[test]
fn an_empty_stream_inflates_to_nothing() {
    assert_eq!(inflate(&hex(EMPTY)).unwrap(), b"");
    assert_eq!(inflate(&zlib_stored(b"")).unwrap(), b"");
}

#[test]
fn a_dynamic_code_block_inflates() {
    assert_eq!(inflate(&hex(DYNAMIC)).unwrap(), hex(DYNAMIC_PLAIN));
}

#[test]
fn a_long_run_of_matches_inflates_to_its_full_size() {
    let out = inflate(&hex(ZEROS)).unwrap();
    assert_eq!(out.len(), 100_000);
    assert!(out.iter().all(|&b| b == 0));
}

#[test]
fn several_stored_blocks_inflate_in_order() {
    let data: Vec<u8> = (0..70_000_u32).map(|i| (i % 251) as u8).collect();
    assert_eq!(inflate(&zlib_stored(&data)).unwrap(), data);
}

#[test]
fn the_reader_stops_at_the_end_of_the_stream() {
    for stream in [STORED, FIXED, MATCH, DYNAMIC] {
        let mut bytes = hex(stream);
        bytes.extend(b"trailing bytes");
        let mut reader: &[u8] = &bytes;
        zlib_inflate(&mut reader, LIMIT).unwrap();
        assert_eq!(reader, b"trailing bytes", "{stream}");
    }
}

#[test]
fn output_past_the_maximum_is_refused_not_stored() {
    let too_large =
        |stream: &str, max| matches!(zlib_inflate(&hex(stream)[..], max), Err(Error::TooLarge(_)));
    assert!(too_large(FIXED, 4));
    assert!(too_large(MATCH, 17));
    assert!(too_large(STORED, 4));
    assert!(too_large(ZEROS, 99_999));
    assert!(too_large(DYNAMIC, 100));
    assert!(zlib_inflate(&hex(FIXED)[..], 5).is_ok());
    assert!(zlib_inflate(&hex(MATCH)[..], 18).is_ok());
    assert!(zlib_inflate(&hex(STORED)[..], 5).is_ok());
    assert!(zlib_inflate(&hex(ZEROS)[..], 100_000).is_ok());
}

#[test]
fn a_bad_header_is_corrupt() {
    for header in [
        [0x79, 0x9c], // not deflate
        [0x78, 0x9d], // check bits wrong
        [0x88, 0x1c], // window larger than 32 KiB
        [0x78, 0xbb], // preset dictionary
    ] {
        let mut bytes = hex(FIXED);
        bytes[..2].copy_from_slice(&header);
        assert!(corrupt(inflate(&bytes)), "{header:x?}");
    }
}

#[test]
fn a_wrong_checksum_is_corrupt() {
    for stream in [STORED, FIXED, DYNAMIC] {
        let mut bytes = hex(stream);
        *bytes.last_mut().unwrap() ^= 1;
        assert!(corrupt(inflate(&bytes)), "{stream}");
    }
}

#[test]
fn a_stream_cut_anywhere_is_corrupt() {
    for stream in [STORED, FIXED, MATCH, DYNAMIC, EMPTY] {
        let bytes = hex(stream);
        for cut in 0..bytes.len() {
            assert!(corrupt(inflate(&bytes[..cut])), "{stream} cut at {cut}");
        }
    }
}

#[test]
fn a_stored_block_whose_length_check_fails_is_corrupt() {
    let mut bytes = hex(STORED);
    bytes[6] ^= 1;
    assert!(corrupt(inflate(&bytes)));
}

#[test]
fn a_block_of_the_reserved_type_is_corrupt() {
    assert!(corrupt(inflate(&[0x78, 0x01, 0x07, 0, 0, 0, 0, 0, 0])));
}

#[test]
fn a_match_before_the_start_of_the_output_is_corrupt() {
    let mut w = BitWriter::default();
    w.bits(1, 1).bits(1, 2);
    w.fixed_lit(97).fixed_lit(257).fixed_dist(1);
    w.fixed_lit(256);
    assert!(corrupt(inflate(&zlib(&w, b"aaaa"))));
}

#[test]
fn a_match_at_distance_one_repeats_the_last_byte() {
    let mut w = BitWriter::default();
    w.bits(1, 1).bits(1, 2);
    w.fixed_lit(97).fixed_lit(257).fixed_dist(0);
    w.fixed_lit(256);
    assert_eq!(inflate(&zlib(&w, b"aaaa")).unwrap(), b"aaaa");
}

#[test]
fn a_length_symbol_with_extra_bits_reads_them() {
    let mut w = BitWriter::default();
    w.bits(1, 1).bits(1, 2);
    // Length symbol 265 has one extra bit: lengths 11 and 12.
    w.fixed_lit(97).fixed_lit(265).bits(1, 1).fixed_dist(0);
    w.fixed_lit(256);
    assert_eq!(inflate(&zlib(&w, &[97; 13])).unwrap(), [97; 13]);
}

#[test]
fn a_distance_symbol_with_extra_bits_reads_them() {
    let mut w = BitWriter::default();
    w.bits(1, 1).bits(1, 2);
    for byte in b"abcdef" {
        w.fixed_lit(u32::from(*byte));
    }
    // Distance symbol 4 has one extra bit: distances 5 and 6.
    w.fixed_lit(257).fixed_dist(4).bits(1, 1);
    w.fixed_lit(256);
    assert_eq!(inflate(&zlib(&w, b"abcdefabc")).unwrap(), b"abcdefabc");
}

#[test]
fn the_unused_length_and_distance_symbols_are_corrupt() {
    for symbol in [286, 287] {
        let mut w = BitWriter::default();
        w.bits(1, 1).bits(1, 2);
        w.fixed_lit(symbol);
        assert!(corrupt(inflate(&zlib(&w, b""))), "{symbol}");
    }
    let mut w = BitWriter::default();
    w.bits(1, 1).bits(1, 2);
    w.fixed_lit(97).fixed_lit(257).fixed_dist(30);
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

#[test]
fn a_dynamic_header_with_too_many_codes_is_corrupt() {
    let mut w = BitWriter::default();
    w.dynamic_header((31, 0, 0), &[(0, 1), (18, 1)]);
    assert!(corrupt(inflate(&zlib(&w, b""))));
    let mut w = BitWriter::default();
    w.dynamic_header((0, 31, 0), &[(0, 1), (18, 1)]);
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

#[test]
fn a_code_length_code_with_too_many_codes_is_corrupt() {
    let mut w = BitWriter::default();
    w.dynamic_header((0, 0, 0), &[(16, 1), (17, 1), (18, 1)]);
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

#[test]
fn a_repeat_with_nothing_to_repeat_is_corrupt() {
    let mut w = BitWriter::default();
    w.dynamic_header((0, 0, 0), &[(0, 1), (16, 1)]);
    w.code(1, 1).bits(0, 2);
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

#[test]
fn code_lengths_that_run_past_the_header_are_corrupt() {
    let mut w = BitWriter::default();
    w.dynamic_header((0, 0, 0), &[(0, 1), (18, 1)]);
    w.code(1, 1).bits(127, 7).code(1, 1).bits(127, 7);
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

#[test]
fn a_literal_code_with_too_many_codes_is_corrupt() {
    // 257 literal codes and one distance code, all one bit long.
    let mut w = BitWriter::default();
    w.dynamic_header((0, 0, 14), &[(0, 1), (1, 1)]);
    (0..258).for_each(|_| {
        w.code(1, 1);
    });
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

/// A dynamic block whose only literal and distance symbol is the
/// end-of-block symbol, with one bit.
fn lone_end_of_block() -> BitWriter {
    let mut w = BitWriter::default();
    w.dynamic_header((0, 0, 14), &[(0, 1), (1, 1)]);
    (0..256).for_each(|_| {
        w.code(0, 1);
    });
    w.code(1, 1).code(1, 1);
    w
}

#[test]
fn an_incomplete_code_still_decodes_the_symbols_it_has() {
    let mut w = lone_end_of_block();
    w.code(0, 1);
    assert_eq!(inflate(&zlib(&w, b"")).unwrap(), b"");
}

#[test]
fn a_bit_pattern_no_symbol_has_is_corrupt() {
    let mut w = lone_end_of_block();
    (0..20).for_each(|_| {
        w.code(1, 1);
    });
    assert!(corrupt(inflate(&zlib(&w, b""))));
}

#[test]
fn adler32_matches_the_known_checksums() {
    assert_eq!(adler32(b""), 1);
    assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    assert_eq!(adler32(b"hello"), 0x062c_0215);
}

#[test]
fn adler32_agrees_with_the_definition_on_large_input() {
    let mut prng = Prng::new(7);
    let data = prng.bytes(1 << 20);
    let a = 1 + data.iter().map(|&b| u64::from(b)).sum::<u64>();
    let b = data.len() as u64
        + data
            .iter()
            .enumerate()
            .map(|(i, &x)| (data.len() - i) as u64 * u64::from(x))
            .sum::<u64>();
    let expected = (((b % 65521) << 16) | (a % 65521)) as u32;
    assert_eq!(adler32(&data), expected);
    assert_eq!(adler32(&vec![0xff; 1 << 20]), {
        let a = 1 + 255 * (1_u64 << 20);
        let b = (1_u64 << 20) + 255 * ((1_u64 << 20) * ((1_u64 << 20) + 1) / 2);
        (((b % 65521) << 16) | (a % 65521)) as u32
    });
}

#[test]
fn a_code_with_more_symbols_than_its_bits_can_tell_apart_is_refused() {
    assert!(Huffman::build(&[1, 1, 1]).is_err());
    assert!(Huffman::build(&[1, 2, 2]).is_ok());
    assert!(Huffman::build(&[1]).is_ok());
    assert!(Huffman::build(&[]).is_ok());
}

#[test]
fn symbols_decode_by_canonical_code() {
    // Lengths 2, 1, 3, 3: B is `0`, A is `10`, C is `110`, D is `111`.
    let code = Huffman::build(&[2, 1, 3, 3]).unwrap();
    let mut w = BitWriter::default();
    w.code(0, 1).code(2, 2).code(6, 3).code(7, 3);
    let mut bits = Bits::new(&w.bytes[..]);
    let got: Vec<u16> = (0..4).map(|_| bits.decode(&code).unwrap()).collect();
    assert_eq!(got, [1, 0, 2, 3]);
}

#[test]
fn a_symbol_cut_short_is_corrupt() {
    let code = Huffman::build(&[2, 1, 3, 3]).unwrap();
    let result = Bits::new(&[][..]).decode(&code);
    assert!(matches!(result, Err(Error::Corrupt(_))));
}

#[test]
fn corrupted_streams_never_panic_and_never_pass_the_maximum() {
    let streams: Vec<Vec<u8>> = [STORED, FIXED, MATCH, EMPTY, ZEROS, DYNAMIC]
        .iter()
        .map(|s| hex(s))
        .collect();
    let mut prng = Prng::new(0x5eed);
    for round in 0..4000 {
        let stream = &streams[round % streams.len()];
        let bytes = prng.corrupt(stream);
        if let Ok(out) = zlib_inflate(&bytes[..], 4096) {
            assert!(out.len() <= 4096);
        }
    }
}

#[test]
fn random_bytes_behind_a_valid_header_never_panic() {
    let mut prng = Prng::new(0xfeed);
    for _ in 0..4000 {
        let len = prng.below(300);
        let mut bytes = vec![0x78, 0x9c];
        bytes.extend(prng.bytes(len));
        if let Ok(out) = zlib_inflate(&bytes[..], 1 << 12) {
            assert!(out.len() <= 1 << 12);
        }
    }
}
