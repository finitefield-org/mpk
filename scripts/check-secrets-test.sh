#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
scanner="$repo_root/scripts/check-secrets.sh"
test_repo=$(mktemp -d "${TMPDIR:-/tmp}/mpk-secret-scan.XXXXXX")
trap 'rm -rf "$test_repo"' EXIT HUP INT TERM

git -C "$test_repo" init --quiet
git -C "$test_repo" config user.email test@example.invalid
git -C "$test_repo" config user.name "MPK secret scanner test"

# Assemble a provider-shaped canary from fragments so the canary is never
# present in the repository, fixtures, snapshots, or this script as a whole.
canary_prefix='AI'
canary_provider='za'
canary_suffix='0123456789abcdefghijklmnopqrstuvwxy'
canary="$canary_prefix$canary_provider$canary_suffix"
printf '{"credential":"%s"}\n' "$canary" > "$test_repo/canary.json"
git -C "$test_repo" add canary.json

if output=$(cd "$test_repo" && "$scanner" 2>&1); then
    printf '%s\n' "secret scanner self-test failed: canary was accepted" >&2
    exit 1
fi

case "$output" in
    *"$canary"*)
        printf '%s\n' "secret scanner self-test failed: canary appeared in output" >&2
        exit 1
        ;;
esac

printf '%s' "$output" | grep -F '[REDACTED]' >/dev/null
printf '%s\n' "secret scanner self-test passed"
