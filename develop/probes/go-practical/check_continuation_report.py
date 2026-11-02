#!/usr/bin/env python3
"""Retained prefix/zero-entry evidence integrity; not a proof replay."""

import json
from pathlib import PurePosixPath
import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
import quote_differential
from run_prefix_probes import depth
from run_proof_pilot import sha
from run_continuation_probes import BOUNDARY_SCOPE, BOUNDED_PREMISES, CASE_SCOPE, COUNTS, PENDING, RECORD, REPORT_SCHEMA, ROOT, SCOPE, STATUS, ZERO_PREMISES, declaration_count, expected_cases, identifier, proof_sources, verify_case


def check(report):
    require(report["schema"] == REPORT_SCHEMA and report["work_item"] == "GO-PRAC-T01-W03", "continuation report schema")
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
        for m in (group["first"], group["second"]):
            require(m["exit_code"] == 0, "construction failed")
            resource(m)
    outcomes = {k: 0 for k in ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}
    for row in report["cases"]:
        case = row["case"]
        n, family, mutation = case["count"], case["family"], case["mutation"]
        d, rooted, natural = depth(n), family.startswith("zero-"), family.endswith("natural")
        require(case["id"] == identifier(n, family, mutation) and case["expected"] == ("accepted" if mutation == 0 or n == 0 and mutation == 1 else "rejected"), "candidate identity")
        require(case["address_depth"] == d and case["storage_capacity"] == 1 << d, "backing capacity identity")
        require(case["source_word_bits"] == 64 and case["quantified_limit_bits"] == 64 and case["quantified_storage_bits"] == 64 << d, "lost full payload quantification")
        require(case["quantified_accumulator_bits"] == (0 if rooted else 64) and case["zero_initialized"] is rooted, "initial accumulator binding")
        require(case["initial_natural_quantified"] is natural, "natural observer quantification")
        require(case["premises"] == (ZERO_PREMISES if rooted else BOUNDED_PREMISES), "lost actual item checks")
        require(case["scope"] == CASE_SCOPE, "overstated candidate scope")
        suffix = {"state": "ContinuationStateClaim", "natural": "ContinuationNatClaim", "zero-state": "ContinuationZeroStateClaim", "zero-natural": "ContinuationZeroNatClaim"}[family]
        require(case["theorem"] == "Probe.GoPractical.Word." + suffix and case["declarations"] == declaration_count(n), "theorem identity")
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
    require(report["outcomes"] == outcomes and outcomes == {"accepted_zero_axiom": 36, "core_type_mismatch": 36, "candidate_fuel_failure_not_false_claim_evidence": 0}, "outcome counts")
    observations = report["quote_observations"]
    quote_differential.check(observations, RECORD)
    require(observations["go_measurement"]["exit_code"] == 0, "Go Quote did not execute")
    require(re.fullmatch(r"[0-9a-f]{64}", observations["go_binary_sha256"]) is not None, "Go Quote binary identity")
    resource(observations["go_measurement"])
    return len(report["cases"])


def check_initial():
    record = RECORD.parent / "continuation-initial"
    report = json.loads((record / "observations.json").read_text())
    require(report["schema"] == "mpk.go_practical.continuation_initial.v0" and report["scope"] == "failed positive zero-complement carry ingredient; not semantic-negative or completion evidence", "initial finding scope")
    paths = [r["path"] for r in report["inputs"]]
    require(len(paths) == len(set(paths)), "duplicate initial source")
    required = {"develop/probes/go-practical/certificates/src/word/balanced/signed/fold/continuation.rs", "crates/mpk-core/src/defeq.rs", "crates/mpk-kernel/src/verifier.rs", "fixtures/core-bool-cases/right-identity.hex", "proofs/std/nat/std-nat.hex"}
    require(required.issubset(paths), "initial source closure")
    for item in report["inputs"]:
        relative = PurePosixPath(item["path"])
        require(not relative.is_absolute() and ".." not in relative.parts, "initial source path")
        snapshot = record / "source" / item["path"].removeprefix("develop/probes/go-practical/")
        source = snapshot if snapshot.exists() else ROOT / relative
        require(sha(source.read_bytes()) == item["sha256"], "initial source identity")
    require(sha((record / "zero-initial.mpcert").read_bytes()) == report["raw_sha256"], "initial certificate identity")
    require(set(report["binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["binaries"].values()), "initial checker binary identity")
    require(set(report["observations"]) == {"rust", "go"}, "initial independent checkers")
    for checker, m in report["observations"].items():
        r = m["report"]
        require(m["exit_code"] == 1 and r["verdict"] == "rejected", "initial positive failure")
        if checker == "rust":
            require(r["error_code"] == "KERNEL_CORE_CHECK" and json.loads(r["error_detail"])["code"] == "CORE_TYPE_MISMATCH", "initial type-failure classification")
        else:
            require(r["error_kind"] == "core_check" and "inferred" in r["error_detail"] and "expected" in r["error_detail"], "initial type-failure classification")
        resource(m)
    return 1


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        check_initial()
        print(f"PASS: {count} continuing-prefix candidates, original zero finding and empirical Go observations; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Continuation report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
