---
n: 3
title: "A stakeholder's comment changes nothing an agent reads"
status: "✅"
result: false
---
# Phase 3: a stakeholder's comment changes nothing an agent reads

Requirements. ENG-28: its row says a review a stakeholder's comment
brings takes the agents' outcome of the same head as its own. No
scenario comes off `@pending`: `@ENG-28` runs already, and its step on
a comment's review gains the new clause with the row.

What the record shows. The reuse is sound only when the comment
changes nothing an agent read. It did: the bundle and each review
context stated the findings that turn on consent, blocking until the
comment and a nit after it. Of 5,852 recorded reports, 292 speak of
consent or CODEOWNERS, and about 30 finding lines carry the missing
consent in their consequence. Such a line outlives the consent, since
the comment's review runs no agent. On pull request 146 one did. Three
written cases carried a consenting comment only so the reviewer would
not repeat the bundle's finding.

The option as it was put, to clear such a line after consent, treats
the line. This phase removes its cause: no agent is shown the consent
state.

The rule. The review bundle writes the bundle and each review context
the same whatever the stakeholder's comments say. The findings that
turn on consent, ENG-21's on a stakeholder path, ENG-29's on a guarded
change and ENG-34's on a lengthened size list, go to the review gate
alone, in either state. The ENG-29 record is named whether or not a
comment consents. Each agent reads one fixed sentence: consent is the
review gate's check, never a finding of its own.

RED. Unit tests of the bundle: the same bundle and contexts for a
change that waits on every consent, with no comment and with one that
consents. The ENG-28 record check of a context, which read the ENG-21
finding as the one on its file, fails once that finding is gone.

GREEN. The analysis sets the findings that turn on consent apart, one
finding a cause, so ENG-29's record and consent are two findings. The
bundle and the contexts leave them out and carry the sentence.

Evaluation. Two clean cases change files on stakeholder paths. One is
for `reviewer-engineer` and one for `lang-markdown`. Each drops its
consenting comment, so no comment consents and the reviewer must still
state no finding. They run on the reviewers' models with and without
the change. The defect and injection cases of both components run
beside them.

Gate. The record check in `review_consent` runs the checkout's bundle
twice on one change and holds every file an agent reads to the same
bytes. Three drift cases fail it: a consent finding shown, the record
named only after consent, and a context without the sentence.
