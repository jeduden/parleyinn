#!/usr/bin/env bash
# Runs the review of HEAD against origin/main here, before a push, so a
# finding costs a minute of this loop and not a CI round (ENG-28).
#
# It prepares the work directory the review workflow prepares: main's
# tree, the head's tree under pr-head/, the diff, the stakeholder's
# comments, the head's agent instructions moved out of the agents' load
# path. It then runs the same tools: review-bundle, review-eval plan and
# review-run. The tools are this checkout's, so a change to them is what
# runs; CI runs main's. The tools expect the finding grammar the head's
# agent definitions define, so the agents are the head's definitions by
# default (--agents head), what main has after the merge: the head's
# .claude/agents/*.md replace the base's. --agents base keeps the base's
# definitions (main's, or REF's under --base REF), as CI runs them today.
# The review reads no skill: review-run dispatches agents only.
#
# Prints the verdict, each finding on one line, the bundle's deterministic
# findings, what the run cost, how long it took and the evaluation plan
# with its estimate. Exits 0 when the review approves and no deterministic
# finding blocks, as the gate decides; 1 when it requests changes or one
# blocks; 2 on a wrong command line or a setup that fails. Run files stay
# under target/review-local/. The agents run with PARINN_HOME and
# CLAUDE_CONFIG_DIR in a temporary directory that holds a copy of the
# login and is removed at the end.
#
# By default the base is origin/main, as in CI. --base REF reviews only
# what changed since REF, for the fix cycle after a review: the commit
# you last reviewed or pushed. Commit first: only committed work is
# reviewed, and uncommitted changes to tracked files stop the run. A
# change to a path CODEOWNERS assigns is a blocking finding until a
# stakeholder comment consents, so pass --consent FILE (the JSON the
# workflow collects) to see the review as it stands with that comment.
#
# usage: scripts/review-local.sh [--base REF] [--agents base|head]
#        [--consent FILE] [--parallel N]
set -Eeuo pipefail
# Any step that fails is a setup that fails: exit 2, never its own status,
# so a caller never reads it as 1, a review that requests changes.
trap 'exit 2' ERR

base=origin/main
consent=""
agents=head
parallel=16
while [ "$#" -gt 0 ]; do
  case "$1" in
    --base) base="${2:?--base needs a ref}"; shift 2 ;;
    --agents) agents="${2:?--agents needs base or head}"; shift 2 ;;
    --consent) consent="${2:?--consent needs a file}"; shift 2 ;;
    --parallel) parallel="${2:?--parallel needs a number}"; shift 2 ;;
    *) echo "usage: scripts/review-local.sh [--base REF] [--agents base|head] [--consent FILE] [--parallel N]" >&2; exit 2 ;;
  esac
done
case "$agents" in
  base | head) ;;
  *) echo "review-local: --agents needs base or head, not '$agents'" >&2; exit 2 ;;
esac
case "$parallel" in
  "" | *[!0-9]* | 0) echo "review-local: --parallel needs a positive number, not '$parallel'" >&2; exit 2 ;;
esac
# The consent file is read after the cd into the work directory below,
# so a relative path is made absolute against the caller's directory now.
case "$consent" in
  "" | /*) ;;
  *) consent="$PWD/$consent" ;;
esac

repo=$(git rev-parse --show-toplevel)
if [ -n "$(git -C "$repo" status --porcelain --untracked-files=no)" ]; then
  echo "review-local: uncommitted changes would not be reviewed; commit them first" >&2
  exit 2
fi
git -C "$repo" fetch -q origin main

start=$SECONDS
work=$(mktemp -d)
# The agents get a home and a Claude Code configuration of their own,
# beside the work directory and outside the tree they review, so a live
# agent never writes to the real ~/.parinn, $PARINN_HOME or ~/.claude
# (ENG-14). Claude Code refuses a changed HOME, so CLAUDE_CONFIG_DIR
# moves its configuration instead.
iso="$work.iso"
cleanup() {
  git -C "$repo" worktree remove --force "$work/pr-head" 2>/dev/null || true
  git -C "$repo" worktree remove --force "$work" 2>/dev/null || true
  git -C "$repo" worktree prune
  rm -rf "${work:?}" "${iso:?}"
}
trap cleanup EXIT
mkdir -m 700 "$iso" "$iso/parinn" "$iso/claude"
# The login is copied, never linked, so a token the agents refresh stays
# in the copy. CLAUDE_CODE_OAUTH_TOKEN or ANTHROPIC_API_KEY, when set,
# reach the agents unchanged.
login="${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.credentials.json"
if [ -f "$login" ]; then
  install -m 600 "$login" "$iso/claude/.credentials.json"
elif [ -z "${CLAUDE_CODE_OAUTH_TOKEN:-}" ] && [ -z "${ANTHROPIC_API_KEY:-}" ]; then
  echo "review-local: no login file at $login and no CLAUDE_CODE_OAUTH_TOKEN; the agents may not authenticate" >&2
fi
export PARINN_HOME="$iso/parinn" CLAUDE_CONFIG_DIR="$iso/claude"

run() {
  if command -v mise > /dev/null; then
    mise exec -- "$@"
  else
    echo "review-local: mise is not on PATH; running $1 unpinned" >&2
    "$@"
  fi
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

# The head's agent definitions run in place of the base's, copied before
# the head's instruction files leave the tree below.
if [ "$agents" = head ]; then
  mkdir -p .claude/agents
  rm -f .claude/agents/*.md
  cp pr-head/.claude/agents/*.md .claude/agents/
fi

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
rm -rf "$repo/target/review-local/review"
cp -R review "$repo/target/review-local/"

if [ "$status" -ne 0 ]; then
  echo "review-local: review-run could not decide the review (exit $status); see target/review-local/review/logs" >&2
  exit 2
fi
jq -r '"verdict: " + .verdict, (.findings[] | "\(.path):\(.line) — \(.severity) — \(.body)")' review/outcome.json
jq -r '.[] | "deterministic: \(.path):\(.line) — \(.severity) — \(.body)"' review/checked.json
jq -r '"evaluation plan: \(length) components"' review/eval-plan.json
LC_NUMERIC=C printf 'cost: $%.2f\n' "$(jq '[.[].cost_usd] | add // 0' review/usage.json)"
echo "elapsed: $((SECONDS - start)) s"
# The gate withholds approval on a blocking deterministic finding
# whatever the agents' verdict, so this does too.
blocked=$(jq '[.[] | select(.severity == "blocking")] | length' review/checked.json)
if [ "$(jq -r .verdict review/outcome.json)" = approve ] && [ "$blocked" -eq 0 ]; then exit 0; else exit 1; fi
