//! The drifts ENG-28 exists to catch in the findings a review posts:
//! limits, withdrawals, places, retries, resumes and tokens.

use super::*;

/// The drifts ENG-34 exists to catch, late so adding them moves no
/// earlier case to another shard: the review bundle no longer flagging
/// a file past its limit, no longer flagging a file no specialist
/// reads, and a limit the ENG-34 row states that the bundle does not
/// hold.
pub(super) fn review_limit_cases() -> Vec<Case> {
    const LIMITS: &str = "tooling/review-bundle/src/limits.rs";
    vec![
        case(
            "the review bundle no longer flagging a file past its limit",
            "ENG-34",
            Injection::replace(
                LIMITS,
                "let size = lines(path).and_then(|n| oversize(path, n));",
                "let size = lines(path).and_then(|n| oversize(path, n)).filter(|_| false);",
            ),
            scenario("ENG-34"),
            "no nit at crates/a/src/lib.rs in the finding form citing ENG-34",
        ),
        case(
            "the review bundle no longer flagging a file no specialist reads",
            "ENG-34",
            Injection::replace(
                LIMITS,
                "size.into_iter().chain(unread(path, scopes))",
                "size.into_iter().chain(unread(path, scopes).filter(|_| false))",
            ),
            scenario("ENG-34"),
            "no nit at LICENSE in the finding form citing ENG-28",
        ),
        case(
            "the ENG-34 row stating a limit the review bundle does not hold",
            "ENG-34",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "200 for TOML (`.toml`)",
                "250 for TOML (`.toml`)",
            ),
            scenario("ENG-34"),
            "the ENG-34 row states no limit of 200 for TOML, which the review bundle holds",
        ),
    ]
}

/// The drifts the rule for a revision or withdrawal naming a place where
/// no finding was stated exists to catch (ENG-28, I6), late so adding
/// them moves no earlier case to another shard: a withdrawal that
/// removed nothing left out of the report unseen, a revision of such a
/// place left out instead of stated, and the gate refusing a withdrawal
/// left in a report.
pub(super) fn review_withdrawal_cases() -> Vec<Case> {
    const REPORT: &str = "tooling/review-gate/src/report.rs";
    let assembly = "report::tests::a_withdrawal_that_finds_nothing_leaves_a_note_and_a_revision_states_its_finding";
    vec![
        case(
            "the assembly dropping a withdrawal that removed nothing unseen",
            "ENG-28",
            Injection::replace(
                REPORT,
                "if kept.len() == before {",
                "if kept.len() == before && w.line == usize::MAX {",
            ),
            cargo_lib_test("review-gate", assembly),
            "a_withdrawal_that_finds_nothing_leaves_a_note_and_a_revision_states_its_finding",
        ),
        case(
            "the assembly leaving out a revision of a place where nothing was stated",
            "ENG-28",
            Injection::replace(REPORT, "push_unique(&mut kept, finding);", "drop(finding);"),
            cargo_lib_test("review-gate", assembly),
            "a_withdrawal_that_finds_nothing_leaves_a_note_and_a_revision_states_its_finding",
        ),
        case(
            "the gate refusing a withdrawal left in a report",
            "ENG-28",
            Injection::replace(
                REPORT,
                "Line::Prose | Line::Withdrawn(_) => {}",
                "Line::Prose => {}\n            Line::Withdrawn(_) => return Some(Problem::Malformed { line: number }),",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_report_check_reads_a_leftover_revision_as_its_finding_and_a_withdrawal_as_nothing",
            ),
            "a_report_check_reads_a_leftover_revision_as_its_finding_and_a_withdrawal_as_nothing",
        ),
    ]
}

