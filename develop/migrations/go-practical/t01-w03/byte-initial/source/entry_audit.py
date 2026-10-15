#!/usr/bin/env python3
"""Record/check the GO-PRAC W01 baseline; never select a production profile.

Run against the W01 source baseline. Later intentional production changes
need an explicit comparison against these retained hashes, not regeneration
that silently replaces the baseline. Git identity/cleanliness is recorded by
the separate /usr/bin/git audit in the ledger; this helper never invokes Git.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[3]
SELECTION = "develop/migrations/go-practical/t01-w01/baseline-selection.json"
REPORT = "develop/migrations/go-practical/t01-w01/entry-audit.json"
CONSUMER_PATTERN = re.compile(
    r"GoFixedV0|mpk\.go\.fixed\.v0|GoFixedConfiguration|goSemanticProfile"
)
EXTRA_OWNERS = (
    "go-tools/go2vir/loader.go",
    "go-tools/go2vir/snapshot.go",
    "go-tools/go2vir/preflight.go",
    "go-tools/go2vir/features.go",
    "go-tools/go2vir/lower_types.go",
    "go-tools/go2vir/contract.go",
    "go-tools/go2vir/source_map.go",
    "go-tools/go2vir/protocol.go",
    "crates/mpk-vc/src/vir.rs",
    "crates/mpk-vc/src/program_wp.rs",
    "crates/mpk-vc/src/call_wp.rs",
    "crates/mpk-vc/src/type_encode.rs",
    "crates/mpk-cli/src/successor_frontend_protocol.rs",
    "crates/mpk-cli/src/successor_frontend_runner.rs",
    "crates/mpk-cli/src/successor_release_bundle.rs",
    "crates/mpk-cli/tests/successor_atomic_cutover.rs",
)


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def read_bytes(root, relative):
    path = root / relative
    if Path(relative).is_absolute() or ".." in Path(relative).parts:
        raise ValueError(f"non-relative input: {relative}")
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError(f"input escapes root: {relative}")
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"input is not a regular file: {relative}")
    return path.read_bytes()


def read_json(root, relative):
    return json.loads(read_bytes(root, relative), object_pairs_hook=strict_object)


def record(root, relative):
    raw = read_bytes(root, relative)
    return {
        "path": relative,
        "bytes": len(raw),
        "sha256": hashlib.sha256(raw).hexdigest(),
    }


def require(condition, message):
    if not condition:
        raise ValueError(message)


def audit(root):
    selection = read_json(root, SELECTION)
    require(selection["schema"] == "mpk.go_practical.baseline_selection.v0", "selection schema")
    for key in ("source_baseline_commit", "source_baseline_tree", "prior_main_commit"):
        require(bool(re.fullmatch(r"[0-9a-f]{40}", selection[key])), f"invalid {key}")
    require(selection["source_baseline_clean_at_capture"] is True, "baseline was not clean")
    require(selection["effective_language_versions"] == ["go1.22", "go1.23", "go1.24", "go1.25"], "language proposal drift")
    require(selection["target"] == {"id": "linux/amd64", "pointer_width": 64, "goamd64": "v1"}, "target proposal drift")

    semantic_path = "release/bundles/semantic-profile-registry.json"
    bundle_path = "release/bundles/bundle-registry.json"
    semantic = read_json(root, semantic_path)
    bundles = read_json(root, bundle_path)
    receipt_path = selection["installed_release_receipt"]
    receipt = read_json(root, receipt_path)
    require(receipt["status"] == "accepted" and receipt["exit_code"] == 0, "historical release receipt is not accepted")
    require(receipt["installed_release_passes"] == 2, "historical receipt lacks two passes")
    for name, data, path in (("semantic_registry", semantic, semantic_path), ("bundle_registry", bundles, bundle_path)):
        require(receipt[name]["raw_sha256"] == record(root, path)["sha256"], f"{name} bytes differ from installed receipt")
        require(receipt[name]["registry_sha256"] == data["registry_sha256"], f"{name} semantic identity differs")
    require(semantic["revision"] == receipt["semantic_registry"]["revision"], "registry revision mismatch")
    go_entries = [p for p in semantic["profiles"] if p["source_language"] == "go"]
    require(len(go_entries) == 1 and go_entries[0]["semantic_profile"] == selection["active_go_profile"], "unexpected active Go profile")
    require(not any(p["semantic_profile"].startswith("mpk.go.practical.") for p in semantic["profiles"]), "practical Go already active")
    tuples = [t for t in bundles["tuples"] if t["semantic_context"]["source_language"] == "go"]
    require(len(tuples) == 1, "Go tuple count differs")
    go_tuple = tuples[0]
    require(go_tuple["semantic_context"]["profile_entry_sha256"] == go_entries[0]["entry_sha256"], "Go entry mismatch")
    require(go_tuple["toolchain_bundle_id"] == selection["toolchain_bundle_id"], "compiler bundle mismatch")
    parameters = go_tuple["semantic_context"]["semantic_parameters"]["value"]
    require(parameters == {"target_id": "linux/amd64", "pointer_width": 64}, "registered Go target mismatch")
    toolchains = [b for b in bundles["toolchain_bundles"] if b["bundle_id"] == selection["toolchain_bundle_id"]]
    require(len(toolchains) == 1, "compiler bundle not unique")
    toolchain = toolchains[0]
    require(all(c["release"] == selection["compiler_release"] for c in toolchain["components"]), "compiler release mismatch")
    compiler = next(c for c in toolchain["components"] if c["kind"] == "executable" and c["name"] == "go")
    content = next(c for c in toolchain["components"] if c["kind"] == "content")
    witness_paths = {"go/VERSION", "go/doc/go_spec.html", "go/src/go/types/version.go", "go/src/go/version/version.go"}
    compiler_witnesses = [f for f in content["inventory"]["files"] if f["path"] in witness_paths]
    require({f["path"] for f in compiler_witnesses} == witness_paths, "compiler/spec witnesses incomplete")

    frontend = read_json(root, "fixtures/vir-go/frontend-index.json")
    derived = read_json(root, "fixtures/vir-go/derived-index.json")
    require(frontend["schema"] == "mpk.go_vir_frontend_corpus.v1", "frontend corpus schema")
    require(derived["schema"] == "mpk.go_vir_derived_corpus.v1", "derived corpus schema")
    require(frontend["positive_source_count"] == len(frontend["cases"]), "positive corpus count")
    require(frontend["negative_source_count"] == len(frontend["negative_cases"]), "negative corpus count")
    artifact_count = 0
    for case in frontend["cases"] + derived["cases"] + frontend["negative_cases"]:
        artifacts = case.get("artifacts", [case["artifact"]] if "artifact" in case else [])
        for artifact in artifacts:
            actual = record(root, "fixtures/vir-go/" + artifact["path"])
            require(actual["bytes"] == artifact["bytes"] and actual["sha256"] == artifact["sha256"], f"corpus drift: {actual['path']}")
            artifact_count += 1
    vectors_path = "develop/specs/vectors/go-vir-profile-v0.json"
    vectors = read_json(root, vectors_path)
    groups = ("profile_cases", "capture_cases", "source_cases", "operation_cases", "contract_cases", "loop_call_cases", "diagnostic_cases", "limit_cases")
    vector_ids = [row["id"] for group in groups for row in vectors[group]]
    require(len(vector_ids) == 130 and len(set(vector_ids)) == 130, "fixed Go vector inventory differs")

    consumers = set(EXTRA_OWNERS)
    references = set()
    for area in ("crates", "go-tools/go2vir", "scripts"):
        for path in sorted((root / area).rglob("*")):
            if not path.is_file() or "vendor" in path.parts or path.suffix not in {".rs", ".go", ".py", ".sh"}:
                continue
            if CONSUMER_PATTERN.search(path.read_text()):
                relative = path.relative_to(root).as_posix()
                references.add(relative)
                consumers.add(relative)
    consumer_records = []
    for relative in sorted(consumers):
        item = record(root, relative)
        item["reason"] = "direct Go profile reference" if relative in references else "reviewed shared/frontend owner"
        item["role"] = "test" if "/tests/" in relative or relative.endswith("_test.go") or relative.endswith("_tests.rs") else "implementation"
        consumer_records.append(item)

    regression_paths = set()
    for area in ("fixtures/vir-go", "fixtures/go-basic", "fixtures/go-alpha", "examples/max64", "examples/order_policy", "examples/payment_policies"):
        for path in (root / area).rglob("*"):
            if path.is_file() and path.suffix in {".go", ".json", ".mod", ".sum", ".hex", ".mpcert"}:
                regression_paths.add(path.relative_to(root).as_posix())
    regression = [record(root, path) for path in sorted(regression_paths)]
    spec_paths = (
        "develop/specs/GO_VIR_PROFILE_V0.md", "develop/specs/VIR_V0.md",
        "develop/specs/VC_V1.md", "develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md",
        "develop/specs/SOURCE_MAP_V0.md", "develop/specs/SOURCE_MANIFEST_V0.md",
        "develop/specs/CORE_V0.md", "develop/specs/CERT_V0.md",
        "develop/specs/TRUST_BOUNDARY_V0.md", vectors_path,
    )
    return {
        "schema": "mpk.go_practical.entry_audit.v0",
        "work_item": "GO-PRAC-T01-W01",
        "evidence_scope": "baseline identity/inventory checks; not practical-Go proof or release acceptance",
        "baseline_selection": selection,
        "selection_file": record(root, SELECTION),
        "installed_release": {
            "receipt": record(root, receipt_path),
            "date": receipt["date"],
            "scope": selection["registered_release_scope"],
            "semantic_registry": record(root, semantic_path),
            "bundle_registry": record(root, bundle_path),
            "semantic_registry_revision": semantic["revision"],
            "profile_count": len(semantic["profiles"]),
            "tuple_count": len(bundles["tuples"]),
        },
        "go_profile_entry": go_entries[0],
        "go_tuple": go_tuple,
        "compiler_candidate": {
            "bundle_id": toolchain["bundle_id"],
            "distribution_sha256": toolchain["distribution_sha256"],
            "executable": compiler,
            "content_sha256": content["content_sha256"],
            "source_witnesses": compiler_witnesses,
            "status": "registered metadata pin; target compiler not executed by this audit",
        },
        "specifications": [record(root, path) for path in spec_paths],
        "consumer_inventory_scope": "direct profile references plus reviewed initial owners; T01-W04 expands to the complete successor migration inventory",
        "consumer_inventory": consumer_records,
        "fixed_go_regression": {
            "positive_sources": len(frontend["cases"]),
            "negative_sources": len(frontend["negative_cases"]),
            "derived_cases": len(derived["cases"]),
            "indexed_artifacts_checked": artifact_count,
            "profile_vectors": len(vector_ids),
            "files": regression,
        },
        "findings": [],
    }


def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    result = audit(root)
    expected = encoded(result)
    if args.write:
        destination = root / REPORT
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(expected)
    else:
        require(read_bytes(root, REPORT) == expected, "entry-audit.json is stale or modified")
    print(f"GO-PRAC W01: {len(result['consumer_inventory'])} owners, {len(result['fixed_go_regression']['files'])} regression files, {result['fixed_go_regression']['indexed_artifacts_checked']} indexed artifacts checked")
    print("Baseline audit PASS; no practical profile or application proof accepted")


if __name__ == "__main__":
    try:
        main()
    except (OSError, KeyError, TypeError, ValueError, StopIteration) as error:
        print(f"Go practical entry audit rejected: {error}", file=sys.stderr)
        raise SystemExit(1)
