#!/usr/bin/env python3
"""Check ordinary decreasing-rank termination with unchanged private kernels."""
import argparse
import json
import os
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

import variant_foundation_build as build
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, json_bytes, sha, observe, timed, verify_host_toolchain, semantic_expectation

RECORD = ROOT / "develop/migrations/go-practical/t01-w04/variant-termination"
TARGET = Path("/private/tmp/mpk-go-practical-variant-target")
MODULE = "Probe.GoPractical.VariantTermination"
SCOPE = "ordinary arbitrary-Nat decreasing-rank finite-stop proof on 128-bit frames, checked by unchanged private Bool.recType/Nat.induct kernels; explicit initial invariant and continuing-backedge preservation/decrease obligations; no production profile or Go source totality claim"
STATUS = "private mathematical termination feasibility checked; W04 remains in progress"
PENDING = ["W04 exact joint core/foundation/overlay/wire/hash/limits/migration freeze and review",
           "source-model binding and totality of loop condition/body/post and called functions; word/range/application composition in T02–T06",
           "W05 cumulative T01 gate; T07 installed activation"]
MUTATIONS = ("valid", "preservation", "decrease", "entry", "bound", "weak-bound", "result", "rank-count-valid", "rank-count-off-by-one", "zero-rank-exit")
REGRESSIONS = (
    ("general-sort0-strong", "develop/migrations/go-practical/t01-w04/nat-termination/strong-induction-valid.mpcert"),
    ("retained-nat-invariant", "develop/migrations/go-practical/t01-w03/control-induction/control-invariant-entry-valid.mpcert"),
    ("legacy-bool", "fixtures/core-bool-cases/right-identity.hex"),
)


def inputs():
    rows = {r["path"]: r["sha256"] for r in build.sources()}
    for name in ("run_variant_probes.py", "check_variant_report.py", "test_variant_report.py",
                 "run_proof_pilot.py", "run_module_probes.py", "check_scope_report.py", "host-toolchain.json"):
        p = HERE / name
        rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    for _, name in REGRESSIONS:
        rows[name] = sha((ROOT / name).read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(rows.items())]


def classify(case, observations):
    if set(observations) != {"rust", "go"}:
        raise ValueError("variant independent checkers")
    if case["expected"] == "accepted":
        semantic_expectation(case, observations, declarations=case["declarations"], module=MODULE)
        return "accepted_zero_axiom"
    r, g = observations["rust"], observations["go"]
    if (r["exit_code"] != 1 or g["exit_code"] != 1
            or r["report"].get("error_code") != "KERNEL_CORE_CHECK"
            or g["report"].get("error_kind") != "core_check"):
        raise ValueError("variant rejection boundary")
    if json.loads(r["report"]["error_detail"])["code"] != "CORE_TYPE_MISMATCH":
        raise ValueError("variant fuel/other failure is not intended rejection")
    if not g["report"]["error_detail"].startswith("term ") or " inferred " not in g["report"]["error_detail"]:
        raise ValueError("variant Go type rejection")
    return "core_type_mismatch"


