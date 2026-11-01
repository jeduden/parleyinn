#!/bin/bash
# Moves every agent instruction file of the pull request's tree into
# review/pr-instructions.txt, as data the agent reads (ENG-28).
#
# Run by .github/workflows/review.yml, which hands it its values in
# the environment; it takes the shell's default, no pipefail.
set -e

find pr-head -path pr-head/.git -prune -o \( -name .claude -o -name CLAUDE.md \
  -o -name CLAUDE.local.md -o -name AGENTS.md -o -name .mcp.json \) -prune -print0 \
  > instructions.list
while IFS= read -r -d '' f; do
  find "$f" -type f -print0 | while IFS= read -r -d '' g; do
    printf '\n===== %s =====\n' "${g#pr-head/}"
    cat "$g"
  done
  rm -rf "$f"
done < instructions.list > review/pr-instructions.txt
