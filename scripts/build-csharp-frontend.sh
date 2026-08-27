#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)

case "$#:${1-}" in
  1:--provision-build-inputs)
    action=provision-build-inputs
    ;;
  1:--check-build-inputs)
    action=check-build-inputs
    ;;
  1:--update-inventory)
    action=update-inventory
    ;;
  1:--check)
    action=check
    ;;
  1:--self-test)
    action=self-test
    ;;
  1:--test-capture)
    action=test-capture
    ;;
  1:--test-roslyn)
    action=test-roslyn
    ;;
  2:--build)
    action=build
    ;;
  *)
    printf '%s\n' CSHARP_BUILD_USAGE >&2
    exit 64
    ;;
esac

if [ "$action" = build ]; then
  exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 \
    /usr/bin/python3 -B "$script_dir/csharp_build_inputs.py" build "$2"
fi

exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 \
  /usr/bin/python3 -B "$script_dir/csharp_build_inputs.py" "$action"
