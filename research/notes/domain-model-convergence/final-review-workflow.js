export const meta = {
  name: 'dm-final-review',
  description: 'The final review of the M1 to M5 domain model and SRS: four area reviewers, each judged by an adversarial verifier',
  phases: [
    { title: 'Review', detail: 'one reviewer per area, following review-brief.md' },
    { title: 'Verify', detail: 'one adversarial verifier per area; its default is not blocking' },
  ],
}
// args: { repo }
//   repo: absolute path of the checkout to review (the committed model and SRS).
const { repo } = args
const BRIEF = `${repo}/research/notes/domain-model-convergence/review-brief.md`
const SCOPE = `${repo}/research/notes/domain-model-convergence/m1-m5-scope.md`
const AREAS = [
  {
    key: 'record',
    area: "the record and the node's lifecycle",
    model: ['record.md', 'git-and-forge.md', 'harness-facts.md'],
    srs: ['05-functional-requirements.md (REC, LMK, RCL)', '05a-administration-requirements.md', '05e-provenance-requirements.md', '08-data-and-storage.md'],
    carry: "The grid's agents named events beyond its 27 rows; check whether the SRS decides what each does to the derived artifacts: a run's start, an MCP server that stops for good, a repository identity bound, recall that returns untrusted content, a hook handler failing open, a provisional branch identity rebound on push, the lack of an unlink act, a locked key store.",
  },
  {
    key: 'trust',
    area: 'trust and the closed paths',
    model: ['trust-and-flow.md', 'harness-facts.md', 'components-and-surfaces.md'],
    srs: ['05-functional-requirements.md (INJ, RCL)', '06-security.md', '06b-boundary-register.md', '05e-provenance-requirements.md', '09-interfaces.md', '09a-command-line-interface.md'],
    carry: 'The model keeps one rule no requirement states: a natural-language landmark headline is no structural field and never reaches a restore block. Check whether it holds against INJ and LMK.',
  },
  {
    key: 'pins',
    area: 'pins and restore',
    model: ['pins-and-context.md', 'places.md'],
    srs: ['05-functional-requirements.md (PIN, INJ)', '05b-lane-requirements.md', '07-non-functional-requirements.md (budgets)'],
    carry: 'The model keeps rules no requirement states; check whether each holds against the SRS: pin priority runs lower first; a personal-room seat can neither leave nor be kicked.',
  },
  {
    key: 'config',
    area: 'configuration, seats and keys',
    model: ['seats-and-keys.md', 'principals-and-agents.md', 'acts-and-roles.md'],
    srs: ['05a-administration-requirements.md', '06-security.md (SEC-10, SEC-11, SEC-22, SEC-27)', '05c-principal-and-peer-requirements.md (OWN)', '05e-provenance-requirements.md', '09-interfaces.md (§9.6 settings keys)'],
    carry: "Two points the chain passes left for this review: OWN-10 says every rule level change the principal makes is a principal act, while ADM-04 says a change that only tightens needs no acceptance; and SEC-27's pending scenario has no step for a seat key missing from where SEC-10 keeps it. The model also keeps rules no requirement states: every change of visibility, admission or a room setting is widening; a role assignment makes ParleyInn refuse no seat's act.",
  },
]
const FINDINGS = {
  type: 'object',
  properties: { findings: { type: 'array', items: { type: 'object', properties: {
    type: { type: 'string', enum: ['invariant-break', 'contradiction', 'gap'] },
    quotes: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, text: { type: 'string' } }, required: ['file', 'text'] } },
    case: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' }, scope: { type: 'string' },
  }, required: ['type', 'quotes', 'case', 'problem', 'fix', 'scope'] } } },
  required: ['findings'],
}
const VERDICTS = {
  type: 'object',
  properties: { themes: { type: 'array', items: { type: 'object', properties: {
    id: { type: 'string', description: '<area>-<n>' },
    title: { type: 'string' },
    findings: { type: 'array', items: { type: 'integer' }, description: 'indices of the findings this theme groups' },
    verdict: { type: 'string', enum: ['blocking', 'known-gap', 'later', 'not-real'] },
    reason: { type: 'string', description: 'why, after trying to refute it against the current text' },
    quotes: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, text: { type: 'string' } }, required: ['file', 'text'] }, description: 'for blocking: the broken invariant clause and the requirement text, or the two statements; exact, one line each' },
    milestone: { type: 'string', description: 'for later: the milestone and feature it waits for' },
    gap: { type: 'string', description: 'for known-gap: one sentence naming what M1 must settle, citing the requirement ids' },
    edits: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, old: { type: 'string' }, new: { type: 'string' } }, required: ['file', 'old', 'new'] }, description: 'for blocking: exact edits that fix it at its rule' },
    scenario: { type: 'object', properties: { file: { type: 'string' }, requirement: { type: 'string' }, step: { type: 'string' } }, required: ['file', 'requirement', 'step'], description: 'for blocking: the step its requirement\'s pending scenario gains' },
  }, required: ['id', 'title', 'findings', 'verdict', 'reason', 'quotes'] } } },
  required: ['themes'],
}
const RULES = `Fix rules, for a blocking theme only. State the rule once, in the SRS requirement row that governs it, never as one more case clause; replace a case clause that restates it with a citation. The model never grows: a model edit is a concept sentence or a citation and leaves the file no longer. docs/srs/invariants.md never changes; a theme whose fix would need it is blocking and says so in its reason. Each SRS file stays at or under 8000 tokens at 1.33 tokens a word: 05b-lane-requirements.md has no room left, so an edit there must not add words. Every edit's old must be an exact, unique substring of the current file. The scenario step goes into the @pending scenario tagged with the requirement whose row the fix edits (grep features/ for the tag); a requirement whose scenario is not @pending cannot take one, so choose the row whose scenario is.`

