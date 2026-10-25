use super::*;
use crate::license_texts::tests::workspace;
use crate::tests::checkout;

fn package(name: &str, license: &str) -> Package {
    Package {
        name: name.into(),
        version: "1.0.0".into(),
        license: Some(license.into()),
        dir: PathBuf::from(format!("/registry/{name}-1.0.0")),
    }
}

fn mit(holder: &str) -> (String, String) {
    (
        "LICENSE".to_owned(),
        format!(
            "MIT License\n\nCopyright (c) 2024 {holder}\n\n{}",
            license_texts::canonical("MIT").unwrap()
        ),
    )
}

fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

#[test]
fn the_committed_file_of_a_target_is_under_licenses() {
    assert_eq!(
        committed_path("aarch64-apple-darwin"),
        "licenses/aarch64-apple-darwin/THIRD_PARTY_LICENSES"
    );
}

#[test]
fn every_target_names_sqlite_and_std_and_a_musl_target_its_c_library() {
    let names = |t: &str| non_crate(t).into_iter().map(|e| e.name).collect::<Vec<_>>();

    assert_eq!(
        names("x86_64-pc-windows-gnu"),
        [
            "SQLite",
            "Rust standard library",
            "mingw-w64 runtime (crt, libmingw32, libmingwex)",
            "libgcc and libgcc_eh"
        ]
    );
    assert_eq!(
        names("aarch64-unknown-linux-musl"),
        ["SQLite", "Rust standard library", "musl C library"]
    );
    assert!(
        non_crate("aarch64-apple-darwin")
            .iter()
            .all(|e| !e.texts.is_empty())
    );
}

#[test]
fn a_crate_entry_keeps_its_copyright_lines_once_and_labels_each_text() {
    let files = vec![
        mit("Someone"),
        (
            "LICENSE-TWO".to_owned(),
            "Copyright (c) 2024 Someone\nCopyright 2020 Else\nUse freely.".to_owned(),
        ),
    ];
    let entry = crate_entry(&package("a", "MIT"), &files);

    assert_eq!(entry.name, "a 1.0.0");
    assert_eq!(entry.license, "MIT");
    assert_eq!(
        entry.copyrights,
        ["Copyright (c) 2024 Someone", "Copyright 2020 Else"]
    );
    assert_eq!(
        entry.texts.iter().map(|t| t.0.as_str()).collect::<Vec<_>>(),
        ["MIT", "LICENSE-TWO"]
    );
    let none = Package {
        license: None,
        ..package("b", "MIT")
    };
    assert_eq!(crate_entry(&none, &[]).license, "");
}

#[test]
fn a_text_loses_its_copyright_lines_and_its_blank_ends() {
    assert_eq!(
        body("\nCopyright (c) 2024 A\n\nTerms.  \nMore.\n\n"),
        "Terms.\nMore."
    );
}

#[test]
fn render_gives_each_text_once_with_the_code_using_it() {
    let entries = vec![
        crate_entry(&package("zed", "MIT"), &[mit("Zed Authors")]),
        crate_entry(&package("alpha", "MIT"), &[mit("Alpha Authors")]),
    ];
    let text = render(&entries);

    assert_eq!(text.matches("Permission is hereby granted").count(), 1);
    let alpha = text.find("alpha 1.0.0 (MIT)").unwrap();
    let zed = text.find("zed 1.0.0 (MIT)").unwrap();
    assert!(alpha < zed);
    assert!(text.contains("    Copyright (c) 2024 Alpha Authors\n"));
    assert!(text.contains("    Copyright (c) 2024 Zed Authors\n"));
    assert!(text.contains("License: MIT\n"));
    assert!(!text.contains("MIT License\n\n\n"));
}

#[test]
fn render_is_the_same_whatever_order_the_code_comes_in() {
    let a = crate_entry(&package("a", "MIT"), &[mit("A")]);
    let b = crate_entry(&package("b", "MIT"), &[mit("B")]);

    assert_eq!(render(&[a.clone(), b.clone()]), render(&[b, a]));
}

#[test]
fn render_names_a_text_by_its_file_when_it_fits_no_canonical_text() {
    let entry = crate_entry(
        &package("a", "Unlicense"),
        &[("UNLICENSE".into(), "Free.".into())],
    );

    assert!(render(&[entry]).contains("License: UNLICENSE\n"));
}

#[test]
fn generate_reads_each_crates_files_and_adds_the_compiled_in_code() {
    let text = generate("x86_64-unknown-linux-musl", &[package("a", "MIT")], |_| {
        Ok(vec![mit("A")])
    })
    .unwrap();

    for name in [
        "a 1.0.0 (MIT)",
        "SQLite",
        "Rust standard library",
        "musl C library",
    ] {
        assert!(text.contains(name), "{name}");
    }
    assert_eq!(
        generate("t", &[package("a", "MIT")], |_| Err("read a: gone".into())).unwrap_err(),
        "read a: gone"
    );
}

fn generated(files: &[(&str, &str)]) -> BTreeMap<String, String> {
    files
        .iter()
        .map(|(t, f)| ((*t).to_owned(), (*f).to_owned()))
        .collect()
}

