//! The drifts of the review's reports: their form, grading and logs.

use super::*;

/// The drift the ENG-28 table's specialist rows exist to catch: a gate
/// that takes a report whatever outcome the specialist answered.
pub(super) fn review_report_case() -> Case {
    case(
        "a specialist's report ending with another outcome than it answered",
        "ENG-28",
        Injection::replace(
            "tooling/review-gate/src/report.rs",
            "    (answered != outcome).then_some(Problem::AnswerDiffers)",
            "    (answered != outcome && answered.is_empty()).then_some(Problem::AnswerDiffers)",
        ),
        scenario("ENG-28"),
        "other outcome answered",
    )
}

/// The review workflow drifts registered after the first ones, late so
/// adding them moves no earlier case to another shard: the agents' run
/// logs leaving their job, and the Claude Code command line, installed
/// by the composite action, in the key's job or starting an agent
/// outside review-run; the ENG-32 evaluation planning every component
/// a shared section change reaches, not one per set; and review-run
/// starting a generalist before its own specialists have ended.
pub(super) fn review_late_cases() -> Vec<Case> {
    vec![
        case(
            "the reviewing agents' run logs uploaded out of their job",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "            review/usage.json\n          if-no-files-found: error\n",
                "            review/usage.json\n            review/logs/\n          if-no-files-found: error\n",
            ),
            scenario("ENG-28"),
            "agent job review uploads review/logs, every tool result its agents read",
        ),
        case(
            "the key's job given the Claude Code command line",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "      - name: Build the review gate\n",
                "      - uses: ./.github/actions/claude-code\n      - name: Build the review gate\n",
            ),
            scenario("ENG-28"),
            "agent job post reads JEDUDEN_REVIEW_AGENT_KEY",
        ),
        case(
            "the review routed by the Claude Code command line instead of review-run",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "          cargo run --locked --quiet --offline -p review-run -- --workdir . --parallel 32 \\\n            --credential \"$RUNNER_TEMP/claude-credential\" > review/run.txt\n",
                "          claude -p review > review/run.txt\n",
            ),
            scenario("ENG-28"),
            "job review starts the Claude Code command line outside review-run and review-eval, which would route the review",
        ),
        case(
            "the evaluation planning every component a shared section change reaches",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/plan.rs",
                "                shared.entry(sections_of(&a)).or_default().push(name);\n",
                "                shared.entry(sections_of(&a)).or_default().push(name.clone());\n                out.push(Planned { component: name, reason: \"shared sections changed\".to_owned() });\n",
            ),
            scenario("ENG-32"),
            "components planned for the sections",
        ),
        case(
            "review-run starting a generalist before its own specialists ended",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/tasks.rs",
                "            .filter_map(|s| g.last.get(s).copied())\n            .collect();\n        g.last.insert(unit.name.clone(), g.tasks.len());\n",
                "            .filter_map(|s| g.last.get(s).copied().filter(|_| false))\n            .collect();\n        g.last.insert(unit.name.clone(), g.tasks.len());\n",
            ),
            scenario("ENG-28"),
            "review-code started before its specialist lang-rust ended",
        ),
    ]
}

/// The drifts the form of a specialist's report exists to catch in the
/// gate's report check, late so adding them moves no earlier case to
/// another shard: a finding line in neither form, an outcome its lines
/// contradict, and a cited id no requirement has.
pub(super) fn review_report_form_cases() -> Vec<Case> {
    const REPORT: &str = "tooling/review-gate/src/report.rs";
    let gate_test = "report::tests::a_report_check_refuses_what_the_gate_refuses";
    vec![
        case(
            "the gate taking a finding line in neither form",
            "ENG-28",
            Injection::replace(
                REPORT,
                "Line::Malformed => return Some(Problem::Malformed { line: number }),",
                "Line::Malformed => {}",
            ),
            cargo_lib_test("review-gate", gate_test),
            "a_report_check_refuses_what_the_gate_refuses",
        ),
        case(
            "the gate taking an outcome of none beside a finding",
            "ENG-28",
            Injection::replace(
                REPORT,
                "(true, true) => Some(Problem::NoneWithFindings),",
                "(true, true) => None,",
            ),
            cargo_lib_test("review-gate", gate_test),
            "a_report_check_refuses_what_the_gate_refuses",
        ),
        case(
            "a report check taking an id no requirement has",
            "ENG-28",
            Injection::replace(
                REPORT,
                "if let Some(id) = unknown_id(&Line::Linked(linked), known) {",
                "if let Some(id) = unknown_id(&Line::Linked(linked), known).filter(|_| false) {",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_report_check_with_the_known_ids_refuses_an_id_no_requirement_has",
            ),
            "a_report_check_with_the_known_ids_refuses_an_id_no_requirement_has",
        ),
    ]
}

