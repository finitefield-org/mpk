#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

if [ "$#" -ne 2 ] || [ "$1" != "--fixture" ] || [ "$2" != "go" ]; then
  printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
  exit 64
fi

exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
  /usr/bin/python3 "$script_dir/release_bundles.py" fixture-go
