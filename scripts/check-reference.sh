#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker_dir="$repo_root/go-tools/mpk-checker-ref"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

cd "$checker_dir"
run go test -count=1 ./...
run go test -count=1 -tags checkeragreement -run TestCheckerAgreementWithRustCLI ./...
