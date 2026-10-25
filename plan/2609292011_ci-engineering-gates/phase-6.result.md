---
n: 6
title: "Main's protection and CODEOWNERS (ENG-21)"
status: "✅"
result: true
summary: >-
  Main's ruleset is committed and checked against the live one nightly, CODEOWNERS is checked against the paths that enforce the SRS and gives the security-sensitive crates to the security reviewer, and the ENG-21 scenario is off pending.
---
## Handoff

What landed:

- `.github/rulesets/main.json`: main's ruleset in GitHub's export
  format. No direct push, no bypass actor, no deletion or history
  rewrite, signed commits, `CI` required, one approval that is not the
  last pusher's, stale approvals dismissed, a code owner's approval
  where CODEOWNERS names one.
- The `protection-check` executable and a `protection` job in the
  nightly workflow: it fetches the live ruleset with the workflow's token
  and fails when the live one is weaker. Reading a ruleset works with
  that token. Its bypass actors do not show, since GitHub gives them
  only to a caller with write access to the ruleset.
- CODEOWNERS checks derived from the SRS: the stakeholder and the
  security reviewer from the index, the security-sensitive crates from
  ENG-11, the enforcing classes from the ENG-21 row. The current
  CODEOWNERS conforms; nothing in it was changed.
- Ten drift cases, six integration gate tests for the ruleset and
  CODEOWNERS, and an end-to-end test of the executable.
- After the merge, the two live tag rulesets were committed beside
  main's: `releases-semver` keeps each `v*.*.*` tag fixed on a signed
  commit that passed `CI`, and `no-other-tags-allowed` refuses every
  other tag. The nightly job compares every file under
  `.github/rulesets` with the live ruleset of its name, and
  `protection-check` also refuses a live ruleset on another target or
  with a `do_not_` flag switched on. The live `no-other-tags-allowed`
  included no ref, so it applied to nothing; the committed one includes
  `~ALL` but the release tags, and a gate test refuses a ruleset with
  no include. The stakeholder applies it on GitHub again.

Scenario: `@ENG-21` is off `@pending`, on the stakeholder's ruling of
2026-10-23. The row asked two approvals on the security-sensitive
crates, one from the security reviewer. A ruleset holds one approval
count for the whole branch and CODEOWNERS one owner per path, so it
took a required check that counted reviews, and only one reviewer
account exists besides the stakeholder, who never counts as the author's
approver: no pull request on those crates could have reached two. A
`Security approvals` check was built, reviewed and closed unmerged
(pull request 72). The row now reads that those crates are owned by the
designated security reviewer, whose approval a change to them needs as
a code owner, and the last step checks that ownership. The ruleset and
CODEOWNERS already held it. A second approval returns as a change to the
row when a second reviewer account is named; nothing here waits on it.

For the stakeholder:

- Apply `.github/rulesets/main.json` on GitHub; until then the nightly
  `protection` job is red. Applied on 2026-10-14; the job passed.
- Decide how a pull request the stakeholder authors on an owned path
  merges: GitHub never counts an author's own approval, and the ruleset
  has no bypass actor.
- AI-generated labelling is in the ENG-21 row but in no scenario step
  and no mechanism; it is still open.
