use std::fs;

use super::*;
use testkit::TempDir;

/// The lines of `body` (a Markdown file) the scan flags.
fn flagged(body: &str) -> Vec<usize> {
    directives("X.md", body)
        .iter()
        .map(|d| {
            let line = d.strip_prefix("X.md:").unwrap();
            line.split(':').next().unwrap().parse().unwrap()
        })
        .collect()
}

/// A fenced shell block holding `lines`.
fn fenced(lines: &str) -> String {
    format!("Run:\n\n```sh\n{lines}\n```\n")
}

#[test]
fn a_destructive_command_at_the_real_home_is_flagged_with_its_line() {
    let got = directives(
        "CLAUDE.md",
        "Intro.\n\nClean up with `rm -rf ~/.parinn` first.\n",
    );

    assert_eq!(
        got,
        vec![
            "CLAUDE.md:3: directs a destructive command at non-isolated state: \
             Clean up with `rm -rf ~/.parinn` first."
                .to_owned()
        ]
    );
}

#[test]
fn every_way_to_reach_the_real_home_counts() {
    for target in [
        "~",
        "~/.claude",
        "~/.claude.json",
        "$HOME/.parinn",
        "${HOME}/x",
        "$PARINN_HOME",
        "${PARINN_HOME}/log",
        "$CLAUDE_CONFIG_DIR",
        "$USERPROFILE",
        "\"$HOME\"",
        "(~)",
    ] {
        let body = fenced(&format!("rm -rf {target}"));

        assert_eq!(flagged(&body), vec![4], "{target}");
    }
}

#[test]
fn every_way_to_destroy_counts() {
    for command in [
        "rm -rf ~/x",
        "rmdir ~/x",
        "unlink ~/x",
        "shred ~/x",
        "truncate -s 0 ~/x",
        "trash ~/x",
        "find ~ -name x -delete",
        "git clean -fdx ~",
        "parinn purge --home ~/.parinn",
        "parinn uninstall ~/.claude",
        "sed -i s/a/b/ ~/.claude/settings.json",
        "echo x | tee ~/.claude/x",
        "echo {} > ~/.claude/settings.json",
        "echo {} >| ~/.claude/settings.json",
        "echo {} >> $HOME/.claude.json",
        "echo {} 2> ~/.parinn/err",
    ] {
        assert_eq!(flagged(&fenced(command)), vec![4], "{command}");
    }
}

#[test]
fn prose_that_tells_an_agent_to_destroy_is_flagged() {
    for sentence in [
        "Delete `~/.parinn` before the run.",
        "Wipe `$PARINN_HOME` between runs.",
        "Remove the `HOME` directory afterwards.",
        "Then overwrite `~/.claude/settings.json` with the new one.",
        "Purge `PARINN_HOME` when done.",
    ] {
        assert_eq!(flagged(sentence), vec![1], "{sentence}");
    }
}

#[test]
fn a_statement_that_forbids_the_act_is_no_directive() {
    for sentence in [
        "Never run a command that deletes from or purges `~/.parinn`.",
        "Do not remove `$HOME`.",
        "Don't delete `~/.claude`.",
        "No instruction may direct an agent to wipe `~`.",
        "It must not overwrite `HOME`.",
        "Tests refuse to delete the real `PARINN_HOME`.",
        "Avoid rm -rf ~.",
        "The guard cannot remove `$HOME`.",
        "A test can’t delete `$HOME`.",
    ] {
        assert_eq!(flagged(sentence), Vec::<usize>::new(), "{sentence}");
    }
}

#[test]
fn a_statement_that_isolates_first_is_no_directive() {
    for sentence in [
        "Point `HOME` and `PARINN_HOME` at a temporary directory, then delete `$HOME/.parinn`.",
        "Remove `~/.parinn` of the isolated home.",
        "Delete everything in `$HOME` made with mktemp.",
    ] {
        assert_eq!(flagged(sentence), Vec::<usize>::new(), "{sentence}");
    }
}

#[test]
fn a_command_isolates_by_assigning_a_temporary_home() {
    for command in [
        "HOME=$(mktemp -d) rm -rf $HOME/.parinn",
        "PARINN_HOME=\"$TMPDIR/c\" parinn purge ~/.parinn",
        "export HOME=/tmp/h; rm -rf ~/.parinn",
        "env HOME=$tmp/home rm -rf ~/x",
    ] {
        assert_eq!(flagged(&fenced(command)), Vec::<usize>::new(), "{command}");
    }
}

