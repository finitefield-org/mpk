#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)

case "$#:${1-}" in
  1:--check-build-inputs)
    action=check-build-inputs
    ;;
  1:--check)
    action=check
    ;;
  1:--update-inventory)
    action=update-inventory
    ;;
  1:--self-test)
    action=self-test
    ;;
  1:--test-capture)
    action=test-capture
    ;;
  1:--test-syntax)
    action=test-syntax
    ;;
  1:--test-ordered)
    action=test-ordered
    ;;
  1:--test-sequences)
    action=test-sequences
    ;;
  1:--test-arrays)
    action=test-arrays
    ;;
  1:--test-structural)
    action=test-structural
    ;;
  1:--test-initialization)
    action=test-initialization
    ;;
  1:--test-construction)
    action=test-construction
    ;;
  1:--test-types)
    action=test-types
    ;;
  *)
    printf '%s\n' CSHARP_PRACTICAL_BUILD_USAGE >&2
    exit 64
    ;;
esac

exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
  /usr/bin/python3 -B "$script_dir/csharp_practical_build_inputs.py" "$action"
