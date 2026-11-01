---
id: ADR-2610300031
title: "The review is run by code, not by an orchestrating agent"
status: proposed
summary: >-
  The review-run tool runs every specialist the review bundle
  dispatches, then every generalist, each as its own agent, and writes
  the review outcome from what they answered, so every dispatched agent
  is consulted by construction. A specialist weighs nothing: it states
  each finding linked to the requirements it breaks, or a requirement
  request, in its run log, which is its report; the generalists alone
  weigh. It replaces the orchestrating agent of ADR-2610262245; the
  layers, gate and required check stay.
---
# ADR-2610300031: The review is run by code

It supersedes ADR-2610262245, which is marked superseded and names
this record. The layers, gate and required check that record decided
stay, as "What stays" says; the reports as files change, as the
decisions on the run log say.

## Context

ADR-2610262245 has one orchestrating agent, on Sonnet at low effort,
call the specialists the review bundle dispatches and then the
generalists, and answer with the review outcome. The gate refuses a
review whose run log shows a dispatched agent left out.

That agent failed to decide the review of three pull requests. It
took a specialist call's reply that the call runs in the background
for an answer. It answered before the generalists were called, and
the gate refused its outcome. Rewording its skill
(jeduden/parleyinn#99) did not remove the failure, and the polling
that change added spends turns of a fixed budget. Calling every name
in a list and waiting for each is what a program does exactly; a model
does it only most of the time.

The pieces a program needs were already there. `review-bundle` writes
the dispatch. `review-eval` starts one agent as `claude --agent` with
the review's tools, write rule and turn budget, and reads its run log.
`review-gate` reads the outcome, the consulted list, the answers and
the reports.

Each specialist also marked its findings `blocking` or `nit` and ended
its report `outcome: sound`, `nits` or `blocking`, and a blocking
report failed the review whatever its generalist decided. The
specialists run on Haiku at low effort and see one language, section
or file. Their severity was a guess made without the whole change in
view, and a guess that blocked a pull request on its own. The
stakeholder ruled that a specialist must not categorize its findings.
A finding instead says what happens if it is not addressed, linked to
the requirement that outcome breaks. Where no requirement exists, the
specialist may ask for one to be added, changed or removed, with an
informal motivation, and the generalist weighs that request like any
other finding.

The report file was a failure class of its own. A specialist had to
write `review/reports/<name>.md` with its one writing tool and rewrite
it as it checked. It then answered two lines: the file's outcome line,
copied character for character, and the file's path. Runs answered an
outcome the file did not end with. They wrapped the outcome line. They
wrote the file late and ran out of turns before answering, leaving no
file at all. The stakeholder asked why the review did not read the
run log instead of asking the agent to log, then asked that a
specialist state each finding as it goes, so a run cut off by its turn
budget still leaves its findings in the log, and that the answer not
repeat them, which costs tokens.

## Decision

- **Code routes the review.** A tooling crate, `review-run`, reads the
  bundle's dispatch, runs every dispatched specialist, many at once,
  and, once every one has ended, every dispatched generalist, told its
  specialists' report files. Each agent runs through
  `review-eval`'s runner and process host with a 900-second deadline,
  and runs once more as the same agent when its run did not answer: it
  would not start, ended without a result, missed its deadline or
  answered nothing. A specialist, whole or one part, also runs once
  more when the check would refuse its assembled report: a line in no
  form, an unknown id, the forms' template place, an outcome its lines
  contradict, no outcome line or an empty report. The second prompt
  names what was wrong, the refused line among it, and the forms
  again, and the second run is judged alone; only a second run that
  again did not answer shows what both stated. The usage file counts
  the runs. No agent routes the review, calls
  another agent or writes the review outcome.
- **A specialist's report has two kinds of finding.** A linked finding,
  `path:line — if not addressed: <consequence> — <ID>[, <ID>...]`, each
  id a requirement id of the SRS or an invariant, `I1` to `I10`. A
  requirement request,
  `path:line — request: add|change|remove <requirement> — <motivation>`.
  A specialist writes no severity word.
- **The run log is the report.** No agent has a writing tool; a
  specialist's tools are Read, Grep and Glob, as a generalist's are. It
  states each finding as one line of its text the moment it finds it,
  between tool calls, never held back for the end. Its final answer is
  the outcome line alone, `outcome: findings — <sentence>` or
  `outcome: none — <sentence>`; a blank or stray line after it is
  tolerated, and the findings are not repeated.
- **A specialist changes its mind in two forms.**
  `revised: <finding line>` replaces every earlier finding at that
  line's `path:line`; a finding whose place moved is withdrawn, then
  stated, so a revision with an arrow, `revised: <old path:line> →
  <finding line>`, is malformed. `withdrawn: path:line — <reason>`
  removes every earlier finding at that place. One rule: the place names
  the finding. A revision naming a place no earlier line found at
  states its finding line, a whole finding in its own right. A
  withdrawal naming such a place removes nothing, so it is no error:
  a specialist that writes one for something it only checked has
  lost nothing. The assembly leaves `note: withdrew nothing at
  path:line — <reason>` in its stead, prose to the check, so the
  withdrawal stays visible (I6).
- **`review-run` assembles the report.** From the specialist's run log
  it replays the finding, request, revision and withdrawal lines of the
  main thread's text, in order, a line identical to an earlier one
  dropped, each finding in its canonical shape, then appends the outcome
  line the answer gives, and writes the result to
  `review/reports/<name>.md`, the file its generalist reads. A malformed
  line stays in the report as written, so the check names it.
- **The form fails closed.** The gate and `review-run` refuse a report
  with a finding, revision or withdrawal line in no form, at the forms'
  template place or citing no id, an outcome of `none` beside a finding
  or `findings` beside none, an answer with no outcome line or another
  outcome than the report ends with: each is read from the report
  alone, `review-run` names a report its second run left refused in a
  blocking finding of the outcome, and the gate then posts no review.
  Only `review-run`, which holds both trees, refuses a report citing an
  id that neither main's SRS nor the pull request's defines; it names
  it in a blocking finding the same way, and the gate, which reads an
  id by its shape alone, posts that outcome as a request for changes,
  so a made-up requirement cannot carry a finding. Free prose is left out of the
  report, but the instructions tell a specialist to write an unlinked
  concern as a request, never as prose, and to start no other line
  with a place and a dash.
- **An incomplete run is salvaged and still fails closed.** A run that
  did not end with a successful result, at its turn budget, its
  deadline, an error or an empty answer, is named in a blocking finding
  of the outcome and is not consulted, so the gate posts no review. Its
  report is assembled all the same, headed `incomplete: <why>` and
  ending with no outcome line, so the findings it stated reach its
  generalist and the author instead of being lost with the run.
- **The generalists alone weigh.** A specialist's outcome never
  becomes a blocking finding by itself. A generalist keeps a linked
  finding as `blocking` only when its consequence breaks the
  requirement or invariant it names on this change, or masks a bug;
  otherwise as a `nit`, or drops it. A request is never blocking: the
  generalist keeps it as a nit that asks the stakeholder the
  requirement question, or drops it with a reason, and `review-run`
  keeps a blocking finding kept from a request as a nit.
- **The verdict is a pure function, and fails closed.** `review-run`
  approves exactly when every dispatched agent answered, every
  specialist's report passes the form check, every generalist's answer
  holds a finding, a dropped line or a `none —` line, and no generalist
  kept a blocking finding. The findings are the generalists' kept
  lines; a line whose first word is `dropped`, or whose severity is, is
  left out, by the rule the evaluation's grader shares. A silent agent,
  a refused report and an answer that says nothing readable are each
  named in a blocking finding, so a silent generalist does not pass the
  review.
- **Personas use the same forms.** A persona agent dispatched by the
  review states its findings and answers as a specialist does; one a
  skill consults answers with its finding lines and then its outcome
  line, so every specialist the review reads speaks one grammar.
- **The evaluation grades the assembled report.** The grader reads the
  run log alone and assembles the report as `review-run` does; a case
  may want a finding stated before the final answer, or withdrawn.
- **A change no rule routes gets one generalist.** The review bundle
  dispatches `review-code`, or its deep instance, so something always
  reads the change. A dispatch that is empty all the same requests
  changes.
- **Stakeholder consent is checked in code.** The review bundle raises
  the ENG-21 finding for a change to a path CODEOWNERS assigns: blocking
  until a stakeholder comment consents, a nit once one does.
- **The gate's inputs stay.** `review-run` writes
  `review/outcome.json`, `review/consulted.json` (every agent that
  answered), `review/answers.json` and `review/usage.json`, the files
  the gate already reads, beside the report files it assembles. The
  gate still refuses to post a review that left a dispatched agent out,
  now a review whose run of that agent did not answer.
- **The models stay.** Each agent runs on the model, effort and turn
  budget its front matter names; `review-run` passes no model.
- **The workflow.** The review job installs the pinned Claude Code
  command line as the evaluate job does, builds `review-run` before the
  Claude Code OAuth token is in any step's environment, and runs it
  offline. The job keeps its read-only permissions, holds no
  reviewer-app key and hands the agents no GitHub token.
- **What stays.** The layers, the deep instances, the generalists
  reading report files, the post job's gate, the required
  `review-gate` check and the triggers stay as ADR-2610262245 has them.

## Alternatives

- **Keep the orchestrator and reword its skill again.** jeduden/parleyinn#99
  tried it; the failure stayed, and each reword is a guess a fixed
  turn budget pays for.
- **Move the orchestrator to Opus.** A stronger model fails less
  often, but still decides whether to wait, and costs more on every
  review.
- **Keep `claude-code-action` and give it a script.** The action runs
  one harness session; it cannot start an agent per dispatched name
  and wait for each.
- **Keep severity and calibrate it.** Better instructions and more
  cases could make a specialist's `blocking` rarer, but the one with
  the narrowest view would still decide alone that a pull request
  cannot land.
- **Let a specialist name a severity the generalist may override.** The
  generalist already weighs every finding; a second severity is a
  label the generalist must argue against, and the stakeholder ruled
  out categorizing.
- **Accept unlinked findings as prose.** A concern no requirement
  covers would then reach the generalist with no requirement question
  attached, and requirement work would lose it.
- **Keep the report file.** The file kept a second copy of what the log
  already held, under a writing tool the agents otherwise never need,
  and every mismatch between file, answer and log was a review that
  failed for its shape, not its findings. A run cut off before writing
  left nothing. The log is written by the harness, not by the model,
  so it is always there and always whole.
- **Have the final answer repeat every finding.** The log would hold
  each finding twice, the answer would cost as many output tokens as
  the review had findings, and a run cut off before its answer would
  lose them all the same. The outcome line is all the answer must add.
- **Discard what a cut-off run stated.** Simpler, and the run fails the
  review either way, but a half-finished check still found what it
  found; the author and the generalist see more with it kept, headed as
  partial, than with it gone.

## Consequences

- ENG-28 asks for the review to be run by code, not by an orchestrating
  agent, and its scenario holds the review workflow to `review-run`
  and refuses a harness session that would route the review.
- ENG-28 also describes the finding grammar, the revision and
  withdrawal forms, the assembly from the run log, its refusals, the
  incomplete report, the second run after a run that did not answer or
  a refused report, that the agents write nothing and that the
  generalists alone weigh; its scenario's gate table takes a report
  with a request, which approves, and one with a malformed finding, an
  incomplete one and one whose answer gives no or another outcome, each
  an error, and one revising or withdrawing a finding never stated,
  which approves.
- The orchestrator's skill, its reviewer component under ENG-32, its
  cases and its prompt and schema in the runner are retired (the
  skill's file goes in a follow-up once main runs `review-run`), together
  with its mentions in the SRS, the docs and the generalists' input
  section. ADR-2609301941 and ADR-2610262245 keep their text as the
  record of what was decided then.
- ADR-2610290002, the reviewer evaluation, also describes the
  orchestrator, and names the review workflow's model for an agent
  whose front matter names none, a model the workflow no longer has.
  ADR-2610311947 supersedes it whole and names the evaluation's
  default model (ENG-32) instead.
- The shared report sections under `docs/agents/`, the specialists'
  and the personas' own report text, the generalists' input section and
  the persona-review skill follow the finding forms; every specialist's
  tools line loses Write. The evaluation cases (ENG-32) expect
  `findings` or `none` and a `linked` or `request` line, may expect a
  finding stated before the answer or withdrawn, and the generalists'
  seeded reports use the new lines; the grader reads the run log alone.
- A requirement request lands with the stakeholder as a nit; the SRS
  changes only through the stakeholder's own change.
- A review runs in waves of agents, each up to 900 seconds and each
  agent at most twice. The repository has 56 specialists and the job
  starts 32 at once, so they take two waves and the generalists a
  third: 3 x 2 x 15 = 90 minutes at worst, and the job's cap is 120.
  Agents end in minutes in practice.
- What a run leaves without an accepted report is said, never read as
  clean: `review/gaps.json` and `review/not-reviewed.md` list each such
  agent or part and why. `review-run --only <gaps> --from <dir>`, behind
  `scripts/review-local.sh --resume`, reruns only those and what
  changed, keyed by `review/run-id.json`: the head, an inputs hash, and
  each definition's and diff's SHA-256. The verdict function is the
  same; nothing is reused across a changed head, definition or diff.
- After every run, failed or not, `review-run` writes
  `review/agent-runs.jsonl`, one entry per agent run with its tokens, the
  cost its log reported and why it was not accepted, and
  `review/report.html`, a static report with every string a run shaped
  escaped; the `review-report` crate builds both from the review's
  files, best effort, so neither changes the verdict. They go with the
  reports; the run logs stay in the job. No server or socket is added.
- A stability measurement, `.github/workflows/stability.yml`, reviews each
  state of `tooling/review-eval/stability-states.toml` three times
  afresh and measures the verdict flip rate and each agent's finding
  overlap (ENG-32). It runs on dispatch alone: about $15 a measurement, so
  its weekly schedule waits, commented out, for the stakeholder.
- The Claude Code OAuth token stays out of every agent's reach: one
  step in each agent job stages it in a credentials file under the
  runner's temporary directory, no step that runs an agent holds it,
  each agent is refused reads of that directory by name in its
  `--disallowedTools`, and of `/proc`, `/sys`, `/dev` and outside its
  trees, and every file and line review-run writes is
  scrubbed of that OAuth token first; a review whose agent's output
  held it requests changes naming the leak and fails, so nothing is
  posted.
- No new third-party dependency.

## Approval

This record is proposed. The stakeholder accepted its first text, the
review run by code, in `jeduden`'s comment 6534291701 on
<https://github.com/jeduden/parleyinn/pull/98>. The decisions on the
finding forms, the run log as the report and the generalists alone
weighing were folded in after that comment and await the
stakeholder's acceptance of the whole record on that pull request.
Until then ADR-2610262245 is marked superseded by a proposed record,
on purpose: the stakeholder accepts this one before that pull request
merges.
