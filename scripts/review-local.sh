#!/usr/bin/env bash
# Runs the review of HEAD against origin/main here, before a push, so a
# finding costs a minute of this loop and not a CI round (ENG-28).
#
# It prepares the work directory the review workflow prepares: main's
# tree, the head's tree under pr-head/, the diff, the stakeholder's
# comments, the head's agent instructions moved out of the agents' load
# path. It then runs the same tools: review-bundle, review-eval plan and
# review-run. The tools are this checkout's, so a change to them is what
# runs; CI runs main's. The agents are main's definitions, as in CI.
#
# Prints the verdict, each finding on one line, what the run cost, how
# long it took and the evaluation plan with its estimate. Exits 0 when the
# review approves, 1 when it requests changes, 2 on a wrong command line
# or a setup that fails. Run files stay under target/review-local/.
#
# By default the base is origin/main, as in CI. --base REF reviews only
# what changed since REF, for the fix cycle after a review: the commit
# you last reviewed or pushed. Commit first: only committed work is reviewed. A change to a path
# CODEOWNERS assigns is a blocking finding until a stakeholder comment
# consents, so pass --consent FILE (the JSON the workflow collects) to see
# the review as it stands with that comment.
#
# usage: scripts/review-local.sh [--base REF] [--consent FILE] [--parallel N]
set -euo pipefail

base=origin/main
consent=""
parallel=16
while [ "$#" -gt 0 ]; do
  case "$1" in
    --base) base="${2:?--base needs a ref}"; shift 2 ;;
    --consent) consent="${2:?--consent needs a file}"; shift 2 ;;
    --parallel) parallel="${2:?--parallel needs a number}"; shift 2 ;;
    *) echo "usage: scripts/review-local.sh [--base REF] [--consent FILE] [--parallel N]" >&2; exit 2 ;;
  esac
done

repo=$(git rev-parse --show-toplevel)
if [ -n "$(git -C "$repo" status --porcelain --untracked-files=no)" ]; then
  echo "review-local: uncommitted changes are not reviewed; commit them first" >&2
fi
git -C "$repo" fetch -q origin main

start=$SECONDS
work=$(mktemp -d)
cleanup() {
  git -C "$repo" worktree remove --force "$work/pr-head" 2>/dev/null || true
  git -C "$repo" worktree remove --force "$work" 2>/dev/null || true
  git -C "$repo" worktree prune
  rm -rf "${work:?}"
}
trap cleanup EXIT

run() {
  if command -v mise > /dev/null; then mise exec -- "$@"; else "$@"; fi
}
tool() {
  run cargo run --locked --quiet --manifest-path "$repo/Cargo.toml" -p "$@"
}

# Main's tree and the head's, as the workflow's two checkouts.
rmdir "$work"
git -C "$repo" worktree add -q --detach "$work" "$base"
git -C "$repo" worktree add -q --detach "$work/pr-head" HEAD
cd "$work"
git -C pr-head diff "$base...HEAD" > pr.diff
if [ -n "$consent" ]; then cp "$consent" stakeholder-comments.json; else echo '[]' > stakeholder-comments.json; fi

mkdir -p review/reports
run mdsmith help patterns -f json > review/mdsmith-patterns.json
tool review-bundle -- --base . --head pr-head --diff pr.diff \
  --consent stakeholder-comments.json --out review > class.txt
tool review-eval -- plan --base . --head pr-head --out review/eval-plan.json

# The head's agent instructions are data to review, never instructions to
# load; they leave the tree as in the workflow.
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

echo "review-local: $(tr '\n' ' ' < class.txt)"
status=0
tool review-run -- --workdir . --parallel "$parallel" || status=$?
mkdir -p "$repo/target/review-local"
cp -R review "$repo/target/review-local/"

if [ "$status" -ne 0 ]; then
  echo "review-local: review-run could not decide the review (exit $status); see target/review-local/review/logs" >&2
  exit 2
fi
jq -r '"verdict: " + .verdict, (.findings[] | "\(.path):\(.line) — \(.severity) — \(.body)")' review/outcome.json
jq -r '"evaluation plan: \(length) components"' review/eval-plan.json
jq -r '"cost: $" + (([.[].cost_usd] | add // 0) * 100 | round / 100 | tostring)' review/usage.json
echo "elapsed: $((SECONDS - start)) s"
[ "$(jq -r .verdict review/outcome.json)" = approve ]
