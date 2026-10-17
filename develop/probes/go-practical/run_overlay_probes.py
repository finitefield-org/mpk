#!/usr/bin/env python3
"""Replay native scope observations and private closed overlay type vectors."""
import argparse
import json
import os
import subprocess
import tempfile
from pathlib import Path
from run_proof_pilot import HERE, ROOT, DEFAULT_GOROOT, json_bytes, sha, verify_host_toolchain
from check_scope_report import strict_json
from overlay_contracts import normalize, OverlayError, OPERATORS
from overlay_vectors import vectors, capture_id

RECORD = ROOT / "develop/migrations/go-practical/t01-w04/overlay-contracts"
SCOPE = "private source-overlay grammar/type/lexical-scope design probe; native original-file type checking; no practical source admission, independent normalized artifact validator, discharged VC or Go totality claim"
FILES = ("overlay_go_scopes.go", "overlay-fixture.go.txt", "overlay_contracts.py", "overlay_vectors.py", "run_overlay_probes.py", "test_overlay_contracts.py")


def inputs():
    names = FILES + ("run_proof_pilot.py", "check_scope_report.py", "host-toolchain.json")
    return [{"path": (HERE / n).relative_to(ROOT).as_posix(), "sha256": sha((HERE / n).read_bytes())} for n in names]


def outcomes(facts):
    rows = []
    for c in vectors(facts):
        identity = capture_id(c, facts["source_sha256"])
        try:
            normalized = normalize(c["overlay"], facts, identity, c["universe"])
            observed = "normalized_pending_variant" if normalized["pending_candidates"] else "normalized_unproved"
            error = ""
        except OverlayError as exc:
            observed, normalized, error = "configuration_rejection", {}, str(exc)
        if c["expected"] != observed or (c["error_contains"] and c["error_contains"] not in error):
            raise RuntimeError(f"{c['id']}: expected {c['expected']}/{c['error_contains']}, observed {observed}/{error}")
        rows.append({"id": c["id"], "outcome": observed, "capture_context_id": identity, "normalized": normalized, "configuration_error": error})
    return rows


def retained_check():
    report = strict_json((RECORD / "observations.json").read_bytes())
    if report["scope"] != SCOPE or report["inputs"] != inputs() or report["production_admission_executed"] is not False or report["proofs_discharged"] is not False:
        raise RuntimeError("overlay scope/source identity")
    facts = strict_json((RECORD / "native-facts.json").read_bytes())
    if sha((HERE / "overlay-fixture.go.txt").read_bytes()) != facts["source_sha256"]:
        raise RuntimeError("overlay original source identity")
    artifacts = {name: sha((RECORD / name).read_bytes()) for name in ("native-facts.json", "vectors.json", "normalized-candidates.json", "operators.json")}
    if artifacts != report["artifacts"] or report["outcomes"] != outcomes(facts):
        raise RuntimeError("overlay retained artifact/normalization identity")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true"); mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    before = inputs()
    inventory = verify_host_toolchain(DEFAULT_GOROOT, json.loads((HERE / "host-toolchain.json").read_text()))
    request = {"path": "policy.go", "source": (HERE / "overlay-fixture.go.txt").read_bytes().decode("utf-8"), "language": "go1.25", "functions": ["Quote", "Scopes", "InputErrors", "Error"]}
    with tempfile.TemporaryDirectory(prefix="mpk-go-overlay-", dir="/private/tmp") as temporary:
        binary = Path(temporary) / "observer"
        env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOFLAGS": "", "GOEXPERIMENT": "", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        build = subprocess.run([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(binary), str(HERE / "overlay_go_scopes.go")], env=env, capture_output=True, timeout=300)
        if build.returncode or build.stdout or build.stderr:
            raise RuntimeError("overlay native build: " + build.stderr.decode())
        digest = sha(binary.read_bytes())
        runs = [subprocess.run([str(binary)], input=json_bytes(request), capture_output=True, timeout=60) for _ in range(2)]
        if any(r.returncode or r.stderr for r in runs) or runs[0].stdout != runs[1].stdout:
            raise RuntimeError("overlay native executions differ")
        facts = strict_json(runs[0].stdout)
        rows = outcomes(facts)
        artifacts = {"native-facts.json": json_bytes(facts), "vectors.json": json_bytes({"cases": vectors(facts)}),
                     "normalized-candidates.json": json_bytes({"cases": rows}),
                     "operators.json": json_bytes({"schema": "mpk.go_practical.overlay_operator_inventory_draft.v0", "source_operators": sorted(OPERATORS),
                                                   "normalized_operators": sorted((OPERATORS - {"local"}) | {"binding"}), "scope": "closed private draft expression heads; not registered VIR tags"})}
        if before != inputs() or sha(binary.read_bytes()) != digest:
            raise RuntimeError("overlay source/observer changed")
        report = {"schema": "mpk.go_practical.w04_overlay_probe.v0", "scope": SCOPE, "inputs": before,
                  "observer_binary_sha256": digest, "host_toolchain_inventory": inventory, "native_executions": 2,
                  "artifacts": {n: sha(raw) for n, raw in artifacts.items()}, "outcomes": rows,
                  "production_admission_executed": False, "proofs_discharged": False,
                  "pending": ["exact production TypeID/SourceObjectID/FunctionDeclID/binding hashes, successor schemas and independent artifact validation",
                              "definedness/domain/source/frame/postcondition/variant and all called-component termination proofs",
                              "remaining W04 limits/migration/foundation freeze; W05 full T01 gate"]}
        if args.write:
            RECORD.mkdir(exist_ok=True)
            for n, raw in artifacts.items(): (RECORD / n).write_bytes(raw)
            (RECORD / "observations.json").write_bytes(json_bytes(report))
        retained = retained_check()
        if retained["observer_binary_sha256"] != digest or any((RECORD / n).read_bytes() != raw for n, raw in artifacts.items()):
            raise RuntimeError("overlay fresh-build replay differs")
        counts = {k: sum(r["outcome"] == k for r in rows) for k in ("normalized_unproved", "normalized_pending_variant", "configuration_rejection")}
        print(f"PASS: native original-file scopes/types twice; {len(OPERATORS)} closed draft expression heads; {len(rows)} vectors {counts}; no source/proof completion")


if __name__ == "__main__": main()
