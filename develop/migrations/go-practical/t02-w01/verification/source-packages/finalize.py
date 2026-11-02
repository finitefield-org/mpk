"""Bind real closed package/source typing checks without promoting compiler facts to proofs."""
from pathlib import Path
import base64
import collections
import datetime
import hashlib
import json


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def identity(source, raw):
    assert set(source) == {"schema", "id", "capability", "path", "kind", "raw_sha256", "bytes"}
    assert source["schema"] == "mpk.go.source_object.v1" and source["raw_sha256"] == hashlib.sha256(raw).hexdigest()
    assert source["bytes"] == len(raw)
    encoded = json.dumps({k: v for k, v in source.items() if k != "id"}, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + encoded).hexdigest() == source["id"]
    return {"source": source, "independently_reconstructed_id": source["id"]}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
current = root / "original-content"
checks = []
for mode in ["selected", "legacy", "vet", "format"]:
    path = current / mode / "result.json"
    record = json.loads(path.read_text())
    assert record["status"] == "passed" and record["exit_code"] == 0 and record["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(record[key]["path"]) == record[key]
    for source in record["sources"]:
        assert seal(source["path"]) == source
        original = seal(Path(record["source_snapshot"]) / source["path"])
        assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
    if mode == "selected":
        assert len(record["root_tests_passed"]) == 24 and len(record["subtests_passed"]) == 201
    if mode == "legacy":
        assert not record["root_tests_passed"] and not record["subtests_passed"]
    checks.append({"record": seal(path), "seconds": record["wall_seconds"]})
selected = current / "selected"
catalog_path = Path("develop/migrations/go-practical/t01-w04/source-admission/source-vectors.json")
facts_path = catalog_path.parent / "native-facts.jsonl"
catalog = json.loads(catalog_path.read_text())["cases"]
facts = [json.loads(line) for line in facts_path.read_text().splitlines()]
typed_path = selected / "all81-original-types.json"
typed = json.loads(typed_path.read_text())
assert len(catalog) == len(facts) == len(typed["cases"]) == 81
assert (typed["typed"], typed["type_errors"], typed["source_errors"]) == (77, 3, 1)
identities, cases, counts = [], [], collections.Counter()
for case, native, observation in zip(catalog, facts, typed["cases"], strict=True):
    assert observation["id"] == case["id"]
    raw = base64.b64decode(observation["raw"], validate=True)
    assert raw == case["source"].encode() and hashlib.sha256(raw).hexdigest() == native["source_sha256"]
    identities.append(identity(observation["source"], raw))
    assert observation["source"]["capability"] == "workspace/main" and observation["source"]["path"] == "policy.go"
    assert not observation["profile_admission_or_source_proof_claimed"]
    before_path = selected / (hashlib.sha256(("TestPracticalSourceTypesRetainAll81OriginalGoTypeResults/" + case["id"]).encode()).hexdigest() + ".original-inputs.json")
    before = json.loads(before_path.read_text())["original_sources"]["0"]
    assert base64.b64decode(before["policy.go"]) == raw and base64.b64decode(before["go.mod"]) == base64.b64decode(observation["module_raw"])
    expected = native["kind"]
    counts[expected] += 1
    if expected in {"type_error", "source_error"}:
        assert observation["error"] and not observation.get("file_versions")
    else:
        assert expected == "typed_original_package" and not observation.get("error")
        assert observation["definitions"] > 0 and observation["complete_syntax_nodes"] > 0
        assert set(observation["file_versions"].values()) == {case["language"]}
    cases.append({"id": case["id"], "source_id": observation["source"]["id"], "original_native_kind": expected,
                  "exact_original_source_and_generated_module_input_retained": True,
                  "effective_languages": sorted(set(observation.get("file_versions", {}).values())),
                  "original_definition_count": observation.get("definitions"), "typecheck_or_source_error": observation.get("error")})
assert dict(counts) == {"typed_original_package": 77, "type_error": 3, "source_error": 1}
build_path = selected / "package-build.json"
build = json.loads(build_path.read_text())
native = json.loads(base64.b64decode(build["native_raw"]))
all_files = {name: base64.b64decode(raw) for name, raw in build["original_files"].items()}
for source in build["active"] + build["excluded"]:
    identities.append(identity(source, all_files[source["path"]]))
assert [source["path"] for source in build["active"]] == native["GoFiles"]
excluded = sorted(native.get("IgnoredGoFiles", []) + native.get("TestGoFiles", []) + native.get("XTestGoFiles", []))
assert [source["path"] for source in build["excluded"]] == excluded
assert native["Name"] == "policy" and {s["path"] for s in build["active"]} == {"policy.go", "tag.go", "target_linux_amd64.go"}
assert excluded == ["cgo.go", "other_windows.go", "policy_test.go"]
assert "goexperiment.dwarf5" in build["all_tags"] and "amd64.v2" in build["all_tags"]
input_records = sorted(selected.glob("*.original-inputs.json"))
assert len(input_records) == 91
runtime_inputs = []
for path in input_records:
    observation = json.loads(path.read_text())
    assert hashlib.sha256(observation["test"].encode()).hexdigest() + ".original-inputs.json" == path.name
    files = []
    for index, sources in observation["original_sources"].items():
        capability = observation["roots"][int(index)]["Capability"]
        for name, encoded in sorted(sources.items()):
            raw = base64.b64decode(encoded, validate=True)
            files.append({"capability": capability, "path": name, "bytes": len(raw), "raw_sha256": hashlib.sha256(raw).hexdigest()})
    runtime_inputs.append({"record": seal(path), "actual_test": observation["test"], "complete_declared_fixture_files": files})
builds_path = selected / "reproduced-builds.json"
inputs_path = selected / "captured-inputs.json"
builds, inputs = json.loads(builds_path.read_text()), json.loads(inputs_path.read_text())
assert builds["byte_exact_equal"] and builds["independent_empty_caches"] and len(builds["builds"]) == 2
assert (builds["source_inputs"], builds["frontend_files"], builds["vendor_files"]) == (173, 37, 133)
assert len(inputs["build_inputs"]) == 173
for source in inputs["build_inputs"]:
    assert source["capability"] == "analysis-frontend"
    identities.append(identity(source, (Path("go-tools/go2vir") / source["path"]).read_bytes()))
for binary in builds["builds"]:
    assert binary["binary_sha256"] == "257d3603a0fa20493b0ba80e2ad5cab731558baac9f9b1c52ba85eadc3dc1d6a" and binary["bytes"] == 8592898
    assert binary["build_info"]["GoVersion"] == "go1.25.0"
assert len(identities) == 260
failed = []
for attempt, mode, reason in [
    ("initial", "regression", "Invalid cross-package fixture required depGo1.23 under mainGo1.22, so actual Go -mod=readonly rejected module loading. Full original test source and test-level stdout/stderr remain. The previous driver did not expose the nested Go child stdout/stderr; those initial child bytes are not claimed retained or reconstructed."),
    ("fixture-diagnostics", "diagnostic", "Fresh diagnostic reproduction retains complete new generated inputs and actual bounded child stdout/stderr, with the original invalid requirements. These fresh physical-root bytes are not relabeled as the initial child observation."),
    ("original-module-language", "regression", "Original changed-archive source correctly rejected; the new control expected a different diagnostic substring. Actual original rejection and all full source/raw outputs remain; the test expectation is corrected without a product relaxation."),
]:
    path = root / attempt / mode / "result.json"
    observation = json.loads(path.read_text())
    assert observation["status"] == "failed" and observation["exit_code"] != 0 and observation["sources_unchanged_during_execution"]
    for key in ["stdout", "stderr"]:
        assert seal(observation[key]["path"]) == observation[key]
    for source in observation["sources"]:
        original = seal(Path(observation["source_snapshot"]) / source["path"])
        assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
    failed.append({"record": seal(path), "original_source_snapshot": observation["source_snapshot"], "classification": reason,
                   "semantic_source_counterexample_or_proof_resource_verdict": False})
diagnostics = sorted((root / "fixture-diagnostics/diagnostic").glob("*.original-Go-failure.json"))
assert len(diagnostics) == 2
for path in diagnostics:
    observation = json.loads(path.read_text())
    assert b"updates to go.mod needed, disabled by -mod=readonly" in base64.b64decode(observation["stderr"])
prior = root.parent / "source-syntax/frozen-design-preservation.json"
frozen = json.loads(prior.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for source in frozen["inputs"]:
    assert seal(source["path"]) == source
frozen["prior_complete_manifest"] = seal(prior)
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {"schema": "mpk.go_practical.source_package_checkpoint.v1", "status": "scoped_checks_passed",
          "entry_commit": "ae1e11cb85931d708b35f32b1dd84be0fa77e96d", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "checks": checks, "affected_roots": 24, "subtests": 201, "original_source_cases": cases,
          "same_original_81_type_results": dict(counts), "actual_native_build_selection": seal(build_path),
          "same_actual_native_build_active_excluded_files": True,
          "original_runtime_input_records": runtime_inputs, "runtime_input_record_count": 91,
          "independent_source_preimages": identities, "complete_source_preimages_checked": 260,
          "actual_frontend_builds": seal(builds_path), "actual_frontend_inputs": seal(inputs_path),
          "original_failed_epochs": failed, "fresh_original_native_failure_diagnostics": [seal(p) for p in diagnostics],
          "frozen_inputs": seal(frozen_path),
          "production_sources": [seal("go-tools/go2vir/practical_package.go"), seal("go-tools/go2vir/practical_typecheck.go"), seal("go-tools/go2vir/practical_resolution.go")],
          "selection_reason": "New closed target package build/whole-source typing consumers, all81 unchanged original source/type results, native target file selection, relative/absolute replacements with no source/mod rewriting, original cross-package nominal/alias distinctions, original unused body errors, whole versioned ZIP/content linkage and import closure failures. Changed discovery diagnostics require affected actual resolver tests. Existing syntax/snapshot consumers and173-input two-fresh-cache frontend builds are affected. Go1.23 compile/link only and current vet/format pass; unrelated scalar/Rust/kernel/archive oracle/full Go/CSharp suites are not rerun.",
          "all_actual_validation_handles_terminal": True, "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
          "T02_W01_complete": False, "R06_status": "pending", "full_T02_gate_passed": False,
          "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
          "source_or_go_mod_bytes_rewritten_or_stubbed": False, "caller_supplied_typed_package_adapter": False,
          "public_profile_source_program_application_or_Linux_release_claim": False,
          "scope": "Private source-only compiler preparation over captured selected source modules. Complete original selected Go files and bodies are checked with actual Go1.25.0 go/types and gc/amd64 sizes; exact file versions remain. Standard library source/export input absence returns errPracticalNotCaptured, with no guessed import signatures or fallback. unsafe retains the actual Go compiler intrinsic during type checking only; selected unsafe remains outside the v1 source profile. Native Go module graph observations remain preparation observations, not an independently replayed authoritative immutable context.",
          "remaining": "Complete original standard-library/source/export/compiler ownership and authoritative immutable Go graph/package/export/selected-closure replay remain due; R06 pending. Full bounded56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation/all6 common members/19 theorem families, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 application demands/T02-T11 remain required. No source/profile/program proof or all65 relation-consumer claim."}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "roots": 24, "subtests": 201, "original_type_results": dict(counts), "source_preimages": 260,
                  "actual_runtime_fixture_records": 91, "fresh_build_inputs": 173, "all_actual_validation_handles_terminal": True}))
