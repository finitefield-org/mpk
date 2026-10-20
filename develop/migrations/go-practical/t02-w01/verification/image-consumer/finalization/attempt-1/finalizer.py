"""Independent immutable image-consumer, multi-snapshot and native replay audit."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import stat
import struct

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "owned-checksums"
native_root = root / "owned-checksums"


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
        assert len(r["root_tests_passed"]) == 59 and len(r["subtests_passed"]) == 290
        assert len(r["tests_skipped"]) == 4
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
assert retained_count == 22668
build_path = current / "selected/reproduced-builds.json"
inputs_path = current / "selected/captured-inputs.json"
builds, inputs = json.loads(build_path.read_text()), json.loads(inputs_path.read_text())
assert builds["byte_exact_equal"] and builds["independent_empty_caches"]
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (182, 46, 133)
assert len(inputs["build_inputs"]) == 182
for obj in inputs["build_inputs"]:
    identity(obj, (Path("go-tools/go2vir") / obj["path"]).read_bytes())
for build in builds["builds"]:
    assert build["bytes"] == 8592898 and build["binary_sha256"] == "fcaf538353eed957519b935ae137e80eb982d4f847d0b879670e10437f50b4fa"
bundle_path = native_root / "linux-prepare/replay-image.json"
bundle = json.loads(bundle_path.read_text())
native_checks = []
for mode in ["prepare", "replay-1", "replay-2"]:
    path = native_root / ("linux-" + mode) / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
    for obj in r["sources"]:
        assert seal(obj["path"]) == obj
    for check in r["checks"]:
        assert seal(check["stdout"]["path"]) == check["stdout"]
        assert seal(check["stderr"]["path"]) == check["stderr"]
    assert all(check["exit_code"] == 0 for check in r["checks"])
    native_checks.append({"record": seal(path), "checks": [{"label": c["label"], "seconds": c["seconds"]} for c in r["checks"]]})
producer = json.loads((native_root / "linux-prepare/result.json").read_text())
scratch = Path(producer["scratch"])


def host_path(private):
    result = scratch / Path(private).relative_to("/replay-output")
    assert result.resolve().is_relative_to(scratch)
    return result


assert len(bundle["inputs"]) == 5674 and bundle["complete_source_typed_packages"] == 181
assert bundle["complete_syntax_nodes"] == 1472495 and len(bundle["mounts"]) == 6
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
def snapshot_digest(capabilities, source_ids):
    framed = hashlib.sha256(b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\0")
    def number(value):
        framed.update(struct.pack("<Q", value))
    def text(value):
        raw = value.encode()
        number(len(raw))
        framed.update(raw)
    number(len(capabilities))
    for capability in capabilities:
        text(capability)
    selected = {key: value for key, value in namespace.items() if key.split("\0")[0] in capabilities}
    number(len(selected))
    for key in sorted(selected):
        capability, relative = key.split("\0")
        text(capability)
        text(relative)
        number(len(selected[key]))
        for entry in selected[key]:
            text(entry["Name"])
            number(entry["Type"])
    number(len(source_ids))
    for source_id in source_ids:
        text(source_id)
    return framed.hexdigest()

snapshots = bundle["original_source_snapshots"]
assert len(snapshots) == 2
covered_roots, covered_sources = set(), set()
for snapshot in snapshots:
    caps, ids = snapshot["roots"], snapshot["source_ids"]
    assert caps == sorted(set(caps)) and ids == sorted(set(ids))
    assert not covered_roots.intersection(caps) and not covered_sources.intersection(ids)
    assert [row["Capability"] for row in snapshot["origins"]] == caps
    for row in snapshot["origins"]:
        assert roots[row["Capability"]]["original"] == row["Directory"]
    assert snapshot_digest(caps, ids) == snapshot["digest"]
    if caps == ["original-go-package-exports"]:
        assert len(ids) == 181 and snapshot["digest"] == bundle["restored_export_snapshot_digest"]
    else:
        assert len(ids) == 5493 and snapshot["digest"] == bundle["restored_source_snapshot_digest"]
    covered_roots.update(caps)
    covered_sources.update(ids)
assert covered_roots == set(roots) and covered_sources == set(source_ids)
assert bundle["original_roots_retired_before_consumer"] and bundle["original_donor_buffers_cleared_before_consumer"]
assert bundle["restored_original_module_bindings"] == 4 and bundle["restored_complete_original_objects"] == 5674
hash_image = hashlib.sha256(b"MPK-GO-PRIVATE-REPLAY-IMAGE-1.0\0")
def add(value):
    raw = value.encode()
    hash_image.update(struct.pack("<Q", len(raw)))
    hash_image.update(raw)
for snapshot in sorted(snapshots, key=lambda row: row["digest"]):
    add(snapshot["digest"])
    for origin in snapshot["origins"]:
        add(origin["Capability"])
        add(origin["Directory"])
add(bundle["compiler_distribution"])
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
    assert r["status"] == "passed" and (r["module_objects"], r["package_objects"], r["export_objects"], r["export_bytes"]) == (4, 182, 181, 79406936)
    for row in r["exports"]:
        assert seal(row["retained_bytes"]["path"]) == row["retained_bytes"]
        raw = base64.b64decode(json.loads(Path(row["retained_bytes"]["path"]).read_text())["base64"], validate=True)
        identity(row["original_export_object"], raw)
    comparisons.append({"record": seal(path), "cache": json.loads((path.parent / "result.json").read_text())["scratch"]})
assert comparisons[0]["cache"] != comparisons[1]["cache"]
# Independently compose physical namespace knowledge and the exact materialized
# file/directory closure, rather than trusting the new view's summary counts.
directory_type = 1 << 31
nodes, observations, expected_files, expected_directories = {}, {}, {}, set()


def node(name, kind):
    assert name not in nodes or nodes[name] == kind
    nodes[name] = kind


def directory(name):
    while True:
        node(name, directory_type)
        expected_directories.add(name)
        parent = str(Path(name).parent)
        if parent == name:
            break
        name = parent


def original_file(name, length, digest, executable):
    directory(str(Path(name).parent))
    node(name, 0)
    expected = (length, digest, 0o555 if executable else 0o444)
    assert name not in expected_files or expected_files[name] == expected
    expected_files[name] = expected


for mount in bundle["mounts"]:
    directory(mount["original"])
for row in bundle["original_namespaces"]:
    name = str(Path(roots[row["capability"]]["original"]) / row["relative"])
    directory(name)
    entries = row["entries"] or []
    assert name not in observations or observations[name] == entries
    observations[name] = entries
    for entry in entries:
        node(str(Path(name) / entry["Name"]), entry["Type"])
for row in bundle["inputs"]:
    for obj in row["objects"]:
        name = str(Path(roots[obj["capability"]]["original"]) / obj["path"])
        original_file(name, obj["bytes"], obj["raw_sha256"], False)
for row in inventory["files"]:
    name = str(Path(bundle["compiler_original_root"]) / row["path"].removeprefix("go/"))
    original_file(name, row["size_bytes"], row["sha256"], row["executable"])
for name, kind in nodes.items():
    parent = str(Path(name).parent)
    if parent != name and parent in observations:
        matches = [entry for entry in observations[parent] if entry["Name"] == Path(name).name]
        assert len(matches) == 1 and matches[0]["Type"] == kind
assert len(nodes) == bundle["reconciled_original_namespace_nodes"] == 7411
assert len(expected_files) == bundle["verified_materialized_files"] == 5771
image_root = host_path(bundle["image_root"])
expected_dirs = {"."} | {str(Path("namespace") / name[1:]) for name in expected_directories}
expected_names = {str(Path("namespace") / name[1:]): value for name, value in expected_files.items()}
seen_dirs, seen_files = set(), set()
for path in [image_root] + sorted(image_root.rglob("*")):
    assert not path.is_symlink()
    name = str(path.relative_to(image_root))
    mode = stat.S_IMODE(path.stat().st_mode)
    if path.is_dir():
        assert name in expected_dirs and mode == 0o555
        seen_dirs.add(name)
    else:
        assert path.is_file() and name in expected_names
        length, digest, expected_mode = expected_names[name]
        row = seal(path)
        assert (row["bytes"], row["sha256"], mode) == (length, digest, expected_mode)
        seen_files.add(name)
assert seen_dirs == expected_dirs and seen_files == set(expected_names)


prior_root = root.parent / "namespace-view"
for name in ["run-linux.py", "compare-linux.py"]:
    assert (root / name).read_bytes() == (prior_root / name).read_bytes()
prior_controls_path = root.parent / "replay-image/initial/linux-controls/result.json"
prior_controls = json.loads(prior_controls_path.read_text())
assert prior_controls["status"] == "passed" and [check["exit_code"] for check in prior_controls["checks"]] == [1, 1, 1]
for check in prior_controls["checks"]:
    for stream in ["stdout", "stderr"]:
        assert seal(check[stream]["path"]) == check[stream]
old_view = (prior_root / "sealed-modes/inputs/go-tools/go2vir/practical_view.go").read_text()
new_view = Path("go-tools/go2vir/practical_view.go").read_text()
def whole_function(source, name):
    start = source.index("func " + name + "(")
    end = source.index("\nfunc ", start + 1)
    return source[start:end]
assert whole_function(old_view, "verifyPracticalReplayMaterialization") == whole_function(new_view, "verifyPracticalReplayMaterialization")
special_control = prior_root / "linux-special-permission/after/result.json"
assert json.loads(special_control.read_text())["status"] == "passed"

epochs = []
for phase, expected_status in [("initial/regression", "failed"), ("restored/regression", "failed"),
                               ("independent/regression", "passed"), ("checksum-source/ownership", "failed"),
                               ("checksum-both/ownership", "failed"), ("owned-checksums/ownership", "passed")]:
    path = root / phase / "result.json"
    result = json.loads(path.read_text())
    assert result["status"] == expected_status and result["sources_unchanged_during_execution"]
    for stream in ["stdout", "stderr"]:
        assert seal(result[stream]["path"]) == result[stream]
    for row in result["sources"]:
        copied = seal(Path(result["source_snapshot"]) / row["path"])
        assert (copied["sha256"], copied["bytes"]) == (row["sha256"], row["bytes"])
    runner = seal(path.parent.parent / "runner.py")
    assert (runner["sha256"], runner["bytes"]) == (result["runner"]["sha256"], result["runner"]["bytes"])
    if phase == "checksum-source/ownership":
        assert result["tests_failed"] == ["TestPracticalImageInputsRebindWholeOriginalModuleGraphAfterRetirement"]
    if phase == "checksum-both/ownership":
        assert len(result["tests_failed"]) == 2
    if phase == "owned-checksums/ownership":
        assert len(result["root_tests_passed"]) == 2 and not result["tests_failed"]
    epochs.append({"record": seal(path), "status": expected_status, "seconds": result["wall_seconds"]})
for phase in ["checksum-source/ownership", "checksum-both/ownership", "owned-checksums/ownership", "owned-checksums/selected"]:
    path = root / phase / "image-checksum-source-ownership.json"
    result = json.loads(path.read_text())
    original = base64.b64decode(result["original_raw"], validate=True)
    returned = base64.b64decode(result["returned_raw"], validate=True)
    identity(result["original_object"], original)
    assert not result["semantic_source_counterexample_claimed"]
    if phase.startswith("checksum-"):
        assert original != returned and result["error"] == "practical source buffer identity is inconsistent"
    else:
        assert original == returned and result["error"] == "<nil>"
for control in ["main-directory", "main-manifest", "language", "version", "replacement-path", "replacement-version",
                "versioned-directory", "checksum", "duplicate", "no-main", "unknown-field", "checksum-owner"]:
    result = json.loads((current / "selected" / ("image-module-" + control + ".json")).read_text())
    assert result["error"] != "<nil>" and not result["semantic_source_counterexample_claimed"]
fixture_path = root / "fixture-payloads.json"
fixture = json.loads(fixture_path.read_text())
assert fixture["status"] == "byte_exact_transfer" and fixture["payloads"] == len(fixture["objects"])
canonical_fixture_payloads = 0
for row in fixture["objects"]:
    assert seal(row["producer_record"]["path"]) == row["producer_record"]
    assert seal(row["retained"]["path"]) == row["retained"]
    value = json.loads(Path(row["producer_record"]["path"]).read_text())
    for key in row["field_path"]:
        value = value[key]
    raw = base64.b64decode(value, validate=True)
    stored = json.loads(Path(row["retained"]["path"]).read_text())
    assert raw == base64.b64decode(stored["base64"], validate=True)
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
    if row["canonical_source_preimage"]:
        identity(row["claimed_source_object"], raw)
        canonical_fixture_payloads += 1
terminal_path = root / "terminal-handles.json"
terminal = json.loads(terminal_path.read_text())
assert terminal["status"] == "passed" and terminal["all_actual_validation_handles_terminal"]
assert not terminal["live_matching_drivers"] and len(terminal["terminal_results"]) == 20
for row in terminal["terminal_results"]:
    assert seal(row["path"]) == {key: row[key] for key in ["path", "sha256", "bytes"]}
limitations_path = root / "original-epoch-limitations.json"
limitations = json.loads(limitations_path.read_text())
assert limitations["missing_original_observations_replaced_by_fresh_runs"] is False
assert len(limitations["limitations"]) == 2

prior = root.parent / "namespace-view/frozen-design-preservation.json"
frozen = json.loads(prior.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for row in frozen["inputs"]:
    assert seal(row["path"]) == row
frozen["prior_complete_manifest"] = seal(prior)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {
    "schema": "mpk.go_practical.image_consumer_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "784d1cea5e2656be1df1b8f2bbbfd86b86378543", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "checks": checks, "affected_Go_roots": 59, "subtests": 290, "native_checks": native_checks,
    "prior_epochs": epochs, "fixture_payloads": seal(fixture_path), "original_epoch_limitations": seal(limitations_path),
    "supplemental_complete_payloads_independently_checked": fixture["payloads"],
    "supplemental_canonical_source_preimages_independently_checked": canonical_fixture_payloads,
    "original_and_image_checksum_byte_map_ownership_regression_reproduced_and_fixed": True,
    "whole_original_source_and_export_snapshots_restored_through_owned_view": True,
    "producer_original_roots_and_donor_buffers_retired_before_all_source_typing": True,
    "source_export_snapshot_components_independently_reconstructed": 2,
    "all_materialized_files_independently_checked": len(expected_files), "original_namespace_nodes_independently_reconciled": len(nodes),
    "Darwin_special_mode_controls_not_materialized": 4, "prior_Linux_six_mode_controls_retained": seal(special_control),
    "exact_mode_guard_function_unchanged": True, "terminal_handles": seal(terminal_path),
    "native_image_manifest": seal(bundle_path), "native_comparisons": comparisons,
    "original_source_preimages_independently_checked": retained_count, "original_source_bytes": retained_bytes,
    "native_original_source_objects": 5674, "source_typed_packages": 181, "source_syntax_nodes": 1472495,
    "full_original_modules_equal": 4, "full_original_packages_equal_except_derived_export_locator": 182,
    "whole_original_exports_equal_per_replay": 181, "whole_original_export_bytes_equal_per_replay": 79406936,
    "complete_original_compiler_inventory_files": 5577, "complete_private_image_unchanged_after_all_kernel_commands": True,
    "source_snapshot_digests_independently_reconstructed": [snapshot["digest"] for snapshot in snapshots], "image_digest_independently_reconstructed": bundle["image_digest"],
    "relative_absolute_and_versioned_replacements_unchanged": True,
    "two_fresh_derived_caches": True, "original_init_or_application_executed": False,
    "prior_kernel_controls": seal(prior_controls_path), "unchanged_kernel_controls_repeated": False,
    "frontend_builds": seal(build_path), "frontend_inputs": seal(inputs_path), "frozen_inputs": seal(frozen_path),
    "all_actual_validation_handles_terminal": True, "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
    "x86_64_emulated_local_Linux": True, "native_Linux_release_claim": False,
    "T02_W01_complete": False, "R06_status": "pending", "registered_context_or_source_program_application_proof": False,
    "full_T02_gate_passed": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "selection_reason": "Original collection and immutable image consumers share complete module binding and source/ZIP/checksum/replacement rules. Full original source/export byte restoration, independently owned snapshots and parsed checksum/source/map cache views, complete SDK/application consumers after original roots and donor buffers retire. Affected archive/module/resolution/image/source/export/analysis consumers only. Complete actual kernel replay covers all original bodies and all182 native packages, not a source extraction/stub or signature-only adapter. Unrelated scalar/Rust/kernel/full-Go/CSharp suites are retained without rerun.",
    "remaining": "Production kernel-sandbox lifecycle/private context ownership and complete general consulted namespace/selected-closure bindings remain due. Full56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation/all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 demands/T02-T11 remain required."
}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "affected_Go_roots", "subtests", "original_source_preimages_independently_checked",
                                      "full_original_modules_equal", "whole_original_exports_equal_per_replay", "R06_status"]}))
