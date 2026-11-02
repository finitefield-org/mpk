#!/usr/bin/env python3
"""Retained halting-return evidence integrity; not an application proof replay."""

import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
import quote_differential
from run_prefix_probes import depth
from run_proof_pilot import sha
from run_returned_probes import BOUNDARY_SCOPE, CASE_SCOPE, COUNTS, PENDING, PREMISES, RECORD, REPORT_SCHEMA, ROOT, SCOPE, STATUS, declaration_count, expected_cases, identifier, proof_sources, verify_case


def check(report):
    require(report["schema"] == REPORT_SCHEMA and report["work_item"] == "GO-PRAC-T01-W03", "return report schema")
    require(report["status"] == STATUS, "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost source/error pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["boundary_probe_scope"] == BOUNDARY_SCOPE, "invented Go capacity admission")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["measurement_child_ceiling_seconds"] == 60 and report["ceiling_scope"] == "private measurement, not a profile budget", "invented resource budget")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require(set(report["checker_binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["checker_binaries"].values()), "checker binary identity")
    require([g["count"] for g in report["construction_groups"]] == list(COUNTS), "construction coverage")
    require([(r["case"]["count"], r["case"]["family"], r["case"]["mutation"]) for r in report["cases"]] == expected_cases(), "case coverage")
    names = [r["case"]["id"] for r in report["cases"]]
    require(len(names) == len(set(names)), "shadowed candidate identity")
    require(sorted(p.name for p in RECORD.glob("*.mpcert")) == sorted(n + ".mpcert" for n in names), "retained certificate coverage")
    for group in report["construction_groups"]:
        require(group["scope"] == "complete candidate group and shared ordinary library", "construction scope")
        for measurement in (group["first"], group["second"]):
            require(measurement["exit_code"] == 0, "construction failed")
            resource(measurement)
    outcomes = {k: 0 for k in ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}
    for row in report["cases"]:
        case = row["case"]
        n, family, mutation = case["count"], case["family"], case["mutation"]
        d = depth(n)
        require(case["id"] == identifier(n, family, mutation) and case["expected"] == ("accepted" if mutation == 0 else "rejected"), "candidate identity")
        require(case["address_depth"] == d and case["storage_capacity"] == 1 << d, "backing capacity identity")
        require(case["source_word_bits"] == 64 and case["quantified_limit_bits"] == 64 and case["quantified_storage_bits"] == 64 << d, "lost full payload quantification")
        require(case["frame_bits"] == 128 and case["root_entry"] is (family not in ("absorbing-return", "first-failure")), "full frame or root entry binding")
        require(case["initial_natural_quantified"] is (family == "success-sum"), "natural observer quantification")
        require(case["premises"] == PREMISES[family], "oracle or altered return premises")
        require(case["scope"] == CASE_SCOPE, "overstated candidate scope")
        require(case["theorem"] == "Probe.GoPractical.Word.ReturnedClaim" + family.replace("-", "") and case["declarations"] == declaration_count(n), "theorem identity")
        require(type(case["terms"]) is int and case["terms"] > 0, "term count")
        raw = (RECORD / (case["id"] + ".mpcert")).read_bytes()
        require(len(raw) == case["bytes"] and row["raw_sha256"] == sha(raw), "certificate byte identity")
        require(set(row["observations"]) == {"rust", "go"}, "missing independent checker")
        try:
            classification = verify_case(case, row["observations"])
        except RuntimeError as error:
            raise ValueError(str(error)) from error
        require(row["classification"] == classification, "incorrect rejection classification")
        outcomes[classification] += 1
        for measurement in row["observations"].values():
            resource(measurement)
    require(report["outcomes"] == outcomes and outcomes == {"accepted_zero_axiom": 54, "core_type_mismatch": 54, "candidate_fuel_failure_not_false_claim_evidence": 0}, "outcome counts")
    observations = report["quote_observations"]
    quote_differential.check(observations, RECORD)
    require(observations["go_measurement"]["exit_code"] == 0, "Go Quote did not execute")
    require(re.fullmatch(r"[0-9a-f]{64}", observations["go_binary_sha256"]) is not None, "Go Quote binary identity")
    resource(observations["go_measurement"])
    return len(report["cases"])


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {count} halting-return candidates and empirical Go observations; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Return report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