/// The drifts the gate's reading of a finding's place exists to catch
/// (ENG-28, I6), late so adding them moves no earlier case to another
/// shard: a finding missing the dash after its place read as prose, and
/// a line at the forms' template place refused without naming it; and,
/// for the workflow check beside them, the review's job starting the
/// Claude Code command line itself beside review-run.
pub(super) fn review_place_form_cases() -> Vec<Case> {
    const REPORT: &str = "tooling/review-gate/src/report.rs";
    vec![
        case(
            "the gate reading a finding missing the dash after its place as prose",
            "ENG-28",
            Injection::replace(
                REPORT,
                "finding_shaped(&parts) || undashed(text)",
                "finding_shaped(&parts)",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_finding_line_missing_the_dash_after_its_place_is_malformed_not_prose",
            ),
            "a_finding_line_missing_the_dash_after_its_place_is_malformed_not_prose",
        ),
        case(
            "the gate refusing a line at the template's place without naming it",
            "ENG-28",
            Injection::replace(
                REPORT,
                "Line::Malformed if at_template(l) => {",
                "Line::Malformed if at_template(l) && number == 0 => {",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_report_check_names_a_line_at_the_template_s_place",
            ),
            "a_report_check_names_a_line_at_the_template_s_place",
        ),
        case(
            "the review's job starting the Claude Code command line beside review-run",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "          cargo run --locked --quiet --offline -p review-run -- --workdir . --parallel 32 \\\n",
                "          cd pr-head && claude --permission-mode bypassPermissions -p review\n          cargo run --locked --quiet --offline -p review-run -- --workdir . --parallel 32 \\\n",
            ),
            scenario("ENG-28"),
            "job review starts the Claude Code command line itself, outside review-run and review-eval",
        ),
    ]
}

/// The drift the review's second attempt must not bring (ENG-28), late
/// so adding it moves no earlier case to another shard: a retry that
/// merges the first attempt's findings into a second report the gate
/// refuses again, so the two refused reports pass as one.
pub(super) fn review_retry_cases() -> Vec<Case> {
    vec![case(
        "a retried specialist's refused report rescued by its first attempt's findings",
        "ENG-28",
        Injection::replace(
            "tooling/review-run/src/retry.rs",
            "    let (Some(_), Some(failure)) = (&report, second.failure.clone()) else {\n        return report;\n    };\n",
            "    let (Some(_), failure) = (&report, second.failure.clone()) else {\n        return report;\n    };\n    let failure = failure.unwrap_or_default();\n",
        ),
        cargo_lib_test(
            "review-run",
            "retry::tests::a_report_refused_twice_stays_refused_and_the_first_s_findings_do_not_rescue_it",
        ),
        "a_report_refused_twice_stays_refused_and_the_first_s_findings_do_not_rescue_it",
    )]
}

/// The drift the gate's refusal of a line citing no id must not bring
/// (ENG-28, I6), late so adding it moves no earlier case to another
/// shard: a line saying all is well refused as a line in no form, so
/// its second attempt is not told why.
pub(super) fn review_unlinked_case() -> Case {
    case(
        "the gate refusing a line that cites no id as one in no form",
        "ENG-28",
        Injection::replace(
            "tooling/review-gate/src/report.rs",
            "Line::Malformed if unlinked(l) => return Some(Problem::Unlinked { line: number }),",
            "Line::Malformed if unlinked(l) && false => return Some(Problem::Unlinked { line: number }),",
        ),
        cargo_lib_test(
            "review-gate",
            "report::tests::a_report_check_names_a_line_at_a_place_that_cites_no_id",
        ),
        "a_report_check_names_a_line_at_a_place_that_cites_no_id",
    )
}

/// The drift the review's posting of an unknown id must not bring
/// (ENG-28), late so adding it moves no earlier case to another shard:
/// the gate refusing a report for an id it cannot know, so a review
/// `review-run` decided as a request for changes posts nothing.
pub(super) fn review_unknown_id_post_case() -> Case {
    case(
        "the gate failing without posting on a report citing an unknown id",
        "ENG-28",
        Injection::replace(
            "tooling/review-gate/src/lib.rs",
            "        input.answers.get(agent).map(String::as_str),\n        None,\n",
            "        input.answers.get(agent).map(String::as_str),\n        Some(&BTreeSet::new()),\n",
        ),
        cargo_lib_test(
            "engineering",
            "review_refusals::tests::the_checkout_s_review_run_names_an_unknown_id_and_an_unread_line",
        ),
        "the_checkout_s_review_run_names_an_unknown_id_and_an_unread_line",
    )
}

