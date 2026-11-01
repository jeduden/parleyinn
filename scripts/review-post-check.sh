#!/bin/bash
# Posts the gate's decision as the review-gate check on the reviewed
# head (ENG-21).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

if [ -s review.json ]; then
  conclusion=failure
  title="Changes requested"
  if [ "$(jq -r .event review.json)" = APPROVE ]; then
    conclusion=success
    title="Approved"
  fi
  gh api --method POST "repos/$REPO/check-runs" -f name=review-gate \
    -f head_sha="$HEAD_SHA" -f status=completed -f conclusion="$conclusion" \
    -f "output[title]=$title" \
    -f "output[summary]=The review gate's review on this head says why. The specialists' reports: $REPORTS (artifact review-reports)."
fi
