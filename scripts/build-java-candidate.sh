#!/bin/sh
set -eu
script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)
case "$#:${1-}" in
    1:--check)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_release_bundles.py" "${1#--}"
        ;;
    *) printf '%s\n' JAVA_CANDIDATE_USAGE >&2; exit 64 ;;
esac
