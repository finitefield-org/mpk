#!/usr/bin/env python3
"""Check retained W03 pilot evidence integrity; this does not run a checker.

Actual source-free proof replay is run_proof_pilot.py --check. Keeping that
distinction explicit prevents a report-only success from becoming a new proof.
"""

import math
import sys

from entry_audit import read_json, require
from run_proof_pilot import RECORD, ROOT, semantic_expectation, sha, sources


def expected_cases():
    result = [("nominal-tag", "accepted"), ("multi-result-projection", "accepted"),
              ("nominal-tag-wrong", "rejected"), ("multi-result-projection-wrong", "rejected")]
    result.extend((name, "accepted") for name in (
        "closed-error-outcome", "return-frame-bit", "continue-post-bit", "full-adder-carry-bit",
        "backing-read-after-write-3"))
    result.append(("backing-read-after-write-3-wrong", "rejected"))
    for kind, capacities in (("string", (0, 1, 32, 256)), ("slice", (0, 1, 8, 32, 128))):
        for capacity in capacities:
            if capacity == 0:
                result.append((f"{kind}-capacity-0-length-zero", "accepted"))
                if kind == "slice":
                    result.append(("slice-nonnil-empty-tag", "accepted"))
            else:
                result.append((f"{kind}-capacity-{capacity}-cell-write", "accepted"))
                result.append((f"{kind}-capacity-{capacity}-cell-write-wrong", "rejected"))
    return result


def check(report):
    require(report["schema"] == "mpk.go_practical.proof_pilot_report.v0", "report schema")
    require(report["work_item"] == "GO-PRAC-T01-W03", "report work item")
    require(report["status"] == "partial feasibility; W03 is not complete", "overstated W03 status")
    require(report["scope"] == "ordinary universal representation/control/carry lemmas; no full Go function proof or public-profile acceptance", "overstated proof scope")
    require(report["pending"] == [
        "complete source/model bindings and validity invariants",
        "index/length/capacity and operation law closure",
        "full signed-integer and exact mathematical-sum bridge",
        "profile-wide resource/boundary freeze"], "lost pending requirements")
    require(report["inputs"] == sources(), "stale proof source inventory")
    require(report["deterministic_generations"] == 2, "missing deterministic generation")
    require(report["host"]["system"] == "Darwin" and report["host"]["machine"] == "arm64", "resource units/host confusion")
    index = read_json(ROOT, (RECORD / "index.json").relative_to(ROOT).as_posix())
    require(index["schema"] == "mpk.go_practical.proof_pilot.v0", "index schema")
    require([(r["case"]["id"], r["case"]["expected"]) for r in report["cases"]] == expected_cases(), "pilot coverage differs")
    require([r["case"] for r in report["cases"]] == index["cases"], "theorem index differs")
    for row in report["cases"]:
        item = row["case"]
        raw = (RECORD / (item["id"] + ".mpcert")).read_bytes()
        require(row["raw_sha256"] == sha(raw) and item["bytes"] == len(raw), "certificate byte identity differs")
        require(item["terms"] > 0 and item["quantified_binders"] > 0, "missing symbolic theorem")
        require(set(row["observations"]) == {"rust", "go"}, "missing independent checker")
        try:
            semantic_expectation(item, row["observations"])
        except RuntimeError as error:
            raise ValueError(str(error)) from error
        require(row["construction"]["exit_code"] == 0, "construction failed")
        for measurement in [row["construction"], *row["observations"].values()]:
            require(measurement["resource_provider"] == "Darwin os.wait4 child rusage", "resource provider differs")
            require(type(measurement["peak_rss_bytes"]) is int and measurement["peak_rss_bytes"] > 0, "missing memory measurement")
            for field in ("wall_seconds", "cpu_user_seconds", "cpu_system_seconds"):
                number = measurement[field]
                require(type(number) in (int, float) and math.isfinite(number) and number >= 0, "invalid resource measurement")
    return len(report["cases"])


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, (RECORD / "observations.json").relative_to(ROOT).as_posix()))
        print(f"PASS: {count} retained pilot candidates; evidence integrity only, W03 incomplete")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Go proof pilot report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
