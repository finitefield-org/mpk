#!/usr/bin/env python3
"""Measure ordinary full-word prefix folds; source-bound Go proofs remain open."""

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import sys
import tempfile

from run_proof_pilot import (
    DEFAULT_GOROOT, HERE, ROOT, TARGET, checked, json_bytes, observe,
    semantic_expectation, sha, sources, timed, verify_host_toolchain,
)

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/prefix-sums"
COUNTS = (0, 1, 3, 8, 32, 127, 128, 129)
FAMILIES = ("conditional", "unconditional")
SCOPE = "ordinary modular64 prefix folds with exact Nat sums, actual carry premises and unconditional carry corrections; no signed or source-bound Go application proof"
PENDING = ["signed/nonnegative prefix interpretation and Go guard discharge", "complete Go source/model and operation bindings",
           "string/slice/descriptor/control law closure", "aggregate boundary and resource freeze"]
BOUNDARY_SCOPE = "129 elements is an internal boundary probe, not Go profile admission; padding remains universally quantified"
CONDITIONAL_SCOPE = "conditional exact fixed-count modular64 prefix fold to Nat sum; signed/nonnegative and Go guard bindings remain pending"
RAW_SCOPE = "unconditional exact fixed-count modular64 prefix fold plus actual step-carry correction to Nat sum; signed/nonnegative and Go bindings remain pending"
GENERATE_COMMAND = "fold-generate"
INDEX_SCHEMA = "mpk.go_practical.prefix_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.prefix_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.prefix_attempt.v0"
STATUS = "prefix ingredients measured; W03 remains incomplete"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in sources()}
    for path in (Path(__file__), HERE / "check_prefix_report.py",
                 HERE / "check_balanced_report.py", HERE / "run_balanced_probes.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": path, "sha256": digest} for path, digest in sorted(result.items())]


def depth(count):
    return max(count - 1, 0).bit_length()


def prefix_nodes(count):
    nodes = set()

    def visit(d, n):
        if (d, n) in nodes:
            return
        nodes.add((d, n))
        if d and n:
            low = min(n, 1 << (d - 1))
            visit(d - 1, low)
            visit(d - 1, n - low)
    visit(depth(count), count)
    if count:
        visit(depth(count), count - 1)
    return nodes


def declaration_count(count):
    return 135 + sum(12 + 2 * bool(d and n) for d, n in prefix_nodes(count))


def variants(count, family):
    return range(3 if family == "unconditional" or count == 0 else 4)


def expected_cases():
    return [(n, family, mutation) for n in COUNTS for family in FAMILIES for mutation in variants(n, family)]


def identifier(count, family, mutation):
    suffix = (("exact-nat-sum", "dropped-initial", "vacuous-empty-safety" if count == 0 else "unrelated-safety", "skipped-last")
              if family == "conditional" else
              ("exact-nat-sum-with-overflow", "empty-overflow-identity" if count == 0 else "dropped-overflow", "raw-dropped-initial"))[mutation]
    return f"prefix-{count}-{suffix}"


def accepted(count, family, mutation):
    return mutation == 0 or count == 0 and mutation == (2 if family == "conditional" else 1)


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["count"]),
                         module=f"Probe.GoPractical.Word.PrefixCount{case['count']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    resource_variant = (case["family"], case["mutation"]) in (("conditional", 3), ("unconditional", 1))
    if resource_variant and rust == "CORE_FUEL_EXHAUSTED" and go == "core reduction fuel exhausted":
        return "candidate_fuel_failure_not_false_claim_evidence"
    raise RuntimeError("unexpected prefix failure; resource failures cannot count as semantic negatives")


def check_retained(report):
    from check_prefix_report import check
    return check(report)


