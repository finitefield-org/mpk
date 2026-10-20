"""Record the terminal affected checks without promoting native data to proofs."""
from pathlib import Path
import datetime
import hashlib
import json

OUT = Path(__file__).resolve().parent


def read(name):
    return json.loads((OUT / name).read_bytes())


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


matrix = ["complete-outputs/selected", "alias-display/owner", "complete-outputs/image", "alias-display/analysis-inputs", "initial/public-boundary", "initial/legacy", "complete-outputs/vet", "complete-outputs/format"]
checks = []
roots, subtests = [], []
for name in matrix:
    result = read(name + "/result.json")
    assert result["status"] == "passed" and result["exit_code"] == 0 and result["whole_source_inputs_unchanged"]
    assert not result.get("failed_tests") and not result.get("skipped_tests")
    for key in ["stdout", "stderr"]:
        assert seal(result[key]["path"]) == result[key]
    roots += [test for test in result.get("passed_tests", []) if "/" not in test]
    subtests += [test for test in result.get("passed_tests", []) if "/" in test]
    checks.append({"result": seal(OUT / name / "result.json"), "command": result["command"], "passed_roots": len([test for test in result.get("passed_tests", []) if "/" not in test]), "passed_subtests": len([test for test in result.get("passed_tests", []) if "/" in test])})
assert len(roots) == len(set(roots)) == 10 and len(subtests) == 77
initial = read("initial/selected/result.json")
assert initial["status"] == "failed" and initial["failed_tests"] == ["TestPracticalStringConstantsOwnOriginalBytesAndCompleteFrozenCarrier/Alias", "TestPracticalStringConstantsOwnOriginalBytesAndCompleteFrozenCarrier"]
builds = [read("build/" + name + "/result.json") for name in ["first", "second"]]
assert all(row["status"] == "passed" and row["whole_source_inputs_unchanged"] for row in builds)
assert builds[0]["output"]["sha256"] == builds[1]["output"]["sha256"]
assert builds[0]["output"]["bytes"] == builds[1]["output"]["bytes"]
analysis = read("alias-display/analysis-inputs/reproduced-builds.json")
assert analysis["source_inputs"] == 186 and analysis["frontend_files"] == 50 and analysis["vendor_files"] == 133
assert analysis["byte_exact_equal"] and analysis["independent_empty_caches"]
replay_path = Path("/private/tmp/mpk-go-practical-string-constants-complete-outputs-replay/replay-image.json")
replay = json.loads(replay_path.read_bytes())
assert len(replay["original_native_string_constants"]) == 2
assert replay["complete_source_typed_packages"] == 181 and replay["complete_syntax_nodes"] == 1472495
record = {
    "schema": "mpk.go_practical.native_string_constant_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "8b54c50c3c917baff244fd0f8f6102d6e83d8230",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "source_language_scope": ["Go", "Rust"], "affected_checks": checks,
    "affected_test_roots": roots, "passed_root_count": 10, "passed_subtest_count": 77,
    "new_string_subtests": 42, "retained_owner_subtests": 15,
    "retained_analysis_rejection_subtests": 9, "retained_public_argument_subtests": 11,
    "legacy_tests_executed": 0, "current_vet_and_changed_file_format": "passed",
    "selection_reason": "Private native string observations add one production Go file and a changed whole-module replay consumer. Select the string, original owner, module/replacement/HTTP image, actual build-input/reproduction and closed public consumers. Preserve earlier passing public/legacy/build/owner outcomes after later edits confined to Go1.25 test expectations and diagnostics records; the production186 build inputs remain exact. Final selected/image/vet/format checks cover the actual last source epoch. Unchanged Rust mathematics/core/protocol and unrelated workspace suites are not repeated.",
    "final_source_epoch": "complete-outputs/inputs.json", "original_source_epochs": ["initial/inputs.json", "alias-display/inputs.json", "complete-outputs/inputs.json"],
    "actual_drivers_saved_before_launch": True, "all_original_test_stdout_and_stderr_retained": True,
    "failure_history": [{"result": "initial/selected/result.json", "cause": "The test expected a canonical target display for a native Go type alias. Actual types.TypeString preserves example.com/policy.AliasText; the display-only expectation is corrected without granting nominal/source identity authority.", "original_native_and_source_bytes_retained": True}],
    "earlier_observation_retention_limit": "Initial and alias-display successful child module observations did not separately archive module diagnostics. Their complete original outer test outputs and recorded native/source bytes remain; the complete-outputs epoch archives actual successful module/package stdout/stderr. No later run supplies missing earlier diagnostics.",
    "full_original_byte_evidence_directories": ["/private/tmp/mpk-go-practical-string-constants-initial-selected-evidence", "/private/tmp/mpk-go-practical-string-constants-alias-display-selected-evidence", "/private/tmp/mpk-go-practical-string-constants-complete-outputs-selected-evidence", "/private/tmp/mpk-go-practical-string-constants-alias-display-replay/evidence", "/private/tmp/mpk-go-practical-string-constants-complete-outputs-replay/evidence"],
    "final_multi_module_observation": {"file": seal(replay_path), "modules": replay["restored_original_module_bindings"], "typed_packages": replay["complete_source_typed_packages"], "syntax_nodes": replay["complete_syntax_nodes"], "source_and_export_objects": replay["restored_complete_original_objects"], "verified_materialized_files": replay["verified_materialized_files"], "observed_namespace_nodes": replay["reconciled_original_namespace_nodes"], "native_string_constants": 2},
    "original_roots_and_donor_buffers_retired_before_consumer": True, "returned_observations_survive_owner_close": True,
    "modeled_shape": {"length_bits": 64, "byte_slots": 256, "bits_per_byte": 8, "unoccupied_slots": "all zero", "native_value_outside_modeled_domain": "retain every original byte; return distinct practicalStringDomainError"},
    "fresh_linux_development_builds": [{"result": seal(OUT / "build" / name / "result.json"), "binary": row["output"]} for name, row in zip(["first", "second"], builds)],
    "fresh_linux_development_builds_byte_identical": True,
    "actual_captured_build_inputs": 186, "actual_captured_frontend_sources": 50, "actual_captured_vendor_files": 133,
    "captured_host_reproductions": {"result": seal(OUT / "alias-display/analysis-inputs/reproduced-builds.json"), "binary_sha256": analysis["builds"][0]["binary_sha256"], "bytes": analysis["builds"][0]["bytes"], "independent_empty_cache_builds": 2, "byte_exact_equal": True},
    "frozen_design_inputs_unchanged": 255, "prior_frontend_inputs_unchanged": 185,
    "registered_binary_or_bundle_changed": False, "registered_frontend_identity_reused_for_development_binary": False,
    "ordinary_source_evaluation_or_nominal_identity_proof": False, "registered_context_or_source_program_proof": False,
    "public_practical_profile_activated": False, "native_release_acceptance": False,
    "R01_complete": False, "R06_complete": False, "W01_complete": False, "full_T_gate_passed": False,
    "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
    "remaining": "Actual ordinary source/constant evaluation, exact curried frozen carrier construction and guarded signed index/nominal-array/padding/empty-dummy/storage/state/program proofs; complete ordinary foundation/private registry/context; production native kernel lifecycle, complete general consulted namespace and selected closure; all169 demands/T02-T11.",
}
(OUT / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "passed_root_count", "passed_subtest_count", "actual_captured_build_inputs", "R01_complete", "R06_complete", "W01_complete", "full_T_gate_passed"]}))
