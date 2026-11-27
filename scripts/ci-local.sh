#!/usr/bin/env bash
# Runs CI's gates here, before a push, from the list ci.yml runs them
# from (scripts/ci-gates.txt, ENG-27), so a defect CI would find shows
# in a minute. By default it runs the fast gates on what the diff
# against origin/main touches: formatting, clippy, the tests and the
# unit-line coverage floor of each touched crate (with a warning naming
# the added lines its unit layer leaves uncovered), the markdown, TOML and
# workflow checks, the srs and scenario gate tests, the cucumber
# scenarios of the requirement ids the diff names, the ENG-34 line
# limits and the drift cases the touched files inject. --full adds the
# gates that need room or the network and runs the crate gates on the
# whole workspace.
#
# Every gate is reported ran, skipped (with the reason: too little disk,
# a core not built, a CI runner, too slow) or failed, and the exit
# status is 1 when one failed. Builds go to target/ci-local with no debug
# information and no incremental cache, which keep a sandbox's disk free
# and its checkout's own target intact; a build step is refused below
# PARINN_CI_MIN_FREE_GB (8 for a heavy gate, half that for a fast one).
#
# usage: scripts/ci-local.sh [--changed] [--base REF] [--full] [--list]
#        [--gate NAME...]
set -Eeuo pipefail
cd "$(git rev-parse --show-toplevel)"
repo=$PWD gates=scripts/ci-gates.txt base=origin/main full=no list=no only=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --changed) shift ;;
    --base) base="${2:?--base needs a ref}"; shift 2 ;;
    --full) full=yes; shift ;;
    --list) list=yes; shift ;;
    --gate) shift; while [ "$#" -gt 0 ] && [ "${1#-}" = "$1" ]; do only+=("$1"); shift; done
      [ "${#only[@]}" -gt 0 ] || { echo "ci-local: --gate needs a name" >&2; exit 2; } ;;
    *) echo "usage: scripts/ci-local.sh [--changed] [--base REF] [--full] [--list] [--gate NAME...]" >&2; exit 2 ;;
  esac
done
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}" CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo/target/ci-local}"
min_gb="${PARINN_CI_MIN_FREE_GB:-8}"
logs="$CARGO_TARGET_DIR/logs"
mkdir -p "$logs"
export TMPDIR="$CARGO_TARGET_DIR/tmp"; mkdir -p "$TMPDIR"

run() { if command -v mise > /dev/null; then mise exec -- "$@"; else "$@"; fi; }
rows() { grep -vE '^(#|$)' "$gates"; }
for g in ${only[@]+"${only[@]}"}; do
  rows | cut -d'|' -f1 | grep -qx "$g" || { echo "ci-local: no gate $g in $gates" >&2; exit 2; }
done

if [ "$list" = yes ]; then
  printf '%-22s %-5s %-8s %-6s %s\n' GATE FIT NEEDS SCOPE LOCAL
  rows | while IFS='|' read -r n fit needs scope ci loc; do
    [ "$loc" = = ] && loc="$ci"
    printf '%-22s %-5s %-8s %-6s %s\n' "$n" "$fit" "$needs" "$scope" "$loc"
  done
  exit 0
fi

git rev-parse -q --verify "$base" > /dev/null || { echo "ci-local: no $base here: git fetch origin main" >&2; exit 2; }
mb=$(git merge-base "$base" HEAD)
files=$({ git diff --name-only "$mb"; git ls-files -o --exclude-standard; } | sort -u)
crates=() dirs=()
while read -r d; do
  [ -f "$d/Cargo.toml" ] || continue
  crates+=("$(sed -n 's/^name = "\(.*\)"/\1/p' "$d/Cargo.toml" | head -1)"); dirs+=("$d")
done < <(printf '%s\n' "$files" | sed -nE 's#^((tooling|crates)/[^/]+)/.*#\1#p' | sort -u)
# The ids of the requirements the diff touches: the SRS rows it changes
# and the scenarios whose lines it changes.
ids=$({
  git diff -U0 "$mb" -- docs/srs | sed -nE 's/^\+\| ([A-Z]{2,4}-[0-9]+) .*/\1/p'
  git diff -U0 "$mb" -- features | awk '/^\+\+\+ b\// { f = substr($2, 3) } /^@@/ { split($3, r, ","); print f, substr(r[1], 2) + 0 }' |
    while read -r f n; do
      awk -v n="$n" '/^ *@/ { for (i = 1; i <= NF; i++) if ($i ~ /^@[A-Z][A-Z]+-[0-9]+$/) id = substr($i, 2) } NR >= n { print id; exit }' "$f"
    done
} | grep . | sort -u || true)
tags=$(printf '%s\n' $ids | grep -v '^NFR-' | grep . | sed 's/^/@/' | paste -sd' ' - | sed 's/ / or /g' || true)
echo "ci-local: $(printf '%s\n' "$files" | grep -c .) changed files; crates: ${crates[*]:-none}; ids: ${ids//$'\n'/ }"
echo "ci-local: target $CARGO_TARGET_DIR, debug=$CARGO_PROFILE_DEV_DEBUG incremental=$CARGO_INCREMENTAL, free-space floor ${min_gb} GB"

report=() failed=0
note() { report+=("$(printf '%-22s %-8s %s' "$1" "$2" "$3")"); echo "ci-local: $1 $2: $3"; }
free_gb() { df -Pk "$CARGO_TARGET_DIR" | awk 'NR == 2 { print int($4 / 1048576) }'; }
floor_of() {
  awk -v c="$1" '$0 ~ "^\\[workspace.metadata.coverage.crates." c "\\.floors\\]" { on = 1; next } /^\[/ { on = 0 } on && /^unit *=/ { print $3; exit }' Cargo.toml
}
default_floor=$(sed -n 's/^floors = .*unit = \([0-9.]*\).*/\1/p' Cargo.toml | head -1)

