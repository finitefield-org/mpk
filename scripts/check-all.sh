#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# The aggregate is a verification-only path. Missing host or frozen tools must
# fail closed instead of letting Cargo, Go, or rustup fetch replacements.
export CARGO_NET_OFFLINE=true
export GOPROXY=off
export GOSUMDB=off
export GOTOOLCHAIN=local
export RUSTUP_DIST_SERVER=http://127.0.0.1:9
export RUSTUP_UPDATE_ROOT=http://127.0.0.1:9/rustup

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

cd "$repo_root"

run "$repo_root/scripts/check-fast.sh"
# This two-pass gate owns the complete successor image, installed frontends,
# checker agreement, and Java release validation. Do not repeat its I/O-heavy
# phases below.
run "$repo_root/scripts/check-java-frontend.sh"
run python3 "$repo_root/scripts/check-spec-vectors.py" --check
run python3 "$repo_root/scripts/check-package-manifest-fixtures.py"
run python3 "$repo_root/scripts/check-package-lock-fixtures.py"
run python3 "$repo_root/scripts/generate-release-report.py" --check

cd "$repo_root/go-tools/go2vir"
run go test -count=1 ./...

cd "$repo_root/examples/order_policy"
run go test -count=1 ./...

cd "$repo_root/examples/order_policy/webapp"
run go test -count=1 ./...

for policy in reserve refund discount fee points; do
  cd "$repo_root/examples/payment_policies/$policy"
  run go test -count=1 ./...
done

cd "$repo_root"
run cargo test -p mpk-vc --test go_vir_corpus
run cargo test -p mpk-cli --test frontend_runner
run cargo test -p mpk-cli --test policy_cli
run cargo test -p mpk-cli --test policy_scan
run cargo test -p mpk-cert hash
run cargo test -p mpk-cert cert_basic
