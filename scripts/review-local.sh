#!/usr/bin/env bash
# Runs the review of HEAD against origin/main here, before a push, so a
# finding costs a minute of this loop and not a CI round (ENG-28).
#
# It prepares the work directory the review workflow prepares (main's
# tree, the head's under pr-head/, the diff, the stakeholder's comments,
# the head's agent instructions moved out of the agents' load path) and
# runs this checkout's review-bundle, review-eval plan and review-run;
# CI runs main's. The agents are the head's .claude/agents/*.md by
# default (--agents head), as main has them after the merge, since the
# tools expect the finding grammar they define; --agents base keeps the
# base's, as CI runs them today. The review reads no skill.
#
# Prints the verdict, each finding on one line, the bundle's
# deterministic findings, the cost, the time taken and the evaluation
# plan. Exits 0 when the review approves and no deterministic finding
# blocks, as the gate decides; 1 when it requests changes or one blocks;
# 2 on a wrong command line or a failed setup. Run files stay under
# target/review-local/. The agents run with PARINN_HOME and
# CLAUDE_CONFIG_DIR in a temporary directory holding a copy of the login.
#
# --base REF reviews only what changed since REF, for the fix cycle
# after a review. Only committed work is reviewed: uncommitted changes
# to tracked files stop the run. A change to a path CODEOWNERS assigns
# blocks until a stakeholder comment consents: --consent FILE (the JSON
# the workflow collects) shows the review with that comment. --resume
# reruns only what the last run left without an accepted report, and
# what changed since, when it reviewed the same head and definitions;
# otherwise it is a full run, and says why.
#
# usage: scripts/review-local.sh [--base REF] [--agents base|head]
#        [--consent FILE] [--parallel N] [--resume]
set -Eeuo pipefail
# Any step that fails is a setup that fails: exit 2, never its own status,
# so a caller never reads it as 1, a review that requests changes.
trap 'exit 2' ERR

base=origin/main
consent=""
agents=head
parallel=16
resume=no
while [ "$#" -gt 0 ]; do
  case "$1" in
    --base) base="${2:?--base needs a ref}"; shift 2 ;;
    --agents) agents="${2:?--agents needs base or head}"; shift 2 ;;
    --consent) consent="${2:?--consent needs a file}"; shift 2 ;;
    --parallel) parallel="${2:?--parallel needs a number}"; shift 2 ;;
    --resume) resume=yes; shift ;;
    *) echo "usage: scripts/review-local.sh [--base REF] [--agents base|head] [--consent FILE] [--parallel N] [--resume]" >&2; exit 2 ;;
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
[ -z "${PARINN_REVIEW_CI_LOCAL:-}" ] || "$repo/scripts/ci-local.sh" >&2 || exit 1; start=$SECONDS

work=$(mktemp -d)
# The agents get a home and a Claude Code configuration of their own, so
# none writes to the real ~/.parinn, $PARINN_HOME or ~/.claude (ENG-14);
# Claude Code refuses a changed HOME, so CLAUDE_CONFIG_DIR moves it.
iso="$work.iso"
cleanup() {
  git -C "$repo" worktree remove --force "$work/pr-head" 2>/dev/null || true
  git -C "$repo" worktree remove --force "$work" 2>/dev/null || true
  git -C "$repo" worktree prune
  rm -rf "${work:?}" "${iso:?}"
}
trap cleanup EXIT
mkdir -m 700 "$iso" "$iso/parinn" "$iso/claude"
# The login is copied, never linked, so a refreshed token stays in the
# copy; with no login file a CLAUDE_CODE_OAUTH_TOKEN is staged there, in
# no agent's environment (ENG-28). An ANTHROPIC_API_KEY reaches them.
login="${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.credentials.json"
stage=no
if [ -f "$login" ]; then
  install -m 600 "$login" "$iso/claude/.credentials.json"
elif [ -n "${CLAUDE_CODE_OAUTH_TOKEN:-}" ]; then
  stage=yes
