#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

cd "$repo_root"

case "$#:${1-}" in
  1:--check)
    run "$repo_root/scripts/check-java-frontend.sh" --check-release-fixtures
    run cargo test --locked -p mpk-cli --test java_release_gate \
      t09_owns_exact_upgrade_corpus_and_keeps_public_activation_closed -- --exact
    exit 0
    ;;
  0:)
    ;;
  *)
    printf '%s\n' JAVA_REHEARSAL_USAGE >&2
    exit 64
    ;;
esac

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" || "$EUID" -ne 0 \
    || ! -w /sys/fs/cgroup/cgroup.subtree_control ]]; then
  printf '%s\n' \
    'the Java rehearsal requires root on native x86-64 Linux with a writable global cgroup-v2 hierarchy' \
    >&2
  exit 69
fi

export CARGO_NET_OFFLINE=true
export GOPROXY=off
export GOSUMDB=off
export GOTOOLCHAIN=local
export RUSTUP_DIST_SERVER=http://127.0.0.1:9
export RUSTUP_UPDATE_ROOT=http://127.0.0.1:9/rustup
export PYTHONDONTWRITEBYTECODE=1

# The currently installed Go/Rust/C# release stays authoritative through T09.
run "$repo_root/scripts/check-csharp-frontend.sh"

# Every Java owner runs from frozen local inputs. Ordinary, non-ignored tests
# already ran in the active workspace gate above.
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
run cargo test --locked -p mpk-cli --test java_frontend_runner

owner="$({
  cargo test --locked -p mpk-cli --test java_frontend_runner --no-run --message-format=json
} | /usr/bin/python3 -B -c '
import json, sys
matches = []
for line in sys.stdin:
    try:
        value = json.loads(line)
    except json.JSONDecodeError:
        continue
    target = value.get("target", {})
    executable = value.get("executable")
    if value.get("reason") == "compiler-artifact" and target.get("name") == "java_frontend_runner" and executable:
        matches.append(executable)
if len(matches) != 1:
    raise SystemExit("JAVA_REHEARSAL_OWNER")
print(matches[0])
')"
[[ "$owner" = /* && -x "$owner" ]] || { printf '%s\n' JAVA_REHEARSAL_OWNER >&2; exit 65; }

rehearsal_root="$(mktemp -d /tmp/mpk-java-t09-rehearsal.XXXXXX)"
cleanup() {
  /bin/rm -rf -- "$rehearsal_root"
}
trap cleanup EXIT INT TERM
image="$rehearsal_root/java-image"

run "$repo_root/scripts/build-java-candidate.sh" --assemble "$image" "$owner"
run "$repo_root/scripts/build-java-candidate.sh" --check-image "$image" "$owner"
run "$repo_root/scripts/check-java-runner.sh" --image "$image" "$owner"
run "$repo_root/scripts/check-java-runner.sh" --native "$image" "$owner"

printf '\nJAVA-03-T09 private rehearsal and active predecessor release gate passed.\n'
