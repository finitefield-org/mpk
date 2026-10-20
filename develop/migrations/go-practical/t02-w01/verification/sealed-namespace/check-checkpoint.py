"""Check retained executions, exact input epochs and unchanged frozen members."""
from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
verification = json.loads((root / "verification.json").read_text())
records = verification["production_sources"] + [verification[key] for key in [
    "tests", "original_failure", "independent_observations", "actual_frontend_reproduction",
    "frozen_inputs", "preceding_word_component_unchanged",
]] + [value["record"] for value in verification["selected_checks"].values()]
for record in records:
    assert seal(record["path"]) == record
executions = []
for attempt, modes in [("original", ["regression"]), ("final", ["selected", "legacy", "vet", "format"])]:
    for mode in modes:
        record_path = root / attempt / mode / "result.json"
        record = json.loads(record_path.read_text())
        assert record["sources_unchanged_during_execution"]
        assert record["status"] == ("failed" if mode == "regression" else "passed")
        assert record["exit_code"] == (1 if mode == "regression" else 0)
        for key in ["stdout", "stderr", "compiler", "runner"]:
            assert seal(record[key]["path"]) == record[key]
        for source in record["sources"]:
            archived = seal(Path(record["source_snapshot"]) / source["path"])
            assert (archived["sha256"], archived["bytes"]) == (source["sha256"], source["bytes"])
            if attempt == "final":
                assert seal(source["path"]) == source
        env = record["environment"]
        assert all(env[key] == "off" for key in ["GOENV", "GOPROXY", "GOSUMDB", "GOTELEMETRY", "GOWORK"])
        assert env["GOTOOLCHAIN"] == "local" and env["GOVCS"] == "*:off"
        if mode in {"regression", "selected", "legacy"}:
            events = [json.loads(line) for line in Path(record["stdout"]["path"]).read_text().splitlines()]
            passed = [e["Test"] for e in events if e["Action"] == "pass" and "Test" in e]
            assert record["root_tests_passed"] == [n for n in passed if "/" not in n]
            assert record["subtests_passed"] == [n for n in passed if "/" in n]
            expected = {"regression": (0, 0), "selected": (34, 101), "legacy": (7, 11)}[mode]
            assert (len(record["root_tests_passed"]), len(record["subtests_passed"])) == expected
            assert all("CSharp" not in n for n in passed)
        if mode == "regression":
            assert record["tests_failed"] == ["TestPracticalSnapshotOwnsBytesAfterCollectorRelease"]
            assert "sealed bytes alias a released collector buffer" in Path(record["stdout"]["path"]).read_text()
        if mode == "format":
            assert Path(record["stdout"]["path"]).read_bytes() == b""
        executions.append(seal(record_path))
frozen = json.loads((root / "frozen-design-preservation.json").read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255 and not frozen["mismatches"]
for record in frozen["inputs"]:
    assert seal(record["path"]) == record
observer = json.loads((root / "independent-observations.json").read_text())
assert observer["status"] == "passed" and observer["complete_preimages_checked"] == 17
assert observer["source_objects_reconstructed"] == 99
for record in [observer["observer"], observer["original_Go_observations"]]:
    assert seal(record["path"]) == record
for vector in observer["vectors"]:
    assert seal(vector["preimage"]["path"]) == vector["preimage"]
    assert vector["sha256"] == vector["preimage"]["sha256"]
build = json.loads((root / "final/selected/reproduced-builds.json").read_text())
assert build["byte_exact_equal"] and build["independent_empty_caches"] and build["source_inputs"] == 170
assert len(build["builds"]) == 2 and {v["bytes"] for v in build["builds"]} == {8592898}
assert {v["binary_sha256"] for v in build["builds"]} == {"2a9d7c49b925818d372c886397c4e195d5fbf09cb53d4e31c2c39cf49ef6a67f"}
inputs = json.loads((root / "final/selected/captured-inputs.json").read_text())
assert len(inputs["build_inputs"]) == 170 and len(inputs["auxiliary_differences"]) == 1
for obj in inputs["build_inputs"]:
    raw = (Path("go-tools/go2vir") / obj["path"]).read_bytes()
    assert len(raw) == obj["bytes"] and hashlib.sha256(raw).hexdigest() == obj["raw_sha256"]
    assert obj["id"] in inputs["snapshot_source_ids"]
requirements = json.loads(Path("develop/migrations/go-practical/t02-w01/requirements.json").read_text())
by_id = {r["id"]: r for r in requirements["requirements"]}
assert len(by_id) == 7 and by_id["T02-W01-R06"]["status"] == "pending"
assert by_id["T02-W01-R02"]["latest_namespace_checkpoint"]["evidence"] == "verification/sealed-namespace/verification.json"
assert not requirements["T02_W01_complete"] and not requirements["public_profile_activated"]
assert verification["remaining_application_demands"] == 169 and not verification["T02_W01_complete"]
assert not verification["CSharp_tests_executed"] and not verification["public_profile_activated"]
record = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "auditor": seal(__file__), "verification": seal(root / "verification.json"), "executions": executions,
          "current_Go_roots": 34, "current_Go_subtests": 101, "legacy_roots": 7, "legacy_subtests": 11,
          "original_alias_failure_preserved": True, "complete_hash_preimages": 17,
          "complete_source_objects_reconstructed": 99, "actual_owned_frontend_inputs": 170,
          "byte_identical_frontend_builds": 2, "frozen_members_unchanged": 255,
          "CSharp_tests_executed": False, "T02_W01_complete": False,
          "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
          "scope": "Retained local evidence and complete current input epochs only. Final changed-input review remains separate. Full56-family foundation/private context and original application source/export/selected-closure immutable loading/replay, all169 application demands and full T02-T11 scope remain required."}
(root / "checkpoint-audit.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "current_Go_roots", "current_Go_subtests", "legacy_roots", "legacy_subtests", "complete_hash_preimages", "complete_source_objects_reconstructed", "actual_owned_frontend_inputs", "frozen_members_unchanged"]}))
