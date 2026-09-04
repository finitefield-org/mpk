#!/usr/bin/env python3
"""Reproduce the W09 ordinary-certificate checker-capacity evidence."""

import argparse
import copy
import hashlib
import json
import os
import platform
from pathlib import Path
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[3]
RECORD = ROOT / "develop/migrations/csharp-03/probes/checker-capacity.json"
TEST = "recursor_feasibility::csharp_03_t01_w09_checker_capacity_bytes_are_reproducible"
TIMEOUT_SECONDS = 60


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode()


def execute(command, *, cwd=ROOT, env=None):
    started = time.monotonic_ns()
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        capture_output=True,
        timeout=TIMEOUT_SECONDS,
    )
    elapsed_ms = (time.monotonic_ns() - started + 999_999) // 1_000_000
    return result, elapsed_ms


def checked(command, *, cwd=ROOT, env=None):
    result, _ = execute(command, cwd=cwd, env=env)
    if result.returncode:
        raise RuntimeError(
            f"command failed: {command!r}\n"
            f"{result.stdout.decode(errors='replace')}\n"
            f"{result.stderr.decode(errors='replace')}"
        )
    return result.stdout.decode()


def source_inventory():
    paths = checked(
        [
            "rg",
            "--files",
            "crates/mpk-core",
            "crates/mpk-kernel",
            "crates/mpk-cert",
            "crates/mpk-cli",
            "go-tools/mpk-checker-ref",
        ]
    ).splitlines()
    paths = [
        path
        for path in paths
        if ("/src/" in path and path.endswith(".rs"))
        or (
            path.startswith("go-tools/")
            and path.endswith(".go")
            and not path.endswith("_test.go")
        )
        or path.endswith(("Cargo.toml", "go.mod"))
    ]
    paths += [
        "Cargo.toml",
        "Cargo.lock",
        "develop/probes/csharp-03/recursor_feasibility.rs",
        "develop/probes/csharp-03/run-checker-capacity-probe.py",
    ]
    return [
        {"path": path, "raw_sha256": digest((ROOT / path).read_bytes())}
        for path in sorted(set(paths))
    ]


def observe(command, checker):
    result, elapsed_ms = execute(command)
    stdout = result.stdout.decode(errors="replace")
    stderr = result.stderr.decode(errors="replace")
    if checker == "reference":
        value = json.loads(stdout)
        accepted = result.returncode == 0 and value.get("verdict") == "accepted"
    else:
        accepted = (
            result.returncode == 0
            and stdout.startswith("ok module=")
            and " axioms=0" in stdout
        )
    if not accepted:
        raise RuntimeError(
            f"unexpected {checker} capacity result: {result.returncode}: {stdout} {stderr}"
        )
    return {
        "result": "accepted",
        "exit_code": result.returncode,
        "stdout": stdout,
        "stdout_sha256": digest(result.stdout),
        "stderr": stderr,
        "stderr_sha256": digest(result.stderr),
        "elapsed_ms": elapsed_ms,
    }


def stable_runs(runs):
    stable = copy.deepcopy(runs)
    for run in stable:
        for observation in run["observations"]:
            observation["rust"].pop("elapsed_ms")
            observation["reference"].pop("elapsed_ms")
    return stable


def assert_timings(runs):
    for run in runs:
        for observation in run["observations"]:
            for checker in ["rust", "reference"]:
                elapsed = observation[checker]["elapsed_ms"]
                if not 0 < elapsed < TIMEOUT_SECONDS * 1_000:
                    raise AssertionError(
                        f"{observation['id']} {checker} elapsed_ms out of range: {elapsed}"
                    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--update", action="store_true")
    group.add_argument("--check", action="store_true")
    args = parser.parse_args()

    checked(["cargo", "build", "-p", "mpk-cli"])
    with tempfile.TemporaryDirectory(prefix="mpk-w09-capacity-") as directory:
        temporary = Path(directory)
        exported = temporary / "certificates"
        env = {
            **os.environ,
            "MPK_W09_CAPACITY_EXPORT": str(exported),
            "PYTHONDONTWRITEBYTECODE": "1",
        }
        checked(
            [
                "cargo",
                "test",
                "-p",
                "mpk-vc",
                "--test",
                "csharp_practical_spec",
                TEST,
                "--",
                "--exact",
            ],
            env=env,
        )
        probe = json.loads((exported / "manifest.json").read_bytes())
        reference = temporary / "mpk-checker-ref"
        checked(
            [
                "go",
                "build",
                "-trimpath",
                "-o",
                str(reference),
                "./cmd/mpk-checker-ref",
            ],
            cwd=ROOT / "go-tools/mpk-checker-ref",
            env={**os.environ, "GOCACHE": str(temporary / "go-cache")},
        )
        runs = []
        for repetition in [1, 2]:
            observations = []
            for case in probe["cases"]:
                certificate = exported / f"{case['id']}.mpcert"
                data = certificate.read_bytes()
                assert digest(data) == case["raw_sha256"]
                rust = observe(
                    [str(ROOT / "target/debug/mpk"), "verify", str(certificate)],
                    "rust",
                )
                reference_result = observe(
                    [str(reference), "verify", str(certificate)], "reference"
                )
                observations.append(
                    {"id": case["id"], "rust": rust, "reference": reference_result}
                )
            runs.append({"repetition": repetition, "observations": observations})

    inventory = source_inventory()
    result = {
        "schema": "mpk.csharp_practical.checker_capacity_evidence.v1",
        "work_item": "CSHARP-03-T01-W09",
        "status": "capacity_measured",
        "activation": "candidate_only",
        "probe": probe,
        "runs": runs,
        "host": {"system": platform.system(), "machine": platform.machine()},
        "toolchain": {
            "rustc": checked(["rustc", "--version"]).strip(),
            "go": checked(["go", "version"]).strip(),
        },
        "timeout_seconds_per_invocation": TIMEOUT_SECONDS,
        "source_inventory": inventory,
        "source_inventory_sha256": digest(canonical(inventory)),
        "claim": (
            "Both independent checkers accept the identical ordinary-term Certificate v0 "
            "at one below, at, and one above each frozen profile ceiling. The profile rejects "
            "each above-ceiling input before checker invocation and therefore retains measured "
            "one-step checker headroom without changing checker acceptance."
        ),
        "release_gate": False,
    }
    assert_timings(result["runs"])

    if args.update:
        RECORD.parent.mkdir(parents=True, exist_ok=True)
        RECORD.write_bytes(canonical(result))
    else:
        retained = json.loads(RECORD.read_bytes())
        for field in [
            "schema",
            "work_item",
            "status",
            "activation",
            "probe",
            "source_inventory",
            "source_inventory_sha256",
            "timeout_seconds_per_invocation",
            "claim",
            "release_gate",
        ]:
            assert retained[field] == result[field], f"drift: {field}"
        assert stable_runs(retained["runs"]) == stable_runs(result["runs"]), "drift: runs"
        assert_timings(retained["runs"])

    print(
        "W09 capacity: 4 counters x 3 boundaries x 2 checkers x 2 runs; "
        "all 48 checker invocations accepted with profile-local +1 rejection headroom."
    )


if __name__ == "__main__":
    main()
