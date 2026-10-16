#!/usr/bin/env python3
"""Retained index-bound evidence integrity; actual replay is run_index_probes."""

import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
import index_differential
from run_proof_pilot import sha
from run_index_probes import BOUNDARY_SCOPE, CASE_SCOPE, COUNTS, PENDING, RECORD, REPORT_SCHEMA, ROOT, SCOPE, STATUS, declaration_count, depth, expected_cases, identifier, premises, proof_sources, verify_case


def check(report):
    require(report["schema"] == REPORT_SCHEMA and report["work_item"] == "GO-PRAC-T01-W03", "index report schema")
    require(report["status"] == STATUS, "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost source/operator pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["boundary_probe_scope"] == BOUNDARY_SCOPE, "invented Go capacity admission")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["measurement_child_ceiling_seconds"] == 60 and report["ceiling_scope"] == "private measurement, not a profile budget", "invented resource budget")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require(set(report["checker_binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["checker_binaries"].values()), "checker binary identity")
    require([g["count"] for g in report["construction_groups"]] == list(COUNTS), "construction coverage")
    require([(r["case"]["capacity"], r["case"]["family"], r["case"]["mutation"]) for r in report["cases"]] == expected_cases(), "case coverage")
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
        n, family, mutation = case["capacity"], case["family"], case["mutation"]
        require(type(n) is int and type(mutation) is int and type(case["count"]) is int, "candidate number types")
        require(case["count"] == n and case["id"] == identifier(n, family, mutation) and case["expected"] == ("accepted" if mutation == 0 else "rejected"), "candidate identity")
        require(case["low_index_bits"] == depth(n), "finite index identity")
        require(all(type(case[k]) is int and case[k] == 64 for k in ("source_word_bits", "full_length_bits", "full_index_bits")), "lost full source domain")
        require(case["premises"] == premises(family, mutation), "invented fit oracle or changed actual premises")
        require(case["scope"] == CASE_SCOPE, "overstated candidate scope")
        require(case["theorem"] == "Probe.GoPractical.Word.IndexBoundClaim" + family.replace("-", "") and case["declarations"] == declaration_count(n), "theorem identity")
        require(type(case["terms"]) is int and case["terms"] > 0, "term count")
        raw = (RECORD / (case["id"] + ".mpcert")).read_bytes()
        require(len(raw) == case["bytes"] and row["raw_sha256"] == sha(raw), "certificate byte identity")
        require(set(row["observations"]) == {"rust", "go"}, "missing independent checker")
        try:
            classification = verify_case(case, row["observations"])
        except RuntimeError as error:
            raise ValueError(str(error)) from error
        require(row["classification"] == classification, "incorrect rejection classification")
        if classification == "accepted_zero_axiom":
            expected_hash = sha(b"MPK-MODULE-CERT-0.1" + bytes([0]) + raw)
            require(all(m["report"]["hashes"]["certificate"] == expected_hash for m in row["observations"].values()), "accepted checker/certificate identity")
        outcomes[classification] += 1
        for measurement in row["observations"].values():
            resource(measurement)
    require(report["outcomes"] == outcomes and outcomes == {"accepted_zero_axiom": 12, "core_type_mismatch": 15, "candidate_fuel_failure_not_false_claim_evidence": 0}, "outcome counts")
    observed = report["index_observations"]
    index_differential.check(observed, RECORD)
    require([w["case_id"] for w in observed["negative_claim_witnesses"]] == [r["case"]["id"] for r in report["cases"] if r["case"]["mutation"]], "false-claim witness coverage")
    require(observed["go_measurement"]["exit_code"] == 0, "Go indexes did not execute")
    require(re.fullmatch(r"[0-9a-f]{64}", observed["go_binary_sha256"]) is not None, "Go index binary identity")
    resource(observed["go_measurement"])
    return len(report["cases"])


if __name__ == "__main__":
    try:
        n = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {n} index-bound candidates and actual-Go guard/panic observations; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Index report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
