#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

if [ "$#" -ne 2 ] || [ "$1" != "--fixture" ]; then
  printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
  exit 64
fi

case "$2" in
  go) action=fixture-go ;;
  all)
    action=fixture-all
    repository_root=$(CDPATH= cd -- "$script_dir/.." && pwd -P)
    cargo_path=$(command -v cargo) || {
      printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
      exit 64
    }
    CARGO_TARGET_DIR="$repository_root/target" "$cargo_path" build --quiet --locked \
      --manifest-path "$repository_root/Cargo.toml" -p mpk-cli --bin mpk
    cli_binary=$repository_root/target/debug/mpk
    ;;
  *)
    printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
    exit 64
    ;;
esac

exec /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent \
  /usr/bin/python3 "$script_dir/release_bundles.py" "$action" ${cli_binary+"$cli_binary"}
