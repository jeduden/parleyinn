#!/usr/bin/env bash
# Installs apt packages on a CI runner under bounded retries: a stalled
# or failed mirror costs one attempt, not the job's whole budget.
#
# Each attempt runs apt-get update and apt-get install together, each
# under a 40-second timeout, with apt's own retries and timeouts set.
# Every failed attempt is printed as a warning and the last as an error
# (I6); under GitHub Actions the number of retries goes to the step
# summary, so the retry rate stays visible.
#
# usage: scripts/apt-install.sh PACKAGE...
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 PACKAGE..." >&2
  exit 2
fi

attempts=3
host=archive.ubuntu.com
opts=(
  -o Acquire::Retries=3
  -o Acquire::http::Timeout=20
  -o Acquire::https::Timeout=20
  -o DPkg::Lock::Timeout=30
)

retries=0
summary() {
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "apt retries: $retries" >> "$GITHUB_STEP_SUMMARY"
  fi
}

for attempt in $(seq 1 "$attempts"); do
  if timeout 40 sudo apt-get "${opts[@]}" update -qq &&
    timeout 40 sudo apt-get "${opts[@]}" install -y -qq --no-install-recommends "$@"; then
    summary
    exit 0
  fi
  echo "::warning::apt attempt $attempt/$attempts failed (host: $host)"
  if [ "$attempt" -lt "$attempts" ]; then
    retries=$((retries + 1))
    sleep 5
  fi
done
summary
echo "::error::apt could not install $* after $attempts attempts (host: $host)"
exit 1
