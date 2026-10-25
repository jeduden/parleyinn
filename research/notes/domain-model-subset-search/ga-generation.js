export const meta = {
  name: 'ga-generation',
  description: 'One memetic GA generation over the domain model: breed, review, record, repair, verify, commit',
  phases: [
    { title: 'Breed', detail: 'Thompson-sampled GA picks k genomes; assemble them' },
    { title: 'Review', detail: 'one blind probe review per candidate' },
    { title: 'Record', detail: 'attribute findings to features; update the ledger' },
    { title: 'Repair', detail: 'fix what tightening can fix, on a copy' },
    { title: 'Verify', detail: 'revert any repair that weakens an invariant or adds a concept' },
    { title: 'Commit', detail: 'snapshot the new version, copy back, commit, push' },
  ],
}
// args: { ga, work, repo, branch, ver, gen, k, trailer }
//   ga: absolute path of this folder; work: a scratch path for this run's copy
//   ver: the text version the candidates are assembled from (annotated-v<ver>
//   equals annotated/); the repair writes version ver+1.
const { ga, work, repo, branch, ver, gen, k, trailer } = args
const next = ver + 1
const tag = `v${ver}-ga${gen}`
const env = `PARINN_REPO=${repo}`

const NAMES = { type: 'object', properties: { cands: { type: 'array', items: { type: 'string' } }, surrogate: { type: 'string' } }, required: ['cands', 'surrogate'] }
const FINDINGS = {
  type: 'object',
  properties: {
    findings: { type: 'array', items: { type: 'object', properties: {
      type: { type: 'string', enum: ['contradiction', 'undefined', 'gap', 'security', 'useless'] },
      quote: { type: 'string' }, problem: { type: 'string' }, fix: { type: 'string' },
      severity: { type: 'string', enum: ['needs-fix', 'minor'] },
    }, required: ['type', 'quote', 'problem', 'fix', 'severity'] } },
  },
  required: ['findings'],
}
const TEXT = { type: 'object', properties: { summary: { type: 'string' } }, required: ['summary'] }
const LENS = 'all three: consistency (contradictions, undefined terms, gaps), security (does the candidate uphold its own invariants) and usefulness (every included concept has a use; the core journeys work)'

phase('Breed')
const bred = await agent(
  `Run these shell commands exactly and report. Do not edit any file by hand.
rm -rf ${work} && cp -r ${ga} ${work} && cd ${work}
python3 ga_round.py ${tag} ${k} --seed ${gen}
then for each genomes/${tag}-*.json written: python3 assemble.py genomes/<name>.json out/<name>
Return the candidate names (e.g. ${tag}-1) and the ga_round.py output as surrogate.`,
  { label: 'breed', phase: 'Breed', schema: NAMES, model: 'haiku' },
)
log(`bred ${bred.cands.join(', ')}`)

phase('Review')
const reviews = await parallel(bred.cands.map(c => () => agent(
  `Read ${ga}/eval-brief.md and follow it. CANDIDATE_DIR = ${work}/out/${c}. LENS = ${LENS}. Edit no file but one: as your last step, write your findings as JSON {"findings": [...]} (the same objects you return) to ${work}/evals/${c}.probe.json. Return the findings through the structured output too (quote copied exactly from candidate.md, one line, 10-200 chars).`,
  { label: `review:${c}`, phase: 'Review', schema: FINDINGS, model: 'sonnet' },
).then(r => ({ cand: c, lens: 'all', model: 'sonnet', findings: r ? r.findings : null }))))

phase('Record')
const rec = await agent(
  `Run: cd ${work} && python3 collect_probes.py ${tag} && python3 ga.py rank | head -15. Return the commands' output as summary. Expected candidates: ${bred.cands.join(', ')}; if one has no ${work}/evals/<cand>.probe.json, say so.`,
  { label: 'record', phase: 'Record', schema: TEXT, model: 'haiku' },
)
log(rec.summary)

