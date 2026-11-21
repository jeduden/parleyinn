---
id: 2611190753
title: "Review stability: the open options, each with its tests"
status: "🔳"
summary: >-
  Plan 2610282358 closed the causes a case, an instruction or the
  bundle held. About one reviewer run in a hundred still departs, and
  the review's own rules turn some of those into a blocked pull
  request. This plan takes the options that were left open, one small
  pull request each: a finding line off its form is read as the finding
  it states, a blocking finding stands until a dropped line withdraws
  it, a stakeholder's comment changes nothing an agent reads, so no
  agent restates a missing consent, a review that runs dry says what to
  do, a generalist starts on what is left of the pull request's cap,
  the evaluation holds ids to the known requirements,
  and its world holds the files a pull request adds.
model: opus
depends-on: [2610282358]
---
# Review stability: the open options, each with its tests

## Goal

A reviewer's run that departs in form, and not in substance, no longer
blocks a pull request or fails an evaluation run by chance. Each rule
that changes lands with the tests that prove it: an evaluation case
where a reviewer's behaviour is part of it, and a scenario, unit tests
and a drift case where only the tool changes.

## Context

Plan [2610282358](../2610282358_reviewer-evaluation/plan.md) measured
the reviewers: 83 of 956 runs failed on main, then 15 of 964, then
about one in a hundred. Its
[phase 3 handoff](../2610282358_reviewer-evaluation/phase-3.result.md)
lists what still fails by chance. At that rate an evaluation of 104
cases blocks by chance about once in thirty, and the weekly run of 241
cases about one week in fourteen.

The 882 graded runs since 17 November were read for lines the review
could not read. Five runs held one. Four of them named their place and
their ids and missed only the form: three ended their sentence with the
id, one wrote a remark after it. Earlier runs show the same shapes: 92
lines cited no id after a dash, 44 were in neither form, 8 wrote more
after their ids.

What was searched, and what is reused:

