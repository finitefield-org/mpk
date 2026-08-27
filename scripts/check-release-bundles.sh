#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)

if [ "$#" -ne 2 ]; then
  printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
  exit 64
fi

case "$1:$2" in
  --fixture:go) action=fixture-go ;;
  --fixture:csharp)
    repository_root=$(CDPATH= cd -- "$script_dir/.." && pwd -P)
    cargo_path=$(command -v cargo) || {
      printf '%s\n' BUNDLE_ASSEMBLER_USAGE >&2
      exit 64
    }
    export CARGO_TARGET_DIR="$repository_root/target"
    exec "$cargo_path" test --quiet --locked --manifest-path "$repository_root/Cargo.toml" \
      -p mpk-cli --test csharp_frontend_runner
    ;;
  --fixture:go-successor)
    exec /usr/bin/env -i PATH=/usr/local/go/bin:/usr/local/bin:/usr/bin:/bin \
      PYTHONDONTWRITEBYTECODE=1 \
      /usr/bin/python3 -B "$script_dir/go_successor_bundles.py" check-fixtures
    ;;
  --fixture:rust-successor|--fixture-update:rust-successor)
    case "$1" in
      --fixture) action=check-fixtures ;;
      --fixture-update) action=update-fixtures ;;
    esac
    exec /usr/bin/env -i PATH=/usr/local/bin:/usr/bin:/bin \
      PYTHONDONTWRITEBYTECODE=1 \
      /usr/bin/python3 -B "$script_dir/rust_successor_bundles.py" "$action"
    ;;
  --fixture:all)
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
