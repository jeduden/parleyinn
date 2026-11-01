#!/bin/bash
# Says whether the reviewer evaluation's plan is empty (ENG-32).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

if [ "$(printf '%s' "$EVAL_PLAN" | jq 'length')" = 0 ]; then
  echo "empty=true" >> "$GITHUB_OUTPUT"
else
  echo "empty=false" >> "$GITHUB_OUTPUT"
fi
