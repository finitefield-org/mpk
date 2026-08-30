#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

if [[ "$(uname -s)" != "Linux" || "$EUID" -ne 0 \
    || ! -w /sys/fs/cgroup/cgroup.subtree_control ]]; then
  printf '%s\n' \
    'the C# release gate requires root in Linux with a writable global cgroup-v2 hierarchy' \
    >&2
  exit 69
fi

# Verification consumes only already-reviewed local tools and frozen caches.
# Any accidental dependency, toolchain, or module fetch must fail closed.
export CARGO_NET_OFFLINE=true
export GOPROXY=off
export GOSUMDB=off
export GOTOOLCHAIN=local
export RUSTUP_DIST_SERVER=http://127.0.0.1:9
export RUSTUP_UPDATE_ROOT=http://127.0.0.1:9/rustup
export PYTHONDONTWRITEBYTECODE=1

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

cd "$repo_root"

run "$repo_root/scripts/build-release-bundles.sh" --check-build-inputs rust
run "$repo_root/scripts/build-csharp-frontend.sh" --check-build-inputs
run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo fmt --all -- --check
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked

# `--fixture successor` executes the `successor_atomic_cutover` installed-image owner.
for pass in 1 2; do
  printf '\n==== Active successor release pass %s of 2 ====\n' "$pass"

  run "$repo_root/scripts/build-csharp-frontend.sh" --check
  run "$repo_root/scripts/build-release-bundles.sh" --check successor

  run cargo test --locked -p mpk-cli \
    --test csharp_profile_vectors \
    --test csharp_release_gate
  run cargo test --locked -p mpk-cli \
    --test successor_go_release \
    --test successor_rust_release \
    --test csharp_frontend_runner \
    --test csharp_emission \
    --test csharp_policy_verify \
    --test csharp_ai_explain
  run cargo test --locked -p mpk-vc \
    --test go_vir_corpus \
    --test rust_positive_corpus \
    --test vir_differential \
    --test successor_go_migration \
    --test successor_rust_migration \
    --test successor_vc
  run cargo test --locked -p mpk-api v2_tests

  (
    cd "$repo_root/go-tools/go2vir"
    run go test -count=1 ./...
  )
  run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked --test differential

  run "$repo_root/scripts/check-release-bundles.sh" --fixture successor
  run "$repo_root/scripts/check-reference.sh"
  run python3 -B "$repo_root/scripts/check-artifact-paths.py"
  run /usr/bin/git diff --check
done

run "$repo_root/scripts/check-fuzz-smoke.sh"
run "$repo_root/scripts/check-no-active-gir.sh" --strict
run python3 -B "$repo_root/scripts/generate-release-report.py" --check

printf '\nActive Go/Rust/C# successor release gate passed twice.\n'
