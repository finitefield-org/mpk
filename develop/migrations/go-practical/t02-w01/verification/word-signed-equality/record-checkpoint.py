"""Bind complete signed equality proofs, exact old identities and scoped evidence."""
from pathlib import Path
from collections import Counter
import datetime
import hashlib
import json
import runpy


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def read(path):
    return json.loads(Path(path).read_text())


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
verification = root / "verification.json"
assert not verification.exists()
checks = {}
for attempt, modes in [("initial", ["construction"]), ("final", ["tests", "clippy", "format"])]:
    for mode in modes:
        path = root / attempt / mode / "result.json"
        value = read(path)
        assert value["status"] == "passed" and value["exit_code"] == 0
        assert not value["changed_inputs_during_execution"]
        for key in ["stdout", "stderr"]:
            assert seal(value[key]["path"]) == value[key]
        for source in value["sources"]:
            original = seal(Path(value["source_snapshot"]) / source["path"])
            assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
            if attempt == "final":
                assert seal(source["path"]) == source
        runner = root / attempt / "runner.py"
        assert seal(runner)["sha256"] == value["runner"]["sha256"] == seal(root / "run-selected.py")["sha256"]
        for cert in value["certificates"]:
            assert seal(cert["path"]) == cert
        checks[attempt + "/" + mode] = {"record": seal(path), "wall_seconds": value["wall_seconds"]}
assert "3 passed; 0 failed" in (root / "final/tests/stdout.log").read_text()
assert not (root / "final/format/stdout.log").read_bytes()
certificate = root / "final/tests/certificates/word-signed-equality.mpcert"
assert certificate.read_bytes() == (root / "initial/construction/certificates/word-signed-equality.mpcert").read_bytes()
assert seal(certificate)["sha256"] == "aff545ade52f37b0c5dcf215e8fd5f5aff1a06d269193a9827cbc4b350e6c06f"
positive = read(certificate.with_suffix(".rust.json"))
assert positive["verdict"] == "accepted" and positive["declaration_count"] == 3489 and positive["axiom_count"] == 0
assert not positive["axiom_report"]["entries"]
reader = root.parent / "word-signed-division/prerequisite-analysis/measure.py"
ns = {"__file__": str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]", 1)[0], str(reader), "exec"), ns)
tables = ns["Tables"](seal(certificate))
assert len(tables.declarations) == 3489 and len(tables.terms) == 72690 and len(tables.raw) == 1007869
paired = root / "paired-final/result.json"
pair = read(paired)
assert pair["status"] == "passed" and pair["same_byte_streams"] == 46 and pair["accepted_zero_axiom_positives"] == 2
assert pair["genuine_core_type_mismatches"] == 44
assert seal(pair["go_binary"]["path"]) == pair["go_binary"]
for source in pair["go_production_sources"]:
    assert seal(source["path"]) == source
subtypes = Counter()
for row in pair["cases"]:
    for key in ["certificate", "rust_report", "stdout", "stderr"]:
        assert seal(row[key]["path"]) == row[key]
    rust, go = row["rust_observation"], row["go_observation"]
    if rust["verdict"] == "accepted":
        assert rust["hashes"] == go["hashes"] and rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
        assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        assert rust["error_code"] == "KERNEL_CORE_CHECK" and go["error_kind"] == "core_check"
        detail = json.loads(rust["error_detail"])
        assert detail["code"] == "CORE_TYPE_MISMATCH"
        subtypes[detail["details"]["kind"]] += 1
assert sum(subtypes.values()) == 44
assert {c["name"] for c in pair["cases"]} == runpy.run_path(str(root / "planned-controls.py"))["names"]() | {"word-signed-equality", "honest-different-signed-equality"}
finite_path = root / "finite-counterparts/result.json"
finite = read(finite_path)
assert finite["status"] == "passed" and finite["original_equality_queries"] == 96
assert finite["all_original_Bool_named_identities_unchanged"] == 1892 and finite["all_original_signed_order_named_identities_unchanged"] == 3457
assert finite["whole_original_result_rows_cross_checked"] == 1640 and finite["original_native_runtime_outcomes_retained"] == 40
assert seal(finite["counterparts"]["path"]) == finite["counterparts"]
native_path = root / "native/result.json"
native = read(native_path)
assert native["status"] == "passed" and native["native_rows"] == 96 and native["two_native_outputs_byte_identical"]
for key in ["original_source", "owned_source", "original_queries", "owned_queries", "complete_original_prepared_image", "compiler", "observer_binary"]:
    assert seal(native[key]["path"]) == native[key]
for command in native["commands"]:
    assert command["exit_code"] == 0
    for key in ["stdout", "stderr"]:
        assert seal(command[key]["path"]) == command[key]
for image in native["complete_original_image_checks"]:
    assert image["status"] == "passed" and image["files"] == 5577 and image["original_bytes"] == 163457770
assert (root / "native/run1.stdout.log").read_bytes() == (root / "native/run2.stdout.log").read_bytes()
native_checks_path = root / "native-checks/result.json"
native_checks = read(native_checks_path)
assert native_checks["status"] == "passed"
assert seal(native_checks["runner"]["path"]) == native_checks["runner"]
assert (root / "native-checks/runner.py").read_bytes() == (root / "check-native.py").read_bytes()
for command in native_checks["commands"]:
    assert command["exit_code"] == 0
    for key in ["stdout", "stderr"]:
        assert seal(command[key]["path"]) == command[key] and command[key]["bytes"] == 0
for image in native_checks["complete_original_image_checks"]:
    assert image["status"] == "passed" and image["files"] == 5577
planning = root / "dependency-analysis/result.json"
assert read(planning)["status"] == "structural_planning_not_checker_authority"
for original in read(planning)["original_certificates"]:
    assert seal(original["path"]) == original
