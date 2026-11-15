#!/bin/bash
# Plans the reviewer evaluation: the reviewer components the pull
# request changes, each with its reason (ENG-32), and with the case
# files it changes sorted by the stakeholder's comments: one a comment
# names judges as the pull request has it, any other as main has it
# (ENG-21).
#
# Run by .github/workflows/review.yml, after the step that collects
# the stakeholder's comments.
set -e

cargo run --locked --quiet -p review-eval -- plan --base . --head pr-head \
  --consent stakeholder-comments.json --out review/eval-plan.json
echo "eval_plan=$(jq -c . review/eval-plan.json)" >> "$GITHUB_OUTPUT"