size_check() {
  local d out nits
  d=$(mktemp -d); mkdir "$d/base" "$d/out"; echo '[]' > "$d/consent.json"
  git archive "$mb" | tar -x -C "$d/base"
  git diff "$mb" > "$d/pr.diff"
  run cargo run --locked --quiet -p review-bundle -- --base "$d/base" --head "$repo" --diff "$d/pr.diff" --consent "$d/consent.json" --out "$d/out" > /dev/null
  nits=$(jq -r '.[] | select(.body | test("ENG-34|ENG-28")) | "  nit: \(.path) \(.body)"' "$d/out/checked.json")
  [ -z "$nits" ] || printf '%s\n' "$nits"
  rm -rf "$d"
}

# Runs command $2 as gate $1, in the log; a failure shows the log's end.
execute() {
  local start=$SECONDS
  echo "ci-local: running $1 ..."
  if [ "$2" = @size ]; then size_check > "$logs/$1.log" 2>&1; else run bash -c "$2" < /dev/null > "$logs/$1.log" 2>&1; fi && { note "$1" ran "$((SECONDS - start)) s $(awk '/^TOTAL/ { print "lines " $10 }' "$logs/$1.log")"; grep '^  nit:' "$logs/$1.log" || true; return 0; }
  tail -n 40 "$logs/$1.log"; note "$1" FAILED "see $logs/$1.log"; failed=1
}

# CI's merge counts a line covered by any layer, which a unit-only run
# cannot see: names the lines the diff adds in crate dir $2 that crate $1's
# unit layer leaves uncovered, as a warning that never fails the gate.
warn_uncovered() {
  local lcov="$logs/$1.lcov" diff="$logs/$1.diff" lines
  git diff -U0 "$mb" -- "$2" > "$diff"
  run cargo llvm-cov report --lcov --output-path "$lcov" -p "$1" --ignore-filename-regex "/tests\\.rs$" > /dev/null 2>&1 || { note "$1" WARNING "no unit coverage report, so the lines the diff adds were not checked"; return 0; }
  lines=$(scripts/uncovered-added.sh "$diff" "$lcov" "$repo" | paste -sd' ' -)
  [ -z "$lines" ] || note "$1" WARNING "CI also counts the integration and e2e layers; these added lines are uncovered by the unit layer: $lines"
}

while IFS='|' read -r name fit needs scope ci loc <&3; do
  if [ "${#only[@]}" -gt 0 ]; then [[ " ${only[*]} " == *" $name "* ]] || continue
  elif [ "$fit" = full ] && [ "$full" = no ]; then note "$name" skipped "heavy ($needs): run with --full"; continue
  fi
  if [ "$fit" = no ]; then note "$name" skipped "needs a CI runner ($needs)"; continue; fi
  [ "$loc" = = ] && loc="$ci"
  case "$loc" in
    @size) ;;
    @* | -) note "$name" skipped "runs in CI as $ci"; continue ;;
  esac
  [ "$needs" != parinn ] || [ -x "$CARGO_TARGET_DIR/release/parinn" ] || { note "$name" skipped "needs a built release core: --gate release-core"; continue; }
  need=$((min_gb / 2)); [ "$fit" = full ] && need=$min_gb
  case "$loc" in *cargo* | @size) [ "$(free_gb)" -ge "$need" ] || { note "$name" skipped "$(free_gb) GB free, below the $need GB floor (PARINN_CI_MIN_FREE_GB)"; continue; } ;; esac
  case "$scope" in
    pkgs | units)
      sel=(${crates[@]+"${crates[@]}"}); [ "$scope" = pkgs ] || { sel=(); for c in ${crates[@]+"${crates[@]}"}; do [ "$c" = scenario ] || sel+=("$c"); done; }
      if [ "$full" = yes ]; then pkgs=--workspace; [ "$scope" = pkgs ] || pkgs="--workspace --exclude scenario"
      elif [ "${#sel[@]}" -eq 0 ]; then note "$name" skipped "no touched crate"; continue
      else pkgs=$(printf -- '-p %s ' "${sel[@]}"); fi
      execute "$name" "${loc//\{pkgs\}/$pkgs}" ;;
    crate)
      [ "${#crates[@]}" -gt 0 ] || { note "$name" skipped "no touched crate"; continue; }
      for i in "${!crates[@]}"; do
        c=${crates[$i]}; floor=$(floor_of "$c"); floor="${floor:-$default_floor}"; cmd="${loc//\{crate\}/$c}"
        execute "$name:$c" "${cmd//\{floor\}/$floor}"
        [ "$name" != coverage-unit ] || warn_uncovered "$c" "${dirs[$i]}"
      done ;;
    tags)
      [ -n "$tags" ] || { note "$name" skipped "no requirement id in the diff (NFR ids need a release core)"; continue; }
      execute "$name" "${loc//\{tags\}/$tags}" ;;
    files)
      [ -n "$files" ] || { note "$name" skipped "no touched file"; continue; }
      DRIFT_TOUCHING="$files" execute "$name" "$loc" ;;
    *) execute "$name" "$loc" ;;
  esac
done 3< <(rows)

echo; echo "ci-local: summary"; printf '  %s\n' "${report[@]}"
[ "$failed" -eq 0 ] || { echo "ci-local: a gate failed" >&2; exit 1; }
