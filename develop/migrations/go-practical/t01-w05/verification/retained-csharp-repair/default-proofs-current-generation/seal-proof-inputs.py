"""Seal a complete corpus only after generation, normal mode and dual checks pass."""

from pathlib import Path
import datetime
import hashlib
import json


def seal(path):
    data = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


def read(path):
    return json.loads(path.read_bytes())


directory = Path("develop/migrations/go-practical/t01-w05/verification/retained-csharp-repair/default-proofs-current-generation")
diagnosis = directory.parent / "default-proofs-diagnosis"
root = Path("develop/migrations/go-practical/t01-w05/retained-csharp/current-proof-inputs")
manifest = root / "manifest.json"
assert not manifest.exists(), "Never replace a sealed proof corpus."
generation = [directory / "foundation-result.json", directory / "identity/identity-result.json", directory / "default/default-result.json"]
normal = [directory / "normal" / (stage + "-result.json") for stage in ["foundation", "identity", "default"]]
checker_paths = [directory / "checker-foundation-identity-repaired-driver/result.json", directory / "checker-default/result.json"]
snapshot_path = directory / "binary-snapshot-before-control-repair.json"
snapshot = read(snapshot_path)
assert seal(Path(snapshot["snapshot"]["path"])) == snapshot["snapshot"]
assert read(directory / "build/build-result.json")["executable"] == snapshot["original_executable"]
checks = {}
for path in generation + normal + checker_paths:
    record = read(path)
    assert record["status"] == "passed", str(path)
    for field in ["stdout", "stderr"]:
        if field in record:
            assert seal(Path(record[field]["path"])) == record[field]
    if "raw_observations" in record:
        assert seal(Path(record["raw_observations"]["path"])) == record["raw_observations"]
        for observation in record["observations"]:
            assert observation["agreed"]
            if observation["expected"] == "zero_axiom_acceptance":
                checks[observation["raw_certificate_sha256"]] = observation["hashes"]
    if path in generation:
        for member in record["generated_members"]:
            assert seal(Path(member["path"])) == member
    if path in generation + normal:
        assert record["expected_one_test_passed"] and record["exit_code"] == 0
    if path in normal:
        assert record["binary"]["sha256"] == snapshot["snapshot"]["sha256"]
    for member in record.get("test_sources", []):
        assert seal(Path(member["path"])) == member

archive_path = diagnosis / "archive-git-audit.json"
archives = read(archive_path)["archives"]
for member in archives:
    actual = seal(Path(member["path"]))
    assert all(actual[key] == member[key] for key in ["path", "sha256", "bytes"])
assert len(archives) == 138
inventory_path = directory / "preseal-inventory.json"
inventory = read(inventory_path)
assert inventory["case_count"] == 45 and len(inventory["members"]) == 189
for member in inventory["members"]:
    actual = seal(Path(member["path"]))
    assert all(actual[key] == member[key] for key in ["path", "sha256", "bytes"])
ids = sorted(path.stem for path in (root / "foundation").glob("*.json"))
assert len(ids) == 45
members = []
for stage, expected in [("foundation", 91), ("identity", 47), ("default", 51)]:
    paths = sorted((root / stage).iterdir())
    assert len(paths) == expected and all(path.is_file() for path in paths)
    assert sorted(path.stem for path in paths if path.suffix == ".json") == ids
    members.extend(seal(path) for path in paths)

cases = []
totals = {"types": 0, "operations": 0, "identity": 0, "default": 0, "supplied": 0, "remaining": 0, "pending_application_ids": 0, "pending_source_use_defaults": 0}
for case_id in ids:
    metadata_paths = {stage: root / stage / (case_id + ".json") for stage in ["foundation", "identity", "default"]}
    data = {stage: read(path) for stage, path in metadata_paths.items()}
    foundation, identity, default = [data[stage] for stage in ["foundation", "identity", "default"]]
    assert identity["foundation"] == foundation and default["identity"] == identity
    certificates = {"foundation": root / "foundation" / (case_id + ".hex")}
    certificates["identity"] = root / "identity" / (case_id + ".hex") if identity["proofs"] else certificates["foundation"]
    certificates["default"] = root / "default" / (case_id + ".hex") if default["proofs"] else certificates["identity"]
    certificate_records = {}
    for stage, path in certificates.items():
        raw = bytes.fromhex(path.read_text())
        digest = hashlib.sha256(raw).hexdigest()
        assert digest in checks, str(path)
        if stage != "foundation":
            assert data[stage]["certificate_sha256"] == checks[digest]["certificate"]
        certificate_records[stage] = {**seal(path), "raw_certificate_sha256": digest, "checked_hashes": checks[digest]}
        assert data[stage]["pending_proof_ids"] == foundation["pending_proof_ids"]
        assert data[stage]["proof_check_pending"] and data[stage]["application_scope_pending"]
    totals["types"] += len(foundation["types"]["proofs"])
    totals["operations"] += len(foundation["operations"]["proofs"])
    totals["identity"] += len(identity["proofs"])
    totals["default"] += len(default["proofs"])
    totals["supplied"] += len(default["supplied_binding_sequent_ids"])
    totals["remaining"] += len(default["remaining_binding_sequent_ids"])
    totals["pending_application_ids"] += len(default["pending_proof_ids"])
    totals["pending_source_use_defaults"] += len(default["pending_defaults"])
    cases.append({"id": case_id, "metadata": {stage: seal(path) for stage, path in metadata_paths.items()}, "certificates": certificate_records})
assert totals == {"types": 87, "operations": 442, "identity": 2, "default": 3, "supplied": 534, "remaining": 453, "pending_application_ids": 987, "pending_source_use_defaults": 32}
assert len(checks) == 49
record = {
    "schema": "mpk.go_practical.retained_current_proof_inputs.v0",
    "status": "sealed_after_all_selected_checks_passed",
    "sealed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "scope": "Separate current component proof records for the same 45 original CSharp contexts. Historical inputs remain at their original paths. The source-owner tests compare complete current metadata/certificate bytes and retain every original proof and negative check.",
    "context_count": len(ids), "current_member_count": len(members), "retained_member_count": len(archives),
    "totals": totals, "current_members": members, "retained_members": archives, "cases": cases,
    "generation": [seal(path) for path in generation], "normal_mode": [seal(path) for path in normal],
    "independent_checkers": [seal(path) for path in checker_paths],
    "original_proof_owner_binary_snapshot": seal(directory / "binary-snapshot-before-control-repair.json"),
    "complete_generation_inventory_before_validation_finished": seal(inventory_path),
    "metadata_and_source_audits": [seal(diagnosis / name) for name in ["foundation-metadata-audit.json", "identity-metadata-audit.json", "source-preservation-audit.json", "source-inputs-git-audit.json", "archive-git-audit.json"]],
    "production_or_checker_source_changes": False,
    "application_proofs_discharged": False, "whole_gate_passed": False, "T01_complete": False,
}
manifest.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"manifest": seal(manifest), "totals": totals, "current_members": len(members), "retained_members": len(archives)}))
