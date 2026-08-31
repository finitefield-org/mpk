#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)

case "$#:${1-}" in
    1:--self-test|1:--check-build-inputs|1:--check|1:--update-inventory)
        action=${1#--}
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_build_inputs.py" "$action"
        ;;
    2:--import-build-inputs|2:--build)
        action=${1#--}
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_build_inputs.py" "$action" "$2"
        ;;
    *)
        printf '%s\n' JAVA_BUILD_USAGE >&2
        exit 64
        ;;
esac
