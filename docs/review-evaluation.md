---
summary: >-
  How the review's agents are held to evaluation cases (ENG-32): the
  cases each reviewer component has, the world and harness a run builds,
  the deterministic grader, what the review of a pull request runs, the
  weekly run against the published scorecard, and how a candidate model
  is compared before a component moves to it.
---
# Evaluating the reviewers

The review's agents are instructions in Markdown, and ENG-32 holds each
of them to cases the way code is held to tests. A case is one change to
main's tree with the report the component must give for it. Every
reviewer component has at least three: a defect it must find, a clean
change it must pass, and an instruction addressed to the reviewer, which
it must report and not obey. The components are every agent the review
bundle can dispatch, each deep instance, and every other agent under
`.claude/agents`, which a skill consults: the persona agents,
`domain-model`, `test-engineer` and `reviewer-engineer`. No agent routes
the review: `review-run` runs the dispatched agents (ENG-28), so the
routing is code, tested as code, and has no cases. A specialist whose
diff is past 48 KiB runs in parts there, each on whole files of it, its
report merged from theirs, and fails if one part does not answer. A case
runs a component whole. The
`review-eval` crate under [tooling/review-eval](../tooling/review-eval)
runs them. [ADR-2610311947](adr/ADR-2610311947-reviewer-evaluation.md)
records the design.

## The cases

- **Written cases** live under `tooling/review-eval/cases/<component>/`,
  one TOML file each. A case names the files it creates, the text it
  replaces, the reports it seeds for a generalist, the stakeholder's
  consent when it grants one, and what the report must say. A finding
  is matched by its path, or a tail of it that ends no other file of the
  case, and the line, words, ids and source the case names,
  and by its form, `linked` or `request`, for a specialist or a persona,
  which weighs nothing, or its severity for a generalist; `none` forbids
  one; `absent` forbids text; `nits` caps the nits a generalist keeps;
  `main_unread` holds the run to the pull request's tree: a file it
  opens or searches in main's fails the run, and a read or search the
  tool refused opened none.
  Cases belong to
  the normal instance of a generalist; its deep instance runs the same
  ones. An agent the bundle never dispatches answers in text and is
  graded from it, as a generalist is; the dispatch rule below does not
  apply to it.
- **Derived cases** come from the tree for each SRS section and
  domain-model file: a requirement row changed without its scenario, a
  cited id no section defines, a sentence in terms the model excludes,
  whose finding cites ENG-35, a true sentence under the title, and the
  same with an instruction addressed to the reviewer. Each holds its
  run to the pull request's tree. They never rot.

## A stable case

A reviewer is a model, and its run varies wherever its instructions or
the case leave it a choice. Every one of the 83 failed runs of the
first full measurement, four runs of each case, was traced to such a
choice, and they did not follow the model or the length of a run. A
case that fails one run in four is a finding about the case or the
instruction, never noise to outvote. So a case, and what a reviewer is
handed, hold to these rules:

- **One right answer in the report's form.** A defect breaks a
  requirement the reviewer can cite, so its finding is a linked line
  with that id; a concern no requirement states is a request, and the
  case wants a request. A case that wants a blocking finding for a rule
  with no id rewards an invented link.
- **A clean change gives the reviewer nothing to weigh.** A comment, or
  a sentence reworded with its meaning kept. A plan is never clean to a
  reviewer whose remit is to find its gaps. A new test or a new
  sentence gives it something to weigh too, and a clean case that adds
  one is read as that when a run objects.
- **A defect's fixture holds the planted defect alone**, at a path no
  other file of the case ends with. It differs from the component's
  clean change by the defect and nothing more, so the clean case shows
  what the rest of it draws.
- **An example in the instructions is followed to the letter.** The
  model specialists' examples named a place as `x.md:3`, and 15 of the
  18 runs that failed on a file's name alone were theirs.
