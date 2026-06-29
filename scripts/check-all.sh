#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

cd "$repo_root"

run "$repo_root/scripts/check-fast.sh"
run "$repo_root/scripts/check-reference.sh"
run python3 "$repo_root/scripts/check-package-manifest-fixtures.py"
run python3 "$repo_root/scripts/check-package-lock-fixtures.py"
run python3 "$repo_root/scripts/generate-release-report.py" --check

cd "$repo_root/go-tools/go2gir"
run go test -count=1 ./...

cd "$repo_root/examples/order_policy"
run go test -count=1 ./...

cd "$repo_root/examples/order_policy/webapp"
run go test -count=1 ./...

cd "$repo_root"
run cargo test -p mpk-vc --test max64_example
run cargo test -p mpk-cert hash
run cargo test -p mpk-cert cert_basic
