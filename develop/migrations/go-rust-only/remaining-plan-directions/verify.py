#!/usr/bin/env python3
"""Record the existing language-scope test and changed documentation inputs."""

from pathlib import Path
import hashlib
import json
import os
import re
import stat
import subprocess
import tarfile
import time


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
DOCS = [
    "develop/docs/05_rust_frontend_design-todo.md",
    "develop/docs/mlang-00-go-rust-shared-boundary-audit.md",
]
COMMAND = [
    "cargo", "test", "--locked", "--offline", "-p", "mpk-vc",
    "--test", "semantic_profile_registry_runtime",
    "installed_registry_contains_only_go_and_rust_and_rejects_other_profiles",
    "--", "--exact",
]


def identity(path):
    raw = path.read_bytes()
    return {
        "path": str(path.relative_to(REPO)),
        "bytes": len(raw),
        "sha256": hashlib.sha256(raw).hexdigest(),
        "mode": stat.S_IMODE(path.stat().st_mode),
    }


def write(name, data):
    with (HERE / name).open("x") as stream:
        json.dump(data, stream, indent=2)
        stream.write("\n")


def main():
    inputs = {REPO / name for name in DOCS}
    inputs.update([
        REPO / "Cargo.toml", REPO / "Cargo.lock", Path(__file__).resolve(),
        REPO / "develop/specs/LANGUAGE_SUPPORT_SCOPE_V1.md",
    ])
    # Retain complete local crate sources, fixtures and literal include inputs;
    # this is input preservation, not an execution of their test suites.
    for name in [
        "crates", "proofs", "fixtures", "develop/specs/vectors", "release/bundles",
    ]:
        inputs.update(path for path in (REPO / name).rglob("*") if path.is_file())
    for name in [".cargo", "rust-toolchain", "rust-toolchain.toml"]:
        path = REPO / name
        if path.is_dir():
            inputs.update(child for child in path.rglob("*") if child.is_file())
        elif path.is_file():
            inputs.add(path)
    ordered = sorted(inputs)
    before = [identity(path) for path in ordered]
    archive = HERE / "prelaunch-inputs.tar.gz"
    with tarfile.open(archive, "x:gz", dereference=True) as packed:
        for path in ordered:
            packed.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    write("prelaunch.json", {"command": COMMAND, "inputs": before,
                              "archive": identity(archive)})
    start = time.monotonic()
    with (HERE / "stdout.log").open("xb") as out, (HERE / "stderr.log").open("xb") as err:
        result = subprocess.run(
            COMMAND, cwd=REPO, env={**os.environ, "CARGO_NET_OFFLINE": "true"},
            stdout=out, stderr=err, check=False,
        )
    after = [identity(path) for path in ordered]
    write("registry-result.json", {
        "command": COMMAND, "exit_code": result.returncode,
        "elapsed_seconds": time.monotonic() - start,
        "status": "passed" if result.returncode == 0 else "failed",
        "inputs_unchanged": before == after,
        "stdout": identity(HERE / "stdout.log"),
        "stderr": identity(HERE / "stderr.log"),
    })
    assert before == after, "A consumed input changed during the real test"
    assert result.returncode == 0, "See the complete original test output"

    originals = json.loads((HERE / "originals.json").read_bytes())
    for row in originals["files"]:
        preserved = REPO / row["complete_original"]
        actual = identity(preserved)
        for field in ["bytes", "sha256", "mode"]:
            assert actual[field] == row[field], (row["path"], field)
    links = []
    for name in DOCS:
        path = REPO / name
        text = path.read_text()
        for label, target in re.findall(r"\[([^\]]+)\]\(([^)]+)\)", text):
            if "://" in target or target.startswith("#"):
                continue
            resolved = path.parent / target.split("#", 1)[0]
            assert resolved.exists(), (name, label, target)
            links.append({"source": name, "target": target, "exists": True})
    write("documentation-review.json", {
        "status": "passed", "current_inputs": [identity(REPO / name) for name in DOCS],
        "originals": "originals.json", "links": links,
        "test_selection": "The existing installed-registry root checks the exact two accepted languages and rejection of removed/unknown profiles. Changed Markdown links and complete prior bytes are checked separately. No executable source changed, so unrelated frontend, kernel proof and release suites are not repeated.",
        "full_gate": "Deferred to final T02-W04; this is an intermediate T02-W01 scope correction.",
    })
    print(json.dumps({"registry": "passed", "documentation": "passed", "links": len(links)}))


if __name__ == "__main__":
    main()
