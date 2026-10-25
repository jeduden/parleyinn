use super::*;

#[test]
fn rust_toolchain_pinned_wants_an_exact_release() {
    assert!(rust_toolchain_pinned("[toolchain]\nchannel = \"1.99.0\"\n").is_ok());
    for (toml, want) in [
        (
            "[toolchain]\nchannel = \"stable\"\n",
            r#"does not pin an exact release: "stable""#,
        ),
        (
            "[toolchain]\nchannel = \"1.99\"\n",
            r#"does not pin an exact release: "1.99""#,
        ),
        (
            "[toolchain]\nchannel = \"1.x.0\"\n",
            r#"does not pin an exact release: "1.x.0""#,
        ),
        ("[toolchain]\n", "names no channel"),
    ] {
        assert_eq!(
            rust_toolchain_pinned(toml).unwrap_err(),
            format!("rust-toolchain.toml {want}")
        );
    }
}

#[test]
fn release_build_reads_the_workflow_and_the_workflows_it_calls() {
    let (_dir, c) = crate::tests::checkout(&[
        (
            ".github/workflows/release.yml",
            "jobs:\n  build:\n    uses: ./.github/workflows/build.yml\n  x:\n    uses: actions/checkout@v1\n",
        ),
        (
            ".github/workflows/build.yml",
            "run: cargo build --remap-path-prefix\n",
        ),
    ]);

    let text = release_build(&c, ".github/workflows/release.yml").unwrap();

    assert!(text.contains("uses: ./.github/workflows/build.yml"));
    assert!(text.contains("--remap-path-prefix"));
}

#[test]
fn release_build_follows_a_called_workflow_s_own_calls_once_each() {
    let (_dir, c) = crate::tests::checkout(&[
        (
            ".github/workflows/release.yml",
            "jobs:\n  build:\n    uses: ./.github/workflows/release-build.yml\n",
        ),
        (
            ".github/workflows/release-build.yml",
            "jobs:\n  build:\n    uses: ./.github/workflows/build.yml\n  again:\n    uses: ./.github/workflows/release.yml\n",
        ),
        (
            ".github/workflows/build.yml",
            "run: cargo build --remap-path-prefix\n",
        ),
    ]);

    let text = release_build(&c, ".github/workflows/release.yml").unwrap();

    assert!(text.contains("--remap-path-prefix"));
    assert_eq!(
        text.matches("uses: ./.github/workflows/build.yml").count(),
        1
    );
    assert_eq!(
        text.matches("uses: ./.github/workflows/release.yml")
            .count(),
        1
    );
}

#[test]
fn release_build_names_a_called_workflow_it_cannot_read() {
    let (_dir, c) = crate::tests::checkout(&[(
        ".github/workflows/release.yml",
        "    uses: ./.github/workflows/gone.yml\n",
    )]);

    let err = release_build(&c, ".github/workflows/release.yml").unwrap_err();

    assert!(err.starts_with("read .github/workflows/gone.yml"), "{err}");
}

#[test]
fn builds_with_wants_the_flag_in_the_release_build() {
    assert_eq!(
        builds_with("a --remap-path-prefix b", "--remap-path-prefix"),
        Ok(())
    );
    assert_eq!(
        builds_with("a b", "PARINN_COMMIT"),
        Err(r#"the release build does not contain "PARINN_COMMIT""#.to_owned())
    );
}

#[test]
fn release_build_leaves_out_comments() {
    let (_dir, c) = crate::tests::checkout(&[(
        ".github/workflows/release.yml",
        "# --remap-path-prefix keeps paths out\nrun: cargo build # --remap-path-prefix\n",
    )]);

    let text = release_build(&c, ".github/workflows/release.yml").unwrap();

    assert_eq!(text, "run: cargo build\n");
}
