#!/usr/bin/env python3
"""Private W09 feasibility test; not a capacity, implementation or release gate."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]
RECORD = ROOT / "develop/migrations/csharp-03/probes/recursor-feasibility.json"
CAPACITY_RECORD = ROOT / "develop/migrations/csharp-03/probes/checker-capacity.json"
TEST = "recursor_feasibility::csharp_03_t01_w09_recursor_probe_bytes_are_reproducible"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode()


def execute(command, *, cwd=ROOT, env=None):
    # A timeout/signal/build error aborts the measurement; it is not a semantic rejection.
    return subprocess.run(command, cwd=cwd, env=env, capture_output=True, timeout=60)


def checked(command, **kwargs):
    result = execute(command, **kwargs)
    if result.returncode:
        raise RuntimeError(f"command failed: {command!r}\n{result.stdout.decode()}\n{result.stderr.decode()}")
    return result.stdout.decode()


def source_inventory():
    paths = checked(["rg", "--files", "crates/mpk-core", "crates/mpk-kernel", "crates/mpk-cert", "crates/mpk-cli", "go-tools/mpk-checker-ref"]).splitlines()
    paths = [path for path in paths if (("/src/" in path and path.endswith(".rs")) or
             (path.startswith("go-tools/") and path.endswith(".go") and not path.endswith("_test.go")) or path.endswith(("Cargo.toml", "go.mod")))]
    paths += ["Cargo.toml", "Cargo.lock", "develop/probes/csharp-03/recursor_feasibility.rs", "develop/probes/csharp-03/run-recursor-probe.py"]
    return [{"path": path, "raw_sha256": digest((ROOT / path).read_bytes())} for path in sorted(paths)]


def observe(command, checker):
    result = execute(command)
    stdout, stderr = result.stdout.decode(), result.stderr.decode()
    if checker == "reference":
        value = json.loads(stdout)
        if result.returncode == 0 and value["verdict"] == "accepted":
            verdict = "accepted"
        elif (result.returncode == 1 and value["verdict"] == "rejected" and value.get("error_kind") == "core_check"
              and re.fullmatch(r"term [0-9]+ inferred (pi|const) but expected const", value.get("error_detail", ""))):
            verdict = "type_mismatch"
        else:
            raise RuntimeError(f"unexpected reference result: {result.returncode}: {stdout} {stderr}")
    elif result.returncode == 0 and stdout.startswith("ok module=") and " axioms=0" in stdout:
        verdict = "accepted"
    elif result.returncode == 1 and "type_mismatch" in stderr:
        verdict = "type_mismatch"
    else:
        raise RuntimeError(f"unexpected Rust result: {result.returncode}: {stdout} {stderr}")
    return {"result": verdict, "exit_code": result.returncode, "stdout": stdout, "stderr": stderr}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--update", action="store_true")
    group.add_argument("--check", action="store_true")
    args = parser.parse_args()
    checked(["cargo", "build", "-p", "mpk-cli"])
    with tempfile.TemporaryDirectory(prefix="mpk-w09-recursor-") as directory:
        temporary = Path(directory)
        exported = temporary / "probes.json"
        env = {**os.environ, "MPK_W09_RECURSOR_EXPORT": str(exported)}
        checked(["cargo", "test", "-p", "mpk-vc", "--test", "csharp_practical_spec", TEST, "--", "--exact"], env=env)
        probe = json.loads(exported.read_bytes())
        reference = temporary / "mpk-checker-ref"
        checked(["go", "build", "-trimpath", "-o", str(reference), "./cmd/mpk-checker-ref"], cwd=ROOT / "go-tools/mpk-checker-ref", env={**os.environ, "GOCACHE": str(temporary / "go-cache")})
        runs = []
        for repetition in [1, 2]:
            observations = []
            for case in probe["cases"]:
                path = temporary / (case["id"] + ".mpcert")
                data = bytes.fromhex(case["certificate_hex"])
                assert digest(data) == case["raw_sha256"]
                path.write_bytes(data)
                rust = observe([str(ROOT / "target/debug/mpk"), "verify", str(path)], "rust")
                other = observe([str(reference), "verify", str(path)], "reference")
                assert rust["result"] == other["result"] == case["expected"], case["id"]
                observations.append({"id": case["id"], "rust": rust, "reference": other})
            runs.append({"repetition": repetition, "observations": observations})
        inventory = source_inventory()
        result = {"schema": "mpk.csharp_practical.recursor_feasibility.v1", "work_item": "CSHARP-03-T01-W09",
                  "status": "replacement_type_feasible", "probe": probe, "runs": runs,
                  "host": {"system": platform.system(), "machine": platform.machine()},
                  "toolchain": {"rustc": checked(["rustc", "--version"]).strip(), "go": checked(["go", "version"]).strip()},
                  "source_inventory": inventory, "source_inventory_sha256": digest(canonical(inventory)),
                  "claim": "The retained cross-result applications still reject, while the Bool-addressed pointwise carrier and static cross-coordinate concrete-transformer fold typecheck unchanged in both checkers without using Nat.rec in the replacement.",
                  "capacity_measurement": {"path": str(CAPACITY_RECORD.relative_to(ROOT)),
                                           "raw_sha256": digest(CAPACITY_RECORD.read_bytes())},
                  "release_gate": False}
        if args.update:
            RECORD.write_bytes(canonical(result))
        else:
            retained = json.loads(RECORD.read_bytes())
            # Host/compiler versions are observations, not the semantic verdict authority.
            for field in ["probe", "runs", "source_inventory", "source_inventory_sha256", "status",
                          "claim", "capacity_measurement", "release_gate"]:
                assert retained[field] == result[field], f"drift: {field}"
        print("W09: 15 cases x 2 checkers x 2 runs; 10 controls/replacement cases accept and 5 retained cross-result applications reject per run. Type feasibility and the separately linked capacity measurement are reproducible.")


if __name__ == "__main__":
    main()
