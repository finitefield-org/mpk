#!/usr/bin/env python3
"""Measure source-shaped add/subtract/order proofs and pinned Go observations."""

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import tempfile

import order_differential
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, TARGET, checked, json_bytes, observe, semantic_expectation, sha, sources, timed, verify_host_toolchain

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/order-guards"
DEPTHS = tuple(range(8))
SCOPE = "ordinary full-word carry composition and source-shaped Quote continuation arithmetic; Go frontend/source binding and whole-loop proof remain pending"
PENDING = ["complete Go source/model and operation binding", "Quote loop/prefix/return composition and collection/control laws",
           "string/slice/descriptor law closure", "aggregate boundary and resource freeze"]
BOUNDARY_SCOPE = "width128 is an internal arithmetic probe, not Go source integer admission"
CARRY_PREMISES = "four arbitrary incoming carries satisfying d + e + r = t + 1; unrelated true replacement is a deliberate mutation"
PATH_PREMISES = "limit and accumulator nonnegative, accumulator <= limit, item nonnegative, item <= wrapping(limit - accumulator); named premise substitutions are deliberate mutations"
CARRY_SCOPE = "private unsigned modular add/subtract carry relation; Go frontend/source binding remains pending"
PATH_SCOPE = "private source-shaped Quote continuation arithmetic; Go frontend/source binding and whole-loop proof remain pending"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in sources()}
    for path in (Path(__file__), HERE / "check_order_report.py", HERE / "order_differential.py", HERE / "order_reference.go",
                 HERE / "check_balanced_report.py", HERE / "run_balanced_probes.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def expected_cases():
    return [(d, family, mutation) for d in DEPTHS for family in (("carry", "path", "natural") if 3 <= d <= 6 else ("carry",))
            for mutation in range(2 if family == "carry" else 3)]


def identifier(d, family, mutation):
    if family == "carry":
        label = ("carry-conservation", "unrelated-carry-premise")[mutation]
    else:
        labels = ("step-invariant", "unrelated-remaining-bound", "unrelated-item-sign") if family == "path" else ("exact-nat-addition", "dropped-accumulator", "unrelated-remaining-bound")
        label = family + "-" + labels[mutation]
    return f"order-{1 << d}-{label}"


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=78 + 37 * case["depth"], module=f"Probe.GoPractical.Word.OrderDepth{case['depth']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("order proof did not fail at the intended type boundary; resource failures are not semantic negatives")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise RuntimeError("pinned host and RSS units require darwin/arm64")
    inventory = verify_host_toolchain(DEFAULT_GOROOT, json.loads((HERE / "host-toolchain.json").read_text()))
    cargo = shutil.which("cargo")
    checked([cargo, "build", "--release", "--offline", "--locked", "--manifest-path", str(HERE / "certificates/Cargo.toml")],
            env={**os.environ, "CARGO_TARGET_DIR": str(TARGET)})
    producer = TARGET / "release/go-practical-proof-probe"
    before, rows, groups = proof_sources(), [], []
    with tempfile.TemporaryDirectory(prefix="mpk-go-order-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        reference, go_operations = temp / "mpk-checker-ref", temp / "go-order-observer"
        env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off", "GOTELEMETRY": "off",
               "CGO_ENABLED": "0", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-o", str(reference), "./cmd/mpk-checker-ref"], cwd=ROOT / "go-tools/mpk-checker-ref", env=env)
        checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-o", str(go_operations), str(HERE / "order_reference.go")], env=env)
        binaries = {"rust": sha(producer.read_bytes()), "go": sha(reference.read_bytes())}

        def attempt(active=None):
            if args.write:
                RECORD.mkdir(parents=True, exist_ok=True)
                (RECORD / "attempt.json").write_bytes(json_bytes({"schema": "mpk.go_practical.order_attempt.v0", "status": "incomplete measurement",
                    "inputs": before, "construction_groups": groups, "cases": rows, "active": active, "checker_binaries": binaries}))
        attempt()
        for d in DEPTHS:
            first, second = temp / f"first-{d}", temp / f"second-{d}"
            one, two = (timed([producer, "order-generate", ROOT, directory, str(d)]) for directory in (first, second))
            if one["exit_code"] or two["exit_code"]:
                raise RuntimeError("order candidate construction failed")
            names = sorted(p.name for p in first.iterdir())
            if names != sorted(p.name for p in second.iterdir()) or any((first/n).read_bytes() != (second/n).read_bytes() for n in names):
                raise RuntimeError("order construction is nondeterministic")
            index = json.loads((first / "index.json").read_text())
            ids = [c["id"] for c in index["cases"]]
            if index["schema"] != "mpk.go_practical.order_pilot.v0" or len(ids) != len(set(ids)) or names != sorted([i + ".mpcert" for i in ids] + ["index.json"]):
                raise RuntimeError("duplicate/shadowed or missing order candidate")
            groups.append({"depth": d, "width": 1 << d, "scope": "complete candidate group and shared ordinary library", "first": one, "second": two})
            for case in index["cases"]:
                path = first / (case["id"] + ".mpcert")
                if args.write:
                    shutil.copyfile(path, RECORD / path.name)
                observations = {}
                for checker, exe in (("rust", producer), ("go", reference)):
                    observations[checker] = observe(exe, path)
                    attempt({"case": case, "observations": observations, "raw_sha256": sha(path.read_bytes())})
                outcome = verify_case(case, observations)
                rows.append({"case": case, "classification": outcome, "raw_sha256": sha(path.read_bytes()), "observations": observations})
                if args.check and (RECORD / path.name).read_bytes() != path.read_bytes():
                    raise RuntimeError("retained order certificate differs")
                attempt()
                print(f"{case['id']}: {outcome}; Rust {observations['rust']['wall_seconds']:.3f}s, Go {observations['go']['wall_seconds']:.3f}s", flush=True)
        differential = order_differential.run(go_operations, temp)
        if before != proof_sources():
            raise RuntimeError("order proof source changed during measurement")
        report = {"schema": "mpk.go_practical.order_report.v0", "work_item": "GO-PRAC-T01-W03", "status": "order/guard ingredients measured; W03 remains incomplete",
                  "scope": SCOPE, "pending": PENDING, "internal_width_128_scope": BOUNDARY_SCOPE, "inputs": before,
                  "host_toolchain_inventory": inventory, "checker_binaries": binaries,
                  "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine(), "cpu_count": os.cpu_count(), "model": None, "physical_memory_bytes": None},
                  "build": "Rust release offline locked; Go 1.25.0 pinned host trimpath", "rustc": checked([shutil.which("rustc"), "--version"]).strip(), "cargo": checked([cargo, "--version"]).strip(),
                  "measurement_child_ceiling_seconds": 60, "ceiling_scope": "private measurement, not a profile budget", "deterministic_generations": 2,
                  "construction_groups": groups, "cases": rows, "differential": differential,
                  "outcomes": {k: sum(r["classification"] == k for r in rows) for k in ("accepted_zero_axiom", "core_type_mismatch")}}
        if args.write:
            for filename in ("go-reference-input.json", "go-reference-output.json"):
                shutil.copyfile(temp / filename, RECORD / filename)
            (RECORD / "observations.json").write_bytes(json_bytes(report))
            (RECORD / "attempt.json").unlink()
        from check_order_report import check
        retained = json.loads((RECORD / "observations.json").read_text())
        check(retained)
        if [r["case"] for r in retained["cases"]] != [r["case"] for r in rows]:
            raise RuntimeError("retained order theorem index differs")
        if args.check and any((RECORD / name).read_bytes() != (temp / name).read_bytes() for name in ("go-reference-input.json", "go-reference-output.json")):
            raise RuntimeError("Go operation observations do not replay")
        print(f"PASS: {len(rows)} proof candidates and {differential['cases']} actual Go observations; W03 remains incomplete")


if __name__ == "__main__":
    main()
