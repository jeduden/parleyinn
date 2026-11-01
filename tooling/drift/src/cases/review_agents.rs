//! The drifts of the review's agents, reach, gate and evaluation.

use super::*;

/// The drifts ENG-28 exists to catch in the review's agents' reach: an
/// agent reaching the approval, or routing the review. [`review_gate_cases`] holds the approval
/// skipping the review gate.
pub(super) fn review_cases() -> Vec<Case> {
    let mut cases = review_reach_cases();
    cases.extend(review_post_cases());
    cases
}

/// The agents' job reaching past its bounds: the wrong trigger, a write
/// permission, the key, a writing tool, or an agent routing the review.
pub(super) fn review_reach_cases() -> Vec<Case> {
    vec![
        case(
            "the reviewer run as the pull request defines it",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "  workflow_run:\n",
                "  pull_request_target:\n  workflow_run:\n",
            ),
            scenario("ENG-28"),
            "triggers on [pull_request_target workflow_run issue_comment], want workflow_run and issue_comment",
        ),
        case(
            "the reviewing agent granted a write permission",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "      pull-requests: read\n",
                "      pull-requests: write\n",
            ),
            scenario("ENG-28"),
            "agent job review is granted a write permission",
        ),
        case(
            "the reviewing agent handed the reviewer app's key",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "        run: |\n          cargo run --locked --quiet --offline -p review-run",
                "          KEY: ${{ secrets.JEDUDEN_REVIEW_AGENT_KEY }}\n        run: |\n          cargo run --locked --quiet --offline -p review-run",
            ),
            scenario("ENG-28"),
            "agent job review reads JEDUDEN_REVIEW_AGENT_KEY",
        ),
        case(
            "the reviewing agents allowed to write anywhere",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "            --credential \"$RUNNER_TEMP/claude-credential\" > review/run.txt\n",
                "            --credential \"$RUNNER_TEMP/claude-credential\" > review/run.txt\n          ./review/agents.sh -p review --allowedTools Write\n",
            ),
            scenario("ENG-28"),
            "may use Write, want Read, Glob, Grep alone",
        ),
        case(
            "the review routed by an agent instead of review-run",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "          cargo run --locked --quiet --offline -p review-run -- --workdir . --parallel 32 \\\n            --credential \"$RUNNER_TEMP/claude-credential\" > review/run.txt\n",
                "          echo routed\n      - uses: anthropics/claude-code-action@1d6de8cb0c237e7c15e9e1bdf973826ebae490cc\n",
            ),
            scenario("ENG-28"),
            "no job runs the review's agents with \"cargo run --locked --quiet --offline -p review-run -- --workdir .\"",
        ),
    ]
}

/// The post job's bounds: the gate's inputs, its check and its order
/// after the agents.
pub(super) fn review_post_cases() -> Vec<Case> {
    vec![
        case(
            "the review gate without the specialists' reports",
            "ENG-28",
            Injection::replace(
                REVIEW_DECIDE_SCRIPT,
                " --reports reports --answers answers.json",
                "",
            ),
            scenario("ENG-28"),
            r#"does not run "--reports reports --answers answers.json""#,
        ),
        case(
            "the review gate's check no longer posted",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "          permission-checks: write\n",
                "          permission-checks: read\n",
            ),
            scenario("ENG-28"),
            "job post does not mint permission-checks: write",
        ),
        case(
            "the review posted after the agent failed",
            "ENG-28",
            Injection::replace(
                REVIEW_WORKFLOW,
                "    needs: [review, evaluate]\n    if: needs.review.outputs.pull != ''\n",
                "    needs: [review, evaluate]\n    if: always() && needs.review.outputs.pull != ''\n",
            ),
            scenario("ENG-28"),
            "job post runs on always(), even after a failed review",
        ),
    ]
}