/// The drifts the form of a specialist's report exists to catch in
/// review-run: a report held to no requirement ids, the pull request's
/// own requirements left out of them, and a blocking finding kept from
/// a requirement request.
pub(super) fn review_run_report_cases() -> Vec<Case> {
    const OUTCOME: &str = "tooling/review-run/src/outcome.rs";
    vec![
        case(
            "review-run holding a report to no requirement ids",
            "ENG-28",
            Injection::replace(
                OUTCOME,
                "report_check(report, Some(answer), Some(known))",
                "report_check(report, Some(answer), Some(known).filter(|_| false))",
            ),
            cargo_lib_test(
                "review-run",
                "outcome::tests::a_report_the_gate_would_refuse_blocks_and_is_named",
            ),
            "a_report_the_gate_would_refuse_blocks_and_is_named",
        ),
        case(
            "review-run reading the requirement ids from main's tree alone",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/requirements.rs",
                "[workdir.join(SRS), workdir.join(PR_HEAD).join(SRS)]",
                "[workdir.join(SRS), workdir.join(SRS).join(PR_HEAD)]",
            ),
            cargo_lib_test(
                "review-run",
                "requirements::tests::the_ids_are_main_s_and_the_pull_request_s_and_the_invariants",
            ),
            "the_ids_are_main_s_and_the_pull_request_s_and_the_invariants",
        ),
        case(
            "review-run keeping a requirement request's finding blocking",
            "ENG-28",
            Injection::replace(
                OUTCOME,
                "if f.severity == BLOCKING && from_request(plan, &name, reports, &f) {",
                "if f.severity == BLOCKING && from_request(plan, &name, reports, &f) && f.line < 0 {",
            ),
            cargo_lib_test(
                "review-run",
                "outcome::tests::a_blocking_finding_kept_from_a_requirement_request_stays_a_nit",
            ),
            "a_blocking_finding_kept_from_a_requirement_request_stays_a_nit",
        ),
    ]
}

/// The drifts the evaluation's grading of a specialist's report exists
/// to catch: a grader that takes a report the review would refuse or
/// reads a specialist's severity word, and a check that lets a
/// generalist's case name a form.
pub(super) fn review_report_grading_cases() -> Vec<Case> {
    vec![
        case(
            "the grader passing a report the review would refuse",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/grade.rs",
                "graded.failures.push(format!(\"report refused: {problem}\"));",
                "graded.slips.push(format!(\"report refused: {problem}\"));",
            ),
            cargo_lib_test(
                "review-eval",
                "grade::tests::a_specialist_report_the_review_would_refuse_fails_its_run",
            ),
            "a_specialist_report_the_review_would_refuse_fails_its_run",
        ),
        case(
            "the grader reading a specialist's line by a severity word",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/grade.rs",
                "kind == Kind::Generalist || (kind == Kind::Consultant",
                "kind != Kind::Consultant || (kind == Kind::Consultant",
            ),
            cargo_lib_test(
                "review-eval",
                "grade::tests::a_specialist_s_line_is_matched_by_its_form_and_never_by_a_severity_word",
            ),
            "a_specialist_s_line_is_matched_by_its_form_and_never_by_a_severity_word",
        ),
        case(
            "the evaluation's check taking a generalist's case that names a form",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/check.rs",
                "if !matches!(component.kind, Kind::Generalist) {",
                "if !matches!(component.kind, Kind::Consultant) {",
            ),
            cargo_lib_test(
                "review-eval",
                "check::tests::a_generalist_s_case_that_names_a_form_is_named",
            ),
            "a_generalist_s_case_that_names_a_form_is_named",
        ),
    ]
}

