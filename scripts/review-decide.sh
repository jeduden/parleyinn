#!/bin/bash
# Decides the review with the review gate; a gate that fails to decide
# posts the review-gate check failed (ENG-21, ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

printf '%s' "$OUTCOME" > outcome.json
printf '%s' "$CHECKED" > bundle-checked.json
jq -s 'add' bundle-checked.json evaluation/findings.json > checked.json
printf '%s' "$DISPATCH" > dispatch.json
printf '%s' "$CONSULTED" > consulted.json
if ! cargo run --locked --quiet -p review-gate --offline -- --outcome outcome.json \
  --check-runs check-runs.jsonl --reviewed "$HEAD_SHA" --head "$(cat head.txt)" \
  --checked checked.json --dispatch dispatch.json \
  --consulted consulted.json --reports reports --answers answers.json > review.json; then
  gh api --method POST "repos/$REPO/check-runs" -f name=review-gate \
    -f head_sha="$HEAD_SHA" -f status=completed -f conclusion=failure \
    -f "output[title]=The review gate did not decide" \
    -f "output[summary]=The review gate failed before deciding; the log of $RUN says why."
  exit 1
fi
