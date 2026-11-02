#!/usr/bin/env python3
"""Measure both ordinary balanced word bridges; W03 still has pending work."""

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import tempfile

from run_proof_pilot import (
    DEFAULT_GOROOT, HERE, ROOT, TARGET, checked, json_bytes, observe,
    semantic_expectation, sha, sources, timed, verify_host_toolchain,
)

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/balanced-words"
KINDS = ("unsigned", "signed")
DEPTHS = tuple(range(8))
SCOPE = "ordinary unsigned and conditional signed Nat word bridges; no mathematical prefix-sum or source-bound Go application proof"
PENDING = ["mathematical prefix-sum composition", "Go source/model bindings and guard discharge",
           "string/slice/descriptor/control law closure", "aggregate boundary and resource freeze"]


def proof_sources():
    result = {r["path"]: r["sha256"] for r in sources()}
    for p in (Path(__file__), HERE / "check_balanced_report.py"):
        result[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def verify_case(case, observations, kind):
    depth = case["depth"]
    semantic_expectation(case, observations,
                         declarations=(45 + 14 * depth if kind == "unsigned" else 55 + 22 * depth),
                         module=f"Probe.GoPractical.Word.{'' if kind == 'unsigned' else 'Signed'}TreeDepth{depth}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    detail = json.loads(observations["rust"]["report"]["error_detail"])
    go_detail = observations["go"]["report"]["error_detail"].lower()
    if detail["code"] == "CORE_TYPE_MISMATCH" and "fuel" not in go_detail:
        return "core_type_mismatch"
    if case["mutation"] == 1 and detail["code"] == "CORE_FUEL_EXHAUSTED" and "fuel" in go_detail:
        return "candidate_fuel_failure_not_false_claim_evidence"
    raise RuntimeError("unexpected proof failure; resource/encoding errors cannot count as ordinary negatives")


def expected_cases():
    return [(kind, depth, mutation) for kind in KINDS for depth in DEPTHS for mutation in range(3)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise RuntimeError("host pin and measured RSS units require darwin/arm64")
    pin = json.loads((HERE / "host-toolchain.json").read_text())
    host_inventory = verify_host_toolchain(DEFAULT_GOROOT, pin)
    cargo = shutil.which("cargo")
    if not cargo:
        raise RuntimeError("Cargo unavailable")
    checked([cargo, "build", "--release", "--offline", "--locked", "--manifest-path", str(HERE / "certificates/Cargo.toml")],
            env={**os.environ, "CARGO_TARGET_DIR": str(TARGET)})
    producer = TARGET / "release/go-practical-proof-probe"
    before = proof_sources()
    rows, groups = [], []
    with tempfile.TemporaryDirectory(prefix="mpk-go-balanced-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        reference = temp / "mpk-checker-ref"
        env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off",
               "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(reference), "./cmd/mpk-checker-ref"],
                cwd=ROOT / "go-tools/mpk-checker-ref", env=env)
        binaries = {"rust": sha(producer.read_bytes()), "go": sha(reference.read_bytes())}

        def retain_attempt(active=None):
            if args.write:
                RECORD.mkdir(parents=True, exist_ok=True)
                (RECORD / "attempt.json").write_bytes(json_bytes({
                    "schema": "mpk.go_practical.balanced_attempt.v0", "status": "incomplete measurement",
                    "inputs": before, "construction_groups": groups, "cases": rows, "active": active,
                    "checker_binaries": binaries}))
        retain_attempt()
        for kind in KINDS:
            for depth in DEPTHS:
                first, second = temp / f"first-{kind}-{depth}", temp / f"second-{kind}-{depth}"
                command = "tree-generate" if kind == "unsigned" else "signed-generate"
                one = timed([producer, command, ROOT, first, str(depth)])
                two = timed([producer, command, ROOT, second, str(depth)])
                if one["exit_code"] or two["exit_code"]:
                    raise RuntimeError("candidate construction failed; W03 remains pending")
                names = sorted(p.name for p in first.iterdir())
                if names != sorted(p.name for p in second.iterdir()) or any(
                        (first / n).read_bytes() != (second / n).read_bytes() for n in names):
                    raise RuntimeError("balanced generation is nondeterministic")
                index = json.loads((first / "index.json").read_text())
                groups.append({"kind": kind, "depth": depth, "width": 1 << depth,
                               "scope": "batch of three candidates and their shared ordinary library", "first": one, "second": two})
                for case in index["cases"]:
                    path = first / (case["id"] + ".mpcert")
                    if args.write:
                        shutil.copyfile(path, RECORD / path.name)
                    observations = {}
                    for checker, exe in (("rust", producer), ("go", reference)):
                        observations[checker] = observe(exe, path)
                        retain_attempt({"case": case, "observations": observations, "raw_sha256": sha(path.read_bytes())})
                    outcome = verify_case(case, observations, kind)
                    row = {"kind": kind, "case": case, "classification": outcome,
                           "raw_sha256": sha(path.read_bytes()), "observations": observations}
                    rows.append(row)
                    if args.check and (RECORD / path.name).read_bytes() != path.read_bytes():
                        raise RuntimeError("retained balanced certificate differs")
                    retain_attempt()
                    print(f"{case['id']}: {outcome}; Rust {observations['rust']['wall_seconds']:.3f}s, Go {observations['go']['wall_seconds']:.3f}s", flush=True)
        if before != proof_sources():
            raise RuntimeError("source changed during balanced measurement")
        report = {"schema": "mpk.go_practical.balanced_report.v0", "work_item": "GO-PRAC-T01-W03",
                  "status": "word bridges measured; W03 remains incomplete", "scope": SCOPE, "pending": PENDING,
                  "inputs": before, "host_toolchain_inventory": host_inventory, "checker_binaries": binaries,
                  "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine(),
                           "cpu_count": os.cpu_count(), "model": None, "physical_memory_bytes": None},
                  "rustc": checked([shutil.which("rustc"), "--version"]).strip(), "cargo": checked([cargo, "--version"]).strip(),
                  "build": "Rust release offline locked; Go 1.25.0 pinned host trimpath",
                  "measurement_child_ceiling_seconds": 60, "ceiling_scope": "private measurement, not a profile budget",
                  "internal_width_128_scope": "widened mathematical-domain probe; no Go 128-bit source integer admission",
                  "deterministic_generations": 2, "construction_groups": groups, "cases": rows,
                  "outcomes": {key: sum(r["classification"] == key for r in rows) for key in
                               ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}}
        if args.write:
            (RECORD / "observations.json").write_bytes(json_bytes(report))
            (RECORD / "attempt.json").unlink()
        else:
            from check_balanced_report import check
            retained = json.loads((RECORD / "observations.json").read_text())
            check(retained)
            if [r["case"] for r in retained["cases"]] != [r["case"] for r in rows]:
                raise RuntimeError("retained balanced theorem index differs")
        print(f"PASS: {len(rows)} measured candidates; signed preconditions remain explicit; W03 incomplete")


if __name__ == "__main__":
    main()
