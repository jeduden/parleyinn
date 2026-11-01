#!/bin/bash
# Reads the pull request's head and the reviewed head's check runs for
# the gate (ENG-21).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

gh api "repos/$REPO/pulls/$PULL" --jq .head.sha > head.txt
gh api --paginate "repos/$REPO/commits/$HEAD_SHA/check-runs?per_page=100" \
  --jq '.check_runs[] | {id, name, status, conclusion, started_at, app: .app.id}' > check-runs.jsonl
