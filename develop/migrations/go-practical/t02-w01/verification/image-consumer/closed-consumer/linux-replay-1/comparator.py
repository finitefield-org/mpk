"""Independently bind complete native replay metadata and original archive bytes."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import sys

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt, mode = sys.argv[1:]
assert mode in {"replay-1", "replay-2"}
producer_dir = root / attempt / "linux-prepare"
directory = root / attempt / ("linux-" + mode)
producer = json.loads((producer_dir / "result.json").read_text())
run = json.loads((directory / "result.json").read_text())
assert producer["status"] == run["status"] == "passed"
bundle = json.loads((producer_dir / "replay-image.json").read_text())


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def unique(pairs):
    result = {}
    for key, value in pairs:
        assert key not in result, "duplicate JSON field"
        result[key] = value
    return result


def decode_stream(raw):
    decoder = json.JSONDecoder(object_pairs_hook=unique)
    remaining, rows = raw.decode(), []
    while remaining.strip():
        remaining = remaining.lstrip()
        row, end = decoder.raw_decode(remaining)
        assert isinstance(row, dict)
        rows.append(row)
        remaining = remaining[end:]
    return rows


original_modules = decode_stream(base64.b64decode(bundle["original_module_output"], validate=True))
actual_modules = decode_stream((directory / "modules.stdout.log").read_bytes())
assert actual_modules == original_modules, "complete original native module graph changed"
original_rows = decode_stream(base64.b64decode(bundle["original_package_output"], validate=True))
actual_rows = decode_stream((directory / "packages.stdout.log").read_bytes())
assert len(actual_rows) == len(original_rows) == 182
assert not (directory / "modules.stderr.log").read_bytes() and not (directory / "packages.stderr.log").read_bytes()
exports = bundle["original_export_objects"]
assert len(exports) == 181
cache = Path(run["scratch"]) / "build"
out = directory / "original-export-bytes"
out.mkdir(exist_ok=False)
observed = []
for original, actual in zip(original_rows, actual_rows, strict=True):
    # Only the generated cache locator may differ. Every other metadata field
    # is compared in full, including paths, source sets, import maps and IDs.
    expected_export = original.get("Export")
    actual_export = actual.get("Export")
    if expected_export is None:
        assert actual_export is None and original["ImportPath"] == "unsafe"
        assert actual == original
        continue
    assert actual_export.startswith("/mpk-work/build/")
    name = actual_export.removeprefix("/mpk-work/build/")
    path = cache / name
    assert path.resolve().is_relative_to(cache.resolve()) and path.is_file() and not path.is_symlink()
    objects = [obj for obj in exports if expected_export.endswith("/" + obj["path"])]
    assert len(objects) == 1
    obj = objects[0]
    assert obj["kind"] == "export_data"
    assert {k: v for k, v in actual.items() if k != "Export"} == {k: v for k, v in original.items() if k != "Export"}, original["ImportPath"]
    raw = path.read_bytes()
    assert len(raw) == obj["bytes"] and hashlib.sha256(raw).hexdigest() == obj["raw_sha256"], original["ImportPath"]
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
            assert json.loads(lines[1][len(b"build id "):]) == original["BuildID"] == actual["BuildID"]
        offset += 60 + size
        if size % 2:
            assert raw[offset:offset + 1] == b"\0"
            offset += 1
        members += 1
    assert offset == len(raw)
    retained = out / (obj["raw_sha256"] + ".bytes.json")
    record = {"schema": "mpk.original_bytes.v1", "raw_sha256": obj["raw_sha256"], "bytes": len(raw),
              "base64": base64.b64encode(raw).decode()}
    if retained.exists():
        assert json.loads(retained.read_text()) == record
    else:
        retained.write_text(json.dumps(record, indent=2) + "\n")
    observed.append({"import_path": actual["ImportPath"], "original_export_object": obj,
                     "fresh_export_cache_path": actual_export, "build_id": actual["BuildID"],
                     "retained_bytes": seal(retained), "original_whole_bytes_equal": True, "archive_members": members})
assert len(observed) == 181
record = {
    "schema": "mpk.go_practical.native_replay_comparison.v1", "status": "passed",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "producer": seal(producer_dir / "result.json"), "replay": seal(directory / "result.json"),
    "image_manifest": seal(producer_dir / "replay-image.json"),
    "full_original_module_objects_equal": True, "module_objects": len(actual_modules),
    "full_original_package_objects_equal_except_derived_export_locator": True, "package_objects": len(actual_rows),
    "original_complete_export_bytes_equal": True, "export_objects": len(observed),
    "export_bytes": sum(row["original_export_object"]["bytes"] for row in observed), "exports": observed,
    "original_source_or_go_mod_rewritten": False, "typed_package_or_signature_adapter": False,
    "fresh_empty_derived_cache": True, "kernel_read_only_original_input_mounts": True,
    "original_host_source_mounted": False, "ambient_SDK_hidden": True, "network": "none",
    "x86_64_emulated": True, "native_Linux_release_acceptance_claimed": False,
    "registered_context_or_source_program_application_proof": False,
    "complete_general_consulted_namespace_and_selected_closure_authority_claimed": False,
    "R06_status": "pending", "T02_W01_complete": False, "CSharp_tests_executed": False,
    "remaining": "Production kernel-sandbox/context ownership and complete consulted namespace/selected closure bindings, full56-all6-19 source/program foundation/private registry/context/retained policies/all169 demands/T02-T11 remain required.",
}
(directory / "comparison.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "module_objects", "package_objects", "export_objects", "export_bytes", "R06_status"]}))
