"""Validate retained mathematical termination evidence without widening its scope."""
import re
from check_scope_report import exact, require, sha, strict_json
from run_variant_probes import RECORD, ROOT, MODULE, SCOPE, STATUS, PENDING, MUTATIONS, REGRESSIONS, inputs, classify
import variant_foundation_build as build
from run_proof_pilot import semantic_expectation


def check(report):
    require(set(report) == {"schema", "scope", "status", "pending", "inputs", "mirrored_checker_inputs", "checker_binaries",
                            "unchanged_checker_proposal", "host_toolchain_inventory", "generations", "direct_child_parallelism",
                            "child_ceiling_seconds", "ceiling_scope", "cases", "regressions", "outcomes"}, "variant closed report shape")
    require(report["schema"] == "mpk.go_practical.variant_termination_report.v0" and report["scope"] == SCOPE and report["status"] == STATUS,
            "variant mathematical scope is not source/public completion")
    require(report["pending"] == PENDING, "variant remaining source/freeze work")
    require(report["inputs"] == inputs(), "variant complete source identities")
    require(report["mirrored_checker_inputs"] == [{"path": p, "sha256": sha(raw)} for p, raw in sorted(build.prior.files().items())],
            "variant mirrored checker identities")
    require(report["unchanged_checker_proposal"] == "develop/migrations/go-practical/t01-w04/nat-termination/observations.json", "variant predecessor checker identity")
    predecessor = strict_json((ROOT / report["unchanged_checker_proposal"]).read_bytes())
    require(report["mirrored_checker_inputs"] == predecessor["mirrored_checker_inputs"], "variant unchanged checker sources")
    require(set(report["checker_binaries"]) == {"rust", "go"} and all(type(h) is str and re.fullmatch("[0-9a-f]{64}", h) for h in report["checker_binaries"].values()),
            "variant checker binary identities")
    require(report["checker_binaries"]["go"] == predecessor["checker_binaries"]["go"], "variant unchanged Go checker")
    require(exact(report["host_toolchain_inventory"], {"files": 14496, "sha256": "85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc"}), "variant exact toolchain")
    require(type(report["direct_child_parallelism"]) is int and report["direct_child_parallelism"] == 1
            and type(report["child_ceiling_seconds"]) is int and report["child_ceiling_seconds"] == 60
            and report["ceiling_scope"] == "private measurement setting, not normative profile limits", "variant private resource scope")
    require(len(report["generations"]) == 2 and all(type(r["exit_code"]) is int and r["exit_code"] == 0 for r in report["generations"]), "variant deterministic construction")
    require([r["case"]["id"] for r in report["cases"]] == ["variant-" + m for m in MUTATIONS], "variant premise/corollary coverage")
    index = strict_json((RECORD / "index.json").read_bytes())
    require(index["schema"] == "mpk.go_practical.variant_termination_pilot.v0" and exact([r["case"] for r in report["cases"]], index["cases"]), "variant exact case index")
    for row in report["cases"]:
        c = row["case"]
        expected = "accepted" if c["mutation"] in {"valid", "rank-count-valid", "zero-rank-exit"} else "rejected"
        require(c["expected"] == expected, "variant expected boundary")
        require(type(c["frame_bits"]) is int and c["frame_bits"] == 128 and c["iteration_domain"] == ("rank zero; arbitrary full frame; universal theorem instance" if c["mutation"] == "zero-rank-exit" else "arbitrary genuine Std.Nat; no finite unrolling"), "variant full-frame/arbitrary-Nat scope")
        raw = (RECORD / (c["id"] + ".mpcert")).read_bytes()
        require(row["raw_sha256"] == sha(raw) and type(c["bytes"]) is int and c["bytes"] == len(raw), "variant canonical candidate bytes")
        require(row["classification"] == classify(c, row["observations"]), "variant observation classification")
        for o in row["observations"].values():
            require(type(o["wall_seconds"]) in (int, float) and 0 <= o["wall_seconds"] < 60
                    and type(o["peak_rss_bytes"]) is int and o["peak_rss_bytes"] > 0, "variant child resource fields")
    require([(r["id"], r["path"]) for r in report["regressions"]] == list(REGRESSIONS), "variant retained regression coverage")
    for row in report["regressions"]:
        p = ROOT / row["path"]
        raw = bytes.fromhex(p.read_text()) if p.suffix == ".hex" else p.read_bytes()
        require(sha(raw) == row["raw_sha256"], "variant retained regression bytes")
        require(set(row["observations"]) == {"rust", "go"}, "variant independent regression checkers")
        r = row["observations"]["rust"]["report"]
        semantic_expectation({"expected": "accepted"}, row["observations"], declarations=r["declaration_count"], module=r["module"])
    require(exact(report["outcomes"], {"ordinary_accepted": 3, "ordinary_type_mismatch": 7, "legacy_regressions": 3}), "variant outcome counts")
    return 13


if __name__ == "__main__":
    print(f"PASS: {check(strict_json((RECORD / 'observations.json').read_bytes()))} retained variant/regression cases; W04 open")
