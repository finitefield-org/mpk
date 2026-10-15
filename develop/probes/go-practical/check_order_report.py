#!/usr/bin/env python3
"""Check order/guard evidence integrity; proof replay is a separate command."""

import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
import order_differential
from run_order_probes import BOUNDARY_SCOPE, CARRY_PREMISES, CARRY_SCOPE, DEPTHS, PATH_PREMISES, PATH_SCOPE, PENDING, RECORD, ROOT, SCOPE, expected_cases, identifier, proof_sources, verify_case
from run_proof_pilot import sha


def check(report):
    require(report["schema"] == "mpk.go_practical.order_report.v0", "order report schema")
    require(report["work_item"] == "GO-PRAC-T01-W03" and report["status"] == "order/guard ingredients measured; W03 remains incomplete", "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost source/loop pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["internal_width_128_scope"] == BOUNDARY_SCOPE, "invented Go source width")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["measurement_child_ceiling_seconds"] == 60 and report["ceiling_scope"] == "private measurement, not a profile budget", "invented resource budget")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require(set(report["checker_binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["checker_binaries"].values()), "checker binary identity")
    require([g["depth"] for g in report["construction_groups"]] == list(DEPTHS), "construction coverage")
    require([(r["case"]["depth"], r["case"]["family"], r["case"]["mutation"]) for r in report["cases"]] == expected_cases(), "case coverage")
    names = [r["case"]["id"] for r in report["cases"]]
    require(len(names) == len(set(names)), "shadowed candidate identity")
    require(sorted(p.name for p in RECORD.glob("*.mpcert")) == sorted(n + ".mpcert" for n in names), "retained certificate coverage")
    for g in report["construction_groups"]:
        require(g["width"] == 1 << g["depth"] and g["scope"] == "complete candidate group and shared ordinary library", "construction width/scope")
        for m in (g["first"], g["second"]):
            require(m["exit_code"] == 0, "construction failed")
            resource(m)
    outcomes = {k: 0 for k in ("accepted_zero_axiom", "core_type_mismatch")}
    for row in report["cases"]:
        case = row["case"]
        d, family, mutation = case["depth"], case["family"], case["mutation"]
        require(case["id"] == identifier(d, family, mutation) and case["expected"] == ("accepted" if mutation == 0 else "rejected"), "candidate identity")
        require(case["width"] == 1 << d and case["operand_binders"] == 3 and case["quantified_operand_bits"] == 3 << d, "lost full operand quantification")
        require(case["initial_natural_quantified"] is (family == "natural"), "natural observer quantification")
        require(case["premises"] == (CARRY_PREMISES if family == "carry" else PATH_PREMISES), "lost actual path premises")
        require(case["scope"] == (CARRY_SCOPE if family == "carry" else PATH_SCOPE), "overstated candidate scope")
        suffix = {"carry": "OrderClaim", "path": "OrderPathClaim", "natural": "OrderPathNatClaim"}[family]
        require(case["theorem"] == "Probe.GoPractical.Word." + suffix and case["declarations"] == 78 + 37 * d, "theorem identity")
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
        for m in row["observations"].values():
            resource(m)
    require(report["outcomes"] == outcomes and outcomes == {"accepted_zero_axiom": 16, "core_type_mismatch": 24}, "outcome counts")
    differential = report["differential"]
    order_differential.check(differential, RECORD)
    require(re.fullmatch(r"[0-9a-f]{64}", differential["go_binary_sha256"]) is not None, "Go observation binary identity")
    require(differential["go_measurement"]["exit_code"] == 0, "Go observations did not execute")
    resource(differential["go_measurement"])
    return len(report["cases"])


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {count} retained order candidates and pinned Go observations; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Order report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
