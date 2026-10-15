#!/usr/bin/env python3
"""Check retained W02 observation identity/coverage without rerunning Go.

This checks evidence integrity and expected observations, never proof
acceptance. Full measurement replay is run_module_probes.py --check.
"""

import re
import sys

from entry_audit import read_json, require
from run_module_probes import HERE, ROOT, REPORT, digest, verify_expectations


def check(report):
    matrix = read_json(ROOT, "develop/probes/go-practical/module-cases.json")
    pin = read_json(ROOT, "develop/probes/go-practical/host-toolchain.json")
    require(report["schema"] == "mpk.go_practical.module_probe_report.v0", "report schema")
    require(report["work_item"] == "GO-PRAC-T01-W02", "report work item")
    require(report["compiler_version"] == "go version go1.25.0 darwin/arm64", "compiler observation")
    require(report["target"] == "linux/amd64" and report["host"] == "darwin/arm64", "host/target confusion")
    for key, filename in (("probe_source_sha256", "module_probe.go"),
                          ("runner_sha256", "run_module_probes.py"),
                          ("case_matrix_sha256", "module-cases.json")):
        require(report[key] == digest((HERE / filename).read_bytes()), f"stale source pin: {filename}")
    require(report["host_archive_sha256"] == pin["archive_sha256"], "host archive pin")
    require(report["specification_sha256"] == pin["shared_source_witness"]["sha256"], "specification pin")
    require(report["deterministic_passes"] == 2, "two-pass observation missing")
    require(report["dependency_snapshot_unchanged"] is True, "dependency snapshot changed")
    require(report["go_downloads_during_measurement"] is False, "Go downloads enabled")
    require("no OS-level" in report["network_scope"], "unsubstantiated network-isolation claim")
    require("not VIR, VC, theorem" in report["scope"], "evidence scope overstated")
    require(report["case_count"] == len(matrix["cases"]), "case count differs")
    require([x["id"] for x in report["cases"]] == [x["id"] for x in matrix["cases"]], "case membership/order differs")
    for case, observed in zip(matrix["cases"], report["cases"]):
        require(observed["source_snapshot_unchanged"] is True, "source snapshot changed")
        require(observed["application_init_or_generate_executed"] is False, "application code executed")
        require(observed["request"]["root"] == "<module-root>", "temporary root leaked")
        require(observed["request"]["package"] == matrix["module"], "original package changed")
        verify_expectations(case, observed["observations"])
        fixture = {x["path"]: x for x in observed["fixture_inputs"]}
        for name, source in case["files"].items():
            require(name in fixture and fixture[name]["sha256"] == digest(source.encode()), f"rewritten fixture source: {case['id']}/{name}")
        for file in observed["observations"]["observed_loaded_inputs"]:
            if file["path"].startswith("<module-root>/"):
                relative = file["path"].removeprefix("<module-root>/")
                require(relative in fixture and file["sha256"] == fixture[relative]["sha256"], f"loaded source identity differs: {case['id']}/{relative}")
            require(bool(re.fullmatch(r"[0-9a-f]{64}", file["sha256"])), "invalid file digest")
    return len(matrix["cases"])


if __name__ == "__main__":
    try:
        count = check(read_json(ROOT, REPORT.relative_to(ROOT).as_posix()))
        print(f"PASS: {count} retained cases; exact source pins, original inputs, scopes and observations")
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f"Go module report rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
