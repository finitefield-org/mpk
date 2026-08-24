#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
launcher=$script_dir/run-rust2vir-toolchain.sh

for target in driver_protocol rust_contract; do
  "$launcher" cargo fuzz run \
    --fuzz-dir /mpk/work/fuzz-project \
    --target x86_64-unknown-linux-gnu \
    --target-dir /mpk/target/fuzz \
    --sanitizer address \
    --jobs 1 \
    "$target" \
    "/mpk/work/fuzz-project/corpus/$target" \
    -- \
    -runs=256 \
    -seed=1 \
    -max_len=1048576 \
    -timeout=5 \
    -rss_limit_mb=1024
done
