use std::collections::BTreeMap;
use std::fs;

use review_eval::component::Kind;
use review_eval::log::{RunLog, Usage};
use testkit::TempDir;

use super::*;

/// A file's section of a diff, `size` bytes long, for `path`.
fn section(path: &str, size: usize) -> String {
    let head = format!("diff --git a/{path} b/{path}\n");
    let body = "+".repeat(size - head.len() - 1);
    format!("{head}{body}\n")
}

/// A work directory holding `diff` as `name`'s.
fn with_diff(name: &str, diff: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let diffs = dir.path().join("review/diffs");
    fs::create_dir_all(&diffs).unwrap();
    fs::write(diffs.join(format!("{name}.diff")), diff).unwrap();
    dir
}

#[test]
fn the_limit_keeps_a_part_within_one_read_of_the_agent_s_file_tool() {
    assert_eq!(LIMIT, 48 * 1024);
}

#[test]
fn a_diff_within_the_limit_is_not_split_and_writes_no_part() {
    let diff = section("a.rs", 100);
    let dir = with_diff("lang-rust", &diff);

    assert_eq!(prepare(dir.path(), "lang-rust", 100).unwrap(), Vec::new());
    assert!(!dir.path().join("review/diffs/lang-rust.1.diff").exists());
}

#[test]
fn a_specialist_without_a_diff_is_not_split() {
    let dir = TempDir::new().unwrap();

    assert_eq!(prepare(dir.path(), "lang-rust", 100).unwrap(), Vec::new());
}

#[test]
fn a_diff_that_does_not_read_fails_the_split() {
    let dir = TempDir::new().unwrap();
    let diff = dir.path().join("review/diffs/lang-rust.diff");
    fs::create_dir_all(&diff).unwrap();

    assert_eq!(
        prepare(dir.path(), "lang-rust", 100),
        Err(format!(
            "read {}: Is a directory (os error 21)",
            diff.display()
        ))
    );
}

#[test]
fn a_diff_past_the_limit_is_written_in_parts_each_told_every_part_s_files() {
    let diff = [
        section("b.rs", 60),
        section("a.rs", 60),
        section("c.rs", 30),
    ]
    .concat();
    let dir = with_diff("lang-rust", &diff);

    let parts = prepare(dir.path(), "lang-rust", 100).unwrap();

    assert_eq!(parts.len(), 2);
    assert_eq!((parts[1].index, parts[1].count), (2, 2));
    let read = |i: usize| {
        fs::read_to_string(dir.path().join(format!("review/diffs/lang-rust.{i}.diff"))).unwrap()
    };
    assert_eq!(read(1), section("a.rs", 60));
    assert_eq!(read(2), [section("b.rs", 60), section("c.rs", 30)].concat());
    assert_eq!(
        parts[1].note,
        "Your diff is too large for one run, so the review runs you in 2 parts and \
         merges their reports: you are part 2 of 2. Read only your part's diff, \
         `review/diffs/lang-rust.2.diff`, not `review/diffs/lang-rust.diff`, and state \
         findings on its files; you may Read and Grep the rest of the tree. The full \
         diff's files, by part: part 1: a.rs; part 2: b.rs, c.rs."
    );
    assert_eq!(parts[1].diff, "review/diffs/lang-rust.2.diff");
    assert_eq!(parts[1].label("lang-rust"), "lang-rust part 2 of 2");
    assert_eq!(parts[1].stem("lang-rust"), "lang-rust.2");
}

#[test]
fn a_part_that_cannot_be_written_fails_the_split() {
    let diff = [section("a.rs", 60), section("b.rs", 60)].concat();
    let dir = with_diff("lang-rust", &diff);
    fs::create_dir_all(dir.path().join("review/diffs/lang-rust.1.diff")).unwrap();

    let err = prepare(dir.path(), "lang-rust", 100).unwrap_err();

    assert!(err.starts_with("write "), "{err}");
    assert!(err.contains("lang-rust.1.diff"), "{err}");
}