def run(command, *, env=None, cwd=ROOT):
    p = subprocess.run(list(map(str, command)), cwd=cwd, env=env, capture_output=True, timeout=300)
    if p.returncode:
        raise RuntimeError(p.stdout.decode() + p.stderr.decode())
    return p.stdout, p.stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true")
    group.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ("Darwin", "arm64"):
        raise RuntimeError("private resource/toolchain receipt requires Darwin arm64")
    before = inputs()
    inventory = verify_host_toolchain(DEFAULT_GOROOT, json.loads((HERE / "host-toolchain.json").read_text()))
    predecessor = json.loads((RECORD.parent / "nat-termination/observations.json").read_text())
    with tempfile.TemporaryDirectory(prefix="mpk-variant-termination-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        workspace = temp / "workspace"
        receipt = build.prepare(workspace)
        if receipt["generated"] != predecessor["mirrored_checker_inputs"]:
            raise RuntimeError("variant checker sources must remain byte-identical")
        env = {**os.environ, "CARGO_TARGET_DIR": str(TARGET)}
        _, err = run(["cargo", "build", "--release", "--offline", "--locked", "--manifest-path", workspace / "Cargo.toml", "-p", "go-practical-variant-probe"], env=env)
        if b"warning:" in err:
            raise RuntimeError("private release compiler warning")
        rust, go = TARGET / "release/go-practical-variant-probe", temp / "checker"
        go_env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off",
                  "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOFLAGS": "", "GOEXPERIMENT": "",
                  "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        run([DEFAULT_GOROOT / "bin/go", "build", "-trimpath", "-buildvcs=false", "-o", go, "./cmd/mpk-checker-ref"],
            cwd=workspace / "go-tools/mpk-checker-ref", env=go_env)
        binaries = {"rust": sha(rust.read_bytes()), "go": sha(go.read_bytes())}
        if binaries["go"] != predecessor["checker_binaries"]["go"]:
            raise RuntimeError("variant unchanged Go checker identity")
        first, second = temp / "first", temp / "second"
        generations = [timed([rust, "variant-generate", ROOT, p]) for p in (first, second)]
        names = sorted(p.name for p in first.iterdir())
        if any(r["exit_code"] for r in generations) or names != sorted(p.name for p in second.iterdir()) or any((first / n).read_bytes() != (second / n).read_bytes() for n in names):
            raise RuntimeError("variant generation differs")
        rows = []
        for case in json.loads((first / "index.json").read_text())["cases"]:
            path = first / (case["id"] + ".mpcert")
            observations = {k: observe(exe, path) for k, exe in (("rust", rust), ("go", go))}
            classification = classify(case, observations)
            rows.append({"case": case, "raw_sha256": sha(path.read_bytes()), "classification": classification, "observations": observations})
            print(case["id"] + ": " + classification, flush=True)
        regressions = []
        for identity, name in REGRESSIONS:
            source = ROOT / name
            raw = bytes.fromhex(source.read_text()) if source.suffix == ".hex" else source.read_bytes()
            path = temp / (identity + ".mpcert")
            path.write_bytes(raw)
            observations = {k: observe(exe, path) for k, exe in (("rust", rust), ("go", go))}
            r = observations["rust"]["report"]
            semantic_expectation({"expected": "accepted"}, observations, declarations=r["declaration_count"], module=r["module"])
            regressions.append({"id": identity, "path": name, "raw_sha256": sha(raw), "observations": observations})
        if before != inputs() or binaries != {"rust": sha(rust.read_bytes()), "go": sha(go.read_bytes())}:
            raise RuntimeError("variant source/binary changed")
        report = {"schema": "mpk.go_practical.variant_termination_report.v0", "scope": SCOPE, "status": STATUS, "pending": PENDING,
                  "inputs": before, "mirrored_checker_inputs": receipt["generated"], "checker_binaries": binaries,
                  "unchanged_checker_proposal": "develop/migrations/go-practical/t01-w04/nat-termination/observations.json",
                  "host_toolchain_inventory": inventory, "generations": generations, "direct_child_parallelism": 1,
                  "child_ceiling_seconds": 60, "ceiling_scope": "private measurement setting, not normative profile limits",
                  "cases": rows, "regressions": regressions, "outcomes": {"ordinary_accepted": 3, "ordinary_type_mismatch": 7, "legacy_regressions": 3}}
        if args.write:
            RECORD.mkdir(exist_ok=True)
            for name in names:
                shutil.copyfile(first / name, RECORD / name)
            (RECORD / "observations.json").write_bytes(json_bytes(report))
        from check_variant_report import check
        retained = json.loads((RECORD / "observations.json").read_text())
        check(retained)
        if retained["checker_binaries"] != binaries or any((RECORD / n).read_bytes() != (first / n).read_bytes() for n in names):
            raise RuntimeError("variant fresh-build replay identity")
        print("PASS: 10 ordinary variant claims and 3 retained regressions; both unchanged private checkers; no Go source totality claim", flush=True)


if __name__ == "__main__":
    main()
