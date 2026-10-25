#!/bin/sh
# Checks that a built parinn executable reports the version and the
# commit the release build stamped into it (ENG-01). The build workflow
# runs it on each binary, on the binary's own platform.
#
# usage: scripts/check-stamp.sh BINARY VERSION COMMIT
set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: $0 BINARY VERSION COMMIT" >&2
  exit 2
fi
bin=$1 version=$2 commit=$3

got=$("$bin" version)
if [ "$got" != "$version" ]; then
  echo "::error::$bin reports $got, expected $version"
  exit 1
fi
want="{\"version\":\"$version\",\"commit\":\"$commit\"}"
got=$("$bin" version --json)
if [ "$got" != "$want" ]; then
  echo "::error::$bin reports $got, expected $want"
  exit 1
fi
echo "$bin: $got"
