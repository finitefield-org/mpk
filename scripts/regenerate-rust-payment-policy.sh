#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

mode=${1:---check}
unset MPK_UPDATE_RUST_PAYMENT_POLICY

case "$mode" in
  --update)
    update=1
    ;;
  --check)
    update=
    ;;
  *)
    echo "usage: $0 [--check|--update]" >&2
    exit 2
    ;;
esac

python3 scripts/rust_build_inputs.py check-build-inputs
build_inputs_sha256=$(python3 -c '
import json
from pathlib import Path

descriptor = Path("release/build-inputs/rust/build-inputs.json")
print(json.loads(descriptor.read_bytes())["build_inputs_sha256"])
')
rust_cache="$repo_root/release/build-input-cache/rust/$build_inputs_sha256"
rust_cargo="$rust_cache/toolchain/bin/cargo"
rustc="$rust_cache/toolchain/bin/rustc"

if [ ! -x "$rust_cargo" ] || [ ! -x "$rustc" ]; then
  echo "missing validated Rust build-input cache: $rust_cache" >&2
  echo "run: python3 scripts/rust_build_inputs.py provision-build-inputs" >&2
  exit 1
fi

if [ -n "$update" ]; then
  export MPK_UPDATE_RUST_PAYMENT_POLICY=1
fi

CARGO_NET_OFFLINE=true \
CARGO_TARGET_DIR="$repo_root/rust-tools/rust2vir/target" \
LD_LIBRARY_PATH="$rust_cache/toolchain/lib" \
RUSTC="$rustc" \
"$rust_cargo" test \
  --manifest-path rust-tools/rust2vir/Cargo.toml \
  --locked \
  --test payment_policy_example

(
  cd examples/rust-payment-policy
  cargo test --locked
)
cargo test -p mpk-cli --test rust_payment_policy
cargo test -p mpk-cli --test rust_policy_verify

if [ -n "$update" ]; then
  python3 scripts/generate-release-report.py --write
fi
python3 scripts/generate-release-report.py --check
git diff --check
