"""Record actual terminal string-component checks without source authority."""
from pathlib import Path
import datetime
import hashlib
import json

OUT = Path(__file__).resolve().parent


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


pair = json.loads((OUT/"paired/result.json").read_bytes())
assert pair["status"] == "passed" and len(pair["cases"]) == 81
assert pair["zero_axiom_acceptances"] == 22 and pair["genuine_core_type_mismatches"] == 59
checks = {}
for name, epoch, mode in [("selected", "native-controls", "selected"), ("lint", "lint-cleanup", "lint"), ("format", "lint-cleanup", "format")]:
    path = OUT/"epochs"/epoch/mode/"result.json"
    row = json.loads(path.read_bytes())
    assert row["status"] == "passed" and row["exit_code"] == 0 and row["whole_source_inputs_unchanged"]
    checks[name] = {"result": seal(path), "stdout": seal(path.parent/"stdout.log"), "stderr": seal(path.parent/"stderr.log")}
foundation = next(row for row in pair["cases"] if row["name"] == "string-foundation")
alphabet = next(row for row in pair["cases"] if row["name"] == "string-all-bytes")
assert foundation["Rust"]["declaration_count"] == 1053 and alphabet["Rust"]["declaration_count"] == 3166
assert len([row for row in pair["cases"] if row["name"].startswith("native-")]) == 18
record = {
    "schema": "mpk.go_practical.ordinary_string_checkpoint.v1", "status": "scoped_checks_passed",
    "entry_commit": "22c964d225582ff6d955c4fe64ad98bbe4ce028f",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "source_language_scope": ["Go", "Rust"],
    "affected_checks": checks, "affected_Rust_test_roots": 5, "independent_comparison": seal(OUT/"paired/result.json"),
    "complete_same_byte_inputs": 81, "zero_axiom_acceptances": 22, "genuine_core_type_mismatches": 59,
    "rejections": {"altered_universal_conclusions": 30, "erased_actual_premises": 18, "well_typed_changed_operations": 5, "changed_actual_literal_bit_claims": 6},
    "component": {"declarations": 1053, "terms": 34875, "bytes": 454703, "new_definitions": 18, "new_laws": 30,
                  "all_original_prerequisite_declarations_exact": 1005, "axioms": 0, "imports": 0, "input": foundation["input"], "hashes": foundation["Rust"]["hashes"]},
    "complete_native_alphabet": {"declarations": 3166, "terms": 40410, "bytes": 706009, "length_bit_laws": 64, "byte_slot_bit_laws": 2048,
                                 "input": alphabet["input"], "hashes": alphabet["Rust"]["hashes"]},
    "actual_original_Go_observations": {"native_inputs": 19, "whole_original_source_and_expression_retained": True,
                                        "full_2112_bit_certificates": 18, "distinct_257_byte_domain_errors": 1,
                                        "original_evidence": "../string-constants/input-review.json",
                                        "source_evaluation_or_nominal_identity_proof": False},
    "ordinary_shape": "(tag:Bool)->Bool.recType (Bool^6->Bool) (Bool^8->Bool^3->Bool) tag; most-significant-head slot/bit addresses",
    "validity": "Actual full unsigned64 Size<=256 and every out-of-prefix Bool8 slot equal to canonical Word8 zero; real product/quantified witnesses required.",
    "zero_validity_proved": True, "native_nonzero_literal_whole_Valid_witnesses_proved": False,
    "source_evaluation_or_nominal_identity_proved": False,
    "selection_reason": "The new source-free string constructor and guarded ordinary operations extend the exact old1005-member component. Check every new universal conclusion, actual supplied range/padding/validity premise, well-typed changed operation, exact owning imports and actual Go native observations. The five target roots plus Clippy/three-file format cover producer/consumer changes; untouched core/ref, Go capture/frontend, protocol and unrelated workspace suites are not repeated. The final edit only changes a fixed domain diagnostic to identical literal bytes; no production recipe or emitted candidate changes.",
    "all_candidates_saved_before_checking": True, "all_actual_test_and_checker_handles_terminal": True,
    "git_whitespace_review": "git-whitespace-review.json: four exact original Cargo stdout EOF blank lines retained; all other staged files pass",
    "original_source_epochs": [str(path.relative_to(OUT)) for path in sorted((OUT/"epochs").glob("*/inputs.json"))],
    "original_failure_history": [
        {"epoch": "initial", "cause": "ZeroValue direct reflexivity assumed unsupported implicit evaluation of the nested numeric recursor. Full original candidate rejects by genuine core type mismatch."},
        {"epoch": "explicit-zero", "cause": "Missing polynomial::mul_right import stops compilation; both actual command outputs remain, with no candidate/outcome invented."},
        {"epoch": "zero-import", "cause": "The ZeroCubeValue1 recipe used its induction hypothesis before explicitly transporting the projected cube equality. Both full original candidate failures remain."},
        {"epoch": "native-controls/lint", "cause": "Clippy rejects unnecessary format! on a fixed native-domain record string; the final identical byte literal fixes only record formatting."},
    ],
    "proof_repair": "One actual Numeric.ValueDefinition per depth plus explicit projected-cube Eq.congr, prior zero values and Nat.MulZero/AddZero. No new core rule, extensionality, semantic axiom or resource extension.",
    "wrong_definition_prefixes": "Five actual changed definition prefixes typecheck; all complete original law streams reject. Prefix typing supplies no partial certificate acceptance.",
    "current_Go_frontend_build_inputs_unchanged": 186, "frozen_design_inputs_unchanged": 255,
    "checker_or_frozen_limit_changes": False, "public_profile_activated": False, "native_release_acceptance": False,
    "R01_complete": False, "R06_complete": False, "W01_complete": False, "full_T_gate_passed": False,
    "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
    "remaining": "Ordinary original source/constant evaluation, actual nominal/alias identity, nonzero literal whole-validity witnesses, signed source-index domain, full string operations/arrays/padding/empty dummy/storage/state/program correspondence; complete ordinary foundation/private registry/context; production native kernel/general namespace/selected closure/all169 demands/T02-T11.",
}
(OUT/"verification.json").write_text(json.dumps(record, indent=2)+"\n")
print(json.dumps({key:record[key] for key in ["status", "affected_Rust_test_roots", "complete_same_byte_inputs", "zero_axiom_acceptances", "genuine_core_type_mismatches", "R01_complete", "R06_complete", "W01_complete"]}))
