#!/bin/sh
set -eu
script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)
case "$#:${1-}" in
    1:--check|1:--update-descriptors)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_release_bundles.py" "${1#--}"
        ;;
    3:--assemble|3:--check-image)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_release_bundles.py" "${1#--}" "$2" "$3"
        ;;
    *) printf '%s\n' JAVA_CANDIDATE_USAGE >&2; exit 64 ;;
esac
