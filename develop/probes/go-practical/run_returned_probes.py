#!/usr/bin/env python3
"""Measure actual halting returns and success-derived arithmetic in a private model."""

import json
from pathlib import Path
import sys

import quote_differential
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, checked, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/halting-returns"
COUNTS = shared.COUNTS
FAMILIES = ("absorbing-return", "failure-zero", "success-bounds", "success-sum", "entry-error", "closed-outcome", "first-failure")
BOUNDARY_SCOPE = shared.BOUNDARY_SCOPE
SCOPE = "ordinary full-payload halting Quote model with absorbing returns, zero failure results and success-derived bounded exact sums; complete Go source/string/slice/operator/descriptor binding remains pending"
PENDING = ["complete Go source/model/operator binding", "actual string/slice/loop/return/closed-error and descriptor binding",
           "remaining byte/collection/control laws", "aggregate boundary and resource freeze"]
CASE_SCOPE = "private full-payload halting Quote model; actual return status derives item checks and zero-entry arithmetic; Go source/operator/string/slice/descriptor binding remains pending"
PREMISES = {
    "absorbing-return": "actual input frame is stopped; all 128 input frame bits are arbitrary",
    "first-failure": "actual low-prefix result is stopped; all 128 input frame bits are arbitrary",
    "failure-zero": "actual rooted return is stopped",
    "success-bounds": "actual rooted return is running; entry and item checks are derived",
    "success-sum": "actual rooted return is running; entry and item checks are derived",
    "entry-error": "actual abstract empty-name or negative-limit entry guard is true",
    "closed-outcome": "none; closed three-outcome representation only",
}
GENERATE_COMMAND = "returned-generate"
INDEX_SCHEMA = "mpk.go_practical.returned_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.returned_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.returned_attempt.v0"
STATUS = "halting return ingredients measured; W03 remains incomplete"
EXTRA_FILES = quote_differential.ARTIFACTS


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_returned_report.py", HERE / "quote_differential.py", HERE / "quote_reference.go", HERE / "order_differential.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def declaration_count(count):
    return 378 + sum(35 + 5 * bool(d and n) for d, n in shared.prefix_nodes(count))


def expected_cases():
    return [(n, family, mutation) for n in COUNTS for family in FAMILIES
            if family != "first-failure" or n > 1 for mutation in range(2)]


def identifier(n, family, mutation):
    return f"returned-{n}-{family}-{'altered' if mutation else 'valid'}"


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["count"]),
                         module=f"Probe.GoPractical.Word.ReturnPrefixCount{case['count']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("halting return proof did not fail at intended type boundary; resource failures cannot count as semantic negatives")


def measure_extra(directory, env):
    executable = directory / "quote-observer"
    checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(executable), str(HERE / "quote_reference.go")], env=env)
    return quote_differential.run(executable, directory)


def check_retained(report):
    from check_returned_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
