#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

case "$#:${1-}:${2-}" in
  2:--check:successor|2:--check:all)
    exec /usr/bin/env -i PATH=/usr/local/bin:/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 \
      /usr/bin/python3 -B "$script_dir/successor_release_bundles.py" check
    ;;
  2:--update-build-inputs:rust|2:--provision-build-inputs:rust|2:--check-build-inputs:rust)
    case "$1" in
      --update-build-inputs) action=update-build-inputs ;;
      --provision-build-inputs) action=provision-build-inputs ;;
      --check-build-inputs) action=check-build-inputs ;;
    esac
    exec /usr/bin/env -i PATH=/usr/bin:/bin \
      /usr/bin/python3 -B "$script_dir/rust_build_inputs.py" "$action"
    ;;
  *)
    printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
    exit 64
    ;;
esac
