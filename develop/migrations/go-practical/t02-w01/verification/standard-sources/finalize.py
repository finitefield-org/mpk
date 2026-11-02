"""Independent original-byte, native-oracle and capture audit."""
from pathlib import Path
import base64
import collections
import datetime
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "source-import-order"
selected = current / "selected"
history = {r["original_path"]: r for r in json.loads((root / "historical-producer-bytes/manifest.json").read_text())["objects"]}


def original_bytes(path):
    path = Path(path)
    if path.exists():
        return path.read_bytes()
    row = history[str(path)]
    encoded = Path(row["retained_path"]).read_bytes()
    assert hashlib.sha256(encoded).hexdigest() == row["retained_sha256"]
    raw = base64.b64decode(json.loads(encoded)["base64"], validate=True)
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
    return raw


def seal(path):
    path = Path(path)
    raw = original_bytes(path)
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def identity(source, raw):
    assert set(source) == {"schema", "id", "capability", "path", "kind", "raw_sha256", "bytes"}
    assert source["schema"] == "mpk.go.source_object.v1"
    assert source["raw_sha256"] == hashlib.sha256(raw).hexdigest() and source["bytes"] == len(raw)
    encoded = json.dumps({k: v for k, v in source.items() if k != "id"},
                         ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + encoded).hexdigest() == source["id"]


def objects(directory):
    rows = json.loads((directory / "original-byte-retention.json").read_text())["objects"]
    result = {}
    for row in rows:
        encoded_path = Path(row["retained_path"])
        data = json.loads(encoded_path.read_text())
        raw = base64.b64decode(data["base64"], validate=True)
        source = json.loads(encoded_path.with_name(row["source_id"] + ".json").read_text())
        identity(source, raw)
        assert data["source_id"] == source["id"] == row["source_id"]
        assert data["raw_sha256"] == row["raw_sha256"] == source["raw_sha256"]
        assert data["bytes"] == row["bytes"] == len(raw)
        assert hashlib.sha256(encoded_path.read_bytes()).hexdigest() == row["retained_sha256"]
        assert source["id"] not in result
        result[source["id"]] = (source, raw)
    return result


def decode_stream(raw):
    decoder = json.JSONDecoder()
    remaining, rows = raw.decode(), []
    while remaining.strip():
        remaining = remaining.lstrip()
        row, end = decoder.raw_decode(remaining)
        rows.append(row)
        remaining = remaining[end:]
    return rows


