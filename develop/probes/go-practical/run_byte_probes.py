#!/usr/bin/env python3
"""Measure canonical 64-bit index values and finite string tail observations."""

import json
from pathlib import Path
import sys

import byte_differential
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, checked, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/byte-tails"
COUNTS = byte_differential.CAPACITIES
FAMILIES = ("canonical-byte", "defined-index", "index-extensional", "finite-source-read", "equality", "lexicographic")
BOUNDARY_SCOPE = "capacities 0/1/32/256 are private string workload probes; no public capacity or application budget is frozen"
SCOPE = "ordinary finite string byte observations with canonical full-width index values and tail unobservability for defined indexing/equality/bytewise order; complete Go source/carrier/operation binding remains pending"
PENDING = ["complete Go source/carrier/operation binding", "remaining substring/concatenation/prefix/suffix and length/content laws",
           "slice/descriptor/control composition", "aggregate boundary and resource freeze"]
CASE_SCOPE = "private string byte observations and tail unobservability; complete Go source/carrier/operation binding and remaining byte laws stay pending"
BACKING_SCOPE = "complete finite Bool-addressed bytes lifted to canonical 64-bit index values; no payload enumeration"
GENERATE_COMMAND = "bytes-generate"
INDEX_SCHEMA = "mpk.go_practical.byte_tail_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.byte_tail_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.byte_tail_attempt.v0"
STATUS = "string byte tail ingredients measured; W03 remains incomplete"
EXTRA_FILES = byte_differential.ARTIFACTS
EXTRA_REPORT_KEY = "byte_observations"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_byte_report.py", HERE / "byte_differential.py", HERE / "byte_reference.go"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def depth(capacity):
    return max(capacity - 1, 0).bit_length()


def declaration_count(capacity):
    return 102 + 6 * depth(capacity)


def expected_cases():
    return [(n, family, mutation) for n in COUNTS for family in FAMILIES
            if n != 0 or family != "defined-index" for mutation in range(2)]


def identifier(n, family, mutation):
    if n <= 1 and family == "index-extensional" and mutation == 1:
        return f"byte-tail-{n}-{family}-single-cell-control"
    return f"byte-tail-{n}-{family}-{'altered' if mutation else 'valid'}"


def accepted(n, family, mutation):
    return mutation == 0 or n <= 1 and family == "index-extensional" and mutation == 1


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["capacity"]),
                         module=f"Probe.GoPractical.Word.ByteTailCapacity{case['capacity']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("byte proof did not fail at intended type boundary; resource failures cannot count as semantic negatives")


def measure_extra(directory, env):
    executable = directory / "byte-observer"
    checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(executable), str(HERE / "byte_reference.go")], env=env)
    return byte_differential.run(executable, directory)


def check_retained(report):
    from check_byte_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