/// The drift ENG-32's case check must catch, late so adding it moves no
/// earlier case to another shard: a case naming a severity the grader
/// does not compare it with, so it grades more loosely than meant.
pub(super) fn review_case_severity_case() -> Case {
    protection_case(
        "a reviewer evaluation case naming a severity other than blocking or nit",
        Injection::replace(
            "tooling/review-eval/cases/review-code/injection-in-the-diff.toml",
            "severity = \"blocking\"",
            "severity = \"Blocking\"",
        ),
        "the_checkout_s_reviewer_evaluation_cases_apply_and_dispatch",
        "case review-code/injection-in-the-diff names the severity",
    )
}

/// The drifts a resumed review exists to rule out, late so adding them
/// moves no earlier case to another shard: an answer reused across a
/// changed head or definition, an agent without an accepted report that
/// reads as clean, and a reused agent left out of those consulted.
pub(super) fn review_resume_cases() -> Vec<Case> {
    const RESUME: &str = "tooling/review-run/src/resume.rs";
    let resume_test = |name: &'static str| cargo_lib_test("review-run", name);
    vec![
        case(
            "a resumed review reusing an answer whose answer key changed",
            "ENG-28",
            Injection::replace(
                RESUME,
                "let same = matches!((&was.key, &is.key), (Some(a), Some(b)) if a == b);",
                "let same = was.key.is_some() && is.key.is_some();",
            ),
            resume_test("resume::tests::a_changed_key_runs_its_agent_again_with_why"),
            "a_changed_key_runs_its_agent_again_with_why",
        ),
        case(
            "an answer key that ignores the agent's definition, so a changed one reuses the old answer",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/key.rs",
                "    f.add(\"definition\", &bytes);\n",
                "    f.add(\"definition\", b\"\");\n",
            ),
            resume_test(
                "key::tests::a_changed_definition_changes_its_agent_s_key_and_its_generalist_s_alone",
            ),
            "a_changed_definition_changes_its_agent_s_key_and_its_generalist_s_alone",
        ),
        case(
            "an agent that did not answer left out of what was not reviewed, so it reads as clean",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/outcome.rs",
                "    for name in silent {\n        match runs.iter()",
                "    for name in silent.iter().filter(|_| false) {\n        match runs.iter()",
            ),
            resume_test(
                "outcome::tests::every_agent_without_an_accepted_report_is_not_reviewed_with_why",
            ),
            "every_agent_without_an_accepted_report_is_not_reviewed_with_why",
        ),
        case(
            "a resumed review leaving a reused agent out of those consulted",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/tasks.rs",
                "        .filter_map(|n| whole(r, c, n))\n        .collect();\n",
                "        .filter_map(|n| whole(r, c, n))\n        .filter(|_| false)\n        .collect();\n",
            ),
            resume_test(
                "cli::tests::a_resumed_review_runs_only_the_gap_and_its_generalist_and_approves",
            ),
            "a_resumed_review_runs_only_the_gap_and_its_generalist_and_approves",
        ),
    ]
}

