"""Bind actual signed-order proof, retained original failures and scoped checks."""
from pathlib import Path
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
for attempt, modes in [("initial", ["construction"]), ("diagnose-original", ["diagnostic"]),
                       ("explicit-sign-choice", ["construction"]), ("final", ["tests", "clippy", "format", "diagnostic-list"])]:
    for mode in modes:
        path = root / attempt / mode / "result.json"
        value = read(path)
        assert value["status"] == ("failed" if attempt == "initial" else "passed")
        assert not value["changed_inputs_during_execution"]
        for key in ["stdout", "stderr"]:
            assert seal(value[key]["path"]) == value[key]
        for source in value["sources"]:
            original = seal(Path(value["source_snapshot"]) / source["path"])
            assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
            if attempt == "final":
                assert seal(source["path"]) == source
        runner = root / "initial/runner.py" if attempt == "initial" else root / "run-selected.py"
        assert seal(runner)["sha256"] == value["runner"]["sha256"]
        checks[attempt + "/" + mode] = {"record": seal(path), "wall_seconds": value["wall_seconds"]}
assert "3 passed; 0 failed" in (root / "final/tests/stdout.log").read_text()
assert "1 test, 0 benchmarks" in (root / "final/diagnostic-list/stdout.log").read_text()
assert not (root / "final/format/stdout.log").read_bytes()
certificate = root / "final/tests/certificates/word-signed-order.mpcert"
assert certificate.read_bytes() == (root / "explicit-sign-choice/construction/certificates/word-signed-order.mpcert").read_bytes()
assert seal(certificate)["sha256"] == "0f61e8eb6a89cac3e2fa786f6027901424e2edaeb730df5dee0c5e8db49e8587"
positive = read(certificate.with_suffix(".rust.json"))
assert positive["verdict"] == "accepted" and positive["declaration_count"] == 3457 and positive["axiom_count"] == 0
reader = root.parent / "word-signed-division/prerequisite-analysis/measure.py"
ns = {"__file__": str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]", 1)[0], str(reader), "exec"), ns)
tables = ns["Tables"](seal(certificate))
assert len(tables.declarations) == 3457 and len(tables.terms) == 71954 and len(tables.raw) == 996816
paired = root / "paired-completed/result.json"
pair = read(paired)
assert pair["status"] == "passed" and pair["same_byte_streams"] == 29 and pair["zero_axiom_positives"] == 2
assert pair["genuine_core_type_mismatches"] == 27 and pair["mismatch_subtypes"] == {"check_type_mismatch": 24, "lambda_domain_mismatch": 3}
assert seal(pair["go_binary"]["path"]) == pair["go_binary"]
for source in pair["go_production_sources"]:
    assert seal(source["path"]) == source
for row in pair["cases"]:
    for key in ["certificate", "rust_report", "stdout", "stderr"]:
        assert seal(row[key]["path"]) == row[key]
assert {c["name"] for c in pair["cases"]} == runpy.run_path(str(root / "planned-controls.py"))["names"]() | {"word-signed-order", "honest-different-signed-order"}
finite_path = root / "finite-counterparts/result.json"
finite = read(finite_path)
assert finite["status"] == "passed" and finite["original_signed_queries"] == 96
assert finite["all_original_Bool_named_identities_unchanged"] == 1892 and finite["all_original_order_named_identities_unchanged"] == 3437
assert finite["whole_original_result_rows_cross_checked"] == 1640 and finite["original_native_runtime_outcomes_retained"] == 40
assert seal(finite["counterparts"]["path"]) == finite["counterparts"]
diagnostic = read(root / "diagnose-original/diagnostic/diagnostic.json")
assert diagnostic["first_failed_declaration_index"] == 3448
assert diagnostic["first_failed_declaration_name"] == "Mpk.Go.Practical.Word.SignedOrder.SignedLessScalar"
assert read(root / "original-Go/validated-result.json")["status"] == "both_full_original_type_rejections_preserved"
frozen = read(root.parent / "sealed-namespace/frozen-design-preservation.json")["inputs"]
assert len(frozen) == 255
for source in frozen:
    assert seal(source["path"]) == source
