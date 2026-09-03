#!/bin/sh
set -eu

probe_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)

case "$#:${1-}" in
  1:--check)
    action=check
    ;;
  1:--update)
    action=update
    ;;
  1:--check-record)
    action=check-record
    ;;
  1:--self-test)
    action=self-test
    ;;
  *)
    printf '%s\n' CSHARP_PRACTICAL_DATA_PROBE_USAGE >&2
    exit 64
    ;;
esac

exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
  /usr/bin/python3 -B "$probe_dir/run-data-construction-probe.py" "$action"