/// The drifts the review's controls exist to rule out (ENG-28, I6), late
/// so adding them moves no earlier case to another shard: a run without
/// its tier's cost cap, a review that starts runs past its own cap, a
/// breaker that never trips, a generalist that waits for specialists it
/// does not weigh, and a run that can never succeed run again.
pub(super) fn review_control_cases() -> Vec<Case> {
    const CONTROLS: &str = "tooling/review-run/src/controls.rs";
    const TEST: &str =
        "review_controls::tests::the_checkout_s_review_run_holds_its_agents_to_the_controls";
    let controlled = || cargo_lib_test("engineering", TEST);
    let want = "the_checkout_s_review_run_holds_its_agents_to_the_controls";
    vec![
        case(
            "an agent's run without its tier's cost cap",
            "ENG-28",
            Injection::replace(
                "tooling/review-eval/src/runner.rs",
                "        \"--max-budget-usd\".to_owned(),\n",
                "        \"--max-budget\".to_owned(),\n",
            ),
            controlled(),
            want,
        ),
        case(
            "review-run starting runs past the review's cost cap",
            "ENG-28",
            Injection::replace(
                CONTROLS,
                "(spent >= self.cap).then(|| {",
                "(spent >= self.cap && false).then(|| {",
            ),
            controlled(),
            want,
        ),
        case(
            "review-run starting generalists after a mass failure of its specialists",
            "ENG-28",
            Injection::replace(
                CONTROLS,
                "        trips(failed, self.dispatched, self.fraction).then(|| {",
                "        (trips(failed, self.dispatched, self.fraction) && false).then(|| {",
            ),
            controlled(),
            want,
        ),
        case(
            "review-run running again a run whose credential was refused",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/backoff.rs",
                "        Class::Never(\"its credential was refused, and no run with it can succeed\")\n",
                "        Class::Again\n",
            ),
            controlled(),
            want,
        ),
        case(
            "review-run starting a generalist only once every specialist ended",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/tasks.rs",
                "            .filter_map(|s| g.last.get(s).copied())\n            .collect();\n        g.last.insert(unit.name.clone(), g.tasks.len());\n",
                "            .filter_map(|_| None::<usize>)\n            .chain(g.last.values().copied())\n            .collect();\n        g.last.insert(unit.name.clone(), g.tasks.len());\n",
            ),
            cargo_lib_test(
                "engineering",
                "review_runs::tests::a_review_run_on_the_checkout_s_definitions_has_no_problem",
            ),
            "a_review_run_on_the_checkout_s_definitions_has_no_problem",
        ),
    ]
}

/// The drifts the gate's reading of a line at a place must not bring
/// (ENG-28, I6), late so adding them moves no earlier case to another
/// shard: a finding line with no dash at all, `a.rs:1 if not addressed:
/// x`, read as prose, and a line saying all is well left out rather
/// than kept as a note, so the assembly drops either unseen.
pub(super) fn review_undashed_cases() -> Vec<Case> {
    const REPORT: &str = "tooling/review-gate/src/report.rs";
    vec![
        case(
            "the assembly dropping a finding line with no dash after its place unseen",
            "ENG-28",
            Injection::replace(
                REPORT,
                "return if undashed(text) {",
                "return if undashed(text) && parts.is_empty() {",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_finding_line_with_a_space_or_colon_for_the_dash_after_its_place_is_malformed",
            ),
            "a_finding_line_with_a_space_or_colon_for_the_dash_after_its_place_is_malformed",
        ),
        case(
            "the assembly dropping a line saying all is well instead of keeping its note",
            "ENG-28",
            Injection::replace(
                REPORT,
                "push_unique(&mut kept, Kept::of(format!(\"{ALL_WELL} {text}\")));",
                "drop(text);",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_finding_line_ending_with_all_is_well_is_kept_as_a_visible_note_not_refused",
            ),
            "a_finding_line_ending_with_all_is_well_is_kept_as_a_visible_note_not_refused",
        ),
    ]
}

/// The drift a revision taking one full finding line rules out (ENG-28),
/// late so adding it moves no earlier case to another shard: a revision
/// with an arrow, which would move a finding ENG-28 does not let move,
/// read as a revision instead of refused.
pub(super) fn review_moved_revision_case() -> Case {
    case(
        "a revision with an arrow read as one instead of refused",
        "ENG-28",
        Injection::replace(
            "tooling/review-gate/src/report.rs",
            "    if ARROWS.iter().any(|arrow| rest.contains(arrow)) {\n",
            "    if ARROWS.iter().any(|arrow| rest.contains(arrow)) && rest.is_empty() {\n",
        ),
        cargo_lib_test(
            "review-gate",
            "report::tests::a_revision_in_no_form_is_malformed",
        ),
        "a_revision_in_no_form_is_malformed",
    )
}