(root / "frozen-design-preservation.json").write_text(json.dumps({"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "frozen_inputs": 255, "inputs": frozen, "mismatches": [], "scope": "All original frozen design/specification/vector inputs remain exact; no checker/import/axiom/fuel/limit or scanner policy changes."}, indent=2) + "\n")
requirements_path = Path("develop/migrations/go-practical/t02-w01/requirements.json")
assert str(requirements_path) not in {r["path"] for r in frozen}
requirements = read(requirements_path)
by = {r["id"]: r for r in requirements["requirements"]}
ordinary = by["T02-W01-R01"]
ordinary.setdefault("prior_ordinary_checkpoints", []).append(ordinary["latest_ordinary_checkpoint"])
ordinary["latest_ordinary_checkpoint"] = {"evidence": "verification/word-signed-order/verification.json",
    "scope": "Actual signed Less/LessEqual at all8/16/32/64 bits agree with the established balanced unbounded integer comparisons, using actual source sign addresses, proved word bounds/projections and genuine unbounded strict-gap closure. All3437 original order named identities and complete old4053-shift bytes remain exact. Both unchanged cores inspect29 complete byte streams, with two zero-axiom acceptances and27 genuine type rejections. This3457-declaration component supplies signed ordering only; complete bounded56-family assembly and source/carrier/nominal/type/operation/storage/lifetime/state/program foundation, descriptor/private registry/context and retained-policy delegation remain due.",
    "not_application_verification_closure": True, "public_profile_activated": False}
checks_owner = by["T02-W01-R07"]
checks_owner.setdefault("prior_scoped_checkpoints", []).append(checks_owner["latest_scoped_checks"])
checks_owner["latest_scoped_checks"] = {"evidence": "verification/word-signed-order/verification.json", "affected_Rust_roots": 3,
    "paired_current_byte_streams": 29, "zero_axiom_positives": 2, "core_type_rejections": 27,
    "reused_original_signed_queries": 96, "all_original_native_rows_preserved": 1640, "original_native_runtime_outcomes_retained": 40,
    "direct_ignored_diagnostic_executed_explicitly": 1, "ignored_helper_listed": True, "unrelated_prior_suites_repeated": False,
    "CSharp_tests_executed": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
assert by["T02-W01-R06"]["status"] == "pending" and not requirements["T02_W01_complete"] and not requirements["public_profile_activated"]
requirements_path.write_text(json.dumps(requirements, ensure_ascii=False, indent=2) + "\n")
record = {"schema": "mpk.go_practical.word_signed_order_checkpoint.v1", "status": "scoped_checks_passed",
          "entry_commit": "63b680e9d3eee4bbd6c30e9869ed692714eaffa2", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "fresh_component": {"certificate": seal(certificate), "declarations": 3457, "terms": 71954, "bytes": 996816,
                              "axioms": 0, "logical_hashes": positive["hashes"], "remaining_declaration_slots": 639},
          "production_sources": [seal("crates/mpk-vc/src/" + path) for path in ["go_practical_ordinary.rs", "go_practical_ordinary/composition.rs", "go_practical_ordinary/word_numeric.rs", "go_practical_ordinary/word_numeric/signed_order.rs"]],
          "checks": checks, "paired_current_cores": seal(paired), "same_byte_streams": 29, "zero_axiom_positives": 2,
          "genuine_core_type_mismatches": 27, "mismatch_subtypes": pair["mismatch_subtypes"], "finite_counterparts": seal(finite_path),
          "original_complete_failed_candidate": seal(root / "initial/construction/certificates/word-signed-order.mpcert"),
          "original_full_Rust_report": seal(root / "initial/construction/certificates/word-signed-order.rust.json"),
          "original_full_Go_observation": seal(root / "original-Go/validated-result.json"),
          "direct_ignored_diagnostic": seal(root / "diagnose-original/diagnostic/diagnostic.json"),
          "observer_failures": [seal(root / p) for p in ["run-original-go.py", "original-Go/observer-diagnostic.log", "observer-failures/finite-original.py", "observer-failures/finite-original-output.log", "observer-failures/paired-original.py", "observer-failures/paired-original-output.log"]],
          "repair_scope": "Explicit ordinary Eq transport of the computed Xor condition to its checked literal before Bool.cases. No new core conversion/iota rule, axiom, fuel, bound or imported assumption. Original insufficient complete recipe remains with both core outcomes and direct suffix diagnosis. Observer repairs retain original sources/raw output, compare actual differing Go/Rust schemas and all40 original native runtime outcomes, reuse28 complete Go streams and execute only the missing canonical positive.",
          "frozen_inputs": seal(root / "frozen-design-preservation.json"), "strict_changed_input_review": "input-review/changed-inputs-review.json",
          "proof_scope": "Unconditional actual signed ordering only. Full3437 original order types/bodies/proofs/levels/families are retained; shared strict-gap proofs are included and rechecked. No source/compiler/application correspondence or complete foundation/context/registry authority is claimed.",
          "CSharp_tests_executed": False, "T02_W01_complete": False, "remaining_application_demands": 169,
          "public_profile_activated": False, "whole_source_program_proof_claimed": False,
          "immutable_application_replay_claimed": False, "native_Linux_gate_claimed": False,
          "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
          "remaining_scope": "Full56-family bounded ordinary foundation and complete source/carrier/nominal/type/operation/storage/lifetime/state/program bindings, descriptor/private registry/context and retained policy delegation. Original application source/export/selected-closure authoritative immutable loading/replay, all169 application demands and original T02-T11 work/gates remain required."}
verification.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "declarations": 3457, "terms": 71954, "bytes": 996816, "same_byte_streams": 29, "genuine_core_type_mismatches": 27, "reused_signed_queries": 96, "frozen_inputs_unchanged": 255}))
