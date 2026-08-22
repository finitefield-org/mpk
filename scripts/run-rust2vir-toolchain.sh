#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

exec /usr/bin/env -i PATH=/usr/bin:/bin \
  /usr/bin/python3 "$script_dir/rust_build_inputs.py" launch "$@"