#[test]
fn a_real_home_assignment_does_not_isolate() {
    for command in [
        "HOME=/root rm -rf ~",
        "PARINN_HOME=$HOME/.parinn parinn purge",
    ] {
        assert_eq!(flagged(&fenced(command)), vec![4], "{command}");
    }
}

#[test]
fn a_block_isolates_the_lines_after_its_assignment_only() {
    let body = fenced("rm -rf ~/before\nexport HOME=$(mktemp -d)\nrm -rf ~/after");

    assert_eq!(flagged(&body), vec![4]);
}

#[test]
fn a_second_block_starts_with_the_real_home_again() {
    let body = format!(
        "{}\n{}",
        fenced("export HOME=$(mktemp -d)\nrm -rf ~/a"),
        fenced("rm -rf ~/b")
    );

    assert_eq!(flagged(&body), vec![11]);
}

#[test]
fn state_inside_the_repository_is_no_real_home() {
    for command in [
        "rm -rf target",
        "rm -rf .claude/worktrees/x",
        "rm -rf ./.parinn-test",
        "git clean -fdx",
        "echo done > out.txt",
        "cargo test 2>&1 | tee run.log",
        "cmd -> ~5 files removed",
        "rm -rf $XDG_CONFIG_HOME/x",
        "~~rm -rf~~ nothing",
    ] {
        assert_eq!(flagged(&fenced(command)), Vec::<usize>::new(), "{command}");
    }
}

#[test]
fn reading_the_real_home_is_no_destruction() {
    for command in [
        "ls ~/.claude",
        "cat $HOME/.claude.json",
        "cd ~",
        "echo $HOME",
    ] {
        assert_eq!(flagged(&fenced(command)), Vec::<usize>::new(), "{command}");
    }
}

#[test]
fn a_wrapped_paragraph_is_read_as_one_statement() {
    let body = "Never run a command that deletes from, or\npurges the real `~/.parinn` of the\nmachine you run on.\n";

    assert_eq!(flagged(body), Vec::<usize>::new());
}

#[test]
fn a_wrapped_directive_is_flagged_at_its_first_line() {
    let body = "Intro.\n\nSometimes you should delete\nthe real `~/.parinn` by hand.\n";

    assert_eq!(flagged(body), vec![3]);
}

#[test]
fn each_sentence_stands_alone() {
    let body = "Never rm `~/.parinn`. Then rm -rf `~/.claude`! Done?\n";

    assert_eq!(directives("X.md", body).len(), 1);
}

#[test]
fn each_list_item_stands_alone() {
    let body = "- Never rm `~/x`\n* rm -rf `~/.parinn`\n1. Never rm `~/y`\n2. wipe `$HOME`\n";

    assert_eq!(flagged(body), vec![2, 4]);
}

#[test]
fn a_table_row_and_a_heading_stand_alone() {
    let body = "# Never delete\n| a | rm ~/x |\n| b | fine |\n";

    assert_eq!(flagged(body), vec![2]);
}

#[test]
fn a_blank_line_ends_a_paragraph() {
    let body = "Never\n\ndelete `~/.parinn`\n";

    assert_eq!(flagged(body), vec![3]);
}

#[test]
fn front_matter_is_read_as_a_paragraph_from_its_first_key() {
    let body = "---\nname: x\ndescription: rm -rf ~/.parinn\n---\nBody.\n";

    assert_eq!(flagged(body), vec![2]);
}

#[test]
fn a_tilde_fence_opens_a_block_too() {
    let body = "~~~\nrm -rf ~\n~~~\n";

    assert_eq!(flagged(body), vec![2]);
}

#[test]
fn a_fence_inside_a_list_item_opens_a_block() {
    let body = "1. Run:\n\n   ```sh\n   rm -rf ~\n   ```\n";

    assert_eq!(flagged(body), vec![4]);
}