- The reader in
  [report.rs](../../tooling/review-gate/src/report.rs) already takes a
  request whose motivation follows a semicolon or `because` (#132). It
  did so silently. Phase 1 makes that one of the near forms, counted.
- The review already lists the lines it sets aside in
  `review/ignored.json` and counts them in its summary. A finding read
  off its form is listed there too, with its own reason and count, so
  no new file or output is needed.
- The grader in
  [grade.rs](../../tooling/review-eval/src/grade.rs) reads a
  specialist's report as the review does, so the evaluation follows
  the reader with one new slip.
- The ENG-28 record check in
  [review_logs.rs](../../tooling/engineering/src/review_logs.rs) runs
  `review-run` and the gate once per kind of log. A new kind proves the
  rule end to end.

Not taken now: a structured call for a finding in place of a line of
text, a small change reviewed by one reviewer, a work directory that
holds one tree, and another word for `request:`. Each changes more than
its evidence asks for today.

## Tasks

1. A finding line off its form is read as the finding it states, its
   slip listed and counted (phase 1, ENG-28).
2. A generalist's blocking finding stands until a `dropped` line
   withdraws it; a later nit at its place no longer replaces it
   (phase 2, ENG-28).
3. A stakeholder's comment changes nothing an agent reads: the bundle
   shows an agent no finding that turns on consent, so the outcome a
   comment's review reuses holds no line that restates a missing
   consent (phase 3, ENG-28).
4. A review that runs out of its pull request's budget names the
   budget, what was spent and how it is reset (phase 4, ENG-28).
5. A generalist whose whole cap can no longer fit starts once half of
   it is left of the pull request's cap, capped at what is left
   (phase 5, ENG-28).
6. The evaluation holds a finding's ids to the known requirements, as
   the review does (phase 6, ENG-32).
7. The evaluation's world holds the files a pull request adds. A mark
   on added lines that stand word for word on main was measured and
   not built (phase 7, ENG-32).
8. A generalist gives its whole weighing in its final answer, after
   the model's API refused one as asked for its reasoning (phase 8).
9. A run that ended on an error holds no answer, and the finding on a
   generalist so ended quotes the error (phase 9, ENG-28).

## Execution

| Phase | Model | Gate                                                                                                                |
| ----- | ----- | ------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | The ENG-28 record check runs `review-run` and the gate on a log with a finding off its form; three drift cases fail |
| 2     | opus  | A generalist's log that states a nit after a blocking finding at one place keeps the blocking one; its case passes  |
| 3     | opus  | The bundle writes an agent the same files whatever the consent; clean cases on stakeholder paths pass unconsented   |
| 4     | opus  | The gate's check summary of a review that ran dry names the budget, the spend and the reset                         |
| 5     | opus  | With 0.90 USD left of the cap a review starts every agent, each generalist capped at the rest                       |
| 6     | opus  | A run citing an id no requirement has fails its evaluation case, as the review blocks it                            |
| 7     | opus  | A world built from a tree that adds a section holds it; four drift cases fail                                       |
| 8     | opus  | The generalists' cases pass on their models, no line before the final answer                                        |
| 9     | opus  | The finding on a generalist that ended on an error quotes it                                                        |

## Phases

<?catalog
glob:
  - "phase-*.md"
  - "phase-*.result.md"
sort: numeric:n
header: |

  | # | Status | Phase |
  |---|--------|-------|
row-expr: |
  [if result {
    "|  | ↳ | \(summary) |"
  }, if !result {
    "| \(n) | \(status) | [\(title)](phase-\(n).md) |"
  }][0]
footer: |

?>

| #   | Status | Phase                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| --- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | ✅     | [A finding line off its form is read as the finding it states](phase-1.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|     | ↳      | ENG-28 now has the review read a finding line that names its place and ids off its form as the finding it states: a remark after the ids, a sentence that ends with them, a request with no motivation set apart. The report holds it in its form, and the slip is listed in review/ignored.json and counted apart. A line that names no id still blocks. The ENG-28 scenario step, a new kind of log in its record check and three drift cases land with it.                                                                                                   |
| 2   | ✅     | [A blocking finding stands until a dropped line withdraws it](phase-2.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|     | ↳      | ENG-28 now has a generalist's blocking finding stand beside a nit it states later at the same place and source, until a dropped line withdraws it. In 582 recorded generalist runs, 4 stated such a nit, each a second point and none a downgrade, and three of them failed an evaluation case for the blocking finding the review had dropped. The ENG-28 scenario step, a fourth run in its weighing check and a drift case land with it; no instruction changes.                                                                                             |
| 3   | ✅     | [A stakeholder's comment changes nothing an agent reads](phase-3.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|     | ↳      | ENG-28 now has the review bundle write an agent the same bundle and review context whatever the stakeholder's comments say: the findings that turn on consent go to the review gate alone, and each agent reads that consent is the gate's check. So the outcome a comment's review reuses holds no line that restates a missing consent. Two clean cases on stakeholder paths with no consenting comment failed 5 of 8 runs before the change and none for consent after it. The ENG-28 scenario step, a record check and three drift cases land with it.      |
| 4   | ✅     | [A review a cost cap stopped says what to do](phase-4.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|     | ↳      | ENG-28 now has a step that fails without deciding post the check failed with its own error in the summary. The error names each dispatched agent that did not run with the reason the review outcome gives. Where a cost cap kept one from starting, that holds what the pull request's reviews cost and the cap, and the error adds how what they spent is reset. The gate's rule is unchanged: it decides no review that left an agent out. A scenario step, a record check and four drift cases land with it.                                                |
| 5   | ✅     | [A generalist starts on what is left of the pull request's cap](phase-5.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|     | ↳      | ENG-28 now has a generalist whose tier's cap can no longer fit in the pull request's cap start all the same once half that cap is left. It is passed what is left as its own cost cap, so no run spends past the pull request's cap. A run paid for that left no result is charged the cap it was passed. Of 596 generalist runs on record one cost 0.70 USD and no other more than 0.25. The ENG-28 scenario step, a record check and three drift cases land with it; no instruction changes.                                                                  |
| 6   | ✅     | [The evaluation holds a finding's ids to the known requirements](phase-6.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
|     | ↳      | ENG-32's grader now refuses a finding that cites an id no requirement or invariant of the case's world has, with the function the review reads the ids with. The model specialists' missing-id case passed 6 of 44 runs under it, and 126 of 132 once their report section said what such a finding ends in. The section now also says that text speaking to the reviewer breaks ENG-28: 4 of 220 injection runs had ended in no id, and none of 132 does. A row of the ENG-32 scenario's grading table and two drift cases land with it.                       |
| 7   | ✅     | [The evaluation's world holds the files a pull request adds](phase-7.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
|     | ↳      | ENG-32's evaluation now builds a run's world with the files the evaluated tree adds: each in both trees, an instruction file in the pull request's tree alone, so only the component under test loads beside the agent. A specialist added with its section had failed a case 2 of 2 runs for the section its world lacked. The mark on added lines that stand on main was measured and not built: a moved row cost 5.4 of 20 turns in 56 runs. Four drift cases land with it.                                                                                  |
| 8   | ✅     | [A generalist gives its whole weighing in its final answer](phase-8.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|     | ↳      | A generalist is now asked for every line of its weighing in its final answer and none before it, after the model's API refused the code generalist of one pull request twice as asking for its reasoning in its text. ENG-28 and ENG-32 keep their rows: the review still reads the whole run log, and the evaluation counts a line stated early and left out of the answer as a slip. The generalists' 26 cases passed in 52 of 52 runs with no slip. The refusal did not occur outside CI, so the fix is not proven against it. One drift case lands with it. |
| 9   | ✅     | [A run that ended on an error says so](phase-9.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
|     | ↳      | ENG-28 keeps its row: the review now keeps the harness's text apart from an agent's answer when a run ended on an error, a usage limit or a call the model's API refused. The finding on a generalist so ended quotes the error, and the summary names an error among the ways a run ends without a successful result, where it named only a turn budget, a deadline or a cost cap. Two drift cases land with it.                                                                                                                                               |
<?/catalog?>

## Acceptance Criteria

- [ ] A finding line that names its place and ids off its form reaches
      its generalist as the finding it states, and the review counts it
- [ ] A line that names no id, or misses the dash after its place, is
      still its own blocking finding
- [ ] A blocking finding a generalist stated is never lost to a later
      nit at its place
- [ ] A pull request whose only open point is consent is approved by
      the review the consenting comment starts
- [ ] A review that ran dry tells the author the budget and the reset
- [ ] A deep review of a large change starts its generalists inside
      the pull request's budget
- [x] The evaluation fails a run that cites an id no requirement has
- [x] A specialist added with the section it checks runs its cases
      in a world that holds that section
- [x] A generalist is asked for its whole weighing in its final
      answer, and its cases pass on its model
- [x] A run that ended on an error is reported with its error, never
      as an agent that answered nothing
