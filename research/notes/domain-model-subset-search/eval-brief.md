# Evaluation brief: review one candidate domain model

You review one CANDIDATE version of ParleyInn's domain model, assembled from a
subset of its features. ParleyInn is a lossless, security-first context layer
for long-running Claude agents: an append-only record of every agent run,
pins restored verbatim after compaction, exact recall on demand, and stored
history never a prompt-injection channel.

Read the candidate fully: CANDIDATE_DIR/candidate.md (it holds the candidate's
invariants, its hub and its concept files, each marked by a FILE comment).
Review ONLY this text. Do not read the repository's docs/domain-model/ or
docs/srs/ (they hold the full model; the candidate deliberately leaves things
out). Requirement ids (REC-01, OWN-11, …) and section numbers cited in the
candidate are references you may not resolve; never report them.

The candidate is a domain model: it defines concepts, their relations and
the rules that make them coherent; mechanisms (how something is
authenticated, stored, enforced or computed) live in the requirements it
cites. Where a sentence cites a requirement id for a rule, assume the rule
holds as stated. Report a missing mechanism only when the candidate states no
rule and cites no requirement for something its own invariants depend on.

Settled by the stakeholder, never a finding: the principal's own tunnel or
`git push` carries data off the machine as the principal's tool, outside
ParleyInn's components and outside I4; room trailers are always on.

A concept that is absent from the candidate is not a finding. Judge the
candidate as a complete model of a product that simply lacks what it does
not mention.

Your lens: LENS.

Report only real defects, each one of:

- contradiction: two statements in the candidate cannot both hold;
- undefined: a term the candidate uses as a ParleyInn concept but never defines,
  or a definition that relies on something the candidate lacks;
- gap: a rule needed for two implementers to build the same thing is missing
  (say exactly what), or a concept that cannot work as defined;
- security: a path from untrusted content to the model outside the closed
  paths, a trust or key rule that can be bypassed, data leaving outside the
  declared boundaries, a silent failure, or an invariant the candidate's own
  concepts break;
- useless: an included concept that has no use in this candidate (nothing can
  create, reach or act on it), or the candidate fails a core journey: an
  agent gets its constraints back verbatim after compaction; it recalls exact
  history; a person sees what agents did; untrusted history never instructs
  an agent.

Do NOT report wording style, missing examples, nice-to-haves, or anything that
would only matter with a concept the candidate lacks. Ask: would this make two
careful implementers build different things, or open a hole? If not, skip it.

Output: one fenced ```json block, an array of
{"type": one of the five above, "quote": EXACT text copied from candidate.md
(10–200 chars, from the line at fault, no line breaks inside), "problem": one
or two sentences, "fix": smallest wording change that fixes it, "severity":
"needs-fix" or "minor"}. Then one line: "FINDINGS: <n needs-fix>". An empty
array is a valid and welcome answer when the candidate holds.