phase('Repair')
const files = bred.cands.map(c => `${work}/out/${c}/attributed.json`).join(', ')
const repair = await agent(
  `WORK = ${work} (a copy of the search folder; edit only there). Read WORK/repair-brief.md and WORK/annotate-brief.md. This generation's findings, each attributed to the feature span its quote sits in ("feature", "file"): ${files}. The annotated sources are WORK/annotated/*.md.ann (version ${ver}).
Decide the repairs yourself, no rulings are given: fix every needs-fix finding whose fix tightens, narrows, deletes or clarifies, inside its feature's span (core text for "core"; nest spans when a fix needs another feature). Fix plainly right minors the same way. Prefer one sentence that settles a family of findings over a clause per finding; prefer deleting to adding; add no concept. Skip a finding that is wrong, that needs a stakeholder decision, that would weaken an invariant, or that asks for a new concept or mechanism. Settled by the stakeholder, keep as is: the principal's own tunnel and git push are its tools outside I4; room trailers are always on; a run's tool process running as the principal's own OS user is §6.1's residual risk R1-R7. Leave invariants.md.ann's text alone.
Write WORK/repair-v${next}-log.md: a heading, then per finding its candidate and index, fixed (before -> after) or skipped (why). Then run: cd ${work} && ${env} python3 check_annotation.py --no-orig (must print OK), and assemble genomes/_test-min.json (core only) into out/v${next}-core to read the core. Do not run git or mdsmith. Return a short summary: fixed and skipped counts, and anything a stakeholder must decide.`,
  { label: 'repair', phase: 'Repair', schema: TEXT },
)
log(repair.summary)

phase('Verify')
const verify = await agent(
  `Adversarially check a repair. Compare ${work}/annotated/*.md.ann (the repair) with ${ga}/annotated/*.md.ann (before it), e.g. with diff. For every changed hunk ask: does it weaken an invariant's protection, contradict invariants.md.ann, add a concept the model did not have, or break span balance or grammar when a feature is removed? Revert each such hunk in ${work}/annotated (only there) and append a line per revert to ${work}/repair-v${next}-log.md. Then run cd ${work} && ${env} python3 check_annotation.py --no-orig (must print OK). Do not run git. Return a summary: hunks checked, hunks reverted and why.`,
  { label: 'verify', phase: 'Verify', schema: TEXT },
)
log(verify.summary)

phase('Commit')
const commit = await agent(
  `Run, stopping at the first failure and reporting it:
cd ${work} && rm -rf annotated-v${next} && cp -r annotated annotated-v${next}
cp -r ${work}/annotated/. ${ga}/annotated/ && cp -r ${work}/annotated-v${next} ${ga}/ && cp ${work}/genomes/${tag}-*.json ${ga}/genomes/ && cp ${work}/evals/${tag}* ${ga}/evals/ && cp ${work}/collect_probes.py ${ga}/ && cp ${work}/ledger.json ${work}/repair-v${next}-log.md ${ga}/
cd ${ga} && ${env} python3 check_annotation.py --no-orig
cd ${repo} && mdsmith fix ${ga}/repair-v${next}-log.md; mdsmith check . | tail -1   (mdsmith is at /root/go/bin/mdsmith if not on PATH; it must end with failures=0)
git add ${ga} && git commit -F <message file> with this message:
research: GA generation ${gen} on v${ver}, repair to v${next}

${rec.summary.split('\n').slice(0, 8).join('\n')}

${trailer}
then git push -u origin ${branch}, retrying up to 4 times with 2, 4, 8, 16 s waits on network errors.
Return a summary with the commit sha.`,
  { label: 'commit', phase: 'Commit', schema: TEXT, model: 'haiku' },
)
return { tag, cands: bred.cands, surrogate: bred.surrogate, record: rec.summary, repair: repair.summary, verify: verify.summary, commit: commit.summary,
  needsFix: reviews.map(r => ({ cand: r.cand, nf: r.findings ? r.findings.filter(f => f.severity === 'needs-fix').length : null })) }
