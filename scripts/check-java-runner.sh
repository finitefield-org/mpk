#!/bin/sh
set -eu
script_dir=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")" && pwd -P)
case "$#:${1-}" in
    1:--check-trace-parser)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_runner_gate.py" --check-trace-parser
        ;;
    3:--image)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_runner_gate.py" --image "$2" "$3"
        ;;
    3:--native)
        exec /usr/bin/env -i PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp \
            /usr/bin/python3 -I -S -B "$script_dir/java_runner_gate.py" "$2" "$3"
        ;;
    *) printf '%s\n' JAVA_NATIVE_GATE_USAGE >&2; exit 64 ;;
esac
