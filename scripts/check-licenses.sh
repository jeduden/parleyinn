#!/bin/sh
# Checks that a built parinn executable prints exactly the third-party
# licenses the release build regenerated for its target (ENG-31). The
# build workflow runs it on each binary, on the binary's own platform.
#
# usage: scripts/check-licenses.sh BINARY REGENERATED_FILE
set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 BINARY REGENERATED_FILE" >&2
  exit 2
fi
bin=$1 want=$2

got=$(mktemp)
trap 'rm -f "$got"' EXIT
"$bin" licenses > "$got"
if ! cmp -s "$got" "$want"; then
  echo "::error::$bin prints licenses that differ from $want"
  exit 1
fi
echo "$bin: licenses equal $want"