/// A part's run of `name`: answered with `answer` at `cost`, or failed
/// for `failure`.
fn part_run(failure: Option<&str>, answer: &str, cost: f64) -> AgentRun {
    let mut usage = BTreeMap::new();
    usage.insert(
        "claude-haiku-5-5".to_owned(),
        Usage {
            calls: 1,
            fresh: 2,
            cache_write: 3,
            cache_read: 4,
            output: 5,
            cost,
        },
    );
    AgentRun {
        name: "lang-rust part".to_owned(),
        kind: Kind::Specialist,
        failure: failure.map(str::to_owned),
        log: RunLog {
            answered: failure.is_none(),
            resulted: true,
            subtype: if failure.is_none() {
                "success"
            } else {
                "error_max_turns"
            }
            .to_owned(),
            answer: answer.to_owned(),
            cost,
            turns: 3,
            duration_ms: 10,
            resolved_model: "claude-haiku-5-5".to_owned(),
            usage,
            texts: vec![answer.to_owned()],
            writes: Vec::new(),
            reads: Vec::new(),
            cwd: String::new(),
            searches: Vec::new(),
        },
        charged: cost,
        ..AgentRun::of(String::new(), Kind::Specialist, None, RunLog::default(), 1)
    }
}

const A: &str = "src/a.rs:3 — if not addressed: a long line hides a bug — I9";
const B: &str = "src/b.rs:5 — if not addressed: a panic stops the hook — I9";

#[test]
fn the_merged_report_is_the_union_of_the_parts_in_order_with_one_outcome() {
    let parts = vec![
        (
            part_run(None, "outcome: findings — one.", 0.25),
            format!("{A}\noutcome: findings — one.\n"),
        ),
        (
            part_run(None, "outcome: none — clean.", 0.5),
            "outcome: none — clean.\n".to_owned(),
        ),
        (
            part_run(None, "outcome: findings — two.", 0.25),
            format!("{B}\n{A}\noutcome: findings — two.\n"),
        ),
    ];

    let (run, report) = merge("lang-rust", parts);

    let outcome = "outcome: findings — in 3 parts: part 1, findings — one.; part 2, none — \
                   clean.; part 3, findings — two.";
    assert_eq!(report, format!("{A}\n{B}\n{outcome}\n"));
    assert_eq!(run.name, "lang-rust");
    assert_eq!(run.kind, Kind::Specialist);
    assert!(run.answered());
    assert_eq!(run.log.answer, outcome);
    assert!(run.log.answered);
    assert_eq!(run.log.subtype, "success");
    assert!((run.log.cost - 1.0).abs() < 1e-9);
    assert_eq!((run.log.turns, run.log.duration_ms), (9, 30));
    assert_eq!(run.log.resolved_model, "claude-haiku-5-5");
    assert_eq!(run.log.usage["claude-haiku-5-5"].calls, 3);
    assert_eq!(run.log.usage["claude-haiku-5-5"].output, 15);
    assert_eq!(run.log.texts.len(), 3);
    assert_eq!((run.attempts, run.parts), (3, 3));
}

#[test]
fn parts_that_all_answer_none_merge_to_none() {
    let parts = vec![
        (
            part_run(None, "outcome: none — a.", 0.0),
            "outcome: none — a.\n".to_owned(),
        ),
        (
            part_run(None, "outcome: none — b.", 0.0),
            "outcome: none — b.\n".to_owned(),
        ),
    ];

    let (run, report) = merge("lang-rust", parts);

    let outcome = "outcome: none — in 2 parts: part 1, none — a.; part 2, none — b.";
    assert_eq!(report, format!("{outcome}\n"));
    assert_eq!(run.log.answer, outcome);
}

#[test]
fn a_part_without_an_outcome_leaves_the_merged_report_without_one() {
    let parts = vec![
        (
            part_run(None, "outcome: findings — one.", 0.0),
            format!("{A}\noutcome: findings — one.\n"),
        ),
        (part_run(None, "I looked.", 0.0), String::new()),
    ];

    let (run, report) = merge("lang-rust", parts);

    assert_eq!(report, format!("{A}\n"));
    assert!(run.answered());
    assert_eq!(run.log.answer, "");
}

