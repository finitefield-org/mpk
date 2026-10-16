#!/usr/bin/env python3
"""Measure full-word descriptor/sharing and complete finite cell-store laws."""

import json
from pathlib import Path
import sys

import slice_differential
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, checked, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/slice-stores"
COUNTS = slice_differential.CAPACITIES
FAMILIES = ("descriptor-copy", "full-reslice", "nil-empty", "same-key-alias", "other-region-frame", "array-value-copy", "tail-exposure")
DATA_FAMILIES = FAMILIES[3:]
BOUNDARY_SCOPE = "capacities 0/1/8/32/127/128/129 are private slice workload/boundary probes; 129 does not admit a larger Go profile and no budget is frozen"
SCOPE = "ordinary two-region descriptor/keyed-store, array-snapshot and tail-exposure laws over full source words and complete 128-bit elements; production absolute-bound/validity/ownership and operation bindings remain pending"
PENDING = ["absolute offset/range/descriptor validity and source binding", "ownership/admission and full operation bindings",
           "general two/three-index slicing, overlapping copy and bounded append",
           "remaining byte/control laws and aggregate resource freeze"]
CASE_SCOPE = "ordinary descriptor/keyed-store and array-snapshot algebra over complete finite backing; absolute-bound/validity/ownership and production Go operation binding remain pending"
GENERATE_COMMAND = "slices-generate"
INDEX_SCHEMA = "mpk.go_practical.slice_store_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.slice_store_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.slice_store_attempt.v0"
STATUS = "slice descriptor/store ingredients measured; W03 remains incomplete"
EXTRA_FILES = slice_differential.ARTIFACTS
EXTRA_REPORT_KEY = "slice_observations"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_slice_report.py", HERE / "slice_differential.py",
                 HERE / "slice_reference.go", HERE / "order_differential.py", HERE / "byte_differential.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def depth(n):
    return max(n - 1, 0).bit_length()


def declaration_count(_n):
    return 273


def expected_cases():
    return [(n, family, mutation) for n in COUNTS for family in FAMILIES
            if n or family not in DATA_FAMILIES for mutation in range(2)]


def identifier(n, family, mutation):
    label = "valid" if not mutation else {"same-key-alias": "fresh-id-per-view",
            "other-region-frame": "erased-region-separation", "array-value-copy": "shared-copy-region",
            "tail-exposure": "zeroed-tail"}.get(family, "altered-result")
    return f"slice-store-{n}-{family}-{label}"


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["capacity"]),
                         module=f"Probe.GoPractical.Word.SliceStoreCapacity{case['capacity']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("slice proof did not fail at intended type boundary; resource failures cannot count as semantic negatives")


def measure_extra(directory, env):
    executable = directory / "slice-observer"
    checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(executable), str(HERE / "slice_reference.go")], env=env)
    return slice_differential.run(executable, directory)


def check_retained(report):
    from check_slice_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
