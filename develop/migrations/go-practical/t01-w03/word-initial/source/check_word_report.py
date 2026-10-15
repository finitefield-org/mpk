#!/usr/bin/env python3
"""Check retained unsigned-word evidence identity; full proof replay is separate."""

import math
import sys

from entry_audit import read_json, require
from run_word_pilot import PENDING, RECORD, ROOT, SCOPE, WIDTHS, verify_case, word_sources
from run_proof_pilot import sha


def check(report):
    require(report["schema"] == "mpk.go_practical.word_report.v0", "word report schema")
    require(report["work_item"] == "GO-PRAC-T01-W03", "word work item")
    require(report["status"] == "unsigned word pilot measured; W03 remains incomplete", "invented completion")
    require(report["scope"] == SCOPE and report["pending"] == PENDING, "lost signed/sum/source pending scope")
    require(report["inputs"] == word_sources(), "stale word source identity")
    require(report["deterministic_generations"] == 2, "missing deterministic generation")
    require(report["measurement_child_ceiling_seconds"] == 600 and report["ceiling_scope"] ==
            "private measurement infrastructure, not a frozen Go profile budget", "invented profile budget")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource host units")
    require(report["build"] == {"rust": "release optimized, offline locked private crate", "go": "Go 1.25.0 pinned host, trimpath, no downloads"}, "checker build identity")
    require([g["width"] for g in report["construction_groups"]] == list(WIDTHS), "construction coverage")
    expected = [(f"word-{w}-nat-conservation{s}", w, e) for w in WIDTHS for s, e in (("", "accepted"), ("-wrong", "rejected"))]
    require([(r["case"]["id"], r["case"]["width"], r["case"]["expected"]) for r in report["cases"]] == expected, "word coverage")
    for group in report["construction_groups"]:
        for measurement in (group["first"], group["second"]):
            require(measurement["exit_code"] == 0, "construction failed")
            resource(measurement)
    for row in report["cases"]:
        item = row["case"]
        raw = (RECORD / (item["id"] + ".mpcert")).read_bytes()
        require(item["bytes"] == len(raw) and row["raw_sha256"] == sha(raw), "word certificate identity")
        require(item["theorem"] == "Probe.GoPractical.Word.Claim" and item["declarations"] == 39 + 6 * item["width"], "word theorem identity")
        require(item["operand_binders"] == 2 * item["width"] and item["initial_natural_quantified"] is True, "lost universal operand scope")
        require(row["construction_group"] == item["width"] and item["terms"] > 0, "word construction binding")
        require(set(row["observations"]) == {"rust", "go"}, "missing independent checker")
        try:
            verify_case(item, row["observations"])
        except RuntimeError as error:
            raise ValueError(str(error)) from error
        for measurement in row["observations"].values():
            resource(measurement)
    return len(report["cases"])


def resource(measurement):
    require(measurement["resource_provider"] == "Darwin os.wait4 child rusage", "resource provider")
    require(type(measurement["peak_rss_bytes"]) is int and measurement["peak_rss_bytes"] > 0, "memory measurement")
    for field in ("wall_seconds", "cpu_user_seconds", "cpu_system_seconds"):
        number = measurement[field]
        require(type(number) in (int, float) and math.isfinite(number) and number >= 0, "invalid resource measurement")


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {count} retained word candidates; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Word proof report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
