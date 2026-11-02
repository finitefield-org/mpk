#!/usr/bin/env python3
"""Retained slice evidence integrity, distinct from actual proof replay."""

import json
from pathlib import PurePosixPath
import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
from run_proof_pilot import sha
from run_slice_probes import BOUNDARY_SCOPE, CASE_SCOPE, COUNTS, PENDING, RECORD, REPORT_SCHEMA, ROOT, SCOPE, STATUS, declaration_count, depth, expected_cases, identifier, proof_sources, verify_case
import slice_differential


def check(report):
    require(report["schema"] == REPORT_SCHEMA and report["work_item"] == "GO-PRAC-T01-W03", "slice report schema")
    require(report["status"] == STATUS, "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost slice validity/operation pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["boundary_probe_scope"] == BOUNDARY_SCOPE, "invented Go capacity admission")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["host_toolchain_inventory"] == {"files": 14496, "sha256": "85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc"}, "host toolchain inventory")
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
        require(type(case["address_depth"]) is int and case["address_depth"] == depth(n), "finite storage identity")
        require(case["source_word_bits"] == 64 and case["descriptor_bits"] == 512 and case["element_bits"] == 128, "lost full source/descriptor/element domain")
        require(case["logical_region_count"] == 2 and case["source_pointer_bits_admitted"] is False, "logical region/pointer scope")
        require(case["quantified_storage_bits"] == 128 << (depth(n) + 1), "lost complete backing/padding domain")
        require(case["scope"] == CASE_SCOPE, "overstated candidate scope")
        require(case["theorem"] == "Probe.GoPractical.Word.SliceClaim" + family.replace("-", "") and case["declarations"] == declaration_count(n), "theorem identity")
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
    require(report["outcomes"] == outcomes and outcomes == {"accepted_zero_axiom": 45, "core_type_mismatch": 45, "candidate_fuel_failure_not_false_claim_evidence": 0}, "outcome counts")
    observed = report["slice_observations"]
    slice_differential.check(observed, RECORD)
    require([w["case_id"] for w in observed["negative_claim_witnesses"]] == [r["case"]["id"] for r in report["cases"] if r["case"]["mutation"]], "false-claim witness coverage")
    require(observed["go_measurement"]["exit_code"] == 0, "Go slices did not execute")
    require(re.fullmatch(r"[0-9a-f]{64}", observed["go_binary_sha256"]) is not None, "Go slice binary identity")
    resource(observed["go_measurement"])
    return len(report["cases"])


def check_initial(report=None):
    archive = RECORD.parent / "slice-initial"
    if report is None:
        report = json.loads((archive / "observations.json").read_text())
    require(report["schema"] == "mpk.go_practical.slice_initial.v0" and report["scope"] == "failed positive library attempts; diagnostic only, not false-claim or completion evidence", "initial slice scope")
    require([g["id"] for g in report["groups"]] == ["tag-predicate", "inline-views"], "initial slice finding coverage")
    count = 0
    for group in report["groups"]:
        require(set(group["checker_binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in group["checker_binaries"].values()), "initial checker identity")
        for item in group["inputs"]:
            path = PurePosixPath(item["path"])
            require(not path.is_absolute() and ".." not in path.parts, "initial source path")
            snapshot = archive / group["id"] / "source" / item["path"].removeprefix("develop/probes/go-practical/")
            source = snapshot if snapshot.exists() else ROOT / path
            require(sha(source.read_bytes()) == item["sha256"], "initial source identity")
        expected_files = ["original-library.mpcert", "stage-constructor.mpcert"] if group["id"] == "tag-predicate" else ["stage-alias.mpcert"]
        require([r["file"] for r in group["cases"]] == expected_files, "initial raw finding coverage")
        for row in group["cases"]:
            raw = (archive / group["id"] / row["file"]).read_bytes()
            require(sha(raw) == row["raw_sha256"], "initial certificate identity")
            require(set(row["observations"]) == {"rust", "go"}, "initial independent checkers")
            fuel = group["id"] == "inline-views"
            for checker, m in row["observations"].items():
                r = m["report"]
                require(m["exit_code"] == 1 and r["verdict"] == "rejected", "initial failed positive attempt")
                if checker == "rust":
                    require(r["error_code"] == "KERNEL_CORE_CHECK" and json.loads(r["error_detail"])["code"] == ("CORE_FUEL_EXHAUSTED" if fuel else "CORE_TYPE_MISMATCH"), "initial failure class")
                    require(r["hashes"]["certificate"] == sha(b"MPK-MODULE-CERT-0.1" + bytes([0]) + raw), "initial checker/certificate identity")
                else:
                    require(r["error_kind"] == "core_check" and (r["error_detail"] == "core reduction fuel exhausted" if fuel else " inferred " in r["error_detail"] and " but expected " in r["error_detail"]), "initial failure class")
                resource(m)
            count += 1
    return count


if __name__ == "__main__":
    try:
        n = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        check_initial()
        print(f"PASS: {n} slice-store candidates, actual-Go observations and original positive diagnostics; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Slice report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