#[test]
fn a_file_that_is_not_markdown_is_read_line_by_line() {
    let body = "{\n  \"hooks\": [\n    \"rm -rf ~/.claude/cache\",\n    \"ls\"\n  ]\n}\n";

    assert_eq!(
        directives("settings.json", body),
        vec![
            "settings.json:3: directs a destructive command at non-isolated state: \
             \"rm -rf ~/.claude/cache\","
                .to_owned()
        ]
    );
}

#[test]
fn a_destructive_command_inside_a_quoted_example_is_still_one() {
    let body = "Example: `rm -rf ~/.parinn`\n";

    assert_eq!(flagged(body), vec![1]);
}

/// A checkout holding `files`, each with a line that would be flagged.
fn checkout(files: &[&str]) -> (TempDir, Checkout) {
    let dir = TempDir::new().unwrap();
    for file in files {
        let path = dir.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "Run `rm -rf ~/.parinn`.\n").unwrap();
    }
    let c = Checkout::new(dir.path());
    (dir, c)
}

#[test]
fn the_instruction_files_are_found_wherever_the_repository_keeps_them() {
    let (dir, _) = checkout(&[
        "CLAUDE.md",
        "AGENTS.md",
        "docs/CLAUDE.md",
        ".claude/agents/a.md",
        ".claude/skills/s/SKILL.md",
        ".claude/settings.json",
        "crates/x/.claude/skills/t.md",
        "README.md",
        "docs/notes.md",
        "src/CLAUDE.md.bak",
    ]);

    assert_eq!(
        instruction_files(dir.path()).unwrap(),
        [
            ".claude/agents/a.md",
            ".claude/settings.json",
            ".claude/skills/s/SKILL.md",
            "AGENTS.md",
            "CLAUDE.md",
            "crates/x/.claude/skills/t.md",
            "docs/CLAUDE.md",
        ]
    );
}

#[test]
fn another_checkout_a_build_and_a_developer_s_own_settings_are_left_out() {
    let (dir, _) = checkout(&[
        ".claude/worktrees/w/CLAUDE.md",
        ".claude/worktrees/w/.claude/agents/a.md",
        ".claude/settings.local.json",
        "target/debug/CLAUDE.md",
        ".git/CLAUDE.md",
        "node_modules/p/CLAUDE.md",
        ".claude/kept.md",
    ]);

    assert_eq!(instruction_files(dir.path()).unwrap(), [".claude/kept.md"]);
}

#[test]
fn a_directory_that_cannot_be_listed_is_named() {
    let err = instruction_files(Path::new("/no/such/dir")).unwrap_err();

    assert!(err.starts_with("list /no/such/dir: "), "{err}");
}

#[test]
fn a_checkout_is_scanned_file_by_file() {
    let (_dir, c) = checkout(&["CLAUDE.md", ".claude/agents/a.md"]);

    let got = no_destructive_instructions(&c);

    assert_eq!(got.len(), 2);
    assert!(got[0].starts_with(".claude/agents/a.md:1: "), "{got:?}");
    assert!(got[1].starts_with("CLAUDE.md:1: "), "{got:?}");
}

#[test]
fn a_checkout_with_clean_instructions_passes() {
    let (dir, c) = checkout(&[]);
    fs::write(dir.path().join("CLAUDE.md"), "Never rm `~/.parinn`.\n").unwrap();

    assert_eq!(no_destructive_instructions(&c), Vec::<String>::new());
}

#[test]
fn a_checkout_that_cannot_be_listed_is_reported() {
    let c = Checkout::new(Path::new("/no/such/dir"));

    assert_eq!(no_destructive_instructions(&c).len(), 1);
}

#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_read_is_reported() {
    let (dir, c) = checkout(&["CLAUDE.md"]);
    fs::remove_file(dir.path().join("CLAUDE.md")).unwrap();
    std::os::unix::fs::symlink("nowhere", dir.path().join("CLAUDE.md")).unwrap();

    let got = no_destructive_instructions(&c);

    assert_eq!(got.len(), 1);
    assert!(got[0].starts_with("read CLAUDE.md: "), "{got:?}");
}

#[test]
fn text_that_is_not_utf8_is_read_lossily() {
    let (dir, c) = checkout(&[]);
    fs::write(dir.path().join("CLAUDE.md"), b"\xff rm -rf ~/x\n").unwrap();

    assert_eq!(no_destructive_instructions(&c).len(), 1);
}
