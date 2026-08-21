#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

case "$#:${1-}:${2-}" in
  2:--update:go|2:--update:all)
    exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
      /usr/bin/python3 "$script_dir/release_bundles.py" update-go
    ;;
  2:--check:go|2:--check:all)
    exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
      /usr/bin/python3 "$script_dir/release_bundles.py" check-go
    ;;
  2:--update-build-inputs:rust|2:--provision-build-inputs:rust|2:--check-build-inputs:rust)
    printf '%s\n' BUNDLE_BUILD_INPUTS_NOT_CONFIGURED >&2
    exit 65
    ;;
  2:--update-candidate:rust|2:--check-candidate:rust)
    printf '%s\n' BUNDLE_BUILD_INPUTS_NOT_CONFIGURED >&2
    exit 65
    ;;
  *)
    printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
    exit 64
    ;;
esac
