#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

case "$#:${1-}:${2-}" in
  2:--update:go)
    exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
      /usr/bin/python3 -B "$script_dir/release_bundles.py" update-go
    ;;
  2:--check:go)
    exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
      /usr/bin/python3 -B "$script_dir/release_bundles.py" check-go
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
  2:--update-candidate:rust|2:--check-candidate:rust)
    case "$1" in
      --update-candidate) action=update-candidate ;;
      --check-candidate) action=check-candidate ;;
    esac
    exec /usr/bin/env -i PATH=/usr/bin:/bin \
      /usr/bin/python3 -B "$script_dir/rust_build_inputs.py" "$action"
    ;;
  2:--update:all|2:--check:all)
    case "$1" in
      --update) action=update-all ;;
      --check) action=check-all ;;
    esac
    exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
      /usr/bin/python3 -B "$script_dir/release_bundles.py" "$action"
    ;;
  *)
    printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
    exit 64
    ;;
esac
