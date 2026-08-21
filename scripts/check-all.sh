#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

cd "$repo_root"

run "$repo_root/scripts/check-fast.sh"
run "$repo_root/scripts/check-no-active-gir.sh" --strict
run "$repo_root/scripts/check-reference.sh"
run "$repo_root/scripts/build-release-bundles.sh" --check go
run "$repo_root/scripts/check-release-bundles.sh" --fixture go
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
run cargo test -p mpk-cli --test policy_report
run cargo test -p mpk-cli --test ai_explain_v1
run cargo test -p mpk-cert hash
run cargo test -p mpk-cert cert_basic
