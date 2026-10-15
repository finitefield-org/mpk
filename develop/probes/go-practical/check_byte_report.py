#!/usr/bin/env python3
"""Retained finite-byte evidence integrity, distinct from source proof replay."""

import re
import json
from pathlib import PurePosixPath
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
import byte_differential
from run_proof_pilot import sha
from run_byte_probes import BACKING_SCOPE, BOUNDARY_SCOPE, CASE_SCOPE, COUNTS, PENDING, RECORD, REPORT_SCHEMA, ROOT, SCOPE, STATUS, accepted, declaration_count, depth, expected_cases, identifier, proof_sources, verify_case


def check(report):
    require(report["schema"] == REPORT_SCHEMA and report["work_item"] == "GO-PRAC-T01-W03", "byte report schema")
    require(report["status"] == STATUS, "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost byte/source pending scope")
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
        d = depth(n)
        require(case["count"] == n and case["id"] == identifier(n, family, mutation) and case["expected"] == ("accepted" if accepted(n, family, mutation) else "rejected"), "candidate identity")
        require(case["address_depth"] == d, "finite capacity identity")
        require(case["source_word_bits"] == 64 and case["full_length_bits"] == 64 and case["full_index_bits"] == 64 and case["payload_bits"] == 8, "lost full source or byte domain")
        require(case["quantified_storage_bits"] == 8 << d and case["two_stores"] is (family in ("equality", "lexicographic")), "lost complete finite backing")
        require(case["backing"] == BACKING_SCOPE and case["scope"] == CASE_SCOPE, "overstated candidate scope")
        require(case["theorem"] == "Probe.GoPractical.Word.ByteTailClaim" + family.replace("-", "") and case["declarations"] == declaration_count(n), "theorem identity")
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
    require(report["outcomes"] == outcomes and outcomes == {"accepted_zero_axiom": 25, "core_type_mismatch": 21, "candidate_fuel_failure_not_false_claim_evidence": 0}, "outcome counts")
    observed = report["byte_observations"]
    byte_differential.check(observed, RECORD)
    require(observed["go_measurement"]["exit_code"] == 0, "Go bytes did not execute")
    require(re.fullmatch(r"[0-9a-f]{64}", observed["go_binary_sha256"]) is not None, "Go byte binary identity")
    resource(observed["go_measurement"])
    return len(report["cases"])


def check_initial():
    archive = RECORD.parent / "byte-initial"
    report = json.loads((archive / "observations.json").read_text())
    require(report["schema"] == "mpk.go_practical.byte_initial.v0" and report["scope"] == "failed proof attempts for true one-cell index-coherence claims; not false-claim or completion evidence", "initial true-claim scope")
    for item in report["inputs"]:
        relative = PurePosixPath(item["path"])
        require(not relative.is_absolute() and ".." not in relative.parts, "initial source path")
        snapshot = archive / "source" / item["path"].removeprefix("develop/probes/go-practical/")
        source = snapshot if snapshot.exists() else ROOT / relative
        require(sha(source.read_bytes()) == item["sha256"], "initial source identity")
    require([(r["case"]["capacity"], r["case"]["family"], r["case"]["mutation"]) for r in report["cases"]] == [(n, "index-extensional", 1) for n in (0, 1)], "initial finding coverage")
    for row in report["cases"]:
        raw = (archive / (row["case"]["id"] + ".mpcert")).read_bytes()
        require(sha(raw) == row["raw_sha256"], "initial certificate identity")
        require(set(row["observations"]) == {"rust", "go"}, "initial independent checkers")
        for checker, measured in row["observations"].items():
            r = measured["report"]
            require(measured["exit_code"] == 1 and r["verdict"] == "rejected", "initial failed proof")
            if checker == "rust":
                require(r["error_code"] == "KERNEL_CORE_CHECK" and json.loads(r["error_detail"])["code"] == "CORE_TYPE_MISMATCH", "initial type failure")
            else:
                require(r["error_kind"] == "core_check" and " inferred " in r["error_detail"] and " but expected " in r["error_detail"], "initial type failure")
            resource(measured)
    return len(report["cases"])


if __name__ == "__main__":
    try:
        n = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        check_initial()
        print(f"PASS: {n} byte-tail candidates and empirical original-Go observations; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Byte report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
