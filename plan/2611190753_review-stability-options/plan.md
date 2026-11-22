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
  do, a generalist reserves what
  generalists cost, the evaluation holds ids to the known requirements,
  and the bundle marks added lines that stand on main.
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
5. A generalist reserves what generalists cost on record, with a
   margin, in place of its whole cap (phase 5, ENG-28).
6. The evaluation holds a finding's ids to the known requirements, as
   the review does (phase 6, ENG-32).
7. The bundle marks added lines that stand word for word on main, and
   the evaluation's world holds the files a pull request adds
   (phase 7, ENG-28 and ENG-32).
8. A generalist gives its whole weighing in its final answer, after
   the model's API refused one as asked for its reasoning (phase 8).
10. A place a generalist spells in the pull request's tree is read as
    the same place, for its weighing and its findings (phase 10,
    ENG-28).
11. A specialist's context quotes whole the requirements its files
    touch, and `lang-gherkin` searches the specification for none
    (phase 11, ENG-28).
12. A read the file system resolves to a specialist's diff covers it,
    whatever path it took (phase 12, ENG-28).
13. A specialist whose recorded runs reached its turn budget gets one
    above the most turns they took (phase 13, ENG-28 and ENG-32).

## Execution

| Phase | Model | Gate                                                                                                                |
| ----- | ----- | ------------------------------------------------------------------------------------------------------------------- |
| 1     | opus  | The ENG-28 record check runs `review-run` and the gate on a log with a finding off its form; three drift cases fail |
| 2     | opus  | A generalist's log that states a nit after a blocking finding at one place keeps the blocking one; its case passes  |
| 3     | opus  | The bundle writes an agent the same files whatever the consent; clean cases on stakeholder paths pass unconsented   |
| 4     | opus  | The gate's check summary of a review that ran dry names the budget, the spend and the reset                         |
| 5     | opus  | A deep review whose generalists cost what they do on record starts every agent inside the cap                       |
| 6     | opus  | A run citing an id no requirement has fails its evaluation case, as the review blocks it                            |
| 7     | opus  | A specialist's case for a moved file passes: no finding at a line main already holds                                |
| 8     | opus  | The generalists' cases pass on their models, no line before the final answer                                        |
| 10    | opus  | The ENG-28 weighing check approves a weighing spelled in the pull request's tree; two drift cases fail              |
| 11    | opus  | The ENG-28 context check finds ENG-28 quoted whole; two drift cases fail; lang-gherkin's cases pass in fewer turns  |
| 12    | opus  | A read that climbs out of the pull request's tree to its diff, or a kept part's read where it ran, covers it        |
| 13    | opus  | The five specialists' cases pass on their models with 30 turns, above the most any recorded run took                |

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
| 8   | ✅     | [A generalist gives its whole weighing in its final answer](phase-8.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|     | ↳      | A generalist is now asked for every line of its weighing in its final answer and none before it, after the model's API refused the code generalist of one pull request twice as asking for its reasoning in its text. ENG-28 and ENG-32 keep their rows: the review still reads the whole run log, and the evaluation counts a line stated early and left out of the answer as a slip. The generalists' 26 cases passed in 52 of 52 runs with no slip. The refusal did not occur outside CI, so the fix is not proven against it. One drift case lands with it. |
| 10  | ✅     | [A place spelled in the pull request's tree is the same place](phase-10.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|     | ↳      | ENG-28 gains one sentence: the review reads a place a generalist spelled in the pull request's tree, `pr-head/<path>`, as the file at that path. The review of pull request 154 had blocked on a weighing that named all four of its places that way, a spelling the evaluation's grader already took. The findings such lines keep are posted and tracked at the repository's path. A fifth run of the ENG-28 weighing check, named in its scenario step, and two drift cases land with it; no instruction changes.                                            |
| 11  | ✅     | [A specialist reads the requirements its files touch in its context](phase-11.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
|     | ↳      | ENG-28 gains a clause: each specialist's review context quotes whole each touched requirement its files change or name, one sentence or clause per line. A requirement is one table line a search shows only in part, and `lang-gherkin` had spent its twenty turns searching ENG-28's 13,856 characters. It now weighs each changed step against main's step, then against its row as its context quotes it, and leaves to CI the tags and bindings CI proves. The ENG-28 context check, two drift cases and a new `lang-gherkin` case land with it.           |
| 12  | ✅     | [A read the file system resolves to the diff covers it](phase-12.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|     | ↳      | ENG-28's review now counts every read of a specialist's diff, by whatever path the file system resolves to it. The local review of §2, §5, §5a and §7 written as blocks had named two specialists' parts partial that read their whole diffs: one by a path that climbed out of the pull request's tree, two kept on a resume and read under the first run's work directory. A climb through a link the pull request plants still covers nothing. No requirement or instruction changes.                                                                        |
| 13  | ✅     | [A specialist's turn budget sits above the turns its runs take](phase-13.md)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
|     | ↳      | The five specialists whose recorded runs reached their budget of 20 turns get 30, above the 26 the longest of them took: domain-model-hub, lang-markdown, lang-gherkin, test-engineer-unit and test-engineer-integration. The review of §2, §5, §5a and §7 written as blocks had named a part of the hub partial, cut off at 20 turns before it stated anything. No requirement or case changes.                                                                                                                                                                |
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
- [ ] The evaluation fails a run that cites an id no requirement has
- [ ] A reviewer is told which added lines stand on main already
- [x] A generalist is asked for its whole weighing in its final
      answer, and its cases pass on its model
- [x] A weighing that names its places in the pull request's tree is
      read as whole
- [x] A specialist's context quotes whole the requirements its files
      touch, and `lang-gherkin`'s cases pass without a search
- [x] A specialist that read its whole diff is never named partial for
      the path it read it by
- [x] Every specialist's turn budget sits above the most turns a
      recorded run of it took
