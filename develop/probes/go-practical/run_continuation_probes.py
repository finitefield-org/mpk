#!/usr/bin/env python3
"""Measure composed source-shaped continuing prefixes with actual zero entry."""

import json
from pathlib import Path
import sys

import quote_differential
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT, HERE, ROOT, checked, semantic_expectation, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w03/continuing-prefixes"
COUNTS = shared.COUNTS
FAMILIES = ("state", "natural", "zero-state", "zero-natural")
BOUNDARY_SCOPE = shared.BOUNDARY_SCOPE
SCOPE = "ordinary all-continuing modular64 prefixes, deriving intermediate bounds and carries from actual item checks and proving zero-entry exact sums; Go source/loop/error/descriptor binding remains pending"
PENDING = ["complete Go source/model/operator binding", "actual loop/return/closed-error and descriptor composition",
           "remaining byte/collection/control laws", "aggregate boundary and resource freeze"]
CASE_SCOPE = "private source-shaped all-continuing modular64 prefix; intermediate bounds and carries are derived; Go source/loop/error/descriptor binding remains pending"
BOUNDED_PREMISES = "initial limit/accumulator nonnegative, accumulator within limit; every actual item passes nonnegative and remaining-bound checks"
ZERO_PREMISES = "limit nonnegative; accumulator is literal zero; every actual item passes nonnegative and remaining-bound checks"
GENERATE_COMMAND = "continuation-generate"
INDEX_SCHEMA = "mpk.go_practical.continuation_pilot.v0"
REPORT_SCHEMA = "mpk.go_practical.continuation_report.v0"
ATTEMPT_SCHEMA = "mpk.go_practical.continuation_attempt.v0"
STATUS = "continuing prefix ingredients measured; W03 remains incomplete"
EXTRA_FILES = quote_differential.ARTIFACTS


def proof_sources():
    result = {r["path"]: r["sha256"] for r in shared.proof_sources()}
    for path in (Path(__file__), HERE / "check_continuation_report.py", HERE / "quote_differential.py", HERE / "quote_reference.go", HERE / "order_differential.py"):
        result[path.relative_to(ROOT).as_posix()] = sha(path.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(result.items())]


def declaration_count(count):
    return 357 + sum(17 + 4 * bool(d and n) for d, n in shared.prefix_nodes(count))


def variants(family):
    return range(3 if family == "natural" else 2)


def expected_cases():
    return [(n, family, mutation) for n in COUNTS for family in FAMILIES for mutation in variants(family)]


def identifier(n, family, mutation):
    label = ("exact-sum" if family.endswith("natural") else "bounded-result") if mutation == 0 else ("vacuous-empty-checks" if n == 0 else "unrelated-checks") if mutation == 1 else "dropped-initial"
    return f"continuation-{n}-{family}-{label}"


def verify_case(case, observations):
    semantic_expectation(case, observations, declarations=declaration_count(case["count"]),
                         module=f"Probe.GoPractical.Word.ContinuationPrefixCount{case['count']}")
    if case["expected"] == "accepted":
        return "accepted_zero_axiom"
    rust = json.loads(observations["rust"]["report"]["error_detail"])["code"]
    go = observations["go"]["report"]["error_detail"]
    if rust == "CORE_TYPE_MISMATCH" and go.startswith("term ") and " inferred " in go and " but expected " in go:
        return "core_type_mismatch"
    raise RuntimeError("continuation proof did not fail at intended type boundary; resource failures cannot count as semantic negatives")


def measure_extra(directory, env):
    executable = directory / "quote-observer"
    checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(executable), str(HERE / "quote_reference.go")], env=env)
    return quote_differential.run(executable, directory)


def check_retained(report):
    from check_continuation_report import check
    return check(report)


if __name__ == "__main__":
    shared.main(sys.modules[__name__])