/// The drifts the report assembled from a specialist's run log exists to
/// catch (ENG-28, ENG-32), late so adding them moves no earlier case to
/// another shard: an assembly that keeps a withdrawn finding, a cut-off
/// run whose salvaged findings make it one that answered, a gate taking
/// an answer with no outcome line, a specialist given a writing tool,
/// and a grader reading a finding stated only in the answer as stated
/// before it.
pub(super) fn review_log_report_cases() -> Vec<Case> {
    const REPORT: &str = "tooling/review-gate/src/report.rs";
    vec![
        case(
            "the assembly keeping a withdrawn finding",
            "ENG-28",
            Injection::replace(
                REPORT,
                "kept.retain(|k| !k.at((&w.path, w.line)));",
                "kept.retain(|_| true);",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_withdrawal_removes_every_earlier_finding_at_its_place",
            ),
            "a_withdrawal_removes_every_earlier_finding_at_its_place",
        ),
        case(
            "a cut-off run counted as answered once its findings are salvaged",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/run.rs",
                "                Some(format!(\"its run ended with {}\", log.subtype))\n",
                "                log.texts\n                    .is_empty()\n                    .then(|| format!(\"its run ended with {}\", log.subtype))\n",
            ),
            cargo_lib_test(
                "review-run",
                "run::tests::a_run_cut_off_before_its_answer_keeps_what_it_stated_as_an_incomplete_report",
            ),
            "a_run_cut_off_before_its_answer_keeps_what_it_stated_as_an_incomplete_report",
        ),
        case(
            "the gate taking a specialist that answered no outcome line",
            "ENG-28",
            Injection::replace(
                REPORT,
                "        return Some(Problem::NoAnswerOutcome);\n",
                "        return None;\n",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_report_check_refuses_what_the_gate_refuses",
            ),
            "a_report_check_refuses_what_the_gate_refuses",
        ),
        protection_case(
            "a specialist given a writing tool",
            Injection::replace(
                ".claude/agents/lang-rust.md",
                "tools: Read, Grep, Glob\n",
                "tools: Read, Grep, Glob, Write\n",
            ),
            "the_checkout_s_review_agents_are_all_defined",
            "lang-rust has tools Read, Grep, Glob, Write, want Read, Grep, Glob",
        ),
        case(
            "the grader reading a finding stated only in the answer as stated before it",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/grade.rs",
                "        .filter(|t| t.trim() != answer)\n",
                "        .filter(|t| t.trim() != answer || true)\n",
            ),
            cargo_lib_test(
                "review-eval",
                "grade::tests::a_finding_stated_only_in_the_answer_was_not_stated_before_it",
            ),
            "a_finding_stated_only_in_the_answer_was_not_stated_before_it",
        ),
    ]
}

/// The drifts running a specialist in parts exists to catch (ENG-28),
/// late so adding them moves no earlier case to another shard: a merge
/// that drops a later part's findings, and a part that did not answer
/// leaving the specialist one that did.
pub(super) fn review_run_parts_cases() -> Vec<Case> {
    const PARTS: &str = "tooling/review-run/src/parts.rs";
    vec![
        case(
            "the merge of a specialist's parts dropping a later part's findings",
            "ENG-28",
            Injection::replace(
                PARTS,
                "} else if !lines.iter().any(|l| l == line) {",
                "} else if at == 0 && !lines.iter().any(|l| l == line) {",
            ),
            cargo_lib_test(
                "review-run",
                "parts::tests::the_merged_report_is_the_union_of_the_parts_in_order_with_one_outcome",
            ),
            "the_merged_report_is_the_union_of_the_parts_in_order_with_one_outcome",
        ),
        case(
            "a specialist's part that did not answer not failing the review",
            "ENG-28",
            Injection::replace(
                PARTS,
                "if let Some(why) = &run.failure {",
                "if let Some(why) = run.failure.as_ref().filter(|_| false) {",
            ),
            cargo_lib_test(
                "review-run",
                "parts::tests::a_failed_part_makes_the_merged_run_incomplete_naming_it_and_keeps_every_part_s_findings",
            ),
            "a_failed_part_makes_the_merged_run_incomplete_naming_it_and_keeps_every_part_s_findings",
        ),
    ]
}
