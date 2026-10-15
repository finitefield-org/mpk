#!/usr/bin/env python3
"""Measure two's-complement prefix cross sums under actual sign-carry guards."""

import json
from pathlib import Path
import sys

import run_prefix_probes as shared
from run_proof_pilot import HERE, ROOT, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/signed-prefix-sums"
COUNTS = shared.COUNTS
BOUNDARY_SCOPE = shared.BOUNDARY_SCOPE
SCOPE = "ordinary conditional two's-complement modular64 prefix cross sums under every actual sign-stage carry guard; no source-bound Go application proof"
PENDING = ["Go source guard/order/subtraction discharge and nonnegative Quote invariant", "complete Go source/model and operation bindings",
           "string/slice/descriptor/control law closure", "aggregate boundary and resource freeze"]
CASE_SCOPE = "conditional exact fixed-count modular64 prefix cross sums for two's-complement differences; Go source guard/order bindings remain pending"
GENERATE_COMMAND = "signed-fold-generate"
INDEX_SCHEMA = "mpk.go_practical.signed_prefix_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.signed_prefix_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.signed_prefix_attempt.v0"
STATUS = "conditional signed prefix ingredient measured; W03 remains incomplete"


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_signed_prefix_report.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": path, "sha256": digest} for path, digest in sorted(result.items())]


def declaration_count(count):
    return 193 + sum(18 + 4 * bool(d and n) for d, n in shared.prefix_nodes(count))


def identifier(count, mutation):
    suffix = ("exact-cross-sum", "dropped-result-sign", "vacuous-empty-guard" if count == 0 else "unrelated-guard", "dropped-initial")[mutation]
    return f"signed-prefix-{count}-{suffix}"


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["count"]),
                         module=f"Probe.GoPractical.Word.SignedPrefixCount{case['count']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    if case["mutation"] == 1 and rust == "CORE_FUEL_EXHAUSTED" and go == "core reduction fuel exhausted":
        return "candidate_fuel_failure_not_false_claim_evidence"
    raise RuntimeError("unexpected signed prefix failure; resource failures cannot count as semantic negatives")


def check_retained(report):
    from check_signed_prefix_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
