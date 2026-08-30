#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mpk_bin="$repo_root/target/debug/mpk"

cd "$repo_root"

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

check_no_github_actions_workflows() {
  local workflow_dir="$repo_root/.github/workflows"
  local workflow_entry=""

  if [[ -L "$workflow_dir" || ( -e "$workflow_dir" && ! -d "$workflow_dir" ) ]]; then
    workflow_entry="$workflow_dir"
  elif [[ -d "$workflow_dir" ]]; then
    workflow_entry="$(find "$workflow_dir" -mindepth 1 -print -quit)"
  fi
  if [[ -n "$workflow_entry" ]]; then
    printf 'GitHub Actions workflow content is forbidden: %s\n' \
      "$workflow_entry" >&2
    return 1
  fi
}

check_accepts() {
  local fixture="$1"
  local output
  printf '\n==> mpk check accepts %s\n' "$fixture"
  require_fixture "$fixture"
  if ! output="$("$mpk_bin" check "$fixture" 2>&1)"; then
    printf '%s\n' "$output" >&2
    return 1
  fi
  if [[ "$output" != *'"verdict":"accepted"'* ]]; then
    printf 'expected accepted verdict for fixture: %s\n%s\n' "$fixture" "$output" >&2
    return 1
  fi
}

check_rejects() {
  local fixture="$1"
  local output
  printf '\n==> mpk check rejects %s\n' "$fixture"
  require_fixture "$fixture"
  if output="$("$mpk_bin" check "$fixture" 2>&1)"; then
    printf 'expected fixture to reject: %s\n%s\n' "$fixture" "$output" >&2
    return 1
  fi
  if [[ "$output" != *'"verdict":"rejected"'* ]]; then
    printf 'expected rejected verdict for fixture: %s\n%s\n' "$fixture" "$output" >&2
    return 1
  fi
}

require_fixture() {
  local fixture="$1"
  if [[ ! -f "$fixture" ]]; then
    printf 'missing fixture: %s\n' "$fixture" >&2
    return 1
  fi
}

run check_no_github_actions_workflows
run cargo fmt --check
run "$repo_root/scripts/check-no-active-gir.sh" --strict
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run cargo build -p mpk-cli

check_accepts fixtures/cert-basic/zero-axiom.hex
check_accepts fixtures/cert-basic/one-theorem.hex
check_rejects fixtures/cert-decode/invalid/bad-magic.hex
check_rejects fixtures/cert-canonical/non-canonical/unsorted-name-table.hex
