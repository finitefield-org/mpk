"""Validate all retained streams and run only the one unfinished Go positive."""
from pathlib import Path
import collections
import datetime
import hashlib
import json
import re
import runpy
import subprocess
import time


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
ex_path = root / "final/tests/result.json"
ex = json.loads(ex_path.read_text())
assert ex["status"] == "passed" and not ex["changed_inputs_during_execution"]
assert "3 passed; 0 failed" in (ex_path.parent / "stdout.log").read_text()
for source in ex["sources"]:
    assert seal(source["path"]) == source
prior_path = root / "paired-final/result.json"
prior = json.loads(prior_path.read_text())
assert prior["status"] == "failed" and len(prior["cases"]) == 28
assert seal(prior["go_binary"]["path"]) == prior["go_binary"]
for source in prior["go_production_sources"]:
    assert seal(source["path"]) == source
expected = runpy.run_path(str(root / "planned-controls.py"))["names"]()
positives = {"word-signed-order", "honest-different-signed-order"}
assert {c["name"] for c in prior["cases"]} == expected | {"honest-different-signed-order"}
assert {Path(c["path"]).stem for c in ex["certificates"]} == expected | positives
out = root / "paired-completed"
out.mkdir(exist_ok=False)


def validate(row):
    for key in ["certificate", "rust_report", "stdout", "stderr"]:
        assert seal(row[key]["path"]) == row[key]
    rust, go = [json.loads(Path(row[key]["path"]).read_text()) for key in ["rust_report", "stdout"]]
    assert rust == row["rust_observation"] and go == row["go_observation"]
    assert not Path(row["stderr"]["path"]).read_bytes() and row["same_original_bytes"]
    if row["name"] in positives:
        assert row["exit_code"] == 0 and rust["verdict"] == go["verdict"] == "accepted"
        assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
        assert rust["hashes"] == go["hashes"] and rust["declaration_count"] == go["declaration_count"]
        assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    else:
        assert row["exit_code"] == 1 and rust["verdict"] == go["verdict"] == "rejected"
        detail = json.loads(rust["error_detail"])
        assert rust["error_code"] == "KERNEL_CORE_CHECK" and detail["code"] == "CORE_TYPE_MISMATCH" and go["error_kind"] == "core_check"
        if detail["details"]["kind"] == "check_type_mismatch":
            a, b = [detail["details"][key] for key in ["inferred_kind", "expected_kind"]]
            assert re.fullmatch(r"term [0-9]+ inferred " + re.escape(a) + " but expected " + re.escape(b), go["error_detail"])
        else:
            assert detail["details"]["kind"] == "lambda_domain_mismatch" and go["error_detail"] == "lambda domain mismatch"


for row in prior["cases"]:
    validate(row)
certificate = ex_path.parent / "certificates/word-signed-order.mpcert"
rust_path = certificate.with_suffix(".rust.json")
command = [prior["go_binary"]["path"], "verify", str(certificate)]
started = time.monotonic()
run = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
stdout, stderr = out / "word-signed-order.stdout.log", out / "word-signed-order.stderr.log"
stdout.write_bytes(run.stdout)
stderr.write_bytes(run.stderr)
row = {"name": "word-signed-order", "certificate": seal(certificate), "rust_report": seal(rust_path),
       "command": command, "exit_code": run.returncode, "seconds": time.monotonic() - started,
       "stdout": seal(stdout), "stderr": seal(stderr), "same_original_bytes": True,
       "rust_observation": json.loads(rust_path.read_text()), "go_observation": json.loads(run.stdout)}
record = {"status": "running", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "original_partial_execution": seal(prior_path), "current_Rust_execution": seal(ex_path),
          "go_binary": prior["go_binary"], "go_production_sources": prior["go_production_sources"],
          "cases": [*prior["cases"], row], "complete_original_Go_streams_reused": 28,
          "missing_complete_Go_streams_executed_once": 1, "observer_source": seal(root / "observer-failures/paired-original.py"),
          "observer_diagnostic": seal(root / "observer-failures/paired-original-output.log"),
          "scope": "All29 complete same-byte streams checked by both unchanged cores. Go does not report axiom_count; zero axioms are bound by the exact checked empty axiom-report hash and full Rust report for the same bytes, never an absent-field default. Reuse all28 original complete Go executions after the observer KeyError; execute only the previously unfinished canonical positive. No source/format/fuel/resource failure is counted as semantic rejection, no checker/axiom/fuel/cap changes or source/program/profile/Linux-release authority."}
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
validate(row)
record.update(status="passed", same_byte_streams=29, zero_axiom_positives=2, genuine_core_type_mismatches=27,
              mismatch_subtypes=dict(collections.Counter(json.loads(c["rust_observation"]["error_detail"])["details"]["kind"] for c in record["cases"] if c["name"] in expected)))
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "same_byte_streams", "zero_axiom_positives", "genuine_core_type_mismatches", "mismatch_subtypes"]}))
