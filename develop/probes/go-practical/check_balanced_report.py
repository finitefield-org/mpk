#!/usr/bin/env python3
"""Check evidence integrity, not a proof replay or completion of W03."""

import math
import sys

from entry_audit import read_json, require
from run_balanced_probes import DEPTHS, KINDS, PENDING, RECORD, ROOT, SCOPE, expected_cases, proof_sources, verify_case
from run_proof_pilot import sha


def check(report):
    require(report["schema"] == "mpk.go_practical.balanced_report.v0", "report schema")
    require(report["work_item"] == "GO-PRAC-T01-W03" and report["status"] == "word bridges measured; W03 remains incomplete", "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost prefix/source pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["internal_width_128_scope"] == "widened mathematical-domain probe; no Go 128-bit source integer admission", "invented source-width admission")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["measurement_child_ceiling_seconds"] == 60 and report["ceiling_scope"] == "private measurement, not a profile budget", "invented resource budget")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require([(g["kind"], g["depth"]) for g in report["construction_groups"]] == [(k,d) for k in KINDS for d in DEPTHS], "construction coverage")
    require([(r["kind"],r["case"]["depth"],r["case"]["mutation"]) for r in report["cases"]] == expected_cases(), "case coverage")
    for group in report["construction_groups"]:
        require(group["width"] == 1 << group["depth"], "construction width")
        for m in (group["first"],group["second"]):
            require(m["exit_code"] == 0, "construction failed"); resource(m)
    counts = {k: 0 for k in ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}
    for row in report["cases"]:
        kind,case = row["kind"],row["case"]
        signed = kind == "signed"
        depth,mutation = case["depth"],case["mutation"]
        require(case["width"] == 1 << depth and case["operand_binders"] == 2 and case["quantified_operand_bits"] == 2 << depth and case["initial_natural_quantified"] is True, "lost full operand quantification")
        require(case["operand_carrier"] == "Bool-addressed cube", "carrier identity")
        expected_id = ("signed-" if signed else "") + f"tree-{1<<depth}-" + (
            "nat-conservation" if mutation == 0 else
            ("dropped-result-sign" if signed else "dropped-carry") if mutation == 1 else
            ("unrelated-overflow-premise" if signed else "low-bit-wrong"))
        require(case["id"] == expected_id and case["expected"] == ("accepted" if mutation == 0 else "rejected"), "candidate identity")
        require(case["kind"] == ("signed-no-overflow-cross-sum" if signed else "unsigned-final-carry"), "arithmetic identity")
        require(case["scope"] == ("conditional two's-complement cross-sum equality in Nat; prefix sums and Go bindings remain open" if signed else
                                  "exact unsigned Nat value with final carry; prefix sums and Go bindings remain open"), "overstated candidate scope")
        premise = ("unrelated true equality (deliberate mutation)" if mutation == 2 else "actual sign-stage incoming/outgoing carry agreement") if signed else "none"
        require(case["required_premise"] == premise and case["incoming_carry"] == ("literal zero" if signed else "universally quantified Bool"), "lost signed precondition binding")
        require(case["theorem"] == "Probe.GoPractical.Word." + ("SignedTreeClaim" if signed else "TreeClaim"), "theorem identity")
        require(case["declarations"] == (55+22*depth if signed else 45+14*depth) and case["terms"] > 0, "declaration identity")
        raw = (RECORD / (case["id"] + ".mpcert")).read_bytes()
        require(len(raw) == case["bytes"] and row["raw_sha256"] == sha(raw), "certificate byte identity")
        require(set(row["observations"]) == {"rust", "go"}, "missing independent checker")
        try:
            classification = verify_case(case,row["observations"],kind)
        except RuntimeError as error:
            raise ValueError(str(error)) from error
        require(row["classification"] == classification, "resource failure counted as semantic negative")
        counts[classification] += 1
        for m in row["observations"].values():resource(m)
    require(report["outcomes"] == counts and counts["accepted_zero_axiom"] == 16, "outcome counts")
    return len(report["cases"])


def resource(m):
    require(m["resource_provider"] == "Darwin os.wait4 child rusage", "resource provider")
    require(type(m["peak_rss_bytes"]) is int and m["peak_rss_bytes"] > 0, "memory measurement")
    for k in ("wall_seconds","cpu_user_seconds","cpu_system_seconds"):
        v=m[k]; require(type(v) in (float,int) and math.isfinite(v) and v >= 0, "invalid resource measurement")


if __name__ == "__main__":
    try:
        count=check(read_json(ROOT,(RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {count} retained balanced candidates; evidence integrity only, W03 incomplete")
    except (OSError,KeyError,TypeError,ValueError) as error:
        print(f"Balanced report rejected: {error}",file=sys.stderr);raise SystemExit(1)