#[test]
fn a_failed_part_makes_the_merged_run_incomplete_naming_it_and_keeps_every_part_s_findings() {
    let parts = vec![
        (
            part_run(None, "outcome: findings — one.", 0.25),
            format!("{A}\noutcome: findings — one.\n"),
        ),
        (
            part_run(Some("its run ended with error_max_turns"), "", 0.5),
            format!("incomplete: its run ended with error_max_turns\n{B}\n"),
        ),
    ];

    let (run, report) = merge("lang-rust", parts);

    assert_eq!(
        report,
        format!("incomplete: part 2 of 2: its run ended with error_max_turns\n{A}\n{B}\n")
    );
    assert!(!run.answered());
    assert_eq!(
        run.failure.as_deref(),
        Some("part 2 of 2: its run ended with error_max_turns")
    );
    assert!(!run.log.answered);
    assert_eq!(run.log.subtype, "error_max_turns");
    assert_eq!(run.log.answer, "");
    assert!((run.log.cost - 0.75).abs() < 1e-9);
    assert_eq!(
        run.failed_parts,
        [(2, "its run ended with error_max_turns".to_owned())]
    );
}

#[test]
fn every_failed_part_is_named() {
    let parts = vec![
        (part_run(Some("x"), "", 0.0), "incomplete: x\n".to_owned()),
        (part_run(Some("y"), "", 0.0), "incomplete: y\n".to_owned()),
    ];

    let (run, report) = merge("lang-rust", parts);

    assert_eq!(
        run.failure.as_deref(),
        Some("part 1 of 2: x; part 2 of 2: y")
    );
    assert_eq!(run.failed_parts, [(1, "x".to_owned()), (2, "y".to_owned())]);
    assert_eq!(report, "incomplete: part 1 of 2: x; part 2 of 2: y\n");
}

#[test]
fn a_merge_of_no_part_is_a_run_that_did_not_answer() {
    let (run, report) = merge("lang-rust", Vec::new());

    assert_eq!(run.failure.as_deref(), Some("it ran in no part"));
    assert_eq!(report, "incomplete: it ran in no part\n");
    assert_eq!((run.attempts, run.parts), (0, 0));
}

/// `run` with its model and usage on `model` alone.
fn on_model(mut run: AgentRun, model: &str) -> AgentRun {
    let usage = run.log.usage.remove("claude-haiku-5-5").unwrap();
    run.log.usage.insert(model.to_owned(), usage);
    run.log.resolved_model = model.to_owned();
    run
}

#[test]
fn a_merge_adds_every_usage_field_and_its_cost_by_model() {
    let parts = vec![
        (
            part_run(None, "outcome: none — a.", 0.25),
            "outcome: none — a.\n".to_owned(),
        ),
        (
            on_model(part_run(None, "outcome: none — b.", 0.5), "claude-opus-5-5"),
            "outcome: none — b.\n".to_owned(),
        ),
        (
            part_run(None, "outcome: none — c.", 0.125),
            "outcome: none — c.\n".to_owned(),
        ),
    ];

    let (run, _) = merge("lang-rust", parts);

    let haiku = Usage {
        calls: 2,
        fresh: 4,
        cache_write: 6,
        cache_read: 8,
        output: 10,
        cost: 0.375,
    };
    let opus = Usage {
        calls: 1,
        fresh: 2,
        cache_write: 3,
        cache_read: 4,
        output: 5,
        cost: 0.5,
    };
    assert_eq!(
        run.log.usage,
        BTreeMap::from([
            ("claude-haiku-5-5".to_owned(), haiku),
            ("claude-opus-5-5".to_owned(), opus),
        ])
    );
    assert!((run.log.cost - 0.875).abs() < 1e-9);
}

#[test]
fn a_merge_keeps_the_model_its_first_part_resolved_to() {
    let parts = vec![
        (
            on_model(part_run(None, "outcome: none — a.", 0.0), "claude-opus-5-5"),
            "outcome: none — a.\n".to_owned(),
        ),
        (
            part_run(None, "outcome: none — b.", 0.0),
            "outcome: none — b.\n".to_owned(),
        ),
    ];

    let (run, _) = merge("lang-rust", parts);

    assert_eq!(run.log.resolved_model, "claude-opus-5-5");
}

