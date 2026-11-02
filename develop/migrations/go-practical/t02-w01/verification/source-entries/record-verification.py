"""Summarize completed affected verification without promoting failed attempts."""
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


def result(epoch):
    path = ROOT / epoch / "result.json"
    value = json.loads(path.read_bytes())
    assert value["status"] == "passed" and value["exit_code"] == 0
    assert value["consumed_inputs_unchanged"]
    for field in ["stdout", "stderr"]:
        assert value[field] == seal(Path(value[field]["path"]))
    return seal(path)


def main():
    rust = result("rust-factored")
    stdout = (ROOT / "rust-factored/stdout.log").read_text()
    expected_roots = ["original_blank_named_and_shadowed_entries_match_native_execution",
                      "source_entries_complete_pure_and_wide_assemblies",
                      "source_entry_controls_reject_premature_locals_and_crossed_arguments"]
    for name in expected_roots:
        assert re.search(r"test " + name + r" \.\.\. ok", stdout)
    assert "3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out" in stdout
    paired = json.loads((ROOT / "paired/comparison.json").read_bytes())
    assert paired["status"] == "passed" and paired["same_bytes_both_full_checkers"]
    assert len(paired["streams"]) == 13 and paired["accepted"] == 8 and paired["rejected"] == 5
    for stream in paired["streams"]:
        assert stream["certificate"] == seal(Path(stream["certificate"]["path"]))
        assert stream["result"] == seal(Path(stream["result"]["path"]))
    rows = json.loads((ROOT / "row-review.json").read_bytes())
    parts = json.loads((ROOT / "entry-review.json").read_bytes())
    assert rows["status"] == parts["status"] == "passed"
    assert parts["independent_rows"] == seal(ROOT / "row-review.json")
    units = []
    for row, part in zip(rows["units"], parts["units"]):
        assert row["unit"] == part["unit"] and row["complete_bodies"] == part["complete_bodies"]
        units.append({k: v for k, v in part.items() if k != "parts"})
    assert [u["complete_bodies"] for u in units] == [47, 117, 4]
    maxima = {key: max(part[key] for unit in parts["units"] for part in unit["parts"]) for key in ["declarations", "terms", "bytes"]}
    assert maxima == {"declarations": 3830, "terms": 130497, "bytes": 1350988}
    traces = json.loads((ROOT / "execution-comparison.json").read_bytes())
    assert traces["status"] == "passed" and traces["complete_original_case_count"] == 127
    assert traces["complete_trace_events_steps_values_faults_and_post_arguments_equal"] and traces["observation_and_input_boundaries_equal"]
    native = json.loads((ROOT / "native/observations.json").read_bytes())
    assert native["status"] == "passed" and native["whole_original_project_executed"] and not native["Linux_release_claim"]
    for original in native["project"]:
        current = seal(Path(original["path"]))
        assert all(current[k] == v for k, v in original.items())
    checks = {"rust": rust, "source_execution": result("matrix"), "native": result("native"), "clippy": result("lint-factored"),
              "format": result("format-checked-factored"), "independent_rows": result("row-inspection"),
              "independent_parts": result("complete-parts-resumed"), "both_checkers": result("paired-resumed")}
    epochs = []
    for receipt in sorted(ROOT.glob("*/inputs.json")):
        path = receipt.with_name("result.json")
        value = json.loads(path.read_bytes())
        assert value["status"] in ["passed", "failed"] and value["inputs"] == seal(receipt)
        epochs.append({"epoch": receipt.parent.name, "status": value["status"], "result": seal(path), "selection_reason": value["selection_reason"]})
    report = {"status": "passed", "driver": seal(Path(__file__).resolve()), "affected_checks": checks,
              "both_checkers": {"comparison": seal(ROOT / "paired/comparison.json"), "streams": 13, "accepted_zero_axiom": 8, "rejected_type_mismatch": 5},
              "independent_rows": seal(ROOT / "row-review.json"), "independent_parts": seal(ROOT / "entry-review.json"), "units": units,
              "entry_members": sum(u["complete_entry_members"] for u in rows["units"]),
              "named_declaration_comparisons_including_common_repetitions": sum(u["whole_original_declaration_comparisons"] for u in rows["units"]),
              "whole_part_maxima": maxima, "unchanged_caps": {"declarations": 4096, "terms": 131072, "bytes": 2097152, "conversion_fuel": 1024},
              "source_execution_comparison": seal(ROOT / "execution-comparison.json"), "native_observations": seal(ROOT / "native/observations.json"),
              "actual_source_epochs": epochs, "retained_interruption": seal(ROOT / "interruption.json"),
              "selection_reason": "New source-owned entry layout and its actual source-interpreter/ordinary-proof consumers. All complete original function/source/type/U scopes and native comparison inputs remain. Unchanged broad kernel and unrelated frontend suites are not repeated; proof-only factoring/packing does not rerun unchanged interpreter/native checks.",
              "review": "Manual source review checks exact binding ownership/type/position, absent locals, blank/unnamed slots, source-derived result zero values, whole-scope hashes, deterministic complete packing, exact import, unchanged checker caps and original candidate retention. Independent row/part reconstruction and actual same-byte checker runs provide separate evidence.",
              "scope": "Entry-value equations only. Not a complete program certificate, source/IR simulation, storage/frame/callee/entry-validity proof or authenticated installed capture/context.",
              "due": ["full56-family foundation and private integration", "all169 source/program demands", "R01", "R06", "T02-W01", "native Linux release gates", "remaining T02-T11"],
              "practical_profile_active": False, "retained_product_languages": ["Go", "Rust"],
              "full_gate": "Deferred to final T02-W04: ./scripts/check-fast.sh"}
    with (ROOT / "verification.json").open("x") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    print(json.dumps({"status": "passed", "bodies": 168, "entry_members": report["entry_members"], "same_byte_streams": 13}))


if __name__ == "__main__":
    main()
