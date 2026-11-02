"""Bind actual terminal checks, complete source epochs and unchanged frozen inputs."""
from pathlib import Path
import collections
import datetime
import hashlib
import json


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
checks = []
for name in ["construction", "tests", "clippy", "format"]:
    path = root / "typed-environment" / name / "result.json"
    record = json.loads(path.read_text())
    assert record["status"] == "passed" and record["exit_code"] == 0
    assert not record["changed_inputs_during_execution"]
    assert seal(path.parent / "stdout.log") == record["stdout"]
    assert seal(path.parent / "stderr.log") == record["stderr"]
    for source in record["sources"]:
        assert seal(source["path"]) == source
        original = seal(Path(record["source_snapshot"]) / source["path"])
        assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
    checks.append({"record": seal(path), "seconds": record["wall_seconds"]})
pair_path = root / "typed-environment/paired/result.json"
paired = json.loads(pair_path.read_text())
assert paired["status"] == "passed" and len(paired["cases"]) == paired["same_byte_streams"] == 84
assert paired["zero_axiom_positives"] == 4 and paired["genuine_core_type_mismatches"] == 80
assert seal(paired["go_binary"]["path"]) == paired["go_binary"]
for source in paired["go_production_sources"]:
    assert seal(source["path"]) == source
types = collections.Counter()
for case in paired["cases"]:
    for key in ["certificate", "rust_report", "stdout", "stderr"]:
        assert seal(case[key]["path"]) == case[key]
    rust, go = case["rust_observation"], case["go_observation"]
    assert rust["verdict"] == go["verdict"]
    if rust["verdict"] == "accepted":
        assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
        assert rust["hashes"] == go["hashes"]
        assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        detail = json.loads(rust["error_detail"])
        assert detail["code"] == "CORE_TYPE_MISMATCH"
        types[detail["details"]["kind"]] += 1
assert dict(types) == {"lambda_domain_mismatch": 28, "check_type_mismatch": 52}
components = []
for name, declarations, terms, size in [("product-relations", 94, 1621, 16265), ("word-product-equality", 3581, 74155, 1029561)]:
    path = root / f"typed-environment/tests/certificates/{name}.mpcert"
    report = json.loads(path.with_suffix(".rust.json").read_text())
    initial = root / f"typed-environment/construction/certificates/{name}.mpcert"
    assert path.read_bytes() == initial.read_bytes() and path.stat().st_size == size
    assert path.with_suffix(".rust.json").read_bytes() == initial.with_suffix(".rust.json").read_bytes()
    assert report["declaration_count"] == declarations and report["axiom_count"] == 0
    components.append({"certificate": seal(path), "declarations": declarations, "terms": terms,
                       "bytes": size, "axioms": 0, "logical_hashes": report["hashes"],
                       "construction_and_final_full_bytes_and_reports_equal": True})
failed_path = root / "initial/construction/result.json"
failed = json.loads(failed_path.read_text())
assert failed["status"] == "failed" and failed["exit_code"] == 101 and not failed["certificates"]
assert not failed["changed_inputs_during_execution"]
assert "E0282" in (failed_path.parent / "stderr.log").read_text()
for key in ["stdout", "stderr"]:
    assert seal(failed[key]["path"]) == failed[key]
for source in failed["sources"]:
    stored = seal(Path(failed["source_snapshot"]) / source["path"])
    assert (stored["sha256"], stored["bytes"]) == (source["sha256"], source["bytes"])
observer_path = root / "observer-initial/result.json"
observer = json.loads(observer_path.read_text())
assert observer["status"] == "failed" and observer["exit_code"] != 0
for key in ["source", "stdout", "stderr"]:
    assert seal(observer[key]["path"]) == observer[key]
prior_frozen = root.parent / "product-foundation/frozen-design-preservation.json"
frozen = json.loads(prior_frozen.read_text())
assert frozen["frozen_inputs"] == len(frozen["inputs"]) == 255
for source in frozen["inputs"]:
    assert seal(source["path"]) == source
frozen["observed_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
frozen["prior_complete_manifest"] = seal(prior_frozen)
frozen_path = root / "frozen-design-preservation.json"
frozen_path.write_text(json.dumps(frozen, indent=2) + "\n")
record = {
    "schema": "mpk.go_practical.product_relations_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "27490ecb06ff728162bb9499c7b5773c23b83514",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "components": components, "new_generic_definitions": 2, "new_generic_laws": 14,
    "mixed_width_pairs": 16, "concrete_word_pair_laws": 32, "arbitrary_integer_representation_shift_laws": 1,
    "complete_original_identities": {"product": 78, "word_signed_equality": 3489,
                                     "old_product_raw_sha256": "06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e",
                                     "old_word_signed_equality_raw_sha256": "aff545ade52f37b0c5dcf215e8fd5f5aff1a06d269193a9827cbc4b350e6c06f"},
    "checks": checks, "both_cores": seal(pair_path), "same_byte_streams": 84,
    "zero_axiom_positives": 4, "genuine_core_type_mismatches": 80, "mismatch_subtypes": dict(types),
    "failed_original_epochs": [{"record": seal(failed_path), "full_source_snapshot": failed["source_snapshot"],
                                "classification": "Rust E0282 missing Bool-case environment type annotation before any certificate generation; not a semantic negative"},
                               {"record": seal(observer_path), "full_source_snapshot": observer["source"],
                                "classification": "Observer assumed30 lambda/50 check subtypes. Actual80 genuine rejections are28 lambda/52 check. Original unchanged observer is rerun once to capture complete stdout/stderr after its first tool-visible assertion. All84 original Rust/Go verdicts remain unchanged; no semantic negative is added or replaced."}],
    "frozen_inputs": seal(frozen_path),
    "selection_reason": "New relational product definitions/laws and concrete applications depend on existing product and signed-equality foundations. Fresh tests compare every old named type/body/proof/level/family identity and full raw generator bytes; no old declaration is pruned or replaced. Four focused roots cover canonical proof acceptance, every16 mixed-width pair and arbitrary Int representative shifts, exact byte ownership versus genuinely valid extensions, and80 actual false-claim/premise/operation controls. Current Clippy/format and independent same-full-byte Go checks pass. The unchanged65-arity layout matrix, original93600 field obligations and prior Go/frontend matrices are retained without unrelated reruns; no new all65 relation-consumer or source-program acceptance claim.",
    "production_sources": [seal("crates/mpk-vc/src/go_practical_ordinary.rs"), seal("crates/mpk-vc/src/go_practical_ordinary/product_relation.rs")],
    "all_actual_validation_handles_terminal": True, "limits": {"declarations": 4096, "terms": 131072, "bytes": 2097152, "conversion_fuel": 1024},
    "axiom_import_kernel_fuel_or_cap_changes": False, "source_JSON_or_directory_exclusions_added": False,
    "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
    "T02_W01_complete": False, "full_T02_gate_passed": False,
    "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "remaining": "Complete bounded56-family source/carrier/nominal/type/operation/storage/lifetime/state/program foundation with all6 common members and19 theorem families, complete original definitions and compiler-derived operation/VC root closure, private descriptor/registry/context/retained policies and authoritative immutable original package/export/selected-closure replay/all169 application demands/T02-T11 remain required. R06 stays pending. Actual Go struct comparability, field order/evaluation/effects/invocation/source-AST binding are not established by these mathematical relation products.",
    "public_profile_source_application_or_Linux_release_claim": False,
}
(root / "verification.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "streams": 84, "zero_axiom_positives": 4, "type_rejections": 80,
                  "all_actual_validation_handles_terminal": True, "frozen_inputs_unchanged": 255}))
