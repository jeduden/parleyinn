use super::*;

fn run(args: &[&str]) -> (u8, Vec<u8>, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = licenses(args.iter().map(|a| (*a).to_owned()), &mut out, &mut err);

    (code, out, String::from_utf8(err).unwrap())
}

#[test]
fn the_build_embeds_the_committed_file_of_its_own_target() {
    let committed = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(EMBEDDED_FILE);

    assert_eq!(
        THIRD_PARTY_LICENSES,
        std::fs::read_to_string(committed).unwrap()
    );
}

#[test]
fn the_embedded_text_names_what_every_build_compiles_in() {
    for name in ["SQLite", "Rust standard library", "License: MIT"] {
        assert!(THIRD_PARTY_LICENSES.contains(name), "{name}");
    }
}

#[test]
fn licenses_prints_the_embedded_text_byte_for_byte() {
    let (code, out, err) = run(&[]);

    assert_eq!((code, err.as_str()), (0, ""));
    assert_eq!(out, THIRD_PARTY_LICENSES.as_bytes());
}

#[test]
fn licenses_takes_no_argument() {
    let (code, out, err) = run(&["--json"]);

    assert_eq!(code, EXIT_USAGE);
    assert!(out.is_empty());
    assert_eq!(err, "parinn licenses: unknown argument \"--json\"\n");
}

#[test]
fn licenses_names_a_failed_write() {
    struct Full;
    impl Write for Full {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut err = Vec::new();

    let code = licenses(std::iter::empty(), &mut Full, &mut err);

    assert_eq!(code, crate::EXIT_FAILURE);
    assert!(String::from_utf8(err).unwrap().contains("disk full"));
}