#[test]
fn committed_problems_names_a_missing_a_stale_and_a_stray_file() {
    let (_dir, c) = checkout(&[
        ("licenses/a/THIRD_PARTY_LICENSES", "same"),
        ("licenses/b/THIRD_PARTY_LICENSES", "old"),
        ("licenses/stray/THIRD_PARTY_LICENSES", "x"),
    ]);
    let want = generated(&[("a", "same"), ("b", "new"), ("c", "new")]);

    assert_eq!(
        committed_problems(&c, &want),
        [
            "c has no committed licenses/c/THIRD_PARTY_LICENSES; run license-check --write",
            "licenses/b/THIRD_PARTY_LICENSES differs from its regeneration; run license-check --write",
            "licenses/stray is no release target's directory",
        ]
    );
    let (_dir, none) = checkout(&[("README.md", "x")]);
    assert_eq!(
        committed_problems(&none, &generated(&[("a", "x")])).len(),
        1
    );
    assert!(
        committed_problems(
            &c,
            &generated(&[("a", "same"), ("b", "old"), ("stray", "x")])
        )
        .is_empty()
    );
}

#[test]
fn write_creates_each_targets_file_and_names_what_it_cannot_write() {
    let (dir, _) = checkout(&[]);
    write(dir.path(), &generated(&[("a", "one"), ("b", "two")])).unwrap();

    assert_eq!(
        fs::read_to_string(dir.path().join("licenses/b/THIRD_PARTY_LICENSES")).unwrap(),
        "two"
    );
    let (blocked, _) = checkout(&[("licenses", "a file where the directory goes")]);
    assert!(
        write(blocked.path(), &generated(&[("a", "x")]))
            .unwrap_err()
            .starts_with("create ")
    );
    let (clash, _) = checkout(&[("licenses/a/THIRD_PARTY_LICENSES/inner", "x")]);
    assert!(
        write(clash.path(), &generated(&[("a", "x")]))
            .unwrap_err()
            .starts_with("write ")
    );
}

const GRAPH: &str = r#"{"workspace_root":"/w","packages":[
  {"id":"app 0","name":"app","version":"0.0.0","source":null,"manifest_path":"/w/crates/app/Cargo.toml","targets":[{"kind":["bin"]}]},
  {"id":"win 1","name":"win","version":"1.0.0","source":"registry+x","license":"MIT","manifest_path":"/r/win/Cargo.toml","targets":[{"kind":["lib"]}]}
 ],"resolve":{"nodes":[{"id":"app 0","deps":[{"pkg":"win 1","dep_kinds":[{"kind":null}]}]}]}}"#;

#[test]
fn linked_per_target_reads_each_targets_metadata() {
    let per = linked_per_target(&["a".to_owned(), "b".to_owned()], |t| {
        Ok(if t == "a" {
            GRAPH.to_owned()
        } else {
            GRAPH
                .replace("win 1", "other 1")
                .replace("\"win\"", "\"other\"")
        })
    })
    .unwrap();

    assert_eq!(per["a"][0].name, "win");
    assert_eq!(per["b"][0].name, "other");
    assert_eq!(
        linked_per_target(&["a".to_owned()], |t| Err(format!("no {t}"))).unwrap_err(),
        "no a"
    );
}

#[test]
fn run_release_writes_then_passes_and_a_stale_file_fails() {
    let (_dir, c) = workspace("[licenses]\nallow = [\"MIT\"]\n");
    let with = Options {
        write: true,
        out: None,
    };

    assert_eq!(
        run_release(&cargo(), &c.root, &with).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(
        run_release(&cargo(), &c.root, &Options::default()).unwrap(),
        Vec::<String>::new()
    );

    let path = c.root.join(committed_path("x86_64-unknown-linux-musl"));
    fs::write(&path, "stale").unwrap();
    assert_eq!(
        run_release(&cargo(), &c.root, &Options::default()).unwrap(),
        [
            "licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES differs from its regeneration; run license-check --write"
        ]
    );
}

#[test]
fn run_release_can_write_the_regeneration_elsewhere_and_leaves_the_committed_files() {
    let (dir, c) = workspace("[licenses]\nallow = [\"MIT\"]\n");
    let out = dir.path().join("regenerated");
    let options = Options {
        write: false,
        out: Some(out.clone()),
    };

    let problems = run_release(&cargo(), &c.root, &options).unwrap();

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        out.join("licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES")
            .is_file()
    );
    assert!(!c.root.join("licenses").exists());
}

#[test]
fn run_release_reports_the_records_it_checks_and_names_what_it_cannot_read() {
    let (_dir, c) = workspace("[licenses]\nallow = [\"WTFPL\"]\n");
    let write = Options {
        write: true,
        out: None,
    };
    assert_eq!(
        run_release(&cargo(), &c.root, &write).unwrap(),
        ["deny.toml grants WTFPL, which the license tool keeps no text for"]
    );

    let (_dir, none) = checkout(&[("Cargo.toml", "")]);
    assert!(
        run_release(&cargo(), &none.root, &Options::default())
            .unwrap_err()
            .starts_with("read .github/workflows/build.yml: ")
    );
    let (_dir, broken) = checkout(&[
        ("Cargo.toml", "not toml"),
        (
            ".github/workflows/build.yml",
            crate::license_texts::tests::BUILD_FIXTURE,
        ),
    ]);
    assert!(
        run_release(&cargo(), &broken.root, &Options::default())
            .unwrap_err()
            .starts_with("cargo metadata for ")
    );
}

#[test]
fn a_verbatim_text_keeps_its_own_copyright_lines() {
    let entries = non_crate("x86_64-pc-windows-gnu");
    let text = render(&entries);

    assert!(text.contains("Copyright (c) 2002 Todd C. Miller"));
    assert!(text.contains("Copyright (C) 2007 Free Software Foundation, Inc."));
    assert!(text.contains("GCC RUNTIME LIBRARY EXCEPTION"));
    assert!(entries.iter().filter(|e| e.verbatim).count() == 2);
    assert_eq!(published("\n a  \r\nb   \n\n"), " a\nb");
}
