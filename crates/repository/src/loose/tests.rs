use std::fs;
use std::path::Path;

use testkit::TempDir;

use super::{read_up_to, rel_of};
use crate::fixture::{Prng, zlib_stored};
use crate::object::{Kind, Object};
use crate::{CommitId, Error};

const MAX: usize = 1 << 20;

fn id(n: u8) -> CommitId {
    CommitId::from_bytes(&[n; 20])
}

/// Stores `content` as the loose file of `id`.
fn put(objects: &Path, id: &CommitId, content: &[u8]) {
    let path = objects.join(rel_of(id));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, zlib_stored(content)).unwrap();
}

fn put_object(objects: &Path, id: &CommitId, kind: &str, body: &[u8]) {
    let header = format!("{kind} {}\0", body.len());
    put(objects, id, &[header.as_bytes(), body].concat());
}

fn corrupt<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Corrupt(_)))
}

#[test]
fn the_file_of_an_id_is_under_its_first_two_digits() {
    assert_eq!(rel_of(&id(0xab)), format!("ab/{}", "ab".repeat(19)));
}

#[test]
fn each_kind_of_object_is_read_back() {
    let dir = TempDir::new().unwrap();
    for (n, (name, kind)) in [
        ("commit", Kind::Commit),
        ("tree", Kind::Tree),
        ("blob", Kind::Blob),
        ("tag", Kind::Tag),
    ]
    .into_iter()
    .enumerate()
    {
        let id = id(n as u8 + 1);
        put_object(dir.path(), &id, name, b"body\nlines");
        let expected = Object {
            kind,
            data: b"body\nlines".to_vec(),
        };
        assert_eq!(read_up_to(dir.path(), &id, MAX).unwrap(), Some(expected));
    }
}

#[test]
fn an_object_with_no_body_is_read() {
    let dir = TempDir::new().unwrap();
    put_object(dir.path(), &id(1), "blob", b"");
    let object = read_up_to(dir.path(), &id(1), MAX).unwrap().unwrap();
    assert!(object.data.is_empty());
}

#[test]
fn an_object_that_is_not_stored_loose_is_none() {
    let dir = TempDir::new().unwrap();
    assert_eq!(read_up_to(dir.path(), &id(1), MAX).unwrap(), None);
    put_object(dir.path(), &id(2), "blob", b"x");
    assert_eq!(read_up_to(dir.path(), &id(1), MAX).unwrap(), None);
}

#[test]
fn a_header_that_does_not_describe_the_body_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let cases: [&[u8]; 8] = [
        b"commit 3\0abcd",
        b"commit 5\0abcd",
        b"commit x\0abcd",
        b"commit \0abcd",
        b"commit4\0abcd",
        b"unknown 4\0abcd",
        b"commit 4 abcd",
        b"",
    ];
    for (n, content) in cases.into_iter().enumerate() {
        let id = id(n as u8 + 1);
        put(dir.path(), &id, content);
        assert!(corrupt(read_up_to(dir.path(), &id, MAX)), "{content:?}");
    }
}

#[test]
fn a_header_that_never_ends_is_corrupt() {
    let dir = TempDir::new().unwrap();
    put(dir.path(), &id(1), &[b'a'; 100]);
    assert!(corrupt(read_up_to(dir.path(), &id(1), MAX)));
}

#[test]
fn a_file_that_is_not_zlib_is_corrupt() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(rel_of(&id(1)));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"this is not zlib").unwrap();
    assert!(corrupt(read_up_to(dir.path(), &id(1), MAX)));
}

#[test]
fn an_object_past_the_maximum_is_too_large() {
    let dir = TempDir::new().unwrap();
    put_object(dir.path(), &id(1), "blob", &[b'x'; 100]);
    let result = read_up_to(dir.path(), &id(1), 50);
    assert!(matches!(result, Err(Error::TooLarge(_))));
    assert!(read_up_to(dir.path(), &id(1), 100).is_ok());
}

#[test]
fn a_directory_in_the_place_of_the_file_is_refused() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join(rel_of(&id(1)))).unwrap();
    let result = read_up_to(dir.path(), &id(1), MAX);
    assert!(matches!(result, Err(Error::Unsupported(_))));
}

#[test]
fn corrupted_loose_files_never_panic() {
    let dir = TempDir::new().unwrap();
    let valid = zlib_stored(b"commit 4\0abcd");
    let mut prng = Prng::new(31);
    let path = dir.path().join(rel_of(&id(1)));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    for _ in 0..500 {
        fs::write(&path, prng.corrupt(&valid)).unwrap();
        let _ = read_up_to(dir.path(), &id(1), 1 << 10);
    }
}

#[test]
fn a_fan_out_directory_that_is_a_file_is_an_io_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("ab"), "not a directory").unwrap();
    assert!(matches!(
        read_up_to(dir.path(), &id(0xab), MAX),
        Err(Error::Io { .. })
    ));
}

#[test]
fn a_body_past_the_maximum_by_less_than_a_header_is_too_large() {
    let dir = TempDir::new().unwrap();
    put_object(dir.path(), &id(1), "blob", &[b'x'; 60]);
    let result = read_up_to(dir.path(), &id(1), 50);
    assert!(matches!(result, Err(Error::TooLarge(_))));
}