frozen = read(root.parent / "word-signed-order/frozen-design-preservation.json")["inputs"]
assert len(frozen) == 255
for source in frozen:
    assert seal(source["path"]) == source
(root / "frozen-design-preservation.json").write_text(json.dumps({"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "frozen_inputs": 255, "inputs": frozen, "mismatches": [], "scope": "Every original frozen design/specification/vector member remains exact. No checker/import/axiom/fuel/limit/scanner policy change."}, indent=2) + "\n")
requirements_path = Path("develop/migrations/go-practical/t02-w01/requirements.json")
assert str(requirements_path) not in {r["path"] for r in frozen}
requirements = read(requirements_path)
by = {r["id"]: r for r in requirements["requirements"]}
ordinary = by["T02-W01-R01"]
ordinary.setdefault("prior_ordinary_checkpoints", []).append(ordinary["latest_ordinary_checkpoint"])
ordinary["latest_ordinary_checkpoint"] = {"evidence": "verification/word-signed-equality/verification.json",
    "scope": "Actual all8/16/32/64 source Equal/NotEqual agree with real balanced unbounded Int.Equal, with both Int.Equivalent reflection directions. Actual full balanced bit equality yields source top-sign coherence in3/4/5/6 checked steps, then real Nat value equality supplies the fully discharged scalar coherence callback. All3457 previous signed-order identities and full old raw bytes remain exact. Both unchanged cores inspect46 full streams: two zero-axiom positives and44 genuine type rejections. This3489-declaration component does not complete bounded56-family assembly/source/carrier/nominal/type/operation/storage/lifetime/state/program foundation, descriptor/private registry/context or retained-policy delegation.",
    "not_application_verification_closure": True, "public_profile_activated": False}
checks_owner = by["T02-W01-R07"]
checks_owner.setdefault("prior_scoped_checkpoints", []).append(checks_owner["latest_scoped_checks"])
checks_owner["latest_scoped_checks"] = {"evidence": "verification/word-signed-equality/verification.json", "affected_Rust_roots": 3,
    "paired_current_byte_streams": 46, "zero_axiom_positives": 2, "core_type_rejections": 44,
    "mismatch_subtypes": dict(subtypes), "reused_original_equality_queries": 96, "fresh_typed_signed_unsigned_native_rows": 96,
    "all_original_native_rows_preserved": 1640, "original_native_runtime_outcomes_retained": 40,
    "unchanged_original_Bool_model_rerun": False, "new_observer_vet_format_passed": True, "unrelated_prior_suites_repeated": False,
    "CSharp_tests_executed": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
assert len(by) == 7 and by["T02-W01-R06"]["status"] == "pending"
assert not requirements["T02_W01_complete"] and not requirements["public_profile_activated"]
requirements_path.write_text(json.dumps(requirements, ensure_ascii=False, indent=2) + "\n")
record = {"schema": "mpk.go_practical.word_signed_equality_checkpoint.v1", "status": "scoped_checks_passed",
          "entry_commit": "c4bd126d0e347f09db444d35df3831c61e1fd4c6", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "fresh_component": {"certificate": seal(certificate), "declarations": 3489, "terms": 72690, "bytes": 1007869,
                              "axioms": 0, "logical_hashes": positive["hashes"], "remaining_declaration_slots": 607},
          "production_sources": [seal("crates/mpk-vc/src/" + path) for path in ["go_practical_ordinary.rs", "go_practical_ordinary/word.rs", "go_practical_ordinary/word_numeric.rs", "go_practical_ordinary/word_numeric/signed_equality.rs"]],
          "checks": checks, "paired_current_cores": seal(paired), "same_byte_streams": 46, "zero_axiom_positives": 2,
          "genuine_core_type_mismatches": 44, "mismatch_subtypes": dict(subtypes), "finite_counterparts": seal(finite_path),
          "fresh_native_equality": seal(native_path), "new_native_observer_vet_format": seal(native_checks_path),
          "dependency_planning": seal(planning), "all_prior_signed_order_named_identities_unchanged": 3457,
          "old_signed_order_raw_sha256": "0f61e8eb6a89cac3e2fa786f6027901424e2edaeb730df5dee0c5e8db49e8587",
          "frozen_inputs": seal(root / "frozen-design-preservation.json"), "strict_changed_input_review": "input-review/changed-inputs-review.json",
          "proof_scope": "Unconditional source signed equality/inequality and both balanced Int equivalence directions, preserving all3457 prior signed-order declarations. Real source equality/tree and established Nat EqualNatural discharge actual sign coherence. No extensionality, new core rule/axiom, imported theorem trust or caller-supplied truth flags. Prior entire signed bounds component remains intact and full56-family assembly remains due; avoiding its unneeded382-declaration half-bound dependency is only a choice within this equality proof recipe.",
          "CSharp_tests_executed": False, "T02_W01_complete": False, "remaining_application_demands": 169,
          "public_profile_activated": False, "whole_source_program_proof_claimed": False,
          "immutable_application_replay_claimed": False, "native_Linux_gate_claimed": False,
          "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
          "remaining_scope": "Full56-family bounded ordinary foundation and complete source/carrier/nominal/type/operation/storage/lifetime/state/program bindings, descriptor/private registry/context and retained policy delegation. Original application source/export/selected-closure authoritative immutable loading/replay, all169 application demands and original T02-T11 work/gates remain required."}
verification.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "declarations": 3489, "terms": 72690, "bytes": 1007869,
                  "same_byte_streams": 46, "genuine_core_type_mismatches": 44, "mismatch_subtypes": dict(subtypes),
                  "original_equality_queries": 96, "frozen_inputs_unchanged": 255}))
