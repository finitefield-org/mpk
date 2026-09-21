#!/usr/bin/env bash
# Remaining string runtime signatures and affected long constructor regressions.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$repo_root"
log_root="${MPK_W09_STRING_LONG_LOGS:-/tmp/mpk-w09-string-hoisted-long}"
mkdir -p "$log_root"
case_list="$(mktemp "${TMPDIR:-/tmp}/mpk-string-cases.XXXXXX")"
trap 'rm -f "$case_list"' EXIT
python3 - <<'PY' > "$case_list"
import json
from pathlib import Path
rows = json.loads(Path('develop/migrations/csharp-03/ordinary-foundation/string-data/pending-runtime.json').read_text())
assert len(rows) == 25
for row in rows:
    print(row['context'] + '\t' + row['operation'])
PY
while IFS=$'\t' read -r context operation; do
  env -u MPK_CORE_MAX_STEPS -u MPK_W09_STRING_DATA_OUT \
    -u MPK_W09_STRING_DATA_RESPONSES -u MPK_W09_STRING_DATA_REQUESTS_OUT \
    MPK_CORE_MAX_SECONDS=600 MPK_W09_STRING_DATA_RUNTIME_PREFIX="$context" \
    MPK_W09_STRING_DATA_OPERATION="$operation" \
    cargo test --release -p mpk-vc --test csharp_practical_vc \
      csharp_03_t06_w09_string_data_original_source -- --nocapture --test-threads=1 \
    2>&1 | tee "$log_root/$context.$operation.log"
done < "$case_list"

# The complete UTF-16 oracle exceeds the agent's one-minute execution window.
env -u MPK_CORE_MAX_STEPS -u MPK_W09_STRING_CONSTRUCT_OUT \
  MPK_CORE_MAX_SECONDS=600 \
  cargo test --release -p mpk-vc --lib \
    string_construction_core_matches_utf16_oracle -- --nocapture --test-threads=1 \
  2>&1 | tee "$log_root/constructor-core-oracle.log"

# Thirteen changed certificates passed locally; only the large binder fixture remains.
(
  cd go-tools/mpk-checker-ref
  go test -v -count=1 -timeout 25m -tags checkeragreement \
    -run '^TestCheckerAgreementWithRustCLIStringConstruction$/^string\.interpolation\.binder_limit\.hex$' .
) 2>&1 | tee "$log_root/constructor-binder-checkers.log"
