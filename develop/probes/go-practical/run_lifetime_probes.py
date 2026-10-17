#!/usr/bin/env python3
"""Replay derived lifetime roots, native values and ordinary reuse/past laws."""
import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from run_proof_pilot import ROOT, HERE, DEFAULT_GOROOT, json_bytes, sha, sources, observe, timed, semantic_expectation, verify_host_toolchain
from check_scope_report import strict_json, exact
from lifetime_cases import model, EXPECTED
from run_storage_view_probes import inputs as predecessor_inputs

RECORD = ROOT / "develop/migrations/go-practical/t01-w04/storage-lifetimes"
TARGET = Path("/private/tmp/mpk-go-practical-lifetime-target")
MODULE = "Probe.GoPractical.LiveStorage"
SCOPE = "private complete modeled-root closure/lifetime/slot-reuse design and 12 native/model values; 2 ordinary masked-frame/past-store laws in unchanged production checkers; no general source/closure/allocator/lifetime binding or profile proof"
CASE_IDS = ("dead-slot-guard-valid", "dead-slot-guard-altered", "masked-replacement-frame-altered", "past-store-namespace-valid", "past-store-namespace-altered")
PENDING = ["ordinary completeness/validity of typed root enumeration, escape rewrite and reusable-slot selection; exact source scope/evaluation/call binding",
           "W04 complete source/limits/schemas/hash/migration freeze and independent import",
           "W05 full T01 gate; complete T02–T11 source/application/release scope"]


def inputs():
    rows = {r["path"]: r["sha256"] for r in predecessor_inputs()}
    for p in list((HERE / "lifetime-certificates").rglob("*")) + [HERE / n for n in ("lifetime_reference.go", "storage_lifetimes.py", "lifetime_cases.py", "test_storage_lifetimes.py", "run_lifetime_probes.py", "check_lifetime_metadata.py")]:
        if p.is_file(): rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(rows.items())]


def classify(case, observed):
    if set(observed) != {"rust", "go"}: raise ValueError("two production checkers required")
    if case["expected"] == "accepted":
        semantic_expectation(case, observed, declarations=case["declarations"], module=MODULE)
        return "accepted_zero_axiom"
    r, g = observed["rust"], observed["go"]
    if r["exit_code"] != 1 or g["exit_code"] != 1 or r["report"].get("error_code") != "KERNEL_CORE_CHECK" or g["report"].get("error_kind") != "core_check":
        raise ValueError("core rejection boundary")
    if json.loads(r["report"]["error_detail"])["code"] != "CORE_TYPE_MISMATCH": raise ValueError("fuel/resource is not false-claim evidence")
    if not g["report"]["error_detail"].startswith("term ") or " inferred " not in g["report"]["error_detail"]: raise ValueError("Go type mismatch boundary")
    return "core_type_mismatch"


def check_retained():
    r = strict_json((RECORD / "observations.json").read_bytes())
    if r["scope"] != SCOPE or r["pending"] != PENDING or r["inputs"] != inputs(): raise ValueError("lifetime scope/source identity")
    if r["production_checker_changed"] is not False or r["complete_source_normalization_proved"] is not False: raise ValueError("no source/profile completion")
    if not exact(strict_json((RECORD / "native-output.json").read_bytes()), EXPECTED) or not exact(model()[0], EXPECTED): raise ValueError("12 native/model/independent observations")
    index = strict_json((RECORD / "index.json").read_bytes())["cases"]
    if [c["id"] for c in index] != list(CASE_IDS) or [row["case"] for row in r["ordinary_cases"]] != index: raise ValueError("ordinary lifetime coverage")
    positives = {"dead-slot-guard-valid", "past-store-namespace-valid"}
    for row in r["ordinary_cases"]:
        c = row["case"]; raw = (RECORD / (c["id"]+".mpcert")).read_bytes()
        if sha(raw) != row["raw_sha256"] or len(raw) != c["bytes"] or c["value_enumeration"] is not False or c["address_enumeration"] is not False: raise ValueError("canonical certificate/value scope")
        if c["expected"] != ("accepted" if c["id"] in positives else "rejected"): raise ValueError("positive/altered claim boundary")
        if row["classification"] != classify(c, row["observations"]): raise ValueError("lifetime classification")
    accepted = [row["raw_sha256"] for row in r["ordinary_cases"] if row["classification"] == "accepted_zero_axiom"]
    if len(accepted) != 2 or len(set(accepted)) != 2: raise ValueError("distinct positive statements")
    native_runs = r["native_runs"]
    if len(native_runs) != 2 or any(x["exit_code"] or not exact(strict_json(x["stdout"]), EXPECTED) for x in native_runs): raise ValueError("retained native runs")
    if r["outcomes"] != {"ordinary_accepted": 2, "ordinary_type_mismatch": 3, "native_observations": 12, "legacy_regressions": 2}: raise ValueError("lifetime outcome counts")
    if [row["id"] for row in r["legacy"]] != ["legacy-bool", "legacy-nat"]: raise ValueError("legacy coverage")
    for row in r["legacy"]:
        raw = bytes.fromhex((ROOT / row["path"]).read_text())
        if sha(raw) != row["raw_sha256"]: raise ValueError("legacy bytes")
        o = row["observations"]["rust"]["report"]
        semantic_expectation({"expected": "accepted"}, row["observations"], declarations=o["declaration_count"], module=o["module"])
    expected_metrics={"overwrite_reused_slot":True,"iteration_peak_positive_slots":1,"iteration_allocations":1000,"iteration_reused_one_slot":True,"zero_access_retired_storage":True,"captured_target_release_retires_old":True}
    if not exact(r["model_metrics"],expected_metrics) or not exact(model()[1],expected_metrics): raise ValueError("lifetime model-only slot metrics")
    return r


