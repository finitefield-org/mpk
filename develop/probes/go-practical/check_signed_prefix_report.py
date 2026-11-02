#!/usr/bin/env python3
"""Check signed-prefix evidence integrity, without claiming proof replay."""

import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
from run_prefix_probes import depth
from run_proof_pilot import sha
from run_signed_prefix_probes import (
    BOUNDARY_SCOPE, CASE_SCOPE, COUNTS, PENDING, RECORD, REPORT_SCHEMA, ROOT,
    SCOPE, STATUS, declaration_count, identifier, proof_sources, verify_case,
)


def check(report):
    require(report["schema"] == REPORT_SCHEMA, "signed prefix report schema")
    require(report["work_item"] == "GO-PRAC-T01-W03" and report["status"] == STATUS, "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost source/guard pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["boundary_probe_scope"] == BOUNDARY_SCOPE, "invented profile admission")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["measurement_child_ceiling_seconds"] == 60 and report["ceiling_scope"] == "private measurement, not a profile budget", "invented resource budget")
    require(set(report["checker_binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["checker_binaries"].values()), "checker binary identity")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require([g["count"] for g in report["construction_groups"]] == list(COUNTS), "construction coverage")
    require([(r["case"]["count"], r["case"]["mutation"]) for r in report["cases"]] == [(n,m) for n in COUNTS for m in range(4)], "case coverage")
    for group in report["construction_groups"]:
        require(group["scope"] == "complete candidate group and shared ordinary library", "construction scope")
        for m in (group["first"], group["second"]):
            require(m["exit_code"] == 0, "construction failed")
            resource(m)
    counts = {k: 0 for k in ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}
    for row in report["cases"]:
        case = row["case"]
        n, mutation = case["count"], case["mutation"]
        d = depth(n)
        require(case["address_depth"] == d and case["storage_capacity"] == 1 << d, "backing capacity identity")
        require(case["source_word_bits"] == 64 and case["quantified_storage_bits"] == 64 << d and case["quantified_accumulator_bits"] == 64 and case["initial_natural_quantified"] is True, "lost full signed payload quantification")
        accepted = mutation == 0 or n == 0 and mutation == 2
        require(case["id"] == identifier(n, mutation) and case["expected"] == ("accepted" if accepted else "rejected"), "candidate identity")
        require(case["scope"] == CASE_SCOPE, "overstated candidate scope")
        premise = "all inactive true equalities (vacuous only for empty prefix)" if mutation == 2 else "each actual intermediate sign-stage incoming/outgoing carry agrees"
        require(case["premise"] == premise, "lost actual sign-carry premise")
        require(case["theorem"] == "Probe.GoPractical.Word.SignedPrefixClaim", "theorem identity")
        require(case["declarations"] == declaration_count(n) and type(case["terms"]) is int and case["terms"] > 0, "declaration identity")
        raw = (RECORD / (case["id"] + ".mpcert")).read_bytes()
        require(len(raw) == case["bytes"] and row["raw_sha256"] == sha(raw), "certificate byte identity")
        require(set(row["observations"]) == {"rust", "go"}, "missing independent checker")
        try:
            classification = verify_case(case, row["observations"])
        except RuntimeError as error:
            raise ValueError(str(error)) from error
        require(row["classification"] == classification, "resource failure counted as semantic negative")
        counts[classification] += 1
        for m in row["observations"].values():
            resource(m)
    require(report["outcomes"] == counts and counts["accepted_zero_axiom"] == 9, "outcome counts")
    return len(report["cases"])


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {count} retained signed prefix candidates; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Signed prefix report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
