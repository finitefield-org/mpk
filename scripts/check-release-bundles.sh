#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

if [ "$#" -ne 2 ]; then
  printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
  exit 64
fi

case "$1:$2" in
  --fixture:successor)
    repository_root=$(CDPATH= cd -- "$script_dir/.." && pwd -P)
    cargo_path=$(command -v cargo) || {
      printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
      exit 64
    }
    export CARGO_TARGET_DIR="$repository_root/target"
    exec "$cargo_path" test --quiet --locked --manifest-path "$repository_root/Cargo.toml" \
      -p mpk-cli --test successor_atomic_cutover -- --full-native-gate
    ;;
  *)
    printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
    exit 64
    ;;
esac
