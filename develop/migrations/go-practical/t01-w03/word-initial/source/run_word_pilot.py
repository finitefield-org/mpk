#!/usr/bin/env python3
"""Measure the compositional unsigned Nat word bridge with both checkers.

The 600-second per-child measurement ceiling is private probe infrastructure,
not a frozen profile budget. A timeout/resource failure leaves W03 incomplete.
The report never claims signed interpretation, prefix sums or Go bindings.
"""

import argparse
import json
import os
import platform
import shutil
import tempfile
from pathlib import Path

from run_proof_pilot import (
    DEFAULT_GOROOT, HERE, ROOT, TARGET, checked, json_bytes, observe,
    semantic_expectation, sha, sources, timed, verify_host_toolchain,
)

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/word-conservation"
WIDTHS = (1, 2, 4, 8, 16, 32, 64)
SCOPE = "exact unsigned word addition and final-carry value conservation in Nat; no signed interpretation, prefix-sum or Go source-binding proof"
PENDING = ["signed two's-complement and no-overflow bridge", "mathematical prefix-sum composition",
           "complete source/model bindings and operation laws", "aggregate resource and boundary freeze"]


def word_sources():
    result = {item["path"]: item["sha256"] for item in sources()}
    for path in (Path(__file__), HERE / "check_word_report.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": path, "sha256": value} for path, value in sorted(result.items())]


def verify_case(item, observations):
    width = item["width"]
    semantic_expectation(item, observations, declarations=39 + 6 * width,
                         module=f"Probe.GoPractical.Word.Width{width}")
    if item["expected"] == "rejected":
        detail = json.loads(observations["rust"]["report"]["error_detail"])
        if detail["code"] != "CORE_TYPE_MISMATCH" or "fuel" in observations["go"]["report"]["error_detail"].lower():
            raise RuntimeError("a resource/encoding failure is not the dropped-carry counterexample")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise RuntimeError("resource units and host compiler pin require darwin/arm64")
    pin = json.loads((HERE / "host-toolchain.json").read_text())
    inventory = verify_host_toolchain(DEFAULT_GOROOT, pin)
    cargo = shutil.which("cargo")
    if not cargo:
        raise RuntimeError("Cargo unavailable")
    checked([cargo, "build", "--release", "--offline", "--locked", "--manifest-path", str(HERE / "certificates/Cargo.toml")],
            env={**os.environ, "CARGO_TARGET_DIR": str(TARGET)})
    rust = TARGET / "release/go-practical-proof-probe"
    before = word_sources()
    groups, rows = [], []
    with tempfile.TemporaryDirectory(prefix="mpk-go-word-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        reference = temp / "mpk-checker-ref"
        env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local",
               "GOPROXY": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0",
               "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-o", str(reference), "./cmd/mpk-checker-ref"],
                cwd=ROOT / "go-tools/mpk-checker-ref", env=env)
        for width in WIDTHS:
            first, second = temp / f"first-{width}", temp / f"second-{width}"
            command = [rust, "word-generate", ROOT, first, str(width)]
            construction = timed(command, limit_seconds=600)
            if construction["exit_code"]:
                raise RuntimeError(f"width {width} construction failure remains pending: {construction}")
            replay = timed([rust, "word-generate", ROOT, second, str(width)], limit_seconds=600)
            if replay["exit_code"]:
                raise RuntimeError(f"width {width} replay construction failed")
            names = sorted(p.name for p in first.iterdir())
            if names != sorted(p.name for p in second.iterdir()) or any(
                    (first / name).read_bytes() != (second / name).read_bytes() for name in names):
                raise RuntimeError("word certificate construction is nondeterministic")
            index = json.loads((first / "index.json").read_text())
            group = {"width": width, "scope": "batch construction of the positive and dropped-carry candidates, including shared lemmas",
                     "first": construction, "second": replay}
            groups.append(group)
            print(f"width {width}: constructed twice; {construction['wall_seconds']:.3f}s", flush=True)
            for item in index["cases"]:
                path = first / (item["id"] + ".mpcert")
                observations = {}
                for checker, executable in (("rust", rust), ("go", reference)):
                    print(f"{item['id']}: checking {checker}", flush=True)
                    observations[checker] = observe(executable, path, limit_seconds=600)
                verify_case(item, observations)
                row = {"case": item, "construction_group": width,
                       "raw_sha256": sha(path.read_bytes()), "observations": observations}
                rows.append(row)
                if args.write:
                    RECORD.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(path, RECORD / path.name)
                elif (RECORD / path.name).read_bytes() != path.read_bytes():
                    raise RuntimeError("retained word certificate differs")
                print(f"{item['id']}: {item['expected']} by both", flush=True)
        if before != word_sources():
            raise RuntimeError("word proof inputs changed during measurement")
        report = {"schema": "mpk.go_practical.word_report.v0", "work_item": "GO-PRAC-T01-W03",
                  "status": "unsigned word pilot measured; W03 remains incomplete", "scope": SCOPE,
                  "pending": PENDING, "inputs": before, "host_toolchain_inventory": inventory,
                  "host": {"system": platform.system(), "machine": platform.machine(), "release": platform.release(),
                           "cpu_count": os.cpu_count(), "model": None, "physical_memory_bytes": None},
                  "build": {"rust": "release optimized, offline locked private crate", "go": "Go 1.25.0 pinned host, trimpath, no downloads"},
                  "rustc": checked([shutil.which("rustc"), "--version"]).strip(),
                  "cargo": checked([cargo, "--version"]).strip(), "measurement_child_ceiling_seconds": 600,
                  "ceiling_scope": "private measurement infrastructure, not a frozen Go profile budget",
                  "checker_binaries": {"rust_sha256": sha(rust.read_bytes()), "go_sha256": sha(reference.read_bytes())},
                  "deterministic_generations": 2, "construction_groups": groups, "cases": rows}
        if args.write:
            (RECORD / "observations.json").write_bytes(json_bytes(report))
        else:
            from check_word_report import check
            retained = json.loads((RECORD / "observations.json").read_text())
            check(retained)
            if [r["case"] for r in retained["cases"]] != [r["case"] for r in rows]:
                raise RuntimeError("retained word theorem index differs")
        print(f"PASS: {len(rows)} compositional unsigned candidates; signed/sum/source gates remain open")


if __name__ == "__main__":
    main()
