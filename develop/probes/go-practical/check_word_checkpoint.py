#!/usr/bin/env python3
"""Validate the two actual diagnostic word runs, without claiming a full matrix.

This is evidence integrity only. The original producer/runner source is kept
beside its certificates, including the runner that stopped at a fuel result.
"""

import json
from pathlib import PurePosixPath
import re
import sys

from entry_audit import read_json, require
from run_proof_pilot import ROOT, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/word-initial"
PROBE_PREFIX = "develop/probes/go-practical/"


def check(report):
    require(report["schema"] == "mpk.go_practical.word_initial_failure.v0", "word checkpoint schema")
    require(report["scope"] == "failed preliminary dropped-final-carry measurement; not a full W03 result", "overstated checkpoint scope")
    inputs = report["inputs"]
    paths = [p["path"] for p in inputs]
    require(len(paths) == len(set(paths)), "duplicate source identity")
    required = {PROBE_PREFIX + p for p in (
        "certificates/src/main.rs", "certificates/src/natural.rs", "certificates/src/word.rs",
        "certificates/Cargo.toml", "certificates/Cargo.lock", "run_proof_pilot.py",
        "run_word_pilot.py", "check_word_report.py", "host-toolchain.json")}
    required.update(("crates/mpk-core/src/lib.rs", "crates/mpk-kernel/src/verifier.rs",
                     "go-tools/mpk-checker-ref/cmd/mpk-checker-ref/main.go",
                     "fixtures/core-bool-cases/right-identity.hex", "proofs/std/nat/std-nat.hex"))
    require(required.issubset(paths), "missing source closure pin")
    for item in inputs:
        path = PurePosixPath(item["path"])
        require(not path.is_absolute() and ".." not in path.parts, "invalid source path")
        snapshot = RECORD / "source" / item["path"].removeprefix(PROBE_PREFIX)
        source = snapshot if item["path"].startswith(PROBE_PREFIX) and snapshot.exists() else ROOT / path
        require(sha(source.read_bytes()) == item["sha256"], f"source identity differs: {path}")
    require(set(report["binaries"]) == {"rust", "go"} and all(
        re.fullmatch(r"[0-9a-f]{64}", digest) for digest in report["binaries"].values()), "missing checker binary identity")
    expected = {"word-32-nat-conservation-wrong": (32, "rejected"), "word-64-nat-conservation": (64, "accepted")}
    require(set(report["observations"]) == set(expected), "diagnostic case scope differs")
    for identifier, (width, verdict) in expected.items():
        row = report["observations"][identifier]
        item = row["case"]
        raw = (RECORD / f"width-{width}" / (identifier + ".mpcert")).read_bytes()
        require(item["id"] == identifier and item["width"] == width and item["expected"] == verdict, "word theorem scope")
        require(row["raw_sha256"] == sha(raw) and item["bytes"] == len(raw), "word certificate identity")
        require(item["operand_binders"] == width * 2 and item["initial_natural_quantified"] is True, "lost operand quantification")
        require(item["declarations"] == 39 + 6 * width, "word declaration identity")
        require(set(row["checkers"]) == {"rust", "go"}, "missing independent checker")
        for checker, observation in row["checkers"].items():
            result = observation["report"]
            require(result["verdict"] == verdict, "checker verdict differs")
            require(observation["peak_rss_bytes"] > 0 and observation["wall_seconds"] > 0, "missing resource measurement")
            require(observation["resource_provider"] == "Darwin os.wait4 child rusage", "resource units differ")
            if verdict == "accepted":
                count = result.get("axiom_count", 0 if checker == "go" else None)
                require(type(count) is int and count == 0 and observation["exit_code"] == 0, "nonzero or unknown axioms")
                require(result["module"] == f"Probe.GoPractical.Word.Width{width}" and result["declaration_count"] == item["declarations"], "checker theorem identity")
            else:
                require(observation["exit_code"] == 1, "resource candidate exit code")
                if checker == "rust":
                    require(result["error_code"] == "KERNEL_CORE_CHECK" and json.loads(result["error_detail"])["code"] == "CORE_FUEL_EXHAUSTED", "fuel result counted as semantic negative")
                else:
                    require(result["error_kind"] == "core_check" and result["error_detail"] == "core reduction fuel exhausted", "Go fuel classification differs")
        if verdict == "accepted":
            require(row["checkers"]["rust"]["report"]["hashes"] == row["checkers"]["go"]["report"]["hashes"], "checker hashes disagree")
    return 2


if __name__ == "__main__":
    try:
        check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print("PASS: 64-bit accepted by both; 32-bit dropped carry is fuel failure, not a semantic negative; W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Word checkpoint rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