#[test]
fn a_merge_keeps_the_subtype_of_its_first_failed_part() {
    let mut late = part_run(Some("its run errored"), "", 0.0);
    late.log.subtype = "error_during_execution".to_owned();
    let parts = vec![
        (
            part_run(None, "outcome: none — a.", 0.0),
            "outcome: none — a.\n".to_owned(),
        ),
        (
            part_run(Some("it ran out of turns"), "", 0.0),
            String::new(),
        ),
        (late, String::new()),
    ];

    let (run, _) = merge("lang-rust", parts);

    assert_eq!(run.log.subtype, "error_max_turns");
}

#[test]
fn what_the_budget_charged_each_part_adds_up() {
    let parts = vec![
        (part_run(None, "outcome: none — a.", 0.25), String::new()),
        (part_run(Some("cut"), "", 0.5), String::new()),
    ];

    let (run, _) = merge("lang-rust", parts);

    assert!((run.charged - 0.75).abs() < 1e-9, "{}", run.charged);
}

#[test]
fn a_text_one_read_returns_whole_needs_no_ranges() {
    assert_eq!(ranges("", 10), Vec::new());
    assert_eq!(ranges("+a\n+b\n", 10), Vec::new());
    assert_eq!(ranges("+a\n+b\n", 6), Vec::new());
}

#[test]
fn a_text_past_one_read_is_read_in_line_ranges_each_within_it_or_one_line() {
    // Lines of 3, 3, 9, 3 and 2 bytes: a line past the limit is a range
    // alone, and the last has no line end.
    let text = "+a\n+b\n+ccccccc\n+d\n+e";

    assert_eq!(ranges(text, 6), [(1, 2), (3, 3), (4, 5)]);
    assert_eq!(ranges(text, 5), [(1, 1), (2, 2), (3, 3), (4, 5)]);
}

#[test]
fn a_part_larger_than_the_limit_is_told_the_line_ranges_to_read_and_one_within_it_is_not() {
    let big = format!(
        "diff --git a/a.md b/a.md\n{}",
        format!("+{}\n", "x".repeat(40)).repeat(5)
    );
    let diff = [big, section("b.rs", 60)].concat();
    let dir = with_diff("lang-markdown", &diff);

    let parts = prepare(dir.path(), "lang-markdown", 100).unwrap();

    assert_eq!(parts.len(), 2);
    assert!(
        parts[0].note.contains(
            "One `Read` does not return `review/diffs/lang-markdown.1.diff` whole: read its 6 \
             lines in 3 calls, each with `offset` and `limit`, lines 1-2, 3-4 and 5-6, and leave \
             no line unread."
        ),
        "{}",
        parts[0].note
    );
    assert!(
        !parts[1].note.contains("does not return"),
        "{}",
        parts[1].note
    );
}

#[test]
fn a_part_of_one_line_past_the_limit_is_told_no_ranges() {
    assert_eq!(ranges("+aaaaaaaa", 4), [(1, 1)]);
    assert_eq!(reading("review/diffs/a.1.diff", "+aaaaaaaa", 4), None);
    assert_eq!(reading("review/diffs/a.1.diff", "+a\n", 4), None);
}

#[test]
fn a_text_of_more_lines_than_one_read_takes_is_read_in_ranges_of_that_many() {
    let most = review_eval::coverage::DEFAULT_LIMIT;
    let text = "+a\n".repeat(2 * most + 1);

    assert_eq!(ranges(&"+a\n".repeat(most), usize::MAX), Vec::new());
    assert_eq!(
        ranges(&text, usize::MAX),
        [
            (1, most),
            (most + 1, 2 * most),
            (2 * most + 1, 2 * most + 1)
        ]
    );
}

#[test]
fn a_whole_diff_that_is_missing_or_that_one_read_returns_has_no_ranges_to_tell() {
    let dir = with_diff("lang-rust", "+a\n");

    assert_eq!(whole_reading(dir.path(), "lang-rust"), None);
    assert_eq!(whole_reading(dir.path(), "lang-toml"), None);
}
