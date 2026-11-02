#!/usr/bin/env python3
"""Measure ordinary full-width bounds implying value-preserving finite indexes."""

import json
from pathlib import Path
import sys

import index_differential
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, checked, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/index-bounds"
COUNTS = index_differential.CAPACITIES
FAMILIES = ("length-sign", "guard-equivalence", "index-coverage", "empty-no-index")
BOUNDARY_SCOPE = "capacities 0/1/32/256 are private index coverage probes; no public capacity or application budget is frozen"
SCOPE = "ordinary full-width length bounds and actual source-shaped signed index guards imply value-preserving finite index embedding; production Go source/operator binding remains pending"
PENDING = ["production Go source/operator and carrier binding", "remaining string operation laws",
           "slice/descriptor/control composition", "aggregate boundary and resource freeze"]
CASE_SCOPE = "private full-width index coverage and source-shaped signed guard; production Go source/operator binding and remaining string/slice laws stay pending"
GENERATE_COMMAND = "bounds-generate"
INDEX_SCHEMA = "mpk.go_practical.index_bound_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.index_bound_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.index_bound_attempt.v0"
STATUS = "index coverage ingredients measured; W03 remains incomplete"
EXTRA_FILES = index_differential.ARTIFACTS
EXTRA_REPORT_KEY = "index_observations"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_index_report.py", HERE / "index_differential.py",
                 HERE / "index_reference.go", HERE / "byte_differential.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def depth(capacity):
    return max(capacity - 1, 0).bit_length()


def declaration_count(capacity):
    count = 221
    if capacity:
        d, p = 6, depth(capacity)
        count += 1
        while True:
            count += 5 + 2 * (0 < p < 1 << d)
            if not p or p >= 1 << d or p == 1 << (d - 1):
                break
            half = 1 << (d - 1)
            d, p = d - 1, p if p < half else p - half
    return count


def expected_cases():
    return [(n, family, mutation) for n in COUNTS for family in FAMILIES
            if (family != "index-coverage" or n) and (family != "empty-no-index" or not n)
            for mutation in range(3 if family == "index-coverage" else 2)]


def identifier(n, family, mutation):
    label = "valid" if not mutation else (("erased-bound" if mutation == 1 else "erased-guard")
            if family in ("index-coverage", "empty-no-index") else "altered-result")
    return f"index-bound-{n}-{family}-{label}"


def premises(family, mutation):
    if family == "index-coverage" and mutation == 1:
        return "length bound deliberately erased; actual source-shaped signed index guard holds; no fit oracle"
    if family == "index-coverage" and mutation == 2:
        return "actual full-width length is within capacity; source guard deliberately erased; no fit oracle"
    if family == "empty-no-index" and mutation == 1:
        return "zero-capacity length bound deliberately erased"
    return {"length-sign": "actual full-width length is within the selected capacity",
            "guard-equivalence": "actual full-width length is within the selected capacity",
            "index-coverage": "actual full-width length is within capacity and actual source-shaped signed index guard holds; no fit oracle",
            "empty-no-index": "actual full-width length is within zero capacity"}[family]


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["capacity"]),
                         module=f"Probe.GoPractical.Word.IndexBoundsCapacity{case['capacity']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("index proof did not fail at intended type boundary; resource failures cannot count as semantic negatives")


def measure_extra(directory, env):
    executable = directory / "index-observer"
    checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(executable), str(HERE / "index_reference.go")], env=env)
    return index_differential.run(executable, directory)


def check_retained(report):
    from check_index_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