/// The drifts ENG-28 exists to catch in the agents the review consults:
/// one the dispatch names but no file defines as named, or one at the
/// wrong effort.
pub(super) fn review_agent_cases() -> Vec<Case> {
    vec![
        protection_case(
            "a section of the specification without its specialist",
            Injection::replace(
                ".claude/agents/srs-security.md",
                "name: srs-security\n",
                "name: srs-secure\n",
            ),
            "the_checkout_s_review_agents_are_all_defined",
            "srs-security names itself srs-secure",
        ),
        protection_case(
            "a specialist raised to medium effort",
            Injection::replace(
                ".claude/agents/lang-rust.md",
                "effort: low\n",
                "effort: medium\n",
            ),
            "the_checkout_s_review_agents_are_all_defined",
            "lang-rust runs at effort medium, want low",
        ),
        protection_case(
            "a language no review specialist reads",
            Injection::create("scripts/rogue.py", "print()\n"),
            "the_checkout_s_languages_each_have_their_specialist",
            "scripts/rogue.py is in no language a review specialist reads",
        ),
        protection_case(
            "a generalist allowed to write",
            Injection::replace(
                ".claude/agents/review-security.md",
                "tools: Read, Grep, Glob\n",
                "tools: Read, Grep, Glob, Write\n",
            ),
            "the_checkout_s_review_agents_are_all_defined",
            "review-security has tools Read, Grep, Glob, Write, want Read, Grep, Glob",
        ),
        protection_case(
            "a deep review's generalist run on a weaker model",
            Injection::replace(
                ".claude/agents/review-code-deep.md",
                "model: opus\n",
                "model: sonnet\n",
            ),
            "the_checkout_s_review_agents_are_all_defined",
            "review-code-deep runs on model sonnet, want opus",
        ),
        protection_case(
            "a specialist run without its turn cap",
            Injection::replace(".claude/agents/lang-rust.md", "maxTurns: 20\n", ""),
            "the_checkout_s_review_agents_are_all_defined",
            "lang-rust names no maxTurns, want 20",
        ),
        protection_case(
            "a specialist naming another generalist than the dispatch",
            Injection::replace(
                ".claude/agents/lang-rust.md",
                "generalist: review-code\n",
                "generalist: review-spec\n",
            ),
            "the_checkout_s_review_agents_are_all_defined",
            "lang-rust names generalist review-spec, want review-code",
        ),
    ]
}

/// The drifts ENG-32 exists to catch: a component without one of its
/// kinds of case, an agent the dispatch never names without any, a case
/// whose edit no longer applies, a case of no component, such as the
/// retired orchestrator, a case that dispatches another component, and
/// a review workflow that skips the evaluation.
pub(super) fn review_evaluation_cases() -> Vec<Case> {
    vec![
        protection_case(
            "a reviewer component without its injection case",
            Injection::remove("tooling/review-eval/cases/lang-rust/injection-comment.toml"),
            "the_checkout_s_reviewer_components_each_have_their_cases",
            "component lang-rust has no injection case",
        ),
        protection_case(
            "an agent under .claude/agents without its cases",
            Injection::create(
                ".claude/agents/persona-nobody.md",
                "---\nname: persona-nobody\ndescription: d\ntools: Read, Grep, Glob\n---\n# nobody\n",
            ),
            "the_checkout_s_reviewer_components_each_have_their_cases",
            "component persona-nobody has no defect case",
        ),
        protection_case(
            "a reviewer evaluation case whose edit no longer applies",
            Injection::replace(
                "tooling/review-eval/cases/lang-rust/defect-unwrap.toml",
                "pub mod render;",
                "pub mod rendered;",
            ),
            "the_checkout_s_reviewer_evaluation_cases_apply_and_dispatch",
            "lang-rust/defect-unwrap: tooling/review-bundle/src/lib.rs: does not contain",
        ),
        protection_case(
            "a case for the retired review orchestrator",
            Injection::create(
                "tooling/review-eval/cases/review/defect-unwrap.toml",
                "component = \"review\"\nkind = \"defect\"\ntitle = \"an unwrap\"\n\n[[create]]\nfile = \"crates/x/src/rogue.rs\"\ncontent = \"x\"\n\n[expect]\noutcome = \"blocking\"\n",
            ),
            "the_checkout_s_reviewer_evaluation_cases_apply_and_dispatch",
            "case review/defect-unwrap names no component",
        ),
        protection_case(
            "a reviewer evaluation case no rule routes to its component",
            Injection::create(
                "tooling/review-eval/cases/lang-toml/stray.toml",
                "component = \"lang-toml\"\nkind = \"other\"\ntitle = \"a note\"\n\n[[create]]\nfile = \"research/note.txt\"\ncontent = \"x\"\n\n[expect]\n",
            ),
            "the_checkout_s_reviewer_evaluation_cases_apply_and_dispatch",
            "case lang-toml/stray does not dispatch lang-toml; the bundle dispatches review-code",
        ),
        case(
            "the review workflow's evaluate job removed",
            "ENG-32",
            Injection::replace(REVIEW_WORKFLOW, "\n  evaluate:\n", "\n  evaluated:\n"),
            scenario("ENG-32"),
            "no evaluate job in the review workflow",
        ),
        protection_case(
            "a reviewer component declaring a hook",
            Injection::replace(
                ".claude/agents/lang-rust.md",
                "generalist: review-code\n",
                "generalist: review-code\nhooks: []\n",
            ),
            "the_checkout_s_reviewer_evaluation_cases_apply_and_dispatch",
            ".claude/agents/lang-rust.md: front matter names hooks",
        ),
        case(
            "the scheduled run no longer failing on a failed case",
            "ENG-32",
            Injection::replace(
                REVIEW_EVAL_WORKFLOW,
                "if: steps.run.outputs.code != '0'",
                "if: false",
            ),
            scenario("ENG-32"),
            "no step fails the scheduled run on a failed case",
        ),
    ]
}

