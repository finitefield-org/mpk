"""Review changed inputs and actual complete source epochs, outside Git sandbox."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = "8b54c50c3c917baff244fd0f8f6102d6e83d8230"
REPORT = OUT / "input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates", "go-tools/mpk-checker-ref", "test-vectors", "release")
assert not Path(".github/workflows").exists()
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_string_constant_input_review")
module.__file__ = str(ROOT / scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(ROOT), "strict"]
exec(compile(code, str(scanner), "exec"), module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"])) == (127, 4, 39)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git("show", BASE + ":" + name)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
findings, inputs, links, decoded = [], [], [], []
cache = {}
path_rules = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name, raw):
    key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m["path"]) for m in path_rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors="ignore"), matchers)]
    findings.extend({**finding, "path": name} for finding in cache[key])


def payload(name, data):
    assert data is None or isinstance(data, str)
    raw = b"" if data is None else base64.b64decode(data, validate=True)
    scan(name, raw)
    decoded.append(seal(name, raw))
    return raw


def source_object(obj, raw):
    assert obj["schema"] == "mpk.go.source_object.v1"
    assert len(raw) == obj["bytes"] and hashlib.sha256(raw).hexdigest() == obj["raw_sha256"]
    data = {key: value for key, value in obj.items() if key != "id"}
    canonical = json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + canonical).hexdigest() == obj["id"]


def native(path, data):
    raw = payload(str(path) + "#original_source", data["original_source"])
    expression = payload(str(path) + "#original_expression", data["original_expression"])
    value = payload(str(path) + "#native_bytes", data["native_bytes"])
    source_object(data["source_object"], raw)
    span = data["span"]
    assert span["source_id"] == data["source_object"]["id"]
    assert 0 <= span["start"] < span["end"] <= len(raw) and raw[span["start"]:span["end"]] == expression
    assert data["package_path"] and data["native_type_display"] and data["image_digest"] and data["compiler_digest"]
    assert data["ordinary_source_evaluation_or_nominal_identity_proof"] is False
    assert data["registered_context_or_program_proof"] is False
    return value


def carrier(data, value, reject=False):
    length, slots = data["length_word"], data["byte_slots"]
    assert len(length) == 64 and all(type(bit) is bool for bit in length)
    assert len(slots) == 256 and all(len(octet) == 8 and all(type(bit) is bool for bit in octet) for octet in slots)
    number = lambda bits: sum((1 << i) for i, bit in enumerate(bits) if bit)
    if reject:
        assert len(value) == 257 and "257 bytes" in data["carrier_error"] and number(length) == 0
        assert not any(number(octet) for octet in slots)
    else:
        assert len(value) <= 256 and number(length) == len(value)
        assert bytes(number(octet) for octet in slots) == value + bytes(256-len(value))


selected = git("diff", "--name-only", "-z", "HEAD", "--") + git("ls-files", "--others", "--exclude-standard", "-z")
paths = sorted(set(selected.decode().split("\0")) - {"", str(REPORT.relative_to(ROOT))})
for name in paths:
    path = ROOT / name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name, raw))
    if name not in configured:
        scan(name, raw)
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#", 1)[0])
            links.append({"document": name, "target": target, "exists": (path.parent / clean).resolve().exists()})

# Resolve the actual original complete epochs, never later reconstructed files.
archives = [json.loads(p.read_bytes()) for p in sorted(OUT.glob("*/inputs.json"))]
assert len(archives) == 3
oids = sorted({row["original_git_blob"] for archive in archives for row in archive["sources"] if "original_git_blob" in row})
raw = subprocess.check_output(["/usr/bin/git", "cat-file", "--batch"], input=("\n".join(oids)+"\n").encode())
originals, cursor = {}, 0
for oid in oids:
    end = raw.index(b"\n", cursor)
    got, kind, size = raw[cursor:end].decode().split()
    assert got == oid and kind == "blob"
    size = int(size)
    originals[oid] = raw[end+1:end+1+size]
    cursor = end+size+2
assert cursor == len(raw)
preimages = 0
for archive in archives:
    for row in archive["sources"]:
        if "original_git_blob" in row:
            raw = originals[row["original_git_blob"]]
            assert row["original_commit"] == BASE
        else:
            raw = Path(row["complete_changed_input"]).read_bytes()
        assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
        preimages += 1
    assert seal(archive["actual_driver_prelaunch"]["path"]) == archive["actual_driver_prelaunch"]
    assert archive["runner"]["sha256"] == archive["actual_driver_prelaunch"]["sha256"]

actual_runs = []
for path in sorted(OUT.glob("*/*/result.json")):
    record = json.loads(path.read_bytes())
    assert record["status"] in {"passed", "failed"} and record["whole_source_inputs_unchanged"]
    for key in ["stdout", "stderr"]:
        assert seal(record[key]["path"]) == record[key]
    if path.parent.parent.name == "initial" and path.parent.name == "selected":
        assert record["status"] == "failed" and record["exit_code"] == 1
        assert record["failed_tests"] == ["TestPracticalStringConstantsOwnOriginalBytesAndCompleteFrozenCarrier/Alias", "TestPracticalStringConstantsOwnOriginalBytesAndCompleteFrozenCarrier"]
    else:
        assert record["status"] == "passed" and record["exit_code"] == 0
        assert not record.get("failed_tests") and not record.get("skipped_tests")
    actual_runs.append(seal(path))

frames, objects, files = [], {}, []
native_observations, accepted_carriers, oversized_carriers = 0, 0, 0
for name in ["initial-selected-evidence", "alias-display-selected-evidence", "complete-outputs-selected-evidence", "alias-display-replay/evidence", "complete-outputs-replay/evidence"]:
    evidence = Path("/private/tmp/mpk-go-practical-string-constants-" + name)
    ids = []
    for path in sorted((evidence / "original-source-objects").glob("*.json")):
        raw = path.with_suffix(".raw").read_bytes()
        metadata = path.read_bytes()
        obj = json.loads(metadata)
        assert obj["id"] == path.stem
        source_object(obj, raw)
        scan(str(path), metadata)
        scan(str(path.with_suffix(".raw")), raw)
        if obj["id"] in objects:
            original = objects[obj["id"]]
            assert seal(path)["sha256"] == original["metadata"]["sha256"] and seal(path.with_suffix(".raw"))["sha256"] == original["raw"]["sha256"]
        else:
            objects[obj["id"]] = {"metadata": seal(path), "raw": seal(path.with_suffix(".raw"))}
        ids.append(obj["id"])
    assert len(ids) == (5674 if "-replay/" in name else 5485)
    frames.append({"directory": str(evidence), "source_object_ids": ids})
    for path in sorted(evidence.glob("*.json")):
        files.append(seal(path))
        data = json.loads(path.read_bytes())
        scan(str(path), path.read_bytes())
        if "source_object" in data:
            if "-damaged-" in path.name:
                for key in ["original_source", "original_expression", "native_bytes"]:
                    payload(str(path) + "#" + key, data[key])
            else:
                value = native(path, data)
                if path.name.endswith("-carrier.json"):
                    reject = len(value) > 256
                    carrier(data, value, reject)
                    oversized_carriers += reject
                    accepted_carriers += not reject
                else:
                    native_observations += 1
        elif "original_files" in data:
            for key, value in data["original_files"].items():
                if isinstance(value, dict):
                    for member, content in value.items():
                        payload(str(path) + "#" + key + "/" + member, content)
                else:
                    payload(str(path) + "#" + key, value)
        else:
            for key in ["stdout", "stderr", "module_output", "module_diagnostics", "module_stdout", "module_stderr", "package_stdout", "package_stderr"]:
                if key in data:
                    payload(str(path) + "#" + key, data[key])
    if "-replay/" in name:
        path = evidence.parent / "replay-image.json"
        record = json.loads(path.read_bytes())
        files.append(seal(path))
        scan(str(path), path.read_bytes())
        assert record["complete_source_typed_packages"] == 181 and record["complete_syntax_nodes"] == 1472495
        assert record["restored_original_module_bindings"] == 4 and record["restored_complete_original_objects"] == 5674
        assert record["original_roots_retired_before_consumer"] and record["original_donor_buffers_cleared_before_consumer"]
        for key in ["original_module_output", "original_package_output", "original_module_diagnostics", "original_package_diagnostics"]:
            if key in record:
                payload(str(path) + "#" + key, record[key])
        for index, data in enumerate(record["original_native_string_constants"]):
            value = native(str(path) + "#" + str(index), data)
            carrier(data, value)
            accepted_carriers += 1
        for item in record["inputs"]:
            raw = Path(item["private"]).read_bytes()
            assert not Path(item["private"]).is_symlink()
            for obj in item["objects"]:
                source_object(obj, raw)
        sdk = next(row for row in record["mounts"] if row["capability"] == "original-go-sdk")
        for item in record["compiler_inventory"]["files"]:
            path = Path(sdk["private"]).parent / item["path"]
            raw = path.read_bytes()
            assert not path.is_symlink() and len(raw) == item["size_bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]

analysis = json.loads((OUT / "alias-display/analysis-inputs/reproduced-builds.json").read_bytes())
assert analysis["source_inputs"] == 186 and analysis["frontend_files"] == 50 and analysis["vendor_files"] == 133
assert analysis["byte_exact_equal"] and analysis["independent_empty_caches"]
assert len({(row["binary_sha256"], row["bytes"]) for row in analysis["builds"]}) == 1
builds = [json.loads((OUT / "build" / name / "result.json").read_bytes()) for name in ["first", "second"]]
assert all(row["status"] == "passed" and seal(row["output"]["path"]) == row["output"] for row in builds)
assert len({(row["output"]["sha256"], row["output"]["bytes"]) for row in builds}) == 1
for row in builds:
    assert seal(row["actual_driver_prelaunch"]["path"]) == row["actual_driver_prelaunch"]
    assert row["native_release_acceptance"] is False and row["registered_binary_installed"] is False
current_inputs = json.loads((OUT / "alias-display/analysis-inputs/captured-inputs.json").read_bytes())["build_inputs"]
assert len(current_inputs) == 186
for row in current_inputs:
    raw = (Path("go-tools/go2vir") / row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
prior_inputs = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/original-packages/integrated/analysis-inputs/captured-inputs.json").read_bytes())["build_inputs"]
assert len(prior_inputs) == 185
assert {row["path"] for row in current_inputs} - {row["path"] for row in prior_inputs} == {"practical_string_constant.go"}
for row in prior_inputs:
    raw = (Path("go-tools/go2vir") / row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
frozen = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json").read_bytes())["inputs"]
assert len(frozen) == 255 and all(seal(row["path"]) == row for row in frozen)
for name in ["develop/docs/go-practical-implementation-traceability-ledger.md", "develop/migrations/go-practical/t02-w01/README.md"]:
    assert Path(name).read_bytes().startswith(git("show", BASE + ":" + name))
original = json.loads(git("show", BASE + ":develop/migrations/go-practical/t02-w01/requirements.json"))
current = json.loads(Path("develop/migrations/go-practical/t02-w01/requirements.json").read_bytes())
for row in current["requirements"]:
    if row["id"] in {"T02-W01-R01", "T02-W01-R03", "T02-W01-R06"}:
        assert row.pop("preparatory_native_string_constant_checkpoint")["status"] == "scoped_checks_passed"
assert current == original
missing = [row for row in links if not row["exists"]]
unchanged = all(seal(row["path"]) == row for row in inputs)
record = {"status": "passed" if not findings and not missing and unchanged else "failed",
          "basis_commit": BASE, "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "inputs": inputs, "source_inputs_unchanged": unchanged, "findings": findings,
          "missing_links": missing, "links_checked": len(links), "complete_actual_runs": actual_runs,
          "original_source_epochs": len(archives), "complete_original_epoch_preimages_verified": preimages,
          "original_source_frames": frames, "unique_complete_original_objects": objects,
          "original_evidence_files": files, "complete_decoded_payloads": decoded,
          "retained_native_observation_occurrences": native_observations,
          "retained_accepted_carrier_occurrences": accepted_carriers, "retained_domain_rejection_occurrences": oversized_carriers,
          "frozen_inputs_unchanged": 255, "prior_frontend_inputs_unchanged": 185, "current_complete_frontend_inputs": 186,
          "matcher_scope_seal_counts": [127, 4, 39], "new_source_JSON_directory_or_blanket_exclusions": False,
          "core_Rust_and_Go_ref_sources_unchanged": True, "native_release_acceptance": False, "full_T_gate_passed": False}
# The report itself is also scanned; its own seal is intentionally nonrecursive.
scan(str(REPORT.relative_to(ROOT)), json.dumps(record, indent=2).encode())
record["findings"] = findings
record["status"] = "passed" if not findings and not missing and unchanged else "failed"
REPORT.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: v for k, v in record.items() if k not in {"inputs", "unique_complete_original_objects", "original_source_frames", "original_evidence_files", "complete_decoded_payloads", "complete_actual_runs"}}))
raise SystemExit(record["status"] != "passed")
