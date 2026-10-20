"""Independent original namespace, owned image-view and native replay audit."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import stat
import struct
import subprocess

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "sealed-modes"
native_root = root / "sealed-modes"


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def identity(obj, raw):
    assert obj["schema"] == "mpk.go.source_object.v1"
    assert obj["bytes"] == len(raw) and obj["raw_sha256"] == hashlib.sha256(raw).hexdigest()
    encoded = json.dumps({k: v for k, v in obj.items() if k != "id"}, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + encoded).hexdigest() == obj["id"]


format_path = root / "final-format/format-correction.json"
format_record = json.loads(format_path.read_text())
assert format_record["status"] == "passed" and format_record["before_formatter_output_equals_current_source"]
assert not format_record["production_Go_source_bytes_changed"]
for key in ["before", "after", "formatter"]:
    assert seal(format_record[key]["path"]) == format_record[key]
formatted = subprocess.run([format_record["formatter"]["path"]],
                           input=Path(format_record["before"]["path"]).read_bytes(),
                           capture_output=True, check=True)
assert not formatted.stderr and formatted.stdout == Path(format_record["after"]["path"]).read_bytes()


def assert_current_source(row, captured_root):
    if seal(row["path"]) == row:
        return
    # Only the one recorded Go test alignment differs. Production sources,
    # frontend build inputs and original application/SDK byte identities stay
    # exact. Every executed test epoch keeps its complete original source.
    assert row["path"] == "go-tools/go2vir/practical_view_test.go"
    original = seal(captured_root / row["path"])
    assert (original["sha256"], original["bytes"]) == (row["sha256"], row["bytes"])
    assert (original["sha256"], original["bytes"]) == (format_record["before"]["sha256"], format_record["before"]["bytes"])
    assert seal(row["path"]) == format_record["after"]


checks = []
for mode in ["selected", "legacy", "vet", "format"]:
    path = (root / "final-format" if mode in {"vet", "format"} else current) / mode / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(r[key]["path"]) == r[key]
    for row in r["sources"]:
        assert_current_source(row, Path(r["source_snapshot"]))
        copied = seal(Path(r["source_snapshot"]) / row["path"])
        assert (copied["sha256"], copied["bytes"]) == (row["sha256"], row["bytes"])
    if mode == "selected":
        assert len(r["root_tests_passed"]) == 30 and len(r["subtests_passed"]) == 192
        assert len(r["tests_skipped"]) == 4
        assert all("TestPracticalReplayViewRejectsSpecialPermissionBits/" in name for name in r["tests_skipped"])
    if mode == "legacy":
        assert not r["root_tests_passed"] and not r["subtests_passed"]
    checks.append({"record": seal(path), "seconds": r["wall_seconds"]})
postformat_path = root / "final-format/permission/result.json"
postformat = json.loads(postformat_path.read_text())
assert postformat["status"] == "passed" and postformat["sources_unchanged_during_execution"]
assert (len(postformat["root_tests_passed"]), len(postformat["subtests_passed"]), len(postformat["tests_skipped"])) == (2, 2, 4)
for stream in ["stdout", "stderr"]:
    assert seal(postformat[stream]["path"]) == postformat[stream]
for source in postformat["sources"]:
    assert seal(source["path"]) == source
format_diagnostic = json.loads((current / "format/result.json").read_text())
assert format_diagnostic["status"] == "failed" and format_diagnostic["exit_code"] == 0
assert Path(format_diagnostic["stdout"]["path"]).read_text() == "practical_view_test.go\n"
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
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (180, 44, 133)
assert len(inputs["build_inputs"]) == 180
for obj in inputs["build_inputs"]:
    identity(obj, (Path("go-tools/go2vir") / obj["path"]).read_bytes())
for build in builds["builds"]:
    assert build["bytes"] == 8592898 and build["binary_sha256"] == "5158c73313b8928b980961e6337fd6a1e0f905b595c06f2f444b3f571b368216"
bundle_path = native_root / "linux-prepare/replay-image.json"
bundle = json.loads(bundle_path.read_text())
export_bytes = sum(obj["bytes"] for obj in bundle["original_export_objects"])
native_checks = []
for mode in ["prepare", "replay-1", "replay-2"]:
    path = native_root / ("linux-" + mode) / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
    for obj in r["sources"]:
        assert_current_source(obj, path.parent / "inputs")
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
    assert r["status"] == "passed" and (r["module_objects"], r["package_objects"], r["export_objects"], r["export_bytes"]) == (4, 182, 181, export_bytes)
    for row in r["exports"]:
        assert seal(row["retained_bytes"]["path"]) == row["retained_bytes"]
        raw = base64.b64decode(json.loads(Path(row["retained_bytes"]["path"]).read_text())["base64"], validate=True)
        identity(row["original_export_object"], raw)
    comparisons.append({"record": seal(path), "cache": json.loads((path.parent / "result.json").read_text())["scratch"]})
assert comparisons[0]["cache"] != comparisons[1]["cache"]

# The kernel harness has no product/source edits. Retain its exact previous
# negative observations instead of needlessly executing those controls again.
prior_root = root.parent / "replay-image"
for name in ["run-linux.py", "compare-linux.py"]:
    assert (root / name).read_bytes() == (prior_root / name).read_bytes()
prior_controls_path = prior_root / "initial/linux-controls/result.json"
prior_controls = json.loads(prior_controls_path.read_text())
assert prior_controls["status"] == "passed"
assert [check["exit_code"] for check in prior_controls["checks"]] == [1, 1, 1]
for check in prior_controls["checks"]:
    for stream in ["stdout", "stderr"]:
        assert seal(check[stream]["path"]) == check[stream]

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
assert len(nodes) == bundle["reconciled_original_namespace_nodes"] == 6441
assert len(expected_files) == bundle["verified_materialized_files"] == 5590
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

# Verify every complete original regression epoch at its own captured inputs.
# The failed producer never becomes a new observation of the fixed producer.
epochs = []
for phase, status in [("original-contradiction/regression", "failed"),
                      ("reconciled/regression", "passed")]:
    path = root / phase / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == status and r["sources_unchanged_during_execution"]
    assert r["exit_code"] == (1 if status == "failed" else 0)
    for stream in ["stdout", "stderr"]:
        assert seal(r[stream]["path"]) == r[stream]
    for source in r["sources"]:
        captured = seal(Path(r["source_snapshot"]) / source["path"])
        assert (captured["sha256"], captured["bytes"]) == (source["sha256"], source["bytes"])
    runner = seal(path.parent.parent / "runner.py")
    assert (runner["sha256"], runner["bytes"]) == (r["runner"]["sha256"], r["runner"]["bytes"])
    observation = json.loads((path.parent / "namespace-contradiction-result.json").read_text())
    assert observation["two_original_epochs"] and not observation["semantic_source_counterexample_claimed"]
    if status == "failed":
        assert r["tests_failed"] == ["TestPracticalReplayImageRejectsContradictoryOriginalViews"]
        assert observation["error"] == "<nil>" and len(observation["materialized_files"]) == 2
    else:
        assert "complete directory namespace" in observation["error"]
    epochs.append({"record": seal(path), "status": status, "seconds": r["wall_seconds"]})
selected = current / "selected"
for control in ["unread-added-name", "file-to-directory", "role", "bytes", "absent-parent"]:
    r = json.loads((selected / ("namespace-" + control + "-result.json")).read_text())
    assert r["error"] != "<nil>" and not r["semantic_source_counterexample_claimed"]
for control in ["bytes", "missing", "mode", "extra-file", "extra-directory", "symlink"]:
    r = json.loads((selected / ("view-" + control + "-result.json")).read_text())
    assert r["error"] != "<nil>" and not r["semantic_source_counterexample_claimed"]
r = json.loads((selected / "view-opened-mutation.json").read_text())
assert r["original"] != r["changed"] and r["error"] != "<nil>"
assert not r["semantic_source_counterexample_claimed"]
supplement = json.loads((root / "original-namespace-payloads.json").read_text())
assert supplement["status"] == "byte_exact_transfer" and len(supplement["objects"]) == supplement["payloads"] == 130
supplement_source_objects = 0
for row in supplement["objects"]:
    assert seal(row["producer_record"]["path"]) == row["producer_record"]
    assert seal(row["retained"]["path"]) == row["retained"]
    encoded = json.loads(Path(row["retained"]["path"]).read_text())
    raw = base64.b64decode(encoded["base64"], validate=True)
    original = json.loads(Path(row["producer_record"]["path"]).read_text())
    for key in row["field"].split("/"):
        original = original[key]
    assert raw == base64.b64decode(original, validate=True)
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
    if row["source_object"] is not None:
        identity(row["source_object"], raw)
        supplement_source_objects += 1
special_records = []
for phase, status in [("special-permission/permission", "failed"),
                      ("linux-special-permission/before", "failed"),
                      ("linux-special-permission/after", "passed")]:
    path = root / phase / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == status and r["sources_unchanged_during_execution"]
    captured_root = Path(r.get("source_snapshot", str(path.parent / "inputs")))
    for source in r["sources"]:
        captured = seal(captured_root / source["path"])
        assert (captured["sha256"], captured["bytes"]) == (source["sha256"], source["bytes"])
    if phase.startswith("linux-"):
        assert len(r["checks"]) == 1 and r["checks"][0]["exit_code"] == (0 if status == "passed" else 1)
        for stream in ["stdout", "stderr"]:
            assert seal(r["checks"][0][stream]["path"]) == r["checks"][0][stream]
        events = [json.loads(line) for line in (path.parent / "controls.stdout.log").read_text().splitlines()]
        action = "pass" if status == "passed" else "fail"
        assert len([event for event in events if event["Action"] == action and "/" in event.get("Test", "")]) == 6
        assert not [event for event in events if event["Action"] == "skip"]
        controls = sorted(path.parent.glob("view-special-*.json"))
        assert len(controls) == 6
        for control in controls:
            observed = json.loads(control.read_text())
            assert observed["materialized"] and observed["actual_mode"] & observed["special_mode"]
            assert (observed["error"] == "<nil>") == (status == "failed")
    else:
        events = [json.loads(line) for line in (path.parent / "stdout.log").read_text().splitlines()]
        assert sum("special permission control was not materialized" in event.get("Output", "") for event in events) == 4
        assert sum("special permission bits acquired sealed image authority" in event.get("Output", "") for event in events) == 2
    special_records.append({"record": seal(path), "status": status})
terminal_path = root / "terminal-handles.json"
terminal = json.loads(terminal_path.read_text())
assert terminal["status"] == "passed" and terminal["all_actual_validation_handles_terminal"]
assert not terminal["live_matching_drivers"] and len(terminal["terminal_results"]) == 22
for row in terminal["terminal_results"]:
    assert seal(row["path"]) == {key: row[key] for key in ["path", "sha256", "bytes"]}
prior = root.parent / "replay-image/frozen-design-preservation.json"
frozen = json.loads(prior.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for row in frozen["inputs"]:
    assert seal(row["path"]) == row
frozen["prior_complete_manifest"] = seal(prior)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {
    "schema": "mpk.go_practical.namespace_view_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "c9e105f340d46562c7c8c9c14934175359fc7225", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "checks": checks, "affected_Go_roots": 30, "subtests": 192, "native_checks": native_checks,
    "original_failed_and_fixed_epochs": epochs, "supplemental_original_payloads": seal(root / "original-namespace-payloads.json"),
    "supplemental_full_payloads_independently_checked": 130, "supplemental_source_objects_independently_checked": supplement_source_objects,
    "original_namespace_alias_contradiction_reproduced_and_fixed": True,
    "original_namespace_nodes_independently_reconciled": len(nodes), "all_materialized_files_independently_checked": len(expected_files),
    "compatible_alias_and_unvisited_nested_refinement_accepted": True, "observed_absence_distinct_from_missing_backing": True,
    "owned_metadata_sticky_failure_and_view_lifetime_tests_passed": True,
    "file_directory_setuid_setgid_sticky_Linux_controls_passed": 6, "Darwin_special_bit_controls_not_materialized": 4,
    "special_permission_records": special_records,
    "rooted_cleanup_preserves_outside_file_bytes_and_mode": True, "terminal_handles": seal(terminal_path),
    "only_post_execution_source_change": seal(format_path),
    "post_format_affected_two_roots": seal(postformat_path), "original_format_failure_preserved": seal(current / "format/result.json"),
    "native_image_manifest": seal(bundle_path), "native_comparisons": comparisons,
    "original_source_preimages_independently_checked": retained_count, "original_source_bytes": retained_bytes,
    "native_original_source_objects": 5493, "source_typed_packages": 181, "source_syntax_nodes": 1472495,
    "full_original_modules_equal": 4, "full_original_packages_equal_except_derived_export_locator": 182,
    "whole_original_exports_equal_per_replay": 181, "whole_original_export_bytes_equal_per_replay": export_bytes,
    "complete_original_compiler_inventory_files": 5577, "complete_private_image_unchanged_after_all_kernel_commands": True,
    "snapshot_digest_independently_reconstructed": snapshot_digest, "image_digest_independently_reconstructed": bundle["image_digest"],
    "relative_absolute_and_versioned_replacements_unchanged": True,
    "two_fresh_derived_caches": True, "original_init_or_application_executed": False,
    "prior_kernel_controls": seal(prior_controls_path), "unchanged_kernel_harness_controls_repeated": False,
    "frontend_builds": seal(build_path), "frontend_inputs": seal(inputs_path), "frozen_inputs": seal(frozen_path),
    "all_actual_validation_handles_terminal": True, "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
    "x86_64_emulated_local_Linux": True, "native_Linux_release_claim": False,
    "T02_W01_complete": False, "R06_status": "pending", "registered_context_or_source_program_application_proof": False,
    "full_T02_gate_passed": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "selection_reason": "Original namespace composition now rejects contradictory aliases/parent observations/roles/bytes. Owned verified image-reader views bind complete materialized closure and independently copied original capability knowledge, sticky failure, active-view image lifetime and rooted cleanup. Affected replay/namespace/snapshot/source/package/export/analysis consumers. Complete actual kernel replay covers all original bodies and all182 native packages, not a source extraction/stub or signature-only adapter. Unrelated scalar/Rust/kernel/full-Go/CSharp suites are retained without rerun.",
    "remaining": "Production kernel-sandbox lifecycle/private context ownership and complete general consulted namespace/selected-closure bindings remain due. Full56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation/all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 demands/T02-T11 remain required."
}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "affected_Go_roots", "subtests", "original_source_preimages_independently_checked",
                                      "full_original_modules_equal", "whole_original_exports_equal_per_replay", "R06_status"]}))
