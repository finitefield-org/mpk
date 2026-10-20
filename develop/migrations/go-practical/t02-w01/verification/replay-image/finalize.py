"""Independent byte-image, compiler inventory, native replay and frozen audit."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import stat
import struct

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "closed-images"
native_root = root / "initial"


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def identity(obj, raw):
    assert obj["schema"] == "mpk.go.source_object.v1"
    assert obj["bytes"] == len(raw) and obj["raw_sha256"] == hashlib.sha256(raw).hexdigest()
    encoded = json.dumps({k: v for k, v in obj.items() if k != "id"}, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + encoded).hexdigest() == obj["id"]


checks = []
for mode in ["selected", "legacy", "vet", "format"]:
    path = current / mode / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(r[key]["path"]) == r[key]
    for row in r["sources"]:
        assert seal(row["path"]) == row
        copied = seal(Path(r["source_snapshot"]) / row["path"])
        assert (copied["sha256"], copied["bytes"]) == (row["sha256"], row["bytes"])
    if mode == "selected":
        assert len(r["root_tests_passed"]) == 22 and len(r["subtests_passed"]) == 177
    if mode == "legacy":
        assert not r["root_tests_passed"] and not r["subtests_passed"]
    checks.append({"record": seal(path), "seconds": r["wall_seconds"]})
retained_count, retained_bytes = 0, 0
for path in sorted(root.glob("*/*/original-byte-retention.json")):
    retention = json.loads(path.read_text())
    for row in retention["objects"]:
        encoded = Path(row["retained_path"])
        assert hashlib.sha256(encoded.read_bytes()).hexdigest() == row["retained_sha256"]
        data = json.loads(encoded.read_text())
        raw = base64.b64decode(data["base64"], validate=True)
        obj = json.loads(encoded.with_name(row["source_id"] + ".json").read_text())
        identity(obj, raw)
        assert row["source_id"] == data["source_id"] == obj["id"]
        assert row["bytes"] == len(raw) and row["raw_sha256"] == obj["raw_sha256"]
        retained_count += 1
        retained_bytes += len(raw)
assert retained_count == 11334
build_path = current / "selected/reproduced-builds.json"
inputs_path = current / "selected/captured-inputs.json"
builds, inputs = json.loads(build_path.read_text()), json.loads(inputs_path.read_text())
assert builds["byte_exact_equal"] and builds["independent_empty_caches"]
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (176, 40, 133)
assert len(inputs["build_inputs"]) == 176
for obj in inputs["build_inputs"]:
    identity(obj, (Path("go-tools/go2vir") / obj["path"]).read_bytes())
for build in builds["builds"]:
    assert build["bytes"] == 8592898 and build["binary_sha256"] == "3fe0ec047ee3e9feb05da1d6679993c66bfa55b3e2a1b0f878f5584f6761c457"
bundle_path = native_root / "linux-prepare/replay-image.json"
bundle = json.loads(bundle_path.read_text())
native_checks = []
for mode in ["prepare", "replay-1", "replay-2", "controls"]:
    path = native_root / ("linux-" + mode) / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
    for obj in r["sources"]:
        assert seal(obj["path"]) == obj
    for check in r["checks"]:
        assert seal(check["stdout"]["path"]) == check["stdout"]
        assert seal(check["stderr"]["path"]) == check["stderr"]
    if mode != "controls":
        assert all(check["exit_code"] == 0 for check in r["checks"])
    else:
        assert [check["exit_code"] for check in r["checks"]] == [1, 1, 1]
        assert "Read-only file system" in (path.parent / "readonly-input.stderr.log").read_text()
        assert "import lookup disabled by -mod=readonly" in (path.parent / "missing-original-source.stderr.log").read_text()
    native_checks.append({"record": seal(path), "checks": [{"label": c["label"], "seconds": c["seconds"]} for c in r["checks"]]})
producer = json.loads((native_root / "linux-prepare/result.json").read_text())
scratch = Path(producer["scratch"])


def host_path(private):
    result = scratch / Path(private).relative_to("/replay-output")
    assert result.resolve().is_relative_to(scratch)
    return result


assert len(bundle["inputs"]) == 5493 and bundle["complete_source_typed_packages"] == 181
assert bundle["complete_syntax_nodes"] == 1472495 and len(bundle["mounts"]) == 5
source_ids = []
for file in bundle["inputs"]:
    path = host_path(file["private"])
    assert path.is_file() and not path.is_symlink()
    raw = path.read_bytes()
    for obj in file["objects"]:
        identity(obj, raw)
        source_ids.append(obj["id"])
roots = {row["capability"]: row for row in bundle["mounts"]}
namespace = {row["capability"] + "\0" + row["relative"]: row["entries"] or [] for row in bundle["original_namespaces"]}
hash_snapshot = hashlib.sha256(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0")


def number(value):
    hash_snapshot.update(struct.pack("<Q", value))


def text(value):
    raw = value.encode()
    number(len(raw))
    hash_snapshot.update(raw)


number(len(roots))
for capability in sorted(roots):
    text(capability)
number(len(namespace))
for key in sorted(namespace):
    capability, relative = key.split("\0")
    text(capability)
    text(relative)
    number(len(namespace[key]))
    for entry in namespace[key]:
        text(entry["Name"])
        number(entry["Type"])
number(len(source_ids))
for source_id in sorted(source_ids):
    text(source_id)
snapshot_digest = hash_snapshot.hexdigest()
hash_image = hashlib.sha256(b"MPK-GO-PRIVATE-REPLAY-IMAGE-1.0\0")
for value in [snapshot_digest] + [part for c in sorted(roots) for part in [c, roots[c]["original"]]] + [bundle["compiler_distribution"]]:
    raw = value.encode()
    hash_image.update(struct.pack("<Q", len(raw)))
    hash_image.update(raw)
assert hash_image.hexdigest() == bundle["image_digest"]
inventory = bundle["compiler_inventory"]
encoded = json.dumps(inventory, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
assert hashlib.sha256(b"MPK-BUNDLE-CONTENT-0.1\0" + encoded).hexdigest() == bundle["compiler_distribution"]
sdk = host_path(roots["original-go-sdk"]["private"])
expected_names = {row["path"].removeprefix("go/") for row in inventory["files"]}
actual_names = set()
for path in sdk.rglob("*"):
    assert not path.is_symlink()
    if path.is_dir():
        assert stat.S_IMODE(path.stat().st_mode) == 0o555
    else:
        assert path.is_file()
        actual_names.add(str(path.relative_to(sdk)))
assert actual_names == expected_names and len(actual_names) == 5577
for row in inventory["files"]:
    path = sdk / row["path"].removeprefix("go/")
    actual = seal(path)
    assert actual["bytes"] == row["size_bytes"] and actual["sha256"] == row["sha256"]
    assert stat.S_IMODE(path.stat().st_mode) == (0o555 if row["executable"] else 0o444)
assert seal(sdk / "bin/go")["sha256"] == "b93cdfdbc72f1afc3f21498c80bf3d155a44a9b95e2d690c940511051574bc25"
comparisons = []
for mode in ["replay-1", "replay-2"]:
    path = native_root / ("linux-" + mode) / "comparison.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and (r["module_objects"], r["package_objects"], r["export_objects"], r["export_bytes"]) == (4, 182, 181, 79406918)
    for row in r["exports"]:
        assert seal(row["retained_bytes"]["path"]) == row["retained_bytes"]
        raw = base64.b64decode(json.loads(Path(row["retained_bytes"]["path"]).read_text())["base64"], validate=True)
        identity(row["original_export_object"], raw)
    comparisons.append({"record": seal(path), "cache": json.loads((path.parent / "result.json").read_text())["scratch"]})
assert comparisons[0]["cache"] != comparisons[1]["cache"]
prior = root.parent / "standard-sources/frozen-design-preservation.json"
frozen = json.loads(prior.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for row in frozen["inputs"]:
    assert seal(row["path"]) == row
frozen["prior_complete_manifest"] = seal(prior)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {
    "schema": "mpk.go_practical.replay_image_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "f7b52db31ae688a074b89339ec6ae70a0e3f1adf", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "checks": checks, "affected_Go_roots": 22, "subtests": 177, "native_checks": native_checks,
    "native_image_manifest": seal(bundle_path), "native_comparisons": comparisons,
    "original_source_preimages_independently_checked": retained_count, "original_source_bytes": retained_bytes,
    "native_original_source_objects": 5493, "source_typed_packages": 181, "source_syntax_nodes": 1472495,
    "full_original_modules_equal": 4, "full_original_packages_equal_except_derived_export_locator": 182,
    "whole_original_exports_equal_per_replay": 181, "whole_original_export_bytes_equal_per_replay": 79406918,
    "complete_original_compiler_inventory_files": 5577, "complete_private_image_unchanged_after_all_kernel_commands": True,
    "snapshot_digest_independently_reconstructed": snapshot_digest, "image_digest_independently_reconstructed": bundle["image_digest"],
    "relative_absolute_and_versioned_replacements_unchanged": True,
    "two_fresh_derived_caches": True, "original_init_or_application_executed": False,
    "kernel_readonly_source_control": "chmod rejects EROFS", "ambient_SDK_control": "masked and not executable",
    "missing_original_source_control": "offline readonly import rejects; not a semantic source counterexample",
    "frontend_builds": seal(build_path), "frontend_inputs": seal(inputs_path), "frozen_inputs": seal(frozen_path),
    "all_actual_validation_handles_terminal": True, "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
    "x86_64_emulated_local_Linux": True, "native_Linux_release_claim": False,
    "T02_W01_complete": False, "R06_status": "pending", "registered_context_or_source_program_application_proof": False,
    "full_T02_gate_passed": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "selection_reason": "New owned byte-image materialization and source/origin/namespace/limit consumers; changed native export original-module-cache selection; affected source/package/module/export and frontend analysis consumers. Complete actual kernel replay covers all original bodies and all182 native packages, not a source extraction/stub or signature-only adapter. Unrelated scalar/Rust/kernel/full-Go/CSharp suites are retained without rerun.",
    "remaining": "Production kernel-sandbox lifecycle/private context ownership and complete general consulted namespace/selected-closure bindings remain due. Full56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation/all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 demands/T02-T11 remain required."
}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "affected_Go_roots", "subtests", "original_source_preimages_independently_checked",
                                      "full_original_modules_equal", "whole_original_exports_equal_per_replay", "R06_status"]}))