def main(model=None):
    model = sys.modules[__name__] if model is None else model
    record = model.RECORD
    parser = argparse.ArgumentParser(description=model.__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise RuntimeError("pinned host and RSS units require darwin/arm64")
    pin = json.loads((HERE / "host-toolchain.json").read_text())
    host_inventory = verify_host_toolchain(DEFAULT_GOROOT, pin)
    cargo = shutil.which("cargo")
    if not cargo:
        raise RuntimeError("Cargo unavailable")
    checked([cargo, "build", "--release", "--offline", "--locked", "--manifest-path", str(HERE / "certificates/Cargo.toml")],
            env={**os.environ, "CARGO_TARGET_DIR": str(TARGET)})
    producer = TARGET / "release/go-practical-proof-probe"
    before = model.proof_sources()
    rows, groups = [], []
    with tempfile.TemporaryDirectory(prefix="mpk-go-prefix-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        reference = temp / "mpk-checker-ref"
        env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off",
               "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(reference), "./cmd/mpk-checker-ref"],
                cwd=ROOT / "go-tools/mpk-checker-ref", env=env)
        binaries = {"rust": sha(producer.read_bytes()), "go": sha(reference.read_bytes())}

        def retain_attempt(active=None):
            if args.write:
                record.mkdir(parents=True, exist_ok=True)
                (record / "attempt.json").write_bytes(json_bytes({
                    "schema": model.ATTEMPT_SCHEMA, "status": "incomplete measurement",
                    "inputs": before, "construction_groups": groups, "cases": rows, "active": active,
                    "checker_binaries": binaries}))
        retain_attempt()
        for count in model.COUNTS:
            first, second = temp / f"first-{count}", temp / f"second-{count}"
            one = timed([producer, model.GENERATE_COMMAND, ROOT, first, str(count)])
            two = timed([producer, model.GENERATE_COMMAND, ROOT, second, str(count)])
            if one["exit_code"] or two["exit_code"]:
                raise RuntimeError("prefix construction failed; W03 remains pending")
            names = sorted(p.name for p in first.iterdir())
            if names != sorted(p.name for p in second.iterdir()) or any(
                    (first / n).read_bytes() != (second / n).read_bytes() for n in names):
                raise RuntimeError("prefix generation is nondeterministic")
            index = json.loads((first / "index.json").read_text())
            ids = [case["id"] for case in index["cases"]]
            if index["schema"] != model.INDEX_SCHEMA or len(ids) != len(set(ids)) or names != sorted([i + ".mpcert" for i in ids] + ["index.json"]):
                raise RuntimeError("prefix index identity or duplicate/shadowed candidate")
            groups.append({"count": count, "scope": "complete candidate group and shared ordinary library", "first": one, "second": two})
            for case in index["cases"]:
                path = first / (case["id"] + ".mpcert")
                if args.write:
                    shutil.copyfile(path, record / path.name)
                observations = {}
                for checker, exe in (("rust", producer), ("go", reference)):
                    observations[checker] = observe(exe, path)
                    retain_attempt({"case": case, "observations": observations, "raw_sha256": sha(path.read_bytes())})
                outcome = model.verify_case(case, observations)
                rows.append({"case": case, "classification": outcome, "raw_sha256": sha(path.read_bytes()), "observations": observations})
                if args.check and (record / path.name).read_bytes() != path.read_bytes():
                    raise RuntimeError("retained prefix certificate differs")
                retain_attempt()
                print(f"{case['id']}: {outcome}; Rust {observations['rust']['wall_seconds']:.3f}s, Go {observations['go']['wall_seconds']:.3f}s", flush=True)
        extra = model.measure_extra(temp, env) if hasattr(model, "measure_extra") else None
        if before != model.proof_sources():
            raise RuntimeError("source changed during prefix measurement")
        report = {"schema": model.REPORT_SCHEMA, "work_item": "GO-PRAC-T01-W03",
                  "status": model.STATUS, "scope": model.SCOPE, "pending": model.PENDING,
                  "inputs": before, "host_toolchain_inventory": host_inventory, "checker_binaries": binaries,
                  "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine(),
                           "cpu_count": os.cpu_count(), "model": None, "physical_memory_bytes": None},
                  "rustc": checked([shutil.which("rustc"), "--version"]).strip(), "cargo": checked([cargo, "--version"]).strip(),
                  "build": "Rust release offline locked; Go 1.25.0 pinned host trimpath",
                  "measurement_child_ceiling_seconds": 60, "ceiling_scope": "private measurement, not a profile budget",
                  "boundary_probe_scope": model.BOUNDARY_SCOPE, "deterministic_generations": 2,
                  "construction_groups": groups, "cases": rows,
                  "outcomes": {key: sum(r["classification"] == key for r in rows) for key in
                               ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}}
        if extra is not None:
            report["quote_observations"] = extra
        if args.write:
            for filename in getattr(model, "EXTRA_FILES", ()):
                shutil.copyfile(temp / filename, record / filename)
            (record / "observations.json").write_bytes(json_bytes(report))
            (record / "attempt.json").unlink()
        retained = json.loads((record / "observations.json").read_text())
        model.check_retained(retained)
        if [r["case"] for r in retained["cases"]] != [r["case"] for r in rows]:
            raise RuntimeError("retained prefix theorem index differs")
        if args.check and any((record / n).read_bytes() != (temp / n).read_bytes() for n in getattr(model, "EXTRA_FILES", ())):
            raise RuntimeError("retained Go observations differ")
        print(f"PASS: {len(rows)} measured candidates; actual guards/corrections explicit; W03 incomplete")


if __name__ == "__main__":
    main()