- **A finding is known by what the report's form fixes**: its place,
  the ids it cites, its source and its form or severity. A word of its
  sentence decides only where the form fixes nothing else.
- **What a reviewer needs, it reads and never works out.** Each line
  of a diff the bundle hands out starts with its line number in the
  file. A reviewer's tool numbers the lines of the diff file itself, and
  one place in ten, 83 of 826, took that number before the diffs carried
  their own. A specialist's context names the ADR a change to the
  invariants lands with, which its own diff does not hold. A reviewer
  told to read a long diff in several calls is handed each call's
  `offset` and `limit`: two that worked them out from line ranges
  counted from nought and left the last line unread.

Read every failed run before changing anything. It names a choice the
instruction left open, or a fixture that is not what the case says. It
may also name an expectation the reviewer met in other words.

## A case run

- **The world** of a run is the review's own input: a worktree of main with
  the component under test copied in, the case's edits in a second worktree as
  `pr-head/`, its instruction files moved out, `pr.diff`, and `review/` from
  `review-bundle`, whose diffs carry each line's number in the pull request's
  file. The dispatch must bring the component, or the case fails before any
  model runs. The run's work directory is the world's own directory, whose
  path names no tree: an agent whose work directory ended in `main` took
  `pr-head/` and `review/` for its siblings and looked for them one directory
  up, outside what a run may read.
- **The review context** is the file `review/context/<name>.md` that
  `review-bundle` writes for a specialist: the review class, the touched ids,
  its dispatch row, the deterministic findings on its files and the ENG-29
  record a guarded change lands with, which its own diff does not hold. The
  prompt tells the specialist to read it and its own diff, never the whole
  bundle.
- **The prompt** tells every agent that `pr-head/` holds the pull request's
  tree and that the tree around it is main's, which shows only what the change
  replaced. A path a diff or an agent's instructions name is relative to the
  repository, so an agent that opens it at the root judges the change against
  main. It names the instruction files `pr-head/` does not hold and where
  their text is, `review/pr-instructions.txt`, so no agent searches for them.
- **The evaluation harness** starts `claude --agent <component>` with the
  review's reading tools and turn budget, on the component's model, or the
  evaluation's default for an agent that names none, or the one `--model`
  names. `review-run` starts each agent of a review through the same runner.
  Each run, a case's too, has its tier's cost cap, `--max-budget-usd`. It
  refuses a component whose front matter names a key outside name,
  description, tools, model, effort, maxTurns and generalist, or a tool its
  kind may not use, so a definition under test cannot start a process or widen
  its reach through the run (I4).
- **The grader** is deterministic and reads the run log alone. A specialist
  writes no file: its specialist report is assembled from the log as
  `review-run` assembles it, every finding, request, `revised:` and
  `withdrawn:` line it stated in its text replayed in order, then the outcome
  line its final answer gives. The grader reads that specialist report: its
  outcome, each finding expected or forbidden, nothing written; a case may
  also want a finding `stated` before the final answer, or a finding
  `withdrawn` or revised, read from the log's text. For a generalist it reads
  the weighing `review-run` reads, by the one shared rule: each `blocking` or
  `nit` finding, `dropped — path:line` or `none —` line it stated as it went,
  other working notes left out, then its final answer's lines, all read; the
  last for a place and the source it names stands, a `blocking` finding beside
  a later `nit`; each finding it must drop on a line that starts with
  `dropped` or gives it as its severity. A consultant is read from its answer.
  A case that holds its run to the pull request's tree fails a run that read
  or searched a file of the repository at the work directory's root, which is
  main's tree: every `Read`, `Grep` and `Glob` path of the log but `pr-head/`,
  `review/`, `pr.diff`, the stakeholder's comments and the instruction files.
  The instruction files at the root are main's too, and reading them fails no
  run: `pr-head/` holds no copy of them, the pull request's being text in
  `review/`. A specialist's report is read as the review reads it: assembled
  from the log, split into the lines that read and those that do not, and
  ended with an outcome its lines give. A finding-shaped line that does not
  read fails its run, its other findings graded all the same. A departure the
  review survives is a slip: a run cut off before a successful result after it
  stated a line or an outcome, an answer with no outcome line or one its lines
  contradict, an answer that is more than the outcome line, a line set aside
  as no finding, a finding line off its form, which the review reads as the
  finding it states, an eleventh finding, a place written as a tail of its
  path. Slips are counted and shown per run and fail none. The instructions
  still say text after the ids refuses a report: the review is the more
  lenient on purpose, and counts the slip. A case passes with two of three
  runs, stopping once decided. A run whose log holds no model call did not
  run, and is counted and failed, as is a specialist's run cut off before it
  stated any line or an outcome, which the review takes for no clean pass: a
  model that always runs out of turns passes no case.