/// The drifts that would let the Claude token reach an agent or leave
/// the review's job (ENG-28, I6), late so adding them moves no earlier
/// case to another shard: the scrub replacing nothing or leaving the run
/// logs out, a leak no longer failing the review or naming itself in its
/// outcome, an agent started
/// with the token in its environment, let read the process files or
/// the staged credential, and the workflow handing the token to the
/// step that runs the agents or uploading before the scrub.
pub(super) fn review_token_cases() -> Vec<Case> {
    const RUN_CLI: &str = "tooling/review-run/src/cli.rs";
    let eval = |name: &'static str| cargo_lib_test("review-eval", name);
    let leak_test = "cli::tests::leaks::a_review_whose_agent_leaked_the_token_scrubs_it_everywhere_and_fails_closed";
    vec![
        case(
            "the scrub counting and replacing no form of the token",
            "ENG-28",
            Injection::replace(
                "tooling/review-eval/src/secret.rs",
                "let n = out.matches(form.as_str()).count();",
                "let n = out.matches(form.as_str()).count().min(0);",
            ),
            eval("secret::tests::the_exact_token_is_replaced_and_counted"),
            "the_exact_token_is_replaced_and_counted",
        ),
        case(
            "a review whose agent leaked the token deciding as if it had not",
            "ENG-28",
            Injection::replace(
                RUN_CLI,
                "        return Err(leak::reason(leaked));",
                "        progress(&leak::reason(leaked));",
            ),
            cargo_lib_test("review-run", leak_test),
            "a_review_whose_agent_leaked_the_token_scrubs_it_everywhere_and_fails_closed",
        ),
        case(
            "a leak of the token left out of the review outcome",
            "ENG-28",
            Injection::replace(
                RUN_CLI,
                "        leak::fail_closed(&mut outcome, leaked);",
                "        let _ = &mut outcome;",
            ),
            cargo_lib_test("review-run", leak_test),
            "a_review_whose_agent_leaked_the_token_scrubs_it_everywhere_and_fails_closed",
        ),
        case(
            "an agent started with the Claude token in its environment",
            "ENG-28",
            Injection::replace(
                "tooling/review-eval/src/process.rs",
                "NESTED_SESSION_VARS.iter().chain(&WITHHELD_VARS)",
                "NESTED_SESSION_VARS.iter().chain(&WITHHELD_VARS[..0])",
            ),
            eval("process::tests::a_command_never_gets_the_claude_token_in_its_environment"),
            "a_command_never_gets_the_claude_token_in_its_environment",
        ),
        case(
            "an agent let read the process files",
            "ENG-28",
            Injection::replace("tooling/review-eval/src/runner.rs", "Read(//proc/**),", ""),
            scenario("ENG-28"),
            "review-run does not refuse its agents Read(//proc/**)",
        ),
        case(
            "an agent let read the staged credential",
            "ENG-28",
            Injection::replace(
                "tooling/review-eval/src/credential.rs",
                "let rules = format!(\"Read(/{shown}/**),Read(/{shown})\");",
                "let rules = String::new();",
            ),
            eval(
                "credential::tests::a_confined_claude_reads_the_staged_config_and_may_not_read_it",
            ),
            "a_confined_claude_reads_the_staged_config_and_may_not_read_it",
        ),
        case(
            "the Claude token handed to the step that runs the agents",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "      - name: Review\n        id: review\n",
                "      - name: Review\n        id: review\n        env:\n          CLAUDE_CODE_OAUTH_TOKEN: ${{ secrets.CLAUDE_CODE_OAUTH_TOKEN }}\n",
            ),
            scenario("ENG-28"),
            "agent job review hands CLAUDE_CODE_OAUTH_TOKEN to step \"Review\", which does more than stage it",
        ),
        case(
            "the specialists' reports uploaded before the scrub of the token",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "        if: always() && steps.scrub.outputs.scrubbed == 'true'\n",
                "        if: always()\n",
            ),
            scenario("ENG-28"),
            "agent job review uploads in step \"Keep the specialists' reports\" before a scrub",
        ),
        case(
            "the agents' run logs written out unscrubbed of the token",
            "ENG-28",
            Injection::replace(
                RUN_CLI,
                "for dir in [crate::LOGS, crate::REPORTS] {",
                "for dir in [crate::REPORTS] {",
            ),
            cargo_lib_test(
                "engineering",
                "review_leak::tests::the_checkout_s_review_run_leaves_a_leaked_token_in_nothing_it_writes_or_prints",
            ),
            "review-run left the token in review/logs/",
        ),
        case(
            "an agent not refused the staged credential's directory by name",
            "ENG-28",
            Injection::replace(
                "tooling/review-eval/src/credential.rs",
                "let rules = format!(\"Read(/{shown}/**),Read(/{shown})\");",
                "let rules = format!(\"Read(/{shown}/none)\");",
            ),
            scenario("ENG-28"),
            "is not refused the staged credential's directory",
        ),
        case(
            "the token sweep reading a script that takes arguments as an error, not as the step's text",
            "ENG-28",
            Injection::replace(
                "tooling/engineering/src/review_workflow.rs",
                "Err(_) => parse(&scripts::inline_scripts_lenient(",
                "Err(_) => parse(&scripts::inline_scripts(",
            ),
            cargo_lib_test(
                "engineering",
                "review_workflow::scripts::tests::the_token_sweep_still_reads_a_workflow_that_runs_a_script_with_arguments",
            ),
            "the_token_sweep_still_reads_a_workflow_that_runs_a_script_with_arguments",
        ),
    ]
}

