#!/usr/bin/env python3
"""Replay closed source/closure candidate observations against pinned Go."""
import argparse
import json
import os
import subprocess
import tempfile
from pathlib import Path
from admission_catalog import generate
from admission_closure import RECORD, PENDING, classify
from admission_vectors import CASES
from run_proof_pilot import ROOT, HERE, DEFAULT_GOROOT, json_bytes, sha, verify_host_toolchain
from run_lifetime_probes import inputs as predecessor_inputs
from check_scope_report import strict_json, require, exact

SCOPE = "private closed 56-AST/18-builtin catalog and selected closure classification from original single-file Go type facts; no production/source-model/effect/proof acceptance"
COUNTS = {"candidate_unproved": 31, "rejected_candidate": 46, "type_error": 3, "source_error": 1}
NEW_SOURCES = ("admission_go_facts.go", "admission_catalog.py", "admission_closure.py", "admission_vectors.py", "run_source_admission.py", "test_source_admission.py", "check_admission_metadata.py")


def vector_bytes():
    return json_bytes({"schema": "mpk.go_practical.source_admission_vectors.v0", "cases": CASES})


def inputs():
    rows = {r["path"]: r["sha256"] for r in predecessor_inputs()}
    for p in [HERE / n for n in NEW_SOURCES] + [RECORD / "catalog.json", RECORD / "source-vectors.json"]:
        rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(rows.items())]


def expected(c, result):
    require(result["status"] == c["expected_status"], "classification: "+c["id"])
    if "expected_code" in c:
        require(any(d["code"] == c["expected_code"] for d in result["diagnostics"]), "rejection boundary: "+c["id"])
    if "expected_closure" in c:
        require([r["name"] for r in result["closure"]] == c["expected_closure"], "declaration closure: "+c["id"])
    if "expected_typecheck_only" in c:
        require(result["typecheck_only"] == c["expected_typecheck_only"], "type-check-only bodies: "+c["id"])
    require(result["pending"] == (PENDING if result["status"] in {"candidate_unproved", "rejected_candidate"} else []), "no source/proof completion")


def native_records():
    raw = (RECORD / "native-facts.jsonl").read_bytes()
    lines = raw.splitlines(keepends=True)
    require(len(lines) == len(CASES), "one complete native JSON line per original source")
    return [(line, strict_json(line)) for line in lines]


def check_retained():
    r = strict_json((RECORD / "observations.json").read_bytes())
    require(r["scope"] == SCOPE and r["pending"] == PENDING and r["inputs"] == inputs(), "scope/input identity")
    require((RECORD / "catalog.json").read_bytes() == json_bytes(generate()) and (RECORD / "source-vectors.json").read_bytes() == vector_bytes(), "closed catalog/vector identity")
    require(r["production_admission_executed"] is False and r["ordinary_source_proofs_checked"] is False and r["application_executed"] is False, "private fact/candidate scope")
    require(r["outcomes"] == COUNTS and len(r["cases"]) == 81, "classification counts")
    counts = {k: 0 for k in COUNTS}
    for c, row, (raw, native) in zip(CASES, r["cases"], native_records()):
        result = classify(c["source"], native, c["root"]); expected(c, result)
        require(row["id"] == c["id"] and exact(row["result"], result) and row["native_stdout_sha256"] == sha(raw), "original/native/classifier linkage")
        require(row["native_exit_codes"] == [0, 0], "two native observer runs")
        counts[result["status"]] += 1
    require(counts == COUNTS and sha((RECORD / "native-facts.jsonl").read_bytes()) == r["native_facts_sha256"], "retained native identity/counts")
    return r


def main():
    parser = argparse.ArgumentParser(description=__doc__); group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true"); group.add_argument("--check", action="store_true"); args = parser.parse_args()
    if args.write:
        RECORD.mkdir(exist_ok=True)
        (RECORD / "catalog.json").write_bytes(json_bytes(generate()))
        (RECORD / "source-vectors.json").write_bytes(vector_bytes())
    before = inputs()
    inventory = verify_host_toolchain(DEFAULT_GOROOT, strict_json((HERE / "host-toolchain.json").read_bytes()))
    env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOFLAGS": "", "GOEXPERIMENT": "", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
    with tempfile.TemporaryDirectory(prefix="mpk-source-admission-", dir="/private/tmp") as temporary:
        exe = Path(temporary) / "observer"
        build = subprocess.run([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(exe), str(HERE / "admission_go_facts.go")], cwd=ROOT, env=env, capture_output=True, timeout=300)
        require(build.returncode == 0 and not build.stderr, "native observer build: "+build.stderr.decode())
        binary = sha(exe.read_bytes()); rows = []; lines = []; counts = {k: 0 for k in COUNTS}
        for c in CASES:
            payload = json_bytes({"source": c["source"], "language": c["language"]})
            runs = [subprocess.run([str(exe)], input=payload, capture_output=True, timeout=60) for _ in range(2)]
            require(all(p.returncode == 0 and not p.stderr for p in runs) and runs[0].stdout == runs[1].stdout, "deterministic native facts: "+c["id"])
            raw = runs[0].stdout; native = strict_json(raw); result = classify(c["source"], native, c["root"]); expected(c, result)
            lines.append(raw); counts[result["status"]] += 1
            rows.append({"id": c["id"], "native_exit_codes": [p.returncode for p in runs], "native_stdout_sha256": sha(raw), "result": result})
        require(counts == COUNTS and before == inputs() and binary == sha(exe.read_bytes()), "counts/source/binary stable")
        native_raw = b"".join(lines)
        report = {"schema": "mpk.go_practical.source_admission_probe.v0", "scope": SCOPE, "pending": PENDING, "inputs": before,
                  "native_binary_sha256": binary, "host_toolchain_inventory": inventory, "native_facts_sha256": sha(native_raw),
                  "cases": rows, "outcomes": counts, "direct_child_parallelism": 1, "child_ceiling_seconds": 60,
                  "production_admission_executed": False, "ordinary_source_proofs_checked": False, "application_executed": False}
        if args.write:
            (RECORD / "native-facts.jsonl").write_bytes(native_raw)
            (RECORD / "observations.json").write_bytes(json_bytes(report))
        retained = check_retained()
        require(retained["native_binary_sha256"] == binary and (RECORD / "native-facts.jsonl").read_bytes() == native_raw and exact(retained["cases"], rows), "fresh native/source/closure replay identity")
        print("PASS: 56 AST dispositions, 18 builtins; 81 original packages (31 unproved candidates, 46 rejections, 3 type errors, 1 syntax error); no application/source-model proof acceptance", flush=True)


if __name__ == "__main__": main()
