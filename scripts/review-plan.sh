#!/bin/bash
# Plans the reviewer evaluation: the reviewer components the pull
# request changes, each with its reason (ENG-32).
#
# Run by .github/workflows/review.yml.
set -e

cargo run --locked --quiet -p review-eval -- plan --base . --head pr-head \
  --out review/eval-plan.json
echo "eval_plan=$(jq -c . review/eval-plan.json)" >> "$GITHUB_OUTPUT"
