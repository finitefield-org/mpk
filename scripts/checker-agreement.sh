#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$repo_root/go-tools/mpk-checker-ref"
go test -count=1 -tags checkeragreement -run TestCheckerAgreementWithRustCLI ./...
