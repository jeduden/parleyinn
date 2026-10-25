# Version 5: core repairs from the v4-core review

Folder GA = research/notes/domain-model-subset-search (the caller gives its
absolute path). GA/annotated/ is version 4 (snapshot GA/annotated-v4/); its
files end in `.md.ann`. Follow GA/repair-brief.md and GA/annotate-brief.md.
Findings: GA/evals/v4-core.findings.json (5 needs-fix, 14 minor, all on
core). Settled by the stakeholder, keep as is: the principal's own tunnel
and `git push` are the principal's tools outside I4; room trailers are
always on.

The v4 core halved the needs-fix count of v3 (10 to 5). Each ruling below
tightens; none adds a concept. Prefer deleting or narrowing words to adding
sentences, and keep the core one node, one principal, never cloned, seat
keys never revoked.

## Needs-fix

S1 Quarantining a pin (finding 0): quarantining a pin quarantines its
   creating event and every version event, and takes it out of every
   restore block (widening when it restored). A quarantined version event
   counts for nothing: the newest version not quarantined restores, and a
   change that removes or reverts a restoring version is widening.
S2 Restore paths and the budget (finding 1): turning off a restore path or
   lowering the pin budget is widening, whatever verb carries it, and
   repository configuration never makes such a change.
S3 Structural fields that reach the model (finding 9): restore blocks and
   compaction guidance carry only ids, kinds, counts, addresses, key
   fingerprints and version numbers. Tool names and paths, though
   sanitized, reach the model only through recall. Keep the trusted
   structural field list for what the trust policy reads; narrow only what
   the closed paths may carry.
S4 Principal acts in the record (finding 10): only the CLI or TUI at a
   principal surface records an `operator` event marked with a principal
   surface. ParleyInn's hook handlers, MCP server and kernel worker never do,
   and an `operator` event without that mark is never a principal act. A
   process of the same OS user that signs with the device key outside
   ParleyInn's code is residual risk R3 (§6.1): only a head receipt kept off
   the machine shows it. Do not invent a new key.
S5 Install and uninstall (finding 11): they run only at a principal
   surface or through the harness's own plugin install or the managed
   install path; from a process the harness or a run started they are
   refused and counted (I6).

## Minor (apply each with the smallest wording change)

- #2: "pins that do not restore" becomes "pins that do not qualify"; a run
  seat may pin any type, and its pins never qualify in core.
- #3: Pin and Unpin name the run seats a run seat replaced in the same run,
  as Contributor does (R8 of v4).
- #4, #14: configuration pins: "any other pin, edit or unpin act on one is
  refused and counted (I6)"; a configuration pin is quarantined like any
  event (S1).
- #5, #13: the capture-off and capture-on acts show the node-wide capture
  gap on Runs and Health, covering runs that began and ended within it.
- #6: say that serving the MCP server's fixed tool definitions and
  instructions is not a write to an agent: they are fixed text ParleyInn ships
  and carry nothing from the record. Do not change I2's or I4's wording.
- #7: "any other seat is kept as another node's or principal's, attributed
  to its seat key and untrusted (I8)".
- #8, #15: Room: "the unit ParleyInn shows" in core; "and shares", and Writer's
  "held by any node", go into the spans of the features that share
  (work-rooms, peer-sync, segment-exchange, export-bundles as fits; nest).
- #12: any CLI or TUI call from a process the harness started, other than
  ParleyInn's own hook handlers, MCP server and harness strip, is recall:
  enveloped, whatever its output.
- #16: repository configuration applies only to runs whose working
  directory lies in that repository, and only to settings keys scoped to
  those runs. Keep it in git-tracking's span if "repository" is only
  defined there, otherwise core.
- #17: Setup is a sibling of Runs, Pins and Health.
- #18: drop Person in core if Principal is defined as a person and nothing
  else needs it; keep it inside the service-accounts span if that feature
  needs the person/service-account split.

Every invariant text change goes in GA/invariant-edits.md (none expected).

When done: `python3 GA/check_annotation.py --no-orig` prints OK; assemble
v4-core and v4-all into GA/out/v5-core and GA/out/v5-all (write
GA/genomes/v5-core.json and v5-all.json with the same features and the new
names) and read the core text end to end. Do not run git. Report per ruling
what you changed (before and after) and what you skipped and why.
