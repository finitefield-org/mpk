#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
candidate_check_temporary=""

cleanup_candidate_check() {
  case "$candidate_check_temporary" in
    /tmp/mpk-rust-candidate-gate.*)
      for name in stdout stderr expected repository-before repository-after \
          cache-before cache-after; do
        [[ ! -e "$candidate_check_temporary/$name" ]] \
          || /usr/bin/unlink "$candidate_check_temporary/$name"
      done
      /usr/bin/rmdir "$candidate_check_temporary"
      ;;
  esac
  candidate_check_temporary=""
}
trap cleanup_candidate_check EXIT

if [[ "$(uname -s)" != "Linux" || "$EUID" -ne 0 \
    || ! -w /sys/fs/cgroup/cgroup.subtree_control ]]; then
  printf '%s\n' \
    'the Rust release gate requires root in Linux with a writable global cgroup-v2 hierarchy' \
    >&2
  exit 69
fi

# Verification must use already installed host tools and the frozen Rust build
# closure. These settings make an accidental package or toolchain fetch fail.
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

gate() {
  printf '\n==== %s ====\n' "$1"
}

snapshot_repository_state() {
  local destination=$1
  # Capture tracked, untracked, and ignored paths. Linux ctime detects an
  # in-place write even if a dirty file keeps the same Git status; atime is
  # deliberately excluded so verification reads do not look like writes.
  /usr/bin/find . -xdev \
    -path ./.git -prune -o \
    -path ./release/build-input-cache/rust -prune -o \
    -printf '%P\t%y\t%m\t%u\t%g\t%i\t%s\t%T@\t%C@\t%l\0' \
    | LC_ALL=C sort -z \
    | /usr/bin/sha256sum >"$destination"
}

snapshot_candidate_cache_parent() {
  local destination=$1
  if [[ ! -e release/build-input-cache/rust ]]; then
    printf 'absent\n' >"$destination"
    return
  fi
  # The 4-GiB frozen cache is ignored by Git. Metadata includes ctime, so an
  # in-place write is detected without hashing all reviewed toolchain bytes.
  /usr/bin/find release/build-input-cache/rust -xdev \
    -printf '%P\t%y\t%m\t%s\t%T@\t%C@\t%l\0' \
    | LC_ALL=C sort -z \
    | /usr/bin/sha256sum >"$destination"
}

reject_registered_candidate_modes() {
  local code mode temporary
  candidate_check_temporary="$(mktemp -d /tmp/mpk-rust-candidate-gate.XXXXXXXX)"
  temporary="$candidate_check_temporary"
  printf 'BUNDLE_CANDIDATE_STATE\n' >"$temporary/expected"
  snapshot_repository_state "$temporary/repository-before"
  snapshot_candidate_cache_parent "$temporary/cache-before"
  for mode in --update-candidate --check-candidate; do
    printf '\n==> registered release rejects %s rust\n' "$mode"
    set +e
    "$repo_root/scripts/build-release-bundles.sh" "$mode" rust \
      >"$temporary/stdout" 2>"$temporary/stderr"
    code=$?
    set -e
    if [[ "$code" -ne 65 || -s "$temporary/stdout" ]] \
        || ! cmp -s "$temporary/expected" "$temporary/stderr"; then
      printf 'unexpected registered Rust candidate result for %s: exit=%s\n' \
        "$mode" "$code" >&2
      sed -n '1,20p' "$temporary/stdout" >&2
      sed -n '1,20p' "$temporary/stderr" >&2
      return 1
    fi
  done
  snapshot_repository_state "$temporary/repository-after"
  snapshot_candidate_cache_parent "$temporary/cache-after"
  if ! cmp -s "$temporary/repository-before" "$temporary/repository-after" \
      || ! cmp -s "$temporary/cache-before" "$temporary/cache-after"; then
    printf 'registered Rust candidate rejection made a persistent write\n' >&2
    return 1
  fi
  cleanup_candidate_check
}

cd "$repo_root"

gate "Frozen build inputs and registered release"
run "$repo_root/scripts/build-release-bundles.sh" --check-build-inputs rust
reject_registered_candidate_modes
run "$repo_root/scripts/build-release-bundles.sh" --check all
run "$repo_root/scripts/check-release-bundles.sh" --fixture all

gate "Root workspace quality"
run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace

gate "Rust frontend"
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo fmt --all -- --check
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked
run cargo test -p mpk-cli --test rust_frontend_runner
run cargo test -p mpk-cli --test rust_frontend_negative
run cargo test -p mpk-vc --test rust_positive_corpus
run cargo test -p mpk-vc --test rust_runtime_safety
run cargo test -p mpk-vc --test rust_calls

gate "Migrated Go path"
(
  cd "$repo_root/go-tools/go2vir"
  run go test -count=1 ./...
)
run cargo test -p mpk-vc --test go_vir_corpus

gate "Rust policy and example"
run cargo test -p mpk-cli --test rust_policy_scan
run cargo test -p mpk-cli --test rust_policy_verify
run cargo test -p mpk-cli --test rust_payment_policy

gate "Source-free checker agreement"
run "$repo_root/scripts/check-reference.sh"

gate "Canonical artifact paths"
run python3 "$repo_root/scripts/check-artifact-paths.py"

gate "Two-clean-build and differential determinism"
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked --test differential
run cargo test -p mpk-vc --test vir_differential

gate "Deterministic limits"
run python3 "$repo_root/scripts/rust_build_inputs.py" self-test
run "$repo_root/scripts/run-rust2vir-toolchain.sh" cargo test --locked --test subset_conformance
run cargo test -p mpk-cli --test frontend_limits
run cargo test -p mpk-cli --test policy_limits
run cargo test -p mpk-vc --test verification_limits

gate "Bounded parser fuzz smoke"
run "$repo_root/scripts/check-fuzz-smoke.sh"

gate "Obsolete interface rejection"
run "$repo_root/scripts/check-no-active-gir.sh" --strict

gate "Untrusted release provenance"
run python3 "$repo_root/scripts/generate-release-report.py" --check
run git diff --check

printf '\nRust frontend release gate passed.\n'