elif [ -z "${ANTHROPIC_API_KEY:-}" ]; then
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
# A staged token's directory is refused to every agent, and what they
# write is scrubbed of the token.
credential=()
if [ "$stage" = yes ]; then
  tool review-eval -- stage-credential --dir "$iso/claude" > /dev/null
  credential=(--credential "$iso/claude")
fi

# The head's agent definitions replace the base's, copied before its instruction
# files leave the tree below; a link, which could copy in any file, is refused.
if [ "$agents" = head ]; then
  links=$(find pr-head/.claude/agents -maxdepth 1 -name '*.md' -type l)
  if [ -L pr-head/.claude ] || [ -L pr-head/.claude/agents ] || [ -n "$links" ]; then
    echo "review-local: refusing a linked agent definition: ${links:-pr-head/.claude/agents}" >&2; exit 2; fi
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
# A resumed run starts from the last run's review directory, kept until
# this run replaces it below, and only on the same head and definitions.
head_sha=$(git -C "$repo" rev-parse HEAD)
prev="$repo/target/review-local/review"
from=()
if [ "$resume" = yes ]; then
  was=$(jq -r '.head // "none"' "$prev/run-id.json" 2>/dev/null || echo none)
  if [ "$was" != "$head_sha" ]; then
    echo "review-local: a full run: the last run reviewed head $was, not $head_sha" >&2
  elif ! command -v sha256sum > /dev/null; then
    echo "review-local: a full run: sha256sum is not on PATH, so the agent definitions cannot be compared" >&2
  elif ! jq -r '.agents | to_entries[] | "\(.value.definition // "-")  .claude/agents/\(.key).md"' \
    "$prev/run-id.json" | sha256sum --check --status --strict; then
    echo "review-local: a full run: an agent definition changed since the last run" >&2
  else
    from=(--only "$prev/gaps.json" --from "$prev")
  fi
fi
status=0; tool review-run -- --workdir . --parallel "$parallel" --head-sha "$head_sha" ${from[@]+"${from[@]}"} ${credential[@]+"${credential[@]}"} || status=$?
rm -rf "$repo/target/review-local/review"; mkdir -p "$repo/target/review-local"
cp -R review "$repo/target/review-local/"

if [ "$status" -ne 0 ]; then
  echo "review-local: review-run could not decide the review (exit $status); see target/review-local/review/logs" >&2
  exit 2
fi
# Agent text loses its control characters: no escape hides a blocking line.
jq -r '"verdict: " + .verdict, (.findings[] | "\(.path):\(.line) — \(.severity) — \(.body)" | gsub("\\p{Cc}"; ""))' review/outcome.json
jq -r '.[] | "deterministic: \(.path):\(.line) — \(.severity) — \(.body)" | gsub("\\p{Cc}"; "")' review/checked.json
jq -r '"evaluation plan: \(length) components"' review/eval-plan.json
if [ "${#from[@]}" -gt 0 ]; then
  jq -r '[.[] | select(.reused | not) | .agent] as $again
    | "resumed: reused \([.[] | select(.reused)] | length) agents, re-ran \($again | length) (\($again | join(", ")))"' review/usage.json
fi
# Nothing without an accepted report reads as clean.
jq -r 'select(length > 0) | "not reviewed: " + ([.[] | .name + (if .part then " part \(.part)" else "" end)] | join(", "))' review/gaps.json
LC_NUMERIC=C printf 'cost: $%.2f\n' "$(jq '[.[].cost_usd] | add // 0' review/usage.json)"
echo "elapsed: $((SECONDS - start)) s"
# A blocking deterministic finding withholds approval, as in the gate.
blocked=$(jq '[.[] | select(.severity == "blocking")] | length' review/checked.json)
if [ "$(jq -r .verdict review/outcome.json)" = approve ] && [ "$blocked" -eq 0 ]; then exit 0; else exit 1; fi
