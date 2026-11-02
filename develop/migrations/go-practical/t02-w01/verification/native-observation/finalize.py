"""Independent complete Go observation ownership and native replay audit."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import stat
import struct

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "owned-observations"
native_root = root / "owned-current"


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def identity(obj, raw):
    assert obj["schema"] == "mpk.go.source_object.v1"
    assert obj["bytes"] == len(raw) and obj["raw_sha256"] == hashlib.sha256(raw).hexdigest()
    encoded = json.dumps({k: v for k, v in obj.items() if k != "id"}, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + encoded).hexdigest() == obj["id"]


def assert_current_or_restored_guard(row):
    current_raw = Path(row["path"]).read_bytes()
    if (hashlib.sha256(current_raw).hexdigest(), len(current_raw)) == (row["sha256"], row["bytes"]):
        return
    assert row["path"] == "go-tools/go2vir/practical_resolution.go"
    before = b"module discovery requires the exact original Go1.25 compiler/target"
    after = b"module discovery requires the exact validated Go1.25 compiler/target"
    assert current_raw.count(after) == 1
    prior_raw = current_raw.replace(after, before)
    assert (hashlib.sha256(prior_raw).hexdigest(), len(prior_raw)) == (row["sha256"], row["bytes"])


checks = []
for mode in ["diagnostic", "legacy", "vet", "format"]:
    path = root / "compatibility" / mode / "result.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(r[key]["path"]) == r[key]
    for row in r["sources"]:
        assert seal(row["path"]) == row
        copied = seal(Path(r["source_snapshot"]) / row["path"])
        assert (copied["sha256"], copied["bytes"]) == (row["sha256"], row["bytes"])
    if mode == "diagnostic":
        assert len(r["root_tests_passed"]) == 2 and not r["tests_failed"]
    if mode == "legacy":
        assert not r["root_tests_passed"] and not r["subtests_passed"]
    checks.append({"record": seal(path), "seconds": r["wall_seconds"]})
path = current / "selected/result.json"
r = json.loads(path.read_text())
assert r["status"] == "failed" and r["sources_unchanged_during_execution"]
assert r["tests_failed"] == ["TestPracticalResolutionRequiresExactCompilerBeforeInputReads"]
assert len(r["root_tests_passed"]) == 61 and len(r["subtests_passed"]) == 295 and len(r["tests_skipped"]) == 4
for key in ["stdout", "stderr"]:
    assert seal(r[key]["path"]) == r[key]
for row in r["sources"]:
    assert_current_or_restored_guard(row)
    copied = seal(Path(r["source_snapshot"]) / row["path"])
    assert (copied["sha256"], copied["bytes"]) == (row["sha256"], row["bytes"])
compatibility_path = root / "diagnostic-compatibility.json"
compatibility = json.loads(compatibility_path.read_text())
assert compatibility["changed_Go_files"] == 1 and not compatibility["runtime_guard_control_flow_or_condition_changed"]
assert sum(row["only_restored_guard_diagnostic_literal"] for row in compatibility["whole_original_Go_sources"]) == 1
checks.append({"record": seal(path), "seconds": r["wall_seconds"], "status": "one_existing_diagnostic_literal_failure_preserved_then_fixed_with_targeted_guard_and_fresh_frontend_builds"})
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
assert retained_count > 22668
build_path = root / "compatibility/diagnostic/reproduced-builds.json"
inputs_path = root / "compatibility/diagnostic/captured-inputs.json"
builds, inputs = json.loads(build_path.read_text()), json.loads(inputs_path.read_text())
assert builds["byte_exact_equal"] and builds["independent_empty_caches"]
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (184, 48, 133)
assert len(inputs["build_inputs"]) == 184
for obj in inputs["build_inputs"]:
    identity(obj, (Path("go-tools/go2vir") / obj["path"]).read_bytes())
assert len(builds["builds"]) == 2
assert len({(row["bytes"], row["binary_sha256"]) for row in builds["builds"]}) == 1
bundle_path = native_root / "linux-owned/replay-image.json"
bundle = json.loads(bundle_path.read_text())
expected_export_bytes = sum(obj["bytes"] for obj in bundle["original_export_objects"])
native_checks = []
path = native_root / "linux-owned/result.json"
r = json.loads(path.read_text())
assert r["status"] == "passed" and r["sources_unchanged_during_execution"]
for obj in r["sources"]:
    assert_current_or_restored_guard(obj)
    copied = seal(path.parent / "inputs" / obj["path"])
    assert (copied["sha256"], copied["bytes"]) == (obj["sha256"], obj["bytes"])
for check in r["checks"]:
    assert seal(check["stdout"]["path"]) == check["stdout"]
    assert seal(check["stderr"]["path"]) == check["stderr"]
assert len(r["checks"]) == 5 and all(check["exit_code"] == 0 for check in r["checks"])
for check in r["checks"]:
    command = check["command"]
    for argument in ["--network=none", "--read-only", "--cap-drop=ALL", "--security-opt=no-new-privileges", "--memory=4g", "--cpus=2", "--pids-limit=256"]:
        assert argument in command
    assert command[command.index("--platform") + 1] == "linux/amd64"
    assert command[command.index("--user") + 1] == "501:501"
    if check["label"] != "producer":
        mounts = [command[index + 1] for index, value in enumerate(command) if value == "--mount"]
        assert len(mounts) == len(bundle["mounts"]) + 1
        assert all(value.endswith(",readonly") for value in mounts[:-1])
        assert "target=/mpk-work" in mounts[-1] and not mounts[-1].endswith(",readonly")
        assert not any("/mpk-frontend" in value or "source=/Users/" in value for value in mounts)
        assert "/usr/local/go:ro,noexec,nosuid,nodev,mode=000,size=4096" in command
        assert command[command.index(r["image"]) + 1] == bundle["compiler_original_root"] + "/bin/go"
assert len(r["owned_comparisons"]) == 2
for row in r["owned_comparisons"]:
    assert seal(row["path"]) == row
native_checks.append({"record": seal(path), "checks": [{"label": c["label"], "seconds": c["seconds"]} for c in r["checks"]]})
producer = json.loads((native_root / "linux-owned/result.json").read_text())
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
    path = native_root / "linux-owned" / ("independent-" + mode) / "comparison.json"
    r = json.loads(path.read_text())
    assert r["status"] == "passed" and (r["module_objects"], r["package_objects"], r["export_objects"], r["export_bytes"]) == (4, 182, 181, expected_export_bytes)
    go_path = native_root / "linux-owned" / (mode + ".go-comparison.json")
    go = json.loads(go_path.read_text())
    assert seal(go_path) == r["owned_Go_comparison"]
    assert go["error"] == "<nil>" and (go["modules"], go["packages"], go["exports"], go["export_bytes"]) == (4, 182, 181, expected_export_bytes)
    assert go["image_digest"] == bundle["image_digest"]
    assert not go["comparison_is_kernel_execution_attestation"] and not go["registered_context_or_source_program_proof"]
    for row in r["exports"]:
        assert seal(row["retained_bytes"]["path"]) == row["retained_bytes"]
        raw = base64.b64decode(json.loads(Path(row["retained_bytes"]["path"]).read_text())["base64"], validate=True)
        identity(row["original_export_object"], raw)
    comparisons.append({"record": seal(path), "cache": str(scratch / "handoff" / mode / "work/build"), "owned_Go_comparison": seal(go_path)})
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
assert len(nodes) == bundle["reconciled_original_namespace_nodes"]
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


prior_root = root.parent / "image-consumer"
prior_controls_path = root.parent / "replay-image/initial/linux-controls/result.json"
prior_controls = json.loads(prior_controls_path.read_text())
assert prior_controls["status"] == "passed" and [check["exit_code"] for check in prior_controls["checks"]] == [1, 1, 1]
for check in prior_controls["checks"]:
    for stream in ["stdout", "stderr"]:
        assert seal(check[stream]["path"]) == check[stream]
old_view = (prior_root / "owned-checksums/inputs/go-tools/go2vir/practical_view.go").read_text()
assert old_view == Path("go-tools/go2vir/practical_view.go").read_text()
special_control = root.parent / "namespace-view/linux-special-permission/after/result.json"
assert json.loads(special_control.read_text())["status"] == "passed"

# Check actual failed and fixed epochs with each complete exact source copy.
epochs = []
for phase, expected_status in [("initial/regression", "failed"), ("source-complete/regression", "passed"),
                               ("full-comparison/regression", "passed")]:
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
    if phase == "initial/regression":
        assert result["tests_failed"] == ["TestPracticalObservationOriginalExportBindsACompletePackageWithoutUnsafe"]
    elif phase == "source-complete/regression":
        assert len(result["root_tests_passed"]) == 3 and len(result["subtests_passed"]) == 5
    else:
        assert len(result["root_tests_passed"]) == 1 and len(result["subtests_passed"]) == 12
        for row in result["sources"]:
            assert_current_or_restored_guard(row)
    epochs.append({"record": seal(path), "status": expected_status, "seconds": result["wall_seconds"]})
for phase in ["initial/regression", "source-complete/regression", "owned-observations/selected"]:
    data = json.loads((root / phase / "observation-no-unsafe-binding.json").read_text())
    assert data["native_packages"] == data["typed_packages"] == 1
    assert (data["error"] == "<nil>") != phase.startswith("initial/")
for control in ["unobserved-copy", "compatible-subset", "transport-replacement", "compiler", "main"]:
    data = json.loads((current / "selected" / ("observation-module-" + control + ".json")).read_text())
    assert data["error"] != "<nil>"
for control in ["identity", "canonical-transport", "module-membership", "module-field", "package-field", "unmodelled-root", "unknown-field", "cache-escape", "archive-bytes", "archive-missing", "module-diagnostics", "package-diagnostics"]:
    data = json.loads((root / "full-comparison/regression" / ("observation-comparison-" + control + ".json")).read_text())
    assert (data["error"] == "<nil>") == (control in {"identity", "canonical-transport"})
    assert data["fixture_copies_original_archives"] and not data["independent_native_command_claimed"]
failed = root / "owned-initial/linux-owned/result.json"
r = json.loads(failed.read_text())
assert r["status"] == "failed" and r["sources_unchanged_during_execution"]
assert len(r["checks"]) == 1 and r["checks"][0]["exit_code"] == 1
for check in r["checks"]:
    for stream in ["stdout", "stderr"]:
        assert seal(check[stream]["path"]) == check[stream]
for row in r["sources"]:
    copied = seal(failed.parent / "inputs" / row["path"])
    assert (copied["sha256"], copied["bytes"]) == (row["sha256"], row["bytes"])
failure = json.loads((failed.parent / "replay-native-original-export.json").read_text())
assert failure["error"].endswith("signal: killed")
assert not base64.b64decode(failure["stdout"] or "") and not base64.b64decode(failure["stderr"] or "")
epochs.append({"record": seal(failed), "status": "failed", "scope": "actual original export preparation120s timeout; whole source observations and actual empty failed stdout/stderr preserved; no successful replay substituted"})
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
assert not terminal["live_matching_drivers"] and len(terminal["terminal_results"]) == 13
for row in terminal["terminal_results"]:
    assert seal(row["path"]) == {key: row[key] for key in ["path", "sha256", "bytes"]}

prior = prior_root / "frozen-design-preservation.json"
frozen = json.loads(prior.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for row in frozen["inputs"]:
    assert seal(row["path"]) == row
frozen["prior_complete_manifest"] = seal(prior)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {
    "schema": "mpk.go_practical.native_observation_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "4e805ee2ee352adfac35119f53ee1396db701920", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "checks": checks, "affected_Go_roots": 63, "subtests": 307, "native_checks": native_checks,
    "prior_epochs": epochs, "fixture_payloads": seal(fixture_path), "diagnostic_compatibility_restoration": seal(compatibility_path),
    "supplemental_complete_payloads_independently_checked": fixture["payloads"],
    "supplemental_canonical_source_preimages_independently_checked": canonical_fixture_payloads,
    "genuine_no_unsafe_complete_source_export_regression_reproduced_and_fixed": True,
    "whole_actual_controlled_original_Go_observation_required": True,
    "live_original_Go_owner_compares_two_independent_native_executions": True,
    "complete_metadata_and_archive_correspondence_controls": 12,
    "source_export_snapshot_components_independently_reconstructed": 2,
    "all_materialized_files_independently_checked": len(expected_files), "original_namespace_nodes_independently_reconciled": len(nodes),
    "Darwin_special_mode_controls_not_materialized": 4, "prior_Linux_six_mode_controls_retained": seal(special_control),
    "exact_mode_guard_function_unchanged": True, "terminal_handles": seal(terminal_path),
    "native_image_manifest": seal(bundle_path), "native_comparisons": comparisons,
    "original_source_preimages_independently_checked": retained_count, "original_source_bytes": retained_bytes,
    "native_original_source_objects": 5674, "source_typed_packages": 181, "source_syntax_nodes": 1472495,
    "full_original_modules_equal": 4, "full_original_packages_equal_except_derived_export_locator": 182,
    "whole_original_exports_equal_per_replay": 181, "whole_original_export_bytes_equal_per_replay": expected_export_bytes,
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
    "selection_reason": "Actual privately owned complete Go observations feed the source/module/export consumers and the Go-owned replay baseline. Compare complete native objects and every archive byte, all metadata including unmodelled Root/unknown-field changes, cache confinement, diagnostics, sticky failure and close ownership. Native driver keeps the actual original owner live across two independent fresh-cache offline readonly runs. A single guard diagnostic literal restores prior compatibility after all actual handles terminate; the exact affected guard and two current frontend builds pass, with whole-source literal-only correspondence for retained unaffected runtime results. Affected frontend inputs/legacy compile-only/vet/format; prior unchanged kernel controls retained. No unrelated suites, CSharp, agents, Actions or intermediate whole gate.",
    "remaining": "Production kernel-sandbox lifecycle/private context ownership and complete general consulted namespace/selected-closure bindings remain due. Full56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation/all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 demands/T02-T11 remain required."
}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "affected_Go_roots", "subtests", "original_source_preimages_independently_checked", "full_original_modules_equal", "whole_original_exports_equal_per_replay", "R06_status"]}))
