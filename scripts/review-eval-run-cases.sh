#!/bin/bash
# Runs the cases of each reviewer component the pull request changes,
# failing on the harness's own failure or a leaked token (ENG-32).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

mkdir -p review-eval
printf '%s' "$EVAL_PLAN" > eval-plan.json
set +e
cargo run --locked --quiet -p review-eval -- run --checkout . --tree pr-head \
  --cases pr-head/tooling/review-eval/cases --base-cases tooling/review-eval/cases \
  --plan eval-plan.json --out review-eval --parallel 12 \
  --credential "$RUNNER_TEMP/claude-credential"
code=$?
set -e
if [ "$code" -eq 3 ]; then
  echo "::error::an agent's output in the reviewer evaluation held the Claude Code OAuth token, now redacted; revoke and rotate it"
  exit 3
fi
if [ "$code" -gt 1 ]; then
  echo "::error::the reviewer evaluation did not run to its end (exit $code)"
  exit "$code"
fi
