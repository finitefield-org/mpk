#!/usr/bin/env bash
# User-owned checks expected to exceed one minute. Run from any directory.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$repo_root"
log_root="${MPK_W09_LIFTED_LONG_LOGS:-/tmp/mpk-w09-lifted-long}"
mkdir -p "$log_root"
for operation in \
  lifted.i32.multiply.checked \
  lifted.i32.divide.checked \
  lifted.i32.remainder.unchecked \
  lifted.i32.divide.unchecked \
  lifted.i64.divide.checked \
  lifted.decimal.equal.unchecked \
  lifted.decimal.not_equal.unchecked \
  lifted.decimal.less.unchecked \
  lifted.decimal.less_equal.unchecked \
  lifted.decimal.greater.unchecked \
  lifted.decimal.greater_equal.unchecked
do
  env -u MPK_CORE_MAX_STEPS -u MPK_W09_LIFTED_DATA_CONTEXT \
    -u MPK_W09_LIFTED_DATA_OUT -u MPK_W09_LIFTED_DATA_RESPONSES \
    -u MPK_W09_LIFTED_DATA_CASE \
    MPK_CORE_MAX_SECONDS=600 MPK_W09_LIFTED_DATA_OPERATION="$operation" \
    cargo test --release -p mpk-vc --test csharp_practical_vc \
      csharp_03_t06_w09_lifted_data_original_source -- --nocapture --test-threads=1 \
    2>&1 | tee "$log_root/$operation.log"
done
cd "$repo_root/go-tools/mpk-checker-ref"
go test -tags checkeragreement -count=1 -v -timeout=30m \
  -run '^TestCheckerAgreementWithRustCLILiftedData$/(i32-checked|i32-unchecked|i64|decimal)\.hex$' \
  2>&1 | tee "$log_root/dual-checkers.log"
