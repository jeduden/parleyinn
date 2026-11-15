#!/bin/bash
# Says whether the reviewer evaluation's plan is empty (ENG-32), and,
# when the review reused an earlier review's verdict on this head
# (ENG-28), keeps that review's evaluation in place of running the
# cases again: `empty` is true for an empty plan, false for cases to
# run, and kept when the earlier run's evaluation, scrubbed there, was
# fetched into review-eval/. An evaluation that cannot be fetched is
# said and its cases run (I6).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

if [ "$(printf '%s' "$EVAL_PLAN" | jq 'length')" = 0 ]; then
  echo "empty=true" >> "$GITHUB_OUTPUT"
  exit 0
fi
if [ -n "$VERDICT_RUN" ]; then
  if artifact=$(gh api "repos/$REPO/actions/runs/$VERDICT_RUN/artifacts?name=review-evaluation" \
      --jq '.artifacts[0].id // empty') && [ -n "$artifact" ] \
    && gh api "repos/$REPO/actions/artifacts/$artifact/zip" > "$RUNNER_TEMP/evaluation.zip" \
    && mkdir -p review-eval && unzip -q -o "$RUNNER_TEMP/evaluation.zip" -d review-eval \
    && jq -e 'type == "array"' review-eval/findings.json > /dev/null; then
    echo "::notice::The reviewer evaluation of run $VERDICT_RUN on this head is kept; no case runs."
    echo "empty=kept" >> "$GITHUB_OUTPUT"
    exit 0
  fi
  echo "::warning::The reviewer evaluation of run $VERDICT_RUN could not be fetched, so its cases run."
  rm -rf review-eval
fi
echo "empty=false" >> "$GITHUB_OUTPUT"