def run(cmd, *, cwd=ROOT, env=None):
    p = subprocess.run(list(map(str, cmd)), cwd=cwd, env=env, capture_output=True, timeout=300)
    if p.returncode or b"warning:" in p.stderr: raise RuntimeError(p.stdout.decode()+p.stderr.decode())
    return p


def main():
    parser = argparse.ArgumentParser(description=__doc__); g = parser.add_mutually_exclusive_group(required=True)
    g.add_argument("--write", action="store_true"); g.add_argument("--check", action="store_true"); args = parser.parse_args()
    before = inputs(); inventory = verify_host_toolchain(DEFAULT_GOROOT, json.loads((HERE / "host-toolchain.json").read_text()))
    run(["cargo", "build", "--release", "--offline", "--locked", "--manifest-path", HERE / "lifetime-certificates/Cargo.toml"], env={**os.environ, "CARGO_TARGET_DIR": str(TARGET)})
    rust = TARGET / "release/go-practical-lifetime-probe"
    with tempfile.TemporaryDirectory(prefix="mpk-lifetimes-", dir="/private/tmp") as temporary:
        temp = Path(temporary); go, native = temp / "checker", temp / "native"
        env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOFLAGS": "", "GOEXPERIMENT": "", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        run([DEFAULT_GOROOT / "bin/go", "build", "-trimpath", "-buildvcs=false", "-o", go, "./cmd/mpk-checker-ref"], cwd=ROOT / "go-tools/mpk-checker-ref", env=env)
        run([DEFAULT_GOROOT / "bin/go", "build", "-trimpath", "-buildvcs=false", "-o", native, HERE / "lifetime_reference.go"], env=env)
        binaries = {"rust": sha(rust.read_bytes()), "go": sha(go.read_bytes()), "native": sha(native.read_bytes())}
        native_runs = [timed([native]) for _ in range(2)]
        if any(x["exit_code"] for x in native_runs) or native_runs[0]["stdout"] != native_runs[1]["stdout"] or not exact(strict_json(native_runs[0]["stdout"]), EXPECTED) or not exact(model()[0], EXPECTED): raise RuntimeError("native/model lifetime observations")
        first, second = temp / "first", temp / "second"
        generations = [timed([rust, "lifetime-generate", ROOT, p]) for p in (first, second)]
        names = sorted(p.name for p in first.iterdir())
        if any(x["exit_code"] for x in generations) or names != sorted(p.name for p in second.iterdir()) or any((first / n).read_bytes() != (second / n).read_bytes() for n in names): raise RuntimeError("lifetime deterministic generation")
        ordinary = []
        for c in strict_json((first / "index.json").read_bytes())["cases"]:
            p = first / (c["id"]+".mpcert"); observed = {k: observe(exe, p) for k, exe in (("rust", rust), ("go", go))}
            kind = classify(c, observed)
            ordinary.append({"case": c, "raw_sha256": sha(p.read_bytes()), "observations": observed, "classification": kind})
            print(c["id"]+": "+kind, flush=True)
        legacy = []
        for identity, name in (("legacy-bool", "fixtures/core-bool-cases/right-identity.hex"), ("legacy-nat", "proofs/std/nat/std-nat.hex")):
            raw = bytes.fromhex((ROOT / name).read_text()); p = temp / (identity+".mpcert"); p.write_bytes(raw)
            observed = {k: observe(exe, p) for k, exe in (("rust", rust), ("go", go))}; o = observed["rust"]["report"]
            semantic_expectation({"expected": "accepted"}, observed, declarations=o["declaration_count"], module=o["module"])
            legacy.append({"id": identity, "path": name, "raw_sha256": sha(raw), "observations": observed})
        if before != inputs() or binaries != {"rust": sha(rust.read_bytes()), "go": sha(go.read_bytes()), "native": sha(native.read_bytes())}: raise RuntimeError("source/binary changed")
        report = {"schema": "mpk.go_practical.lifetime_probe.v0", "scope": SCOPE, "pending": PENDING, "inputs": before,
                  "checker_binaries": {k: binaries[k] for k in ("rust", "go")}, "native_binary_sha256": binaries["native"], "host_toolchain_inventory": inventory,
                  "native_runs": native_runs, "generations": generations, "ordinary_cases": ordinary, "legacy": legacy,
                  "direct_child_parallelism": 1, "child_ceiling_seconds": 60, "ceiling_scope": "private measurement setting, not normative profile limits",
                  "production_checker_changed": False, "complete_source_normalization_proved": False,
                  "model_metrics": model()[1], "outcomes": {"ordinary_accepted": 2, "ordinary_type_mismatch": 3, "native_observations": 12, "legacy_regressions": 2}}
        if args.write:
            RECORD.mkdir(exist_ok=True)
            for n in names: shutil.copyfile(first / n, RECORD / n)
            (RECORD / "native-output.json").write_bytes(json_bytes(EXPECTED))
            (RECORD / "observations.json").write_bytes(json_bytes(report))
        old = check_retained()
        if old["checker_binaries"] != report["checker_binaries"] or old["native_binary_sha256"] != binaries["native"] or any((RECORD / n).read_bytes() != (first / n).read_bytes() for n in names): raise RuntimeError("fresh replay identity")
        print("PASS: 12 native/model observations, 5 ordinary claims in unchanged production checkers, 2 legacy certificates; no source-normalization completion", flush=True)


if __name__ == "__main__": main()