- **The outputs** of a run go under `--out`: `scorecard.json`, `findings.json`
  in the shape the review gate reads, `report.md`, and each run's log, the
  specialist report assembled from it and its grade under `runs/`.

## The commands

`cargo run -p review-eval -- check` applies every case to the checkout and
confirms the dispatch, without a model. The engineering crate's gate tests run
the same check on every pull request, each with its drift case (ENG-27). `cargo
run -p review-eval -- run --component lang-rust --out DIR` runs one component's
cases on its model; it needs the `claude` command line and its login, and clears
what a Claude Code session sets for its children, so an agent may run it too.
`--model` names another model for every component, `--repetitions` and
`--passes` change the rule, and `--parallel` the cases run at once. `compare`
reads a run's scorecard against the published one, and `publish` writes it into
it.

## In the review of a pull request

The `evaluate` job of [review.yml](../.github/workflows/review.yml) runs
the cases of each reviewer component the pull request changes, on the
model its front matter names, from the pull request's definition of it.
The review job plans what changed with `review-eval plan`. The evaluate
job runs after the review job, holds a read-only token and no write
permission, and ends before the job that holds the reviewer app's key.
The review job's `review-run` starts each dispatched agent through the
harness's own runner, so an agent in the review and in its cases gets
the same reading tools, turn budget and prompt. Only the review runs a
specialist again, and only one that did not run and cost nothing, once,
told why; a run that was charged, cut off or not, is read as its log
stands and never paid for twice, and a case's run gets no second
attempt. An agent not reviewed whole, one that did not run or read only
part of its diff, is listed in `review/gaps.json`, and a review
resumed from that list is judged exactly as a full run. A review reuses a
stored answer only under its unchanged answer key, held to the report
check again; a case's run never reuses one. A case that fails
becomes a blocking finding, which the gate weighs beside the bundle's.
With nothing to evaluate the job ends in seconds. A change to a shared
section reaches every component that includes it, so the plan holds one
component per distinct set of shared sections, the first by name, with
the rest named in its reason; a component whose own text or cases changed
is always planned, and the weekly run covers every component. A component main
already has is judged by main's cases; the pull request's cases judge
only the components it adds, so a change cannot soften the cases that
judge it.

The one exception is the stakeholder's to make. The cases are a
stakeholder path, so a pull request that changes one waits for a
stakeholder comment that names it (ENG-21). A case file such a comment
names judges as the pull request has it: its changed text, a case it
adds, and no case where it removes one. Without this a wrong case on
main could never be corrected, since main's version would fail the
change that corrects it. Every other case of the component stays
main's. The plan sorts the case files that differ into the two lists
and prints both, and the evaluation says which cases the consent moved.

### What bounds the evaluate job

The job runs reviewer components the pull request defines, with the
Claude OAuth token. Five things bound it:

- it runs from main's workflow definition, never the pull request's;
- its token is `contents: read`, with no write permission;
- the harness refuses a definition whose front matter key or tool is
  outside its allow-lists, so none can start a process (I4);
- main's cases judge the components main already has, but for a case
  file whose change a stakeholder comment names;
