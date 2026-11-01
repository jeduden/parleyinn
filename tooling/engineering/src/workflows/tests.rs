use super::*;

const SHA: &str = "3d3c42e5aac5ba805825da76410c181273ba90b1";

#[test]
fn unpinned_takes_a_full_commit_sha_with_its_version_and_local_actions() {
    let body = format!(
        "steps:\n      - uses: actions/checkout@{SHA} # v7.0.1\n        uses: ./.github/actions/setup-mise\n    uses: \"owner/flow/.github/workflows/x.yml@{SHA}\" # v1\n      - uses: docker://alpine@sha256:{}\n",
        "a".repeat(64)
    );

    assert!(unpinned("w.yml", &body).is_empty());
}

#[test]
fn unpinned_names_a_tag_a_short_sha_a_missing_comment_and_an_undigested_image() {
    let body = format!(
        "      - uses: actions/checkout@v4\n        uses: owner/act@3d3c42e # v1\n        uses: owner/act@{SHA}\n        uses: owner/act\n        uses: docker://alpine:3.20\n        uses: 'owner/act@main' # v1\n"
    );

    assert_eq!(
        unpinned("w.yml", &body),
        [
            "w.yml:1 uses actions/checkout@v4, not pinned to a full commit sha".to_owned(),
            "w.yml:2 uses owner/act@3d3c42e, not pinned to a full commit sha".to_owned(),
            format!("w.yml:3 uses owner/act@{SHA}, with no version in a comment"),
            "w.yml:4 uses owner/act, not pinned to a full commit sha".to_owned(),
            "w.yml:5 uses docker://alpine:3.20, not pinned to a sha256 digest".to_owned(),
            "w.yml:6 uses owner/act@main, not pinned to a full commit sha".to_owned(),
        ]
    );
}

#[test]
fn unpinned_names_a_sha_of_forty_non_hex_characters() {
    let body = format!("        uses: owner/act@{} # v1\n", "g".repeat(40));

    assert_eq!(
        unpinned("w.yml", &body),
        [format!(
            "w.yml:1 uses owner/act@{}, not pinned to a full commit sha",
            "g".repeat(40)
        )]
    );
}

#[test]
fn unpinned_names_a_docker_digest_too_short_or_not_hex() {
    let body = format!(
        "        uses: docker://alpine@sha256:abc\n        uses: docker://alpine@sha256:{}\n",
        "z".repeat(64)
    );

    assert_eq!(
        unpinned("w.yml", &body),
        [
            "w.yml:1 uses docker://alpine@sha256:abc, not pinned to a sha256 digest".to_owned(),
            format!(
                "w.yml:2 uses docker://alpine@sha256:{}, not pinned to a sha256 digest",
                "z".repeat(64)
            ),
        ]
    );
}

#[test]
fn unpinned_names_a_sha_whose_version_comment_is_empty() {
    let body = format!("        uses: owner/act@{SHA} #\n        uses: owner/act@{SHA} #   \n");

    assert_eq!(
        unpinned("w.yml", &body),
        [
            format!("w.yml:1 uses owner/act@{SHA}, with no version in a comment"),
            format!("w.yml:2 uses owner/act@{SHA}, with no version in a comment"),
        ]
    );
}

#[test]
fn unpinned_reads_no_uses_a_comment_names() {
    assert!(unpinned("w.yml", "      # uses: actions/checkout@v4\n  uses:\n").is_empty());
}

#[test]
fn run_scripts_reads_inline_and_block_scripts_with_their_first_line() {
    let body = "steps:\n  - run: cargo test # a comment\n  - name: build\n    run: |\n      echo one\n\n      echo two\n    env:\n      A: b\n  - run: >-\n      folded\n";

    assert_eq!(
        run_scripts(body),
        [
            (2, "cargo test".to_owned()),
            (5, "      echo one\n\n      echo two".to_owned()),
            (11, "      folded".to_owned()),
        ]
    );
}

#[test]
fn untrusted_expansions_names_each_untrusted_context_a_script_expands() {
    let body = "    steps:\n      - run: echo \"${{ github.event.pull_request.title }}\"\n      - run: |\n          echo ok\n          git switch ${{ github.head_ref }} ${{inputs.ref}}\n          x=${{ github.event['comment'] }}\n";

    assert_eq!(
        untrusted_expansions("w.yml", body),
        [
            "w.yml:2 expands github.event.pull_request.title in a run script, not through env",
            "w.yml:5 expands github.head_ref in a run script, not through env",
            "w.yml:5 expands inputs.ref in a run script, not through env",
            "w.yml:6 expands github.event['comment'] in a run script, not through env",
        ]
    );
}

