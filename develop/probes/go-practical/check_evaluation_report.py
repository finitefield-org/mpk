"""Validate retained evaluation facts; never promote observations to proofs."""
import re
from check_scope_report import exact, json_bytes, require, sha, strict_json
from evaluation_cases import HERE, PENDING, RECORD, ROOT, SCOPE, VECTORS, expected, inputs, vectors


def sources():
    paths = [HERE / name for name in (
        "evaluation_reference.go", "evaluation_cases.py", "run_evaluation_probes.py",
        "check_evaluation_report.py", "test_evaluation_report.py",
        "host-toolchain.json", "run_module_probes.py", "check_scope_report.py",
    )] + [VECTORS]
    return [{"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
             "sha256": sha(p.read_bytes())} for p in sorted(paths)]


def check(report, record=RECORD):
    require(set(report) == {"schema", "scope", "status", "pending", "inputs", "toolchain_inventory",
                            "observer_binary_sha256", "build", "artifacts", "cases", "source_forms"},
            "evaluation report closed shape")
    require(report["schema"] == "mpk.go_practical.w04_evaluation_observations.v0"
            and report["scope"] == SCOPE and report["status"] == "native facts retained; W04 in progress",
            "evaluation scope is not proof/admission")
    require(report["pending"] == PENDING, "remaining evaluation work")
    require(exact(strict_json(VECTORS.read_bytes()), vectors()), "evaluation draft vectors")
    require(exact(report["inputs"], sources()), "evaluation source identity")
    require(exact(report["toolchain_inventory"], {
        "files": 14496, "sha256": "85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc"}),
        "evaluation pinned Go distribution")
    require(re.fullmatch("[0-9a-f]{64}", report["observer_binary_sha256"]) is not None,
            "evaluation binary identity")
    require(exact(report["build"], {
        "release": "go1.25.0", "host": "darwin/arm64", "trimpath": True, "buildvcs": False,
        "specification_sha256": "42a4ff1c01efe8dc23eca4378b8fb03d76e9f2f079dab01298ffd74d8b0e3149",
        "runtime_scope": "host facts, not target release or verified application"}),
        "evaluation build/scope identity")
    require(set(report["artifacts"]) == {"native-input.json", "native-output.json"}, "evaluation artifact coverage")
    for name, pin in report["artifacts"].items():
        raw = (record / name).read_bytes()
        require(exact(pin, {"bytes": len(raw), "sha256": sha(raw)}), "evaluation raw artifact identity")
    require(exact(strict_json((record / "native-input.json").read_bytes()), inputs()),
            "evaluation complete input coverage")
    output = strict_json((record / "native-output.json").read_bytes())
    require(exact(output, [expected(row) for row in inputs()]), "independent evaluation facts")
    require(type(report["cases"]) is int and report["cases"] == len(inputs())
            and type(report["source_forms"]) is int
            and report["source_forms"] == len({r["id"] for r in inputs()}), "evaluation counts")
    return len(inputs())


if __name__ == "__main__":
    print(f"PASS: {check(strict_json((RECORD/'observations.json').read_bytes()))} original-Go evaluation cases; no profile/proof promotion")