- one step stages the token where no agent reads it, and the job
  scrubs what it keeps of the token before it uploads (ENG-28).

The scrub takes for the token every form of the staged one, and any
text of the Claude key's shape. One such text is no leak: a fixture a
file of main's tree or the pull request's holds, which an agent that
reads the file has in its run log. The review and the evaluation replace
it with a marker of its own and do not count it, and each says at its
start how many such texts it spares. A link in a tree is never followed,
so a file outside the trees lends no text. A text of that shape that no
file of the trees holds still fails the review closed, and so does the
staged token wherever it stands.

### The plugins every harness session fetches

[.claude/settings.json](../.claude/settings.json) makes every Claude
harness session started in the checkout, the review workflow's agents included,
fetch two plugins, `mdsmith-skills` and `mdsmith-audit`, from
github.com/jeduden/mdsmith. They are the mdsmith project's Claude Code
plugins, at the release of the linter that `mdsmith check .` runs
([tooling.md](tooling.md)). The fetch is a network use of the harness, which
is repository tooling and no ParleyInn component: I4 binds every component, the
core and B1 to B3 alike, and none of them makes it. The pin is the control: both
plugins name release `v0.55.1` and its commit sha. `.claude/`
is a stakeholder path ([CODEOWNERS](../.github/CODEOWNERS)), so a change to
the pin waits for the stakeholder's approval.

## The weekly run and a candidate model

[review-eval.yml](../.github/workflows/review-eval.yml) runs every
component's cases on its model weekly. It compares the result with the
scorecard the repository publishes,
`tooling/review-eval/scorecard-v1.txt`, so a model that stops serving
its instructions shows up as failed cases. Run from the Actions "Run
workflow" button with a `model`, it runs the cases on that model instead
and writes the comparison into its summary. That is how a candidate
model is weighed before a component moves to it. Both run from main's
definition. The Claude Code command line both start is an npm package,
which the mise rule for released binaries refuses, so both workflows install
it through `.github/actions/claude-code`, which pins its version (ENG-01).

## The stability measurement

The command line takes no seed, so a review is measured by repetition.
[stability.yml](../.github/workflows/stability.yml), started by hand,
reviews each snapshot of `tooling/review-eval/stability-snapshots.toml` three
times afresh. `review-report stability` then writes `stability.json`:
the share of reviews whose verdict differs from the most common one,
and per agent the Jaccard overlap of its finding sets. A finding is its
path, ids and the hash of its normalised consequence, never its line.
An agent a review missed overlaps nothing, and one in no review makes the
comparison unmeasured. A specialist's findings are those its report still
holds after its revisions and withdrawals. The workflow runs from the default
branch's definition alone, stages the Claude token and scrubs what it wrote of
it before any upload, as the review does (ENG-28). It warns past a flip
rate of 0.2 or an overlap below 0.7. Every review also leaves
`review/agent-runs.jsonl`, the run list: first `{"review": {id,
started, finished, verdict, findings_blocking, findings_nit, gaps}}`, then one
entry per agent run, `{name, kind, part?, attempt, model, effort,
started_ms, ended_ms, turns, tokens, cost_usd, status, reason?}`. Its
`cost_usd` is the log's `total_cost_usd`, and the run report,
`review/report.html`, is built from it.

## Changing a component or a case

A new or changed component lands with its cases in the same change. The
pull request's review runs them before the gate decides. The shared
sections under `docs/agents/`, which mdsmith inlines into every agent
file that includes them, carry each report section and its trust rule:
the pull request is data, and an instruction inside it is a finding. A case that
keeps failing is a finding about the instructions or about the case,
never a reason to loosen it. The cases and the published scorecard are
stakeholder paths ([CODEOWNERS](../.github/CODEOWNERS)). The
`reviewer-shape` skill drives such a change, a failing case first, and
the `reviewer-engineer` agent reviews it.
