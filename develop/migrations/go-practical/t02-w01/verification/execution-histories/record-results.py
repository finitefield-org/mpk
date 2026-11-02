"""Audit actual checks and every complete emitted mathematical component."""
from pathlib import Path
import hashlib
import importlib.util
import json

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("history_result_inspection", ROOT / "inspect-statements.py")
observer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(observer)


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def main():
    results = {p.parent.name: json.loads(p.read_bytes()) for p in ROOT.glob("*/result.json")}
    for epoch in ["rust-controls-equalities", "lint", "final-format", "statement-syntax-fixed", "paired-error-format-fixed", "reference-build"]:
        assert results[epoch]["status"] == "passed" and results[epoch]["consumed_inputs_unchanged"]
    original = (ROOT / "rust-borrow-fixed/stdout.log").read_text()
    for name in ["unbounded_invariant_partial_run_and_mapping_use_actual_edges", "whole_ordinary_history_component_is_checked_and_exactly_imported"]:
        assert "test " + name + " ... ok" in original
    assert results["rust-borrow-fixed"]["status"] == "failed"
    paired = json.loads((ROOT / "paired/comparison.json").read_bytes())
    assert paired["status"] == "passed" and (paired["accepted"], paired["rejected"]) == (5, 10)
    foundation = ROOT / "rust-borrow-fixed/certificates/foundation.mpcert"
    base_raw, _, base, base_term = observer.decoder.decode(foundation)
    base_names = list(base)
    common, metrics = 0, []
    for row in paired["streams"]:
        path = Path(row["certificate"]["path"])
        assert seal(path) == {k: row["certificate"][k] for k in ["path", "bytes", "sha256"]}
        raw, _, table, term = observer.decoder.decode(path)
        names = list(table)
        for name, original in base.items():
            current = table[name]
            assert current[1] == original[1] and term(current[2]) == base_term(original[2]), name
            if current[1] in [1, 2]:
                assert term(current[3]) == base_term(original[3]), name
                if current[1] == 1:
                    assert current[4] == original[4]
            elif current[1] in [4, 5]:
                assert names[current[3]] == base_names[original[3]] and current[4] == original[4]
            common += 1
        reader = observer.decoder.m.Sections(raw)
        reader.at = 7
        reader.string(); reader.string(); reader.string()
        assert reader.number() == 0
        reader.vec(reader.string); reader.vec(reader.level)
        terms = len(reader.vec(reader.term))
        assert len(table) <= 4096 and terms <= 131072 and len(raw) <= 2097152
        metrics.append({"certificate": str(path), "declarations": len(table), "terms": terms, "bytes": len(raw)})
    statement = json.loads((ROOT / "statement-review.json").read_bytes())
    assert statement["status"] == "passed" and statement["inspection"]["sha256"] == hashlib.sha256(base_raw).hexdigest()
    record = {
        "status": "passed", "scope": "Private ordinary mathematical history library; not closed Go/IR steps, an application proof, complete foundation/context, public activation or Linux release acceptance.",
        "component": statement["inspection"], "all_original_named_declaration_identity_comparisons": common,
        "checkers": {"same_byte_streams": len(metrics), "accepted_zero_axiom_streams": paired["accepted"], "genuine_type_rejections": paired["rejected"], "complete_comparison": seal(ROOT / "paired/comparison.json")},
        "maxima": {key: max(row[key] for row in metrics) for key in ["declarations", "terms", "bytes"]},
        "metrics": metrics,
        "tests": {"passed_original_roots": 2, "passed_isolated_corrected_control_roots": 1, "original_failed_parent_preserved": True,
                  "selection_reason": "Only the new generator/importer and its direct history/invariant/normal-run consumers changed. The two original passing roots and current production definitions remain; only the failed control root reran after test-proof corrections. Changed-file format and strict library/test Clippy passed. Unchanged source/capture, other frontend and large kernel matrices were not repeated."},
        "actual_epochs": [{"epoch": name, "status": row["status"], "result": seal(ROOT / name / "result.json")} for name, row in sorted(results.items())],
        "failure_retention": "Initial source/test syntax errors, Rust borrow error, both rejected full endpoint proof candidates, observer syntax error and both report-format comparison failures remain with complete actual inputs and raw outputs. Explicit Eq.congr computation fixes the endpoint proof without changing its true/false/true history. Original Go results are reused only by exact retained inputs and command; no live check is restarted.",
        "full_gate": "Deferred to final T02-W04: ./scripts/check-fast.sh. R01/R06/W01 and all remaining T02-T11 requirements remain open. Go/Rust only."
    }
    with (ROOT / "verification.json").open("x") as stream:
        json.dump(record, stream, indent=2)
        stream.write("\n")
    print(json.dumps({"status": "passed", "streams": len(metrics), "original_declaration_comparisons": common, "maxima": record["maxima"]}))


if __name__ == "__main__":
    main()
