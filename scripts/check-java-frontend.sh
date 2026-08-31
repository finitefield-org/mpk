#!/bin/sh
set -eu
script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)
case "$#:${1-}" in
    1:--check-fixtures|1:--run|1:--check-admission-fixtures|1:--run-admission|1:--check-lowering-fixtures|1:--run-lowering|1:--run-runner)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_frontend_tests.py" "${1#--}"
        ;;
    *)
        printf '%s\n' JAVA_FRONTEND_TEST_USAGE >&2
        exit 64
        ;;
esac
