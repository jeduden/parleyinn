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
# (5 s dpkg recovery + 3 s pause) = 151 s, after up to 60 s installing
# from a cache. The whole run is also held to a total budget of 150 s:
# each command's timeout is cut to the time left, so the script prints
# its error and retry count and exits within about 150 s. Callers give
# the step a 3-minute timeout, above that.
#
# With APT_CACHE naming a directory, the packages install from the .deb
# files in it first, without the mirrors, so a build does not depend on
# the mirrors once one install succeeded. An install from it that
# fails is a warning and falls back to the mirrors. After an install
# from the mirrors, the .deb files it downloaded are copied there for a
# CI cache to keep; the directory is made only when there are some, so
# an empty one is never kept.
#
# usage: [APT_CACHE=DIR] scripts/apt-install.sh PACKAGE...
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 PACKAGE..." >&2
  exit 2
fi

attempts=3
update_timeout=15
install_timeout=30
cache_timeout=60
recover_timeout=5
pause=3
total=150
hosts="azure.archive.ubuntu.com, archive.ubuntu.com, security.ubuntu.com"
opts=(
  -o Acquire::Retries=3
  -o Acquire::http::Timeout=10
  -o Acquire::https::Timeout=10
  -o DPkg::Lock::Timeout=10
  -o APT::Keep-Downloaded-Packages=true
)
cache=${APT_CACHE:-}
# With a cache, apt downloads into a directory of its own, which no
# image's clean-up hook empties after the install.
fetched=()
if [ -n "$cache" ]; then
  archives=$(mktemp -d)
  mkdir -p "$archives/partial"
  chmod 755 "$archives"
  fetched=(-o "Dir::Cache::archives=$archives")
fi

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

# Installs the cache's .deb files with dpkg, which never reaches the
# network: apt, given local files, still reads its lists and may wait
# on a mirror. Fails when there are none, when they do not hold every
# package the install needs, or when unpacking them outlasts its bound;
# dpkg says which.
from_cache() {
  local debs=()
  if [ -n "$cache" ] && [ -d "$cache" ]; then
    debs=("$cache"/*.deb)
  fi
  if [ "${#debs[@]}" -eq 0 ] || [ ! -e "${debs[0]}" ]; then
    return 1
  fi
  bounded "$cache_timeout" sudo dpkg --install --skip-same-version "${debs[@]}"
}

# Removes each package a failed install from the cache left unpacked
# but not configured, so the mirrors' install starts as it would have
# without the cache.
rollback() {
  local left
  left=$(dpkg-query -W -f '${db:Status-Status} ${Package}\n' | awk '$1 != "installed" && $1 != "not-installed" && $1 != "config-files" {print $2}')
  if [ -n "$left" ]; then
    # shellcheck disable=SC2086 # one package name per word
    bounded "$recover_timeout" sudo dpkg --remove --force-depends $left ||
      echo "::warning::dpkg could not roll back $left"
  fi
}

# Copies the .deb files the install downloaded into the cache, owned by
# the user that runs the script, so the CI cache can read them.
keep() {
  if [ -z "$cache" ]; then
    return 0
  fi
  local debs=("$archives"/*.deb)
  if [ ! -e "${debs[0]}" ]; then
    return 0
  fi
  mkdir -p "$cache"
  sudo cp "${debs[@]}" "$cache"/ && sudo chown -R "$(id -u):$(id -g)" "$cache" ||
    echo "::warning::could not keep the apt packages in $cache"
}

if [ -n "$cache" ]; then
  if from_cache; then
    echo "apt packages from the cache in $cache"
    report
    exit 0
  fi
  if [ -d "$cache" ]; then
    echo "::warning::apt could not install from the cache in $cache; trying the mirrors"
    rollback
  fi
fi

for attempt in $(seq 1 "$attempts"); do
  if [ "$attempt" -gt 1 ]; then
    retries=$((retries + 1))
    sleep "$pause"
    bounded "$recover_timeout" sudo dpkg --configure -a ||
      echo "::warning::dpkg --configure -a failed before apt attempt $attempt/$attempts"
  fi
  if bounded "$update_timeout" sudo apt-get "${opts[@]}" update -qq &&
    bounded "$install_timeout" sudo apt-get "${opts[@]}" "${fetched[@]}" install -y -qq \
      --no-install-recommends "$@"; then
    keep
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