#[test]
fn untrusted_expansions_reads_a_context_in_any_case_and_through_brackets() {
    let body = "      - run: |\n          echo ${{ GITHUB.head_ref }}\n          echo ${{ github['head_ref'] }}\n          echo ${{ github[\"Event\"].issue.title }}\n          echo ${{ Inputs.ref }}\n          echo ${{ github [ 'event' ] }} ${{ github['event_name'] }}\n";

    assert_eq!(
        untrusted_expansions("w.yml", body),
        [
            "w.yml:2 expands GITHUB.head_ref in a run script, not through env",
            "w.yml:3 expands github['head_ref'] in a run script, not through env",
            "w.yml:4 expands github[\"Event\"].issue.title in a run script, not through env",
            "w.yml:5 expands Inputs.ref in a run script, not through env",
            "w.yml:6 expands github [ 'event' ] in a run script, not through env",
        ]
    );
}

#[test]
fn untrusted_expansions_passes_trusted_contexts_env_and_values_outside_scripts() {
    let body = "    name: ${{ github.event.pull_request.title }}\n    steps:\n      - env:\n          TITLE: ${{ github.event.pull_request.title }}\n        run: |\n          echo \"$TITLE\" ${{ matrix.shard }} ${{ github.event_name }} ${{ steps.inputs_x.outputs.y }} ${{ steps.a.outputs.l[0] }} ${{ x[ 'a' }} ${{ y['b }}\n          echo ${{ unclosed\n";

    assert!(untrusted_expansions("w.yml", body).is_empty());
}

#[test]
fn runs_zizmor_wants_a_step_that_runs_it() {
    assert!(runs_zizmor("      - run: zizmor .github\n").is_empty());
    assert!(runs_zizmor("      - run: |\n          zizmor .github\n").is_empty());
    assert_eq!(
        runs_zizmor("  # run: zizmor .github\n      - run: zizmor --version\n"),
        [r#"the CI workflow does not run "zizmor .github""#]
    );
}

#[test]
fn files_lists_workflows_and_composite_actions() {
    let (_dir, c) = crate::tests::checkout(&[
        (".github/workflows/ci.yml", ""),
        (".github/workflows/b.yaml", ""),
        (".github/workflows/notes.md", ""),
        (".github/actions/one/action.yml", ""),
        (".github/actions/two/action.yaml", ""),
        (".github/actions/three/README.md", ""),
    ]);

    assert_eq!(
        files(&c).unwrap(),
        [
            ".github/actions/one/action.yml",
            ".github/actions/two/action.yaml",
            ".github/workflows/b.yaml",
            ".github/workflows/ci.yml",
        ]
    );
}

#[test]
fn files_without_actions_lists_the_workflows_and_without_workflows_fails() {
    let (_dir, c) = crate::tests::checkout(&[(".github/workflows/ci.yml", "")]);
    assert_eq!(files(&c).unwrap(), [".github/workflows/ci.yml"]);

    let (_dir, c) = crate::tests::checkout(&[("README.md", "")]);
    assert!(
        files(&c)
            .unwrap_err()
            .starts_with("read .github/workflows: ")
    );
}

#[test]
fn check_names_every_problem_of_every_file() {
    let (_dir, c) = crate::tests::checkout(&[
        (
            ".github/workflows/ci.yml",
            "      - uses: actions/checkout@v4\n      - run: zizmor .github\n",
        ),
        (
            ".github/actions/one/action.yml",
            "    - run: echo ${{ inputs.tools }}\n      shell: bash\n",
        ),
    ]);

    assert_eq!(
        check(&c),
        [
            ".github/actions/one/action.yml:1 expands inputs.tools in a run script, not through env",
            ".github/workflows/ci.yml:1 uses actions/checkout@v4, not pinned to a full commit sha",
        ]
    );
}

#[test]
fn check_fails_on_no_workflow_or_an_unreadable_one() {
    let (_dir, c) = crate::tests::checkout(&[(".github/workflows/notes.md", "")]);
    assert_eq!(check(&c), [".github/workflows holds no workflow"]);

    let (_dir, c) = crate::tests::checkout(&[("README.md", "")]);
    assert!(check(&c)[0].starts_with("read .github/workflows: "));

    let (dir, c) = crate::tests::checkout(&[(".github/workflows/ci.yml", "")]);
    fs::create_dir(dir.path().join(".github/workflows/dir.yml")).unwrap();
    assert!(check(&c)[0].starts_with("read .github/workflows/dir.yml: "));
}