const results = await pipeline(
  AREAS,
  a => agent(
    `Read ${BRIEF} and follow it. The repository is ${repo}. Edit no file. Your area: ${a.area}. Its model files, under docs/domain-model/: ${a.model.join(', ')}; read the hub index.md too. Its SRS, under docs/srs/: ${a.srs.join('; ')}, with §8.5 of 08-data-and-storage.md and §12.2 of 12-delivery-plan.md. ${a.carry} Return the findings through the structured output.`,
    { label: `review:${a.key}`, phase: 'Review', schema: FINDINGS },
  ),
  (review, a) => {
    const findings = review ? review.findings : []
    if (!findings.length) return { area: a.key, findings, themes: [] }
    return agent(
      `You are the adversarial verifier of ParleyInn's final review of the domain model and SRS at milestones M1 to M5. The repository is ${repo}; edit no file. Sources are docs/srs/invariants.md, docs/domain-model/ and docs/srs/ only; ADRs and plan notes are history. Scope is M1 to M5 as §12.2 of docs/srs/12-delivery-plan.md and ${SCOPE} set it.\n\nBelow are the findings of the reviewer of the area "${a.area}". Group findings that are one defect into one theme. For each theme, try to refute it first: read every quoted line and the requirements it cites, search the SRS and the model for a rule that settles it, and check that the behaviour ships in M1 to M5. The reviewer's type labels count for nothing. Your default verdict is not blocking.\n\n- blocking: only when you can quote, exactly and from the current files, either an invariant clause and the requirement text that, built as written, breaks it in M1 to M5 behaviour, with the concrete case; or two normative statements that cannot both hold.\n- known-gap: real and in M1 to M5, but neither: give one sentence naming what M1's implementation must settle, citing the requirement ids.\n- later: it arises only from M6 to M9 features; name the milestone and feature.\n- not-real: refuted; say by which text.\n\nFor each blocking theme, give the exact edits that fix it and the scenario step. ${RULES}\n\nFindings:\n${JSON.stringify(findings.map((f, i) => ({ index: i, ...f })))}`,
      { label: `verify:${a.key}`, phase: 'Verify', schema: VERDICTS },
    ).then(v => ({ area: a.key, findings, themes: v ? v.themes : null }))
  },
)
return results
