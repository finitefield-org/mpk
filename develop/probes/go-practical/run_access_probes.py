#!/usr/bin/env python3
"""Measure concrete slice validity, actual local guards and full-word coverage."""

import json
from pathlib import Path
import sys

import access_differential
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, checked, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/slice-access"
COUNTS = access_differential.CAPACITIES
FAMILIES = access_differential.FAMILIES
DATA_FAMILIES = FAMILIES[:5]
BOUNDARY_SCOPE = "semantic capacities 0/1/8/32/127/128/129 are private probes; power-of-two padding never raises the storage bound or admits a public Go capacity/budget"
SCOPE = "ordinary computed descriptor validity and actual local guards imply source-width address safety, physical bounds, live-region and complete canonical low-bit reconstruction/read laws; production capture/ownership/operation and remaining byte/control/aggregate closure stay pending"
PENDING = ["production Go capture/model/ownership/operation bindings", "general slicing, overlapping copy and bounded construction/append",
           "remaining byte/control composition", "aggregate boundary and resource freeze"]
CASE_SCOPE = "ordinary computed descriptor validity and actual index guard discharge absolute full-width address safety/coverage; production Go capture/ownership/operation and remaining laws stay pending"
GENERATE_COMMAND = "access-generate"
INDEX_SCHEMA = "mpk.go_practical.slice_access_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.slice_access_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.slice_access_attempt.v0"
STATUS = "slice validity/access ingredients measured; W03 remains incomplete"
EXTRA_FILES = access_differential.ARTIFACTS
EXTRA_REPORT_KEY = "access_observations"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_access_report.py", HERE / "access_differential.py", HERE / "access_reference.go",
                 HERE / "slice_differential.py", HERE / "order_differential.py", HERE / "byte_differential.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def depth(n):
    return max(n - 1, 0).bit_length()


def padding(n):
    return 1 << depth(n)


def declaration_count(n):
    return 384 if n == 0 else 383 if n == 1 else 399 if n == 129 else 420


def expected_cases():
    return [(n, f, m) for n in COUNTS for f in FAMILIES
            if (n or f not in DATA_FAMILIES) and (not n or f != "empty-no-access") for m in range(2)]


def identifier(n, family, mutation):
    label = "valid" if not mutation else "erased-validity" if family in ("physical-bounds", "live-region") else "erased-storage-bound" if family in ("full-value-coverage", "empty-no-access") else "zeroed-read" if family == "finite-read" else "altered-result"
    return f"slice-access-{n}-{family}-{label}"


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["capacity"]),
                         module=f"Probe.GoPractical.Word.SliceAccessCapacity{case['capacity']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("access proof did not fail at intended type boundary; resource failure is not semantic-negative evidence")


def measure_extra(directory, env):
    executable = directory / "access-observer"
    checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(executable), str(HERE / "access_reference.go")], env=env)
    return access_differential.run(executable, directory)


def check_retained(report):
    from check_access_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
