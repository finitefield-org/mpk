#!/bin/sh
set -eu

# This local entry point deliberately emits only rule IDs and paths. The
# matching value is never printed, so a finding cannot become a second leak in
# the scanner log. The repository does not use a hosted action or workflow.
exec python3 - "$@" <<'PY'
import pathlib
import re
import subprocess
import sys


RULES = (
    ("google-api-key", re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b")),
    ("google-oauth-access-token", re.compile(r"\bya29\.[0-9A-Za-z_-]{20,}\b")),
    ("google-oauth-refresh-token", re.compile(r"\b1//[0-9A-Za-z._~-]{20,}\b")),
    (
        "private-key",
        re.compile(r"-----BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
    ),
    ("aws-access-key-id", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("github-token", re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b")),
    (
        "bearer-token",
        re.compile(r"(?i)\bBearer[ \t]+[0-9A-Za-z._~+/=-]{20,}"),
    ),
    (
        "credential-assignment",
        re.compile(
            r"(?i)\b(api[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret)"
            r"\b[ \t]*[:=][ \t]*[\"']?[0-9A-Za-z._~+/=-]{20,}"
        ),
    ),
)


def repository_root() -> pathlib.Path:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        print("secret scan failed: not inside a Git repository", file=sys.stderr)
        raise SystemExit(2)
    return pathlib.Path(result.stdout.strip()).resolve()


def repository_files(root: pathlib.Path):
    try:
        result = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        print("secret scan failed: could not enumerate repository files", file=sys.stderr)
        raise SystemExit(2)

    for raw_path in result.stdout.split(b"\0"):
        if not raw_path:
            continue
        yield root / pathlib.Path(raw_path.decode("utf-8", "surrogateescape"))


def scan_file(path: pathlib.Path):
    if path.is_symlink() or not path.is_file():
        return []
    try:
        text = path.read_bytes().decode("utf-8", "ignore")
    except OSError:
        return []

    return [rule_id for rule_id, pattern in RULES if pattern.search(text)]


root = repository_root()
findings = []
checked = 0
seen_paths = set()
for path in repository_files(root):
    relative = path.relative_to(root).as_posix()
    if relative in seen_paths:
        continue
    seen_paths.add(relative)
    checked += 1
    for rule_id in scan_file(path):
        findings.append((relative, rule_id))

if findings:
    print("secret scan failed: credential-shaped material detected", file=sys.stderr)
    for relative, rule_id in findings:
        print(f"  path={relative} rule={rule_id} secret=[REDACTED]", file=sys.stderr)
    raise SystemExit(1)

print(f"secret scan passed: checked {checked} tracked/unignored files")
PY
