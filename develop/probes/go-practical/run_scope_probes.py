#!/usr/bin/env python3
"""Build/replay original Go observations, without executing a practical frontend."""
import argparse
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

from check_scope_report import check, json_bytes, sha, sources, strict_json
from run_module_probes import DEFAULT_GOROOT, verify_host_toolchain
from scope_cases import CASES, HERE, PENDING, RECORD, SCOPE, VECTORS, vectors


def run(argv, *, env, stdin=None):
    result = subprocess.run(list(map(str, argv)), input=stdin, capture_output=True, env=env, timeout=60)
    if result.returncode != 0 or result.stderr:
        raise RuntimeError(f"native observer failed: {result.returncode}\n{result.stderr.decode()}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true")
    group.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise RuntimeError("this retained native receipt is Darwin arm64; no host fallback")
    if args.write:
        VECTORS.write_bytes(json_bytes(vectors()))
    elif strict_json(VECTORS.read_bytes()) != vectors():
        raise RuntimeError("draft vectors changed")
    before = sources()
    toolchain = verify_host_toolchain(DEFAULT_GOROOT, strict_json((HERE / "host-toolchain.json").read_bytes()))
    env = {
        "PATH": str(DEFAULT_GOROOT / "bin") + ":/usr/bin:/bin",
        "GOROOT": str(DEFAULT_GOROOT), "GOENV": "off", "GOWORK": "off",
        "GOTOOLCHAIN": "local", "GOPROXY": "off", "GOSUMDB": "off", "CGO_ENABLED": "0",
        "GOTELEMETRY": "off", "GOOS": "darwin", "GOARCH": "arm64",
        "GOFLAGS": "", "GOEXPERIMENT": "",
        "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache",
    }
    native_input = json_bytes({"sources": [{"id": row["id"], "source": row["source"]} for row in CASES]})
    with tempfile.TemporaryDirectory(prefix="mpk-w04-scope-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        observer = temp / "observer"
        run([DEFAULT_GOROOT / "bin/go", "build", "-trimpath", "-buildvcs=false",
             "-o", observer, HERE / "scope_reference.go"], env=env)
        binary = sha(observer.read_bytes())
        outputs = [run([observer], env=env, stdin=native_input) for _ in range(2)]
        if outputs[0] != outputs[1]:
            raise RuntimeError("native output is not deterministic")
        native_counts, draft_counts = {}, {}
        for row in CASES:
            k = row["native_expectation"]["status"]
            native_counts[k] = native_counts.get(k, 0) + 1
            k = row["draft_outcome"]
            draft_counts[k] = draft_counts.get(k, 0) + 1
        artifacts = {"native-input.json": native_input, "native-output.json": outputs[0]}
        report = {
            "schema": "mpk.go_practical.w04_scope_observations.v0",
            "work_item": "GO-PRAC-T01-W04", "scope": SCOPE,
            "status": "native observations retained; W04 freeze remains in progress",
            "pending": PENDING, "inputs": before, "observer_binary_sha256": binary,
            "toolchain_inventory": toolchain,
            "build": {"release": "go1.25.0", "host": "darwin/arm64",
                      "type_target": "gc/amd64", "effective_language": "go1.25.0",
                      "trimpath": True, "buildvcs": False,
                      "runtime_target_scope": "host observations, not Linux release"},
            "artifacts": {name: {"bytes": len(raw), "sha256": sha(raw)}
                          for name, raw in artifacts.items()},
            "native_type_counts": native_counts, "draft_outcome_counts": draft_counts,
        }
        if sources() != before or sha(observer.read_bytes()) != binary:
            raise RuntimeError("native source/binary changed during observation")
        for name, raw in artifacts.items():
            (temp / name).write_bytes(raw)
        check(report, temp)
        if args.write:
            RECORD.mkdir(parents=True, exist_ok=True)
            for name in artifacts:
                shutil.copyfile(temp / name, RECORD / name)
            (RECORD / "observations.json").write_bytes(json_bytes(report))
        else:
            retained = strict_json((RECORD / "observations.json").read_bytes())
            check(retained)
            if retained != report or any((RECORD / name).read_bytes() != raw for name, raw in artifacts.items()):
                raise RuntimeError("native replay/source/binary identity changed")
    print(f"PASS: {len(CASES)} intact original-Go cases, two identical executions; practical profile remains unimplemented")


if __name__ == "__main__":
    main()
