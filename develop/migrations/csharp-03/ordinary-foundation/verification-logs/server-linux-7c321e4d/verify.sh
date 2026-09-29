#!/usr/bin/env bash
set -euo pipefail
cd /root/mpk
EVIDENCE=/root/mpk/develop/migrations/csharp-03/ordinary-foundation/verification-logs/server-linux-7c321e4d
mkdir -p "$EVIDENCE"
HEAD=$(/usr/bin/git rev-parse HEAD)
printf '%s\n' "$HEAD" > "$EVIDENCE/head.txt"
test "$HEAD" = 7c321e4dd103941d72f864ef8d1877400ca8c521
export CARGO_TARGET_DIR=/root/mpk/target
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_TEST_OPT_LEVEL=2
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
export CARGO_PROFILE_TEST_OVERFLOW_CHECKS=true
export CARGO_INCREMENTAL=0
printf 'running definitions %s\n' "$(date -u +%FT%TZ)" > "$EVIDENCE/status.txt"
run_test() {
  local label="$1" filter="$2" start end code
  start="$(date -u +%FT%TZ)"
  set +e
  /usr/bin/cargo test -p mpk-vc --test csharp_practical_vc "$filter" -- --nocapture --test-threads=1 > "$EVIDENCE/$label.log" 2>&1
  code=$?
  set -e
  end="$(date -u +%FT%TZ)"
  printf '%s %s %s %s\n' "$label" "$start" "$end" "$code" >> "$EVIDENCE/runs.txt"
  sha256sum "$EVIDENCE/$label.log" >> "$EVIDENCE/log-hashes.txt"
  if test "$code" -ne 0; then
    printf 'failed %s %s\n' "$label" "$end" > "$EVIDENCE/status.txt"
    exit "$code"
  fi
}
run_test definitions csharp_03_t06_w09_concrete_operations_original_source_certificates
printf 'running decimal-pins %s\n' "$(date -u +%FT%TZ)" > "$EVIDENCE/status.txt"
run_test decimal-pins csharp_03_t06_w09_decimal_fixed_formats_pinned_sources
printf 'running real-values %s\n' "$(date -u +%FT%TZ)" > "$EVIDENCE/status.txt"
run_test real-values csharp_03_t06_w09_concrete_operations_real_values_and_guards
printf 'passed %s\n' "$(date -u +%FT%TZ)" > "$EVIDENCE/status.txt"
