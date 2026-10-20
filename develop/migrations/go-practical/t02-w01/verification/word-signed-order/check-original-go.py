"""Classify the already completed original Go stream without repeating it."""
from pathlib import Path
import hashlib
import json
import re


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
p = root / "original-Go/result.json"
original = json.loads(p.read_text())
for key in ["certificate", "go_binary", "stdout", "stderr"]:
    assert seal(original[key]["path"]) == original[key]
for source in original["go_production_sources"]:
    assert seal(source["path"]) == source
rust_path = root / "initial/construction/certificates/word-signed-order.rust.json"
rust = json.loads(rust_path.read_text())
detail = json.loads(rust["error_detail"])
go = original["observation"]
assert original["exit_code"] == 1 and go["verdict"] == rust["verdict"] == "rejected"
assert go["error_kind"] == "core_check" and detail["code"] == "CORE_TYPE_MISMATCH"
assert detail["details"]["kind"] == "check_type_mismatch"
inferred, expected = [detail["details"][key] for key in ["inferred_kind", "expected_kind"]]
assert re.fullmatch(r"term [0-9]+ inferred " + re.escape(inferred) + " but expected " + re.escape(expected), go["error_detail"])
record = {"status": "both_full_original_type_rejections_preserved", "original_go_run": seal(p),
          "original_rust_report": seal(rust_path), "observer_source": seal(root / "run-original-go.py"),
          "observer_diagnostic": seal(root / "original-Go/observer-diagnostic.log"), "corrected_classifier": seal(__file__),
          "Go_process_repeated": False,
          "scope": "The original observer wrongly assumed JSON inside the Go core's human error_detail. Preserve that source/traceback and classify the exact already-completed raw Go report against the structured Rust mismatch. This is an insufficient original proof recipe, not a semantic control or a Go source counterexample."}
(root / "original-Go/validated-result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "Go_process_repeated": False}))