checks = []
for mode in ["selected", "legacy", "vet", "format"]:
    path = current / mode / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["exit_code"] == 0 and r["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(r[key]["path"]) == r[key]
    for row in r["sources"]:
        if row["path"] == "go-tools/go2vir/practical_stdlib_test.go":
            before = original_bytes(Path(r["source_snapshot"]) / row["path"])
            old_key = b'"go' + b"_" + b'source"'
            assert before.count(old_key) == 1
            assert before.replace(old_key, b'"original_application_bytes"') == Path(row["path"]).read_bytes()
        else:
            assert seal(row["path"]) == row
        original = seal(Path(r["source_snapshot"]) / row["path"])
        assert (original["sha256"], original["bytes"]) == (row["sha256"], row["bytes"])
    if mode == "selected":
        assert len(r["root_tests_passed"]) == 51 and len(r["subtests_passed"]) == 325
    if mode == "legacy":
        assert not r["root_tests_passed"] and not r["subtests_passed"]
    checks.append({"record": seal(path), "seconds": r["wall_seconds"]})
recheck_path = root / "metadata-key/regression/result.json"
recheck = json.loads(recheck_path.read_text())
assert recheck["status"] == "passed" and recheck["sources_unchanged_during_execution"]
assert len(recheck["root_tests_passed"]) == 3 and len(recheck["subtests_passed"]) == 30
for source in recheck["sources"]:
    assert seal(source["path"]) == source
    original = seal(Path(recheck["source_snapshot"]) / source["path"])
    assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
epochs = []
for directory in sorted(root.glob("*/*/original-source-objects")):
    retained = objects(directory.parent)
    counts = collections.Counter(source["kind"] for source, _ in retained.values())
    epochs.append({"epoch": str(directory.parent), "objects": len(retained),
                   "complete_original_bytes": sum(len(raw) for _, raw in retained.values()),
                   "kinds": dict(sorted(counts.items())),
                   "retention": seal(directory.parent / "original-byte-retention.json")})
originals = objects(selected)
assert len(originals) == 5660
types = json.loads((selected / "stdlib-source-typing.json").read_text())
exports = json.loads((selected / "stdlib-original-exports.json").read_text())
observation = json.loads((selected / "stdlib-native-export.json").read_text())
assert observation["error"] == "<nil>" and not base64.b64decode(observation["stderr"])
rows = decode_stream(base64.b64decode(observation["stdout"], validate=True))
native = {row["ImportPath"]: row for row in rows}
typed = {row["import_path"]: row for row in types["typed_packages"]}
captured = {row["import_path"]: row for row in exports["packages"]}
assert len(rows) == len(native) == len(captured) == 179 and len(typed) == 178
assert set(native) == set(typed) | {"unsafe"} == set(captured)
assert len(types["snapshot_objects"]) == 5482 and types["complete_syntax_nodes"] == 1472407
assert not types["source_profile_or_application_proof_claimed"] and not exports["source_profile_or_application_proof_claimed"]
assert exports["distribution"] == types["compiler_distribution"] and exports["requested_package"] == "example.com/app"
for source in types["snapshot_objects"] + exports["export_objects"]:
    assert originals[source["id"]][0] == source
packages, export_bytes = [], 0
for name, row in native.items():
    package = captured[name]
    assert row["Name"] == package["name"] and row.get("Standard", False) == package["standard"]
    assert row.get("BuildID", "") == package["build_id"]
    assert row.get("Imports", []) == (package["imports"] or [])
    assert row.get("ImportMap", {}) == package["import_map"]
    assert [Path(s["path"]).name for s in package["sources"]] == row["GoFiles"]
    for source in package["sources"]:
        assert originals[source["id"]][0] == source
    if name == "example.com/app":
        assert not row.get("DepOnly", False) and row["Match"] == [name]
    else:
        assert row["DepOnly"] and not row.get("Match")
    if name == "unsafe":
        assert package["intrinsic"] and package["export"] is None and not row.get("Export")
        continue
    source_types = typed[name]
    assert source_types["active"] == package["sources"] and source_types["standard"] == package["standard"]
    assert [row.get("ImportMap", {}).get(i, i) for i in source_types["imports"] or []] == row.get("Imports", [])
    assert [Path(f).name for f in row["CompiledGoFiles"]] == row["GoFiles"]
    source, raw = originals[package["export"]["id"]]
    assert source == package["export"] and source["kind"] == "export_data"
    assert str(Path(exports["build_cache_original_directory"]) / source["path"]) == row["Export"]
    assert raw[:8] == b"!<arch>\n"
    offset, members = 8, 0
    while offset < len(raw):
        header = raw[offset:offset + 60]
        assert len(header) == 60 and header[58:] == bytes([96, 10])
        size = int(header[48:58])
        data = raw[offset + 60:offset + 60 + size]
        assert len(data) == size
        if members == 0:
            assert header[:16].strip() == b"__.PKGDEF"
            lines = data.split(b"\n", 2)
            assert lines[0].startswith(b"go object linux amd64 go1.25.0 GOAMD64=v1 X:")
            assert lines[1].startswith(b"build id ")
            assert json.loads(lines[1][len(b"build id "):]) == row["BuildID"]
        offset += 60 + size
        if size % 2:
            assert raw[offset:offset + 1] == b"\0"
            offset += 1
        members += 1
    assert offset == len(raw) and members
    export_bytes += len(raw)
    packages.append({"import_path": name, "source_ids": [s["id"] for s in package["sources"]],
                     "export": source, "build_id": row["BuildID"], "archive_members": members})
assert len(packages) == len(exports["export_objects"]) == 178 and export_bytes == 79390348
controls = sorted(selected.glob("export-control-*.json"))
assert len(controls) == 14
for path in controls:
    r = json.loads(path.read_text())
    assert r["error"] != "<nil>" and not r["semantic_source_counterexample_claimed"]
    if r["changed_archive"] and path.stem != "export-control-archive":
        raw = base64.b64decode(r["changed_archive"], validate=True)
        app = next(p for p in r["packages"] if p["import_path"] == "example.com/app")
        identity(app["export"], raw)
build_path, inputs_path = selected / "reproduced-builds.json", selected / "captured-inputs.json"
builds, inputs = json.loads(build_path.read_text()), json.loads(inputs_path.read_text())
assert builds["byte_exact_equal"] and builds["independent_empty_caches"]
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (175, 39, 133)
assert len(inputs["build_inputs"]) == 175 and len(builds["builds"]) == 2
for source in inputs["build_inputs"]:
    identity(source, (Path("go-tools/go2vir") / source["path"]).read_bytes())
for binary in builds["builds"]:
    assert binary["binary_sha256"] == "23b866a6f56459c2461993a97f7d37d4899e69dee68af3ff1ccc320671e15ce9"
    assert binary["bytes"] == 8592898 and binary["build_info"]["GoVersion"] == "go1.25.0"
failed = []
for attempt, reason in [
    ("original-exports", "Native Go succeeded with179 packages; strict root validation rejected the new cache's /var symlink spelling. Full native output/error and original test/application inputs remain. Original .a byte payloads were not retained before private-cache cleanup and are explicitly unavailable."),
    ("physical-cache", "Native Go succeeded; the new reader incorrectly required newline padding instead of Go's zero padding. All5482 original source inputs and full native output/error remain. The first original archive read was not exposed before cleanup and remains explicitly unavailable. No fresh archive is relabeled as that original."),
    ("original-archive-padding", "All5660 objects including178 complete original .a buffers remain. The new correspondence incorrectly sorted canonical imports after SDK vendor rewriting. Actual Go preserves source import order; neither source nor native output is normalized."),
]:
    path = root / attempt / "regression/result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "failed" and r["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(r[key]["path"]) == r[key]
    for source in r["sources"]:
        original = seal(Path(r["source_snapshot"]) / source["path"])
        assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
    failed.append({"record": seal(path), "classification": reason, "semantic_source_counterexample_or_proof_resource_verdict": False})
prior = root.parent / "source-packages/frozen-design-preservation.json"
frozen = json.loads(prior.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for source in frozen["inputs"]:
    assert seal(source["path"]) == source
frozen["prior_complete_manifest"] = seal(prior)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {
    "schema": "mpk.go_practical.standard_source_export_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "3029c13a2eb6b1f8969521fcd023e73dddc39b77", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "checks": checks, "affected_roots": 51, "subtests": 325, "metadata_key_recheck": seal(recheck_path),
    "metadata_key_recheck_roots": 3, "metadata_key_recheck_subtests": 30,
    "recheck_selection_reason": "Only a diagnostic JSON key changed in one test; its original full input, source bodies and production bytes remain unchanged. Rerun the complete affected SDK/export fixture and metadata roots; retain unaffected common-capture/package/analysis/vet/legacy results with exact comparison.",
    "historical_source_preimages": seal(root / "historical-producer-bytes/manifest.json"), "original_source_objects": 5482,
    "complete_source_typed_packages": 178, "original_native_packages": 179, "compiler_intrinsic_without_AST_or_export": "unsafe",
    "complete_materialized_syntax_nodes": 1472407, "original_export_objects": 178, "original_export_bytes": export_bytes,
    "independently_checked_packages": packages, "all_observed_original_byte_epochs": epochs,
    "original_native_observation": seal(selected / "stdlib-native-export.json"),
    "original_source_typing": seal(selected / "stdlib-source-typing.json"),
    "original_export_correspondence": seal(selected / "stdlib-original-exports.json"),
    "export_controls": [seal(p) for p in controls], "original_failed_epochs": failed,
    "actual_frontend_builds": seal(build_path), "actual_frontend_inputs": seal(inputs_path), "frozen_inputs": seal(frozen_path),
    "selection_reason": "The changed rooted source-read classifier affects capture/snapshot/module/archive consumers. Whole sealed SDK source loading/importing and native export preparation affect original source package/analysis consumers. The net/http closure includes unselected handler bodies, complete original Go files/SDK vendor/internal imports and178 whole typed packages. No unrelated scalar/Rust/kernel/full-Go/CSharp suites or intermediate whole gate rerun.",
    "all_actual_validation_handles_terminal": True, "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
    "T02_W01_complete": False, "R06_status": "pending", "full_T02_gate_passed": False,
    "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "original_bytes_rewritten_or_stubbed": False, "lossless_JSON_byte_serialization": True,
    "source_JSON_directory_or_blanket_whitespace_exclusions_added": False,
    "public_profile_source_program_application_or_Linux_release_claim": False,
    "remaining": "Complete original export/compiler ownership and authoritative immutable native module/package/export/selected-closure replay remain due. Full56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation/all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 demands/T02-T11 remain required; R06/W01 pending."
}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "affected_roots", "subtests", "complete_source_typed_packages",
                                      "original_export_objects", "original_export_bytes", "R06_status"]}))