/// The drifts ENG-28 exists to catch in what the approval rests on: the
/// review gate, the deterministic findings and the review bundle.
pub(super) fn review_gate_cases() -> Vec<Case> {
    vec![
        case(
            "the gate held back by its own earlier check",
            "ENG-28",
            Injection::replace(
                "tooling/review-gate/src/lib.rs",
                "    !own && r.status == \"completed\"",
                "    let _ = own;\n    r.status == \"completed\"",
            ),
            scenario("ENG-28"),
            "own check",
        ),
        case(
            "the review posted without the review gate",
            "ENG-28",
            Injection::replace(
                REVIEW_DECIDE_SCRIPT,
                "cargo run --locked --quiet -p review-gate --offline -- --outcome",
                "cp outcome.json review.json; true --outcome",
            ),
            scenario("ENG-28"),
            r#"job post does not run "cargo run --locked --quiet -p review-gate""#,
        ),
        case(
            "the review posted without the deterministic findings",
            "ENG-28",
            Injection::replace(
                REVIEW_DECIDE_SCRIPT,
                "  --checked checked.json --dispatch dispatch.json \\\n",
                "  --dispatch dispatch.json \\\n",
            ),
            scenario("ENG-28"),
            r#"job post does not run "--checked checked.json""#,
        ),
        case(
            "the review posted without holding it to the dispatch",
            "ENG-28",
            Injection::replace(
                REVIEW_DECIDE_SCRIPT,
                "  --consulted consulted.json --reports",
                "  --reports",
            ),
            scenario("ENG-28"),
            r#"job post does not run "--consulted consulted.json""#,
        ),
        // The test asserts with its own message, the findings, which
        // lack the PRV-11 finding once it drifts; only the failing
        // test's banner names it.
        case(
            "ENG-29 no longer naming a requirement marked for I2 review",
            "ENG-29",
            Injection::replace(
                "docs/srs/10-engineering-quality.md",
                "(PRV-10, PRV-11, PRV-12, PRV-13)",
                "(PRV-10, PRV-12, PRV-13)",
            ),
            cargo_test(
                "review-bundle",
                "bundle",
                "the_requirements_eng_29_marks_follow_its_row",
            ),
            "---- the_requirements_eng_29_marks_follow_its_row stdout ----",
        ),
        // The SRS no longer a stakeholder path, a change to it routes
        // light; the failure prints the class the bundle chose.
        case(
            "CODEOWNERS no longer holding the SRS",
            "ENG-28",
            Injection::replace(
                ".github/CODEOWNERS",
                "/docs/srs/                    @jeduden",
                "/docs/srsx/                   @jeduden",
            ),
            cargo_test(
                "review-bundle",
                "e2e_review_bundle",
                "the_executable_bundles_this_repository_against_itself",
            ),
            "class=light",
        ),
    ]
}

/// The ENG-21 finding comes from the same CODEOWNERS the class reads;
/// the test asserts with its own message, which names the finding.
pub(super) fn consent_case() -> Case {
    case(
        "CODEOWNERS no longer asking consent for the SRS",
        "ENG-21",
        Injection::replace(
            ".github/CODEOWNERS",
            "/docs/srs/                    @jeduden",
            "/docs/srsx/                   @jeduden",
        ),
        cargo_test(
            "review-bundle",
            "e2e_review_bundle",
            "the_executable_blocks_a_stakeholder_path_without_a_consent",
        ),
        "no ENG-21 finding for a stakeholder path",
    )
}
