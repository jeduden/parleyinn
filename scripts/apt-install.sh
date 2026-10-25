#!/usr/bin/env bash
# Installs apt packages on a CI runner under bounded retries: a stalled
# or failed mirror costs one attempt, not the job's whole budget.
#
# Each attempt runs apt-get update and apt-get install together, each
# under its own timeout, with apt's own retries and timeouts set. Before
# a retry it finishes any package an interrupted install left half
# configured, so the retry does not fail on dpkg's state. Every failed
# attempt is printed as a warning and the last as an error (I6); the
# number of retries is printed and, under GitHub Actions, written to the
# step summary, so the retry rate stays visible.
#
# Worst case: 3 attempts x (15 s update + 30 s install) + 2 retries x
# (5 s dpkg recovery + 3 s pause) = 151 s. The whole run is also held
# to a total budget of 150 s: each command's timeout is cut to the time
# left, so the script prints its error and retry count and exits within
# about 150 s. Callers give the step a 3-minute timeout, above that.
#
# usage: scripts/apt-install.sh PACKAGE...
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 PACKAGE..." >&2
  exit 2
fi

attempts=3
update_timeout=15
install_timeout=30
recover_timeout=5
pause=3
total=150
hosts="azure.archive.ubuntu.com, archive.ubuntu.com, security.ubuntu.com"
opts=(
  -o Acquire::Retries=3
  -o Acquire::http::Timeout=10
  -o Acquire::https::Timeout=10
  -o DPkg::Lock::Timeout=10
)

deadline=$((SECONDS + total))
retries=0

# Runs a command under the smaller of its own timeout and the time left
# in the total budget; fails at once when no time is left.
bounded() {
  local limit=$1 left=$((deadline - SECONDS))
  shift
  if [ "$left" -le 0 ]; then
    return 124
  fi
  if [ "$limit" -gt "$left" ]; then
    limit=$left
  fi
  timeout "$limit" "$@"
}

report() {
  echo "apt retries: $retries"
  if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "apt retries: $retries" >> "$GITHUB_STEP_SUMMARY"
  fi
}

for attempt in $(seq 1 "$attempts"); do
  if [ "$attempt" -gt 1 ]; then
    retries=$((retries + 1))
    sleep "$pause"
    bounded "$recover_timeout" sudo dpkg --configure -a ||
      echo "::warning::dpkg --configure -a failed before apt attempt $attempt/$attempts"
  fi
  if bounded "$update_timeout" sudo apt-get "${opts[@]}" update -qq &&
    bounded "$install_timeout" sudo apt-get "${opts[@]}" install -y -qq --no-install-recommends "$@"; then
    report
    exit 0
  fi
  echo "::warning::apt attempt $attempt/$attempts failed (hosts: $hosts)"
  if [ $((deadline - SECONDS)) -le 0 ]; then
    break
  fi
done
report
echo "::error::apt could not install $* after $attempt of $attempts attempts within ${total} s (hosts: $hosts)"
exit 1