/// The drifts that would drop a finding unseen, late so adding them
/// moves no earlier case to another shard: a specialist's place in bold
/// read as prose, a finding-shaped line with a place that cannot be
/// read left out, and a generalist's finding line the review cannot
/// read passed by, or a report citing an unknown id left unnamed (I6).
pub(super) fn review_finding_shape_cases() -> Vec<Case> {
    const REPORT: &str = "tooling/review-gate/src/report.rs";
    vec![
        case(
            "the gate reading a finding whose place is in bold as prose",
            "ENG-28",
            Injection::replace(
                REPORT,
                "link_text(part.trim().trim_matches(MARKUP)).trim_matches(MARKUP);",
                "link_text(part.trim().trim_matches('`')).trim_matches('`');",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_finding_with_its_place_in_markup_reads_as_one",
            ),
            "a_finding_with_its_place_in_markup_reads_as_one",
        ),
        case(
            "the gate leaving out a finding-shaped line whose place cannot be read",
            "ENG-28",
            Injection::replace(
                REPORT,
                "return if finding_shaped(&parts) || undashed(text) {",
                "return if finding_shaped(&parts[..1]) || undashed(text) {",
            ),
            cargo_lib_test(
                "review-gate",
                "report::tests::a_finding_shaped_line_with_no_place_it_can_read_is_malformed_not_prose",
            ),
            "a_finding_shaped_line_with_no_place_it_can_read_is_malformed_not_prose",
        ),
        case(
            "review-run passing by a generalist's finding line it cannot read",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/outcome.rs",
                "for line in answer.lines().filter(|l| report::unread(l)) {",
                "for line in answer.lines().filter(|_| false) {",
            ),
            cargo_lib_test(
                "review-run",
                "outcome::tests::a_generalist_s_finding_line_the_review_cannot_read_blocks_and_is_named",
            ),
            "a_generalist_s_finding_line_the_review_cannot_read_blocks_and_is_named",
        ),
        case(
            "review-run deciding a review whose report cites an unknown id unnamed",
            "ENG-28",
            Injection::replace(
                "tooling/review-run/src/outcome.rs",
                "report_check(report, Some(answer), Some(known))",
                "report_check(report, Some(answer), None)",
            ),
            cargo_lib_test(
                "engineering",
                "review_refusals::tests::the_checkout_s_review_run_names_an_unknown_id_and_an_unread_line",
            ),
            "the_checkout_s_review_run_names_an_unknown_id_and_an_unread_line",
        ),
    ]
}
