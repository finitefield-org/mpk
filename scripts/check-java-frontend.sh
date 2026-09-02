#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

if [[ "$#" -ne 0 ]]; then
  printf '%s\n' 'usage: sudo ./scripts/check-java-frontend.sh' >&2
  exit 64
fi

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" || "$EUID" -ne 0 \
    || ! -w /sys/fs/cgroup/cgroup.subtree_control || ! -x /usr/bin/strace ]]; then
  printf '%s\n' \
    'the Java release gate requires root on native x86-64 Linux, /usr/bin/strace, and a writable global cgroup-v2 hierarchy' \
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
run "$repo_root/scripts/build-java-frontend.sh" --check-build-inputs
run "$repo_root/scripts/build-java-candidate.sh" --check
run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo fmt --all -- --check
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked

# Execute every provisioned Java owner that ordinary workspace tests skip.
# These remain offline and consume only the frozen JDK/image caches checked
# above; omitting them would silently weaken the completed T02-T09 gates.
run cargo test --locked -p mpk-cli --test java_build_inputs \
  offline_java_candidate_builds_twice_and_refuses_ambient_options -- --ignored --exact --nocapture
run cargo test --locked -p mpk-cli --test java_frontend_vectors \
  pinned_java_capture_compiler_and_diagnostic_vectors_execute -- --ignored --exact --nocapture
run cargo test --locked -p mpk-cli --test java_subset \
  pinned_source_admission_executes_every_owned_case_and_preserves_full_closure -- --ignored --exact --nocapture
run cargo test --locked -p mpk-cli --test java_contracts \
  pinned_contract_executor_matches_independent_normalized_hashes_and_all_refusals -- --ignored --exact --nocapture
run cargo test --locked -p mpk-cli --test java_lowering -- --ignored --nocapture
run cargo test --locked -p mpk-cli --test java_source_maps -- --ignored --nocapture
run cargo test --locked -p mpk-cli --test java_policy_verify \
  pinned_java_reaches_same_byte_certificate_and_private_consumers -- --ignored --exact --nocapture
run cargo test --locked -p mpk-cli --test java_release_gate \
  pinned_t09_release_rehearsal_builds_and_runs_twice -- --ignored --exact --nocapture

# `--fixture successor` executes the four-language `successor_atomic_cutover`
# installed-image owner, including the active JVM syscall/resource boundary.
# Each pass also rebuilds and exercises the fixed Java candidate in its
# private, networkless JDK test container.
for pass in 1 2; do
  printf '\n==== Active Go/Rust/C#/Java successor release pass %s of 2 ====\n' "$pass"

  run "$repo_root/scripts/build-csharp-frontend.sh" --check
  run "$repo_root/scripts/build-java-frontend.sh" --check
  run "$repo_root/scripts/build-java-candidate.sh" --check
  run "$repo_root/scripts/build-release-bundles.sh" --check successor
  run /usr/bin/python3 -I -S -B "$repo_root/scripts/java_frontend_tests.py" run-release

  run cargo test --locked -p mpk-cli \
    --test csharp_profile_vectors \
    --test csharp_release_gate \
    --test java_activation \
    --test java_build_inputs \
    --test java_frontend_runner \
    --test java_release_gate
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
    --test successor_vc \
    --test java_profile_spec \
    --test java_profile_vectors
  run cargo test --locked -p mpk-api v2_tests

  (
    cd "$repo_root/go-tools/go2vir"
    run go test -count=1 ./...
  )
  run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked --test differential

  run "$repo_root/scripts/check-release-bundles.sh" --fixture successor
  run "$repo_root/scripts/check-reference.sh"
  run /usr/bin/python3 -B "$repo_root/scripts/check-artifact-paths.py"
  run /usr/bin/python3 -B "$repo_root/scripts/check-spec-vectors.py" --check
  run /usr/bin/git diff --check
done

run "$repo_root/scripts/check-fuzz-smoke.sh"
run "$repo_root/scripts/check-no-active-gir.sh" --strict
run /usr/bin/python3 -B "$repo_root/scripts/generate-release-report.py" --check

printf '\nActive Go/Rust/C#/Java successor release gate passed twice.\n'

exit 0
