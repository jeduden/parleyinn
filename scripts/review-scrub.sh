#!/bin/bash
# Scrubs whatever review-run wrote of the Claude token once more, a run
# cut off before its own scrub included, before any step reads or
# uploads it; a form found fails the job. Then the credential goes
# (ENG-28).
#
# Run by .github/workflows/review.yml; it takes the shell's default, no
# pipefail. Writes `scrubbed` to the step's outputs when the files are
# clean of the token, redacted or never holding it.
set +e
cargo run --locked --quiet --offline -p review-eval -- scrub \
  --credential "$RUNNER_TEMP/claude-credential" \
  --path review/logs --path review/reports --path review/answers.json --path review/usage.json \
  --path review/outcome.json --path review/consulted.json --path review/gaps.json \
  --path review/not-reviewed.md --path review/ignored.json --path review/run-id.json \
  --path review/findings.json --path review/findings.md --path review/spend.json \
  --path review/run.txt --path review/agent-runs.jsonl --path review/report.html \
  --path review/dispatch.json
code=$?
set -e
rm -rf "$RUNNER_TEMP/claude-credential"
if [ "$code" -eq 0 ] || [ "$code" -eq 3 ]; then
  echo "scrubbed=true" >> "$GITHUB_OUTPUT"
fi
if [ "$code" -eq 3 ]; then
  echo "::error::what the review wrote held the Claude Code OAuth token, now redacted; revoke and rotate it, and nothing is posted"
fi
exit "$code"
