#!/bin/bash
# Decides the review with the review gate; a gate that fails to decide
# posts the review-gate check failed, its summary holding the gate's
# own error: which agent did not run and why, and for a review the
# pull request's cost cap stopped, how the spend is reset (ENG-21,
# ENG-28).
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
  --consulted consulted.json --reports reports --answers answers.json > review.json \
  2> gate-error.txt; then
  cat gate-error.txt >&2
  # The gate's error is its own line, the one that starts with its
  # name: a usage error is followed by the usage. A gate that never
  # started printed none, and the last line cargo printed says why. It
  # goes in a code span, its own backticks dropped, so nothing in it is
  # read as Markdown. A check's summary holds 65535 characters: the cut
  # leaves room for the text around it, and a list of every agent fits
  # whole with the reset that ends it.
  said=$(grep -m 1 '^review-gate: ' gate-error.txt || tail -n 1 gate-error.txt)
  why="\`$(printf '%s' "$said" | tr -d '\`' | head -c 65000)\`"
  gh api --method POST "repos/$REPO/check-runs" -f name=review-gate \
    -f head_sha="$HEAD_SHA" -f status=completed -f conclusion=failure \
    -f "output[title]=The review gate did not decide" \
    -f "output[summary]=The review gate failed before deciding: $why. The log of $RUN has the rest."
  exit 1
fi
cat gate-error.txt >&2
