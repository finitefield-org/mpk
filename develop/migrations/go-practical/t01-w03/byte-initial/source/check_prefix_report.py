#!/usr/bin/env python3
"""Check retained prefix evidence integrity; this is not proof replay."""

import json
from pathlib import PurePosixPath
import re
import sys

from check_balanced_report import resource
from entry_audit import read_json, require
from run_prefix_probes import (
    BOUNDARY_SCOPE, CONDITIONAL_SCOPE, COUNTS, PENDING, RAW_SCOPE, RECORD, ROOT,
    SCOPE, accepted, declaration_count, depth, expected_cases, identifier, proof_sources, verify_case,
)
from run_proof_pilot import sha


def check(report):
    require(report["schema"] == "mpk.go_practical.prefix_report.v0", "prefix report schema")
    require(report["work_item"] == "GO-PRAC-T01-W03" and report["status"] == "prefix ingredients measured; W03 remains incomplete", "invented W03 completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost source/signed pending scope")
    require(report["inputs"] == proof_sources(), "stale source identity")
    require(report["deterministic_generations"] == 2, "missing deterministic construction")
    require(report["boundary_probe_scope"] == BOUNDARY_SCOPE, "invented profile admission")
    require(report["build"] == "Rust release offline locked; Go 1.25.0 pinned host trimpath", "checker build identity")
    require(report["measurement_child_ceiling_seconds"] == 60 and report["ceiling_scope"] == "private measurement, not a profile budget", "invented resource budget")
    require(set(report["checker_binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["checker_binaries"].values()), "checker binary identity")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require([g["count"] for g in report["construction_groups"]] == list(COUNTS), "construction coverage")
    require([(r["case"]["count"], r["case"]["family"], r["case"]["mutation"]) for r in report["cases"]] == expected_cases(), "case coverage")
    for group in report["construction_groups"]:
        require(group["scope"] == "complete candidate group and shared ordinary library", "construction scope")
        for m in (group["first"], group["second"]):
            require(m["exit_code"] == 0, "construction failed")
            resource(m)
    counts = {k: 0 for k in ("accepted_zero_axiom", "core_type_mismatch", "candidate_fuel_failure_not_false_claim_evidence")}
    for row in report["cases"]:
        case = row["case"]
        n, family, mutation = case["count"], case["family"], case["mutation"]
        d = depth(n)
        require(case["address_depth"] == d and case["storage_capacity"] == 1 << d, "backing capacity identity")
        require(case["source_word_bits"] == 64 and case["quantified_storage_bits"] == 64 << d and case["quantified_accumulator_bits"] == 64 and case["initial_natural_quantified"] is True, "lost full payload quantification")
        require(case["id"] == identifier(n, family, mutation) and case["expected"] == ("accepted" if accepted(n, family, mutation) else "rejected"), "candidate identity")
        require(case["scope"] == (CONDITIONAL_SCOPE if family == "conditional" else RAW_SCOPE), "overstated candidate scope")
        premise = "none" if family == "unconditional" else "all inactive false equalities (vacuous only for empty prefix)" if mutation == 2 else "each actual intermediate unsigned carry equals false"
        require(case["premise"] == premise, "lost actual carry premise")
        require(case["theorem"] == "Probe.GoPractical.Word." + ("PrefixClaim" if family == "conditional" else "PrefixRawClaim"), "theorem identity")
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
    require(report["outcomes"] == counts and counts["accepted_zero_axiom"] == 18, "outcome counts")
    return len(report["cases"])


def check_initial():
    record = RECORD.parent / "prefix-initial"
    report = json.loads((record / "observations.json").read_text())
    require(report["schema"] == "mpk.go_practical.prefix_initial.v0" and report["scope"] == "failed preliminary prefix positive, including empty prefix; not completion", "initial finding scope")
    pins = report["inputs"]
    paths = [p["path"] for p in pins]
    require(len(paths) == len(set(paths)), "duplicate initial source")
    required = {"develop/probes/go-practical/certificates/src/" + p for p in (
        "main.rs", "natural.rs", "word.rs", "word/balanced.rs", "word/balanced/signed.rs", "word/balanced/signed/fold.rs")}
    required.update(("crates/mpk-core/src/defeq.rs", "crates/mpk-kernel/src/verifier.rs",
                     "fixtures/core-bool-cases/right-identity.hex", "proofs/std/nat/std-nat.hex",
                     "go-tools/mpk-checker-ref/cmd/mpk-checker-ref/main.go"))
    require(required.issubset(paths), "missing initial source closure")
    for item in pins:
        relative = PurePosixPath(item["path"])
        require(not relative.is_absolute() and ".." not in relative.parts, "invalid initial source path")
        snapshot = record / "source" / item["path"].removeprefix("develop/probes/go-practical/")
        source = snapshot if snapshot.exists() else ROOT / relative
        require(sha(source.read_bytes()) == item["sha256"], "initial source identity")
    require(set(report["binaries"]) == {"rust", "go"} and all(re.fullmatch(r"[0-9a-f]{64}", h) for h in report["binaries"].values()), "initial checker binary identity")
    require(sha((record / "prefix-0-exact-nat-sum.mpcert").read_bytes()) == report["raw_sha256"], "initial certificate identity")
    require(set(report["observations"]) == {"rust", "go"}, "missing initial independent checker")
    for checker, m in report["observations"].items():
        resource(m)
        r = m["report"]
        require(m["exit_code"] == 1 and r["verdict"] == "rejected", "initial positive failure")
        if checker == "rust":
            require(r["error_code"] == "KERNEL_CORE_CHECK" and json.loads(r["error_detail"])["code"] == "CORE_FUEL_EXHAUSTED", "initial fuel classification")
        else:
            require(r["error_kind"] == "core_check" and r["error_detail"] == "core reduction fuel exhausted", "initial fuel classification")
    return 1


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        check_initial()
        print(f"PASS: {count} retained prefix candidates and original fuel finding; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Prefix report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
