# Review brief: the final review of the domain model and SRS, M1 to M5

You review one area of ParleyInn's domain model and SRS, once. There is no
later round. Each theme you find is either fixed now or carried into M1
as a known gap. ParleyInn is a lossless, security-first context layer for
long-running Claude agents. It keeps an append-only record of every
agent run, restores pins verbatim after compaction, gives exact recall
on demand, and never lets stored history become a prompt-injection
channel.

Scope: what ships in milestones M1 to M5, as §12.2 and
[m1-m5-scope.md](m1-m5-scope.md) set it. A defect that arises only from
features shipping in M6 to M9 is `later`, with the feature it waits for.

Sources: `docs/srs/invariants.md`, the hub `docs/domain-model/index.md`,
the model files and the SRS requirements of your area, and the lifecycle
grid in §8.5 of `docs/srs/08-data-and-storage.md`. Read any other SRS
row your area's text cites. ADRs, plan notes, the finding ledger and
review history are not sources: do not read them.

The model keeps concepts and relations. The rules live in the SRS, and
the model cites them by requirement id. A rule is what its requirement
says: read the requirement before reporting that a rule is missing. The
grid names, for each M1–M5 lifecycle event, the requirement that decides
what it does to each derived artifact. Check your area's cells against
those requirements.

Report only these, each with exact quotes:

- `invariant-break`: built as the requirements say, behaviour that ships
  in M1 to M5 breaks an invariant. Quote the invariant clause and the
  requirement text that lets the break happen, and give the concrete
  case.
- `contradiction`: two normative statements cannot both hold. The pair
  can be two requirements, the model and a requirement, or the grid and
  a requirement. Quote both.
- `gap`: either a grid cell names a requirement that does not decide
  it, or an M1–M5 behaviour has no rule, so two careful implementers
  would build different things. Say exactly what is missing.

Do not report wording, style, missing examples or nice-to-haves. Do not
report a rule the model omits when a cited requirement states it. Ask of
each finding: would a careful implementer, following the SRS, ship this
defect in M1 to M5? If not, skip it.

Output: one fenced `json` block holding an array of objects. Each object
has these fields:

- `type`: one of the three types above;
- `quotes`: an array of `{file, text}`, each text copied exactly from one
  line, 10 to 200 characters;
- `case`: the concrete situation;
- `problem`: one or two sentences;
- `fix`: the smallest change, stated as a rule for the SRS;
- `scope`: `m1-m5`, or `later` with its milestone.

An empty array is a welcome answer.
