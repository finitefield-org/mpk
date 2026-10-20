"""Inspect complete original product-foundation streams in both unchanged cores."""
from pathlib import Path
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
execution = root / "final/tests/result.json"
ex = json.loads(execution.read_text())
assert ex["status"] == "passed" and not ex["changed_inputs_during_execution"]
assert "4 passed; 0 failed" in (execution.parent / "stdout.log").read_text()
for source in ex["sources"]:
    assert seal(source["path"]) == source
    original = seal(Path(ex["source_snapshot"]) / source["path"])
    assert (original["sha256"], original["bytes"]) == (source["sha256"], source["bytes"])
prior_path = root.parent / "core-interfaces/paired/result.json"
prior = json.loads(prior_path.read_text())
binary = prior["go_binary"]
assert seal(binary["path"]) == binary
for source in prior["go_production_sources"]:
    assert seal(source["path"]) == source
expected = runpy.run_path(str(root / "planned-controls.py"))["names"]()
positives = {"product-foundation", "honest-different-product"} | {f"balanced-shape-{n:02}" for n in range(65)}
certificates = sorted((execution.parent / "certificates").glob("*.mpcert"))
assert {p.stem for p in certificates} == positives | expected
assert ex["certificates"] == [seal(p) for p in certificates]
out = root / "paired-final"
out.mkdir(exist_ok=False)
record = {"status": "running", "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "Rust_execution": seal(execution), "go_binary": binary, "go_production_sources": prior["go_production_sources"],
          "unchanged_go_build_evidence": seal(prior_path), "cases": [],
          "scope": "Generic heterogeneous products and all65 complete independently constructed balanced-shape consumers. Every complete original closed byte stream is inspected by both unchanged production cores. Acceptances require zero axioms and exact three hashes. All50 altered claims/actual product update and validity definitions/real observation and validity premises require genuine CORE_TYPE_MISMATCH; no format/fuel/resource rejection counts. No kernel/import/axiom/fuel/cap widening, prefix acceptance, source/application/context/profile/Linux-release claim or CSharp tests. All169 application demands and full56-family/T02-T11 scope remain open."}


def save():
    (out / "result.json").write_text(json.dumps(record, indent=2) + "\n")


save()
try:
    for certificate in certificates:
        name = certificate.stem
        rust_path = certificate.with_suffix(".rust.json")
        rust = json.loads(rust_path.read_text())
        command = [binary["path"], "verify", str(certificate)]
        started = time.monotonic()
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        stdout, stderr = out / (name + ".stdout.log"), out / (name + ".stderr.log")
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        go = json.loads(result.stdout)
        row = {"name": name, "certificate": seal(certificate), "rust_report": seal(rust_path),
               "command": command, "exit_code": result.returncode, "seconds": time.monotonic() - started,
               "stdout": seal(stdout), "stderr": seal(stderr), "rust_observation": rust,
               "go_observation": go, "same_original_bytes": True}
        record["cases"].append(row)
        save()
        assert not result.stderr
        if name in positives:
            assert result.returncode == 0 and rust["verdict"] == go["verdict"] == "accepted"
            assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
            assert go["hashes"]["axiom_report"] == rust["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
            assert rust["hashes"] == go["hashes"] and rust["declaration_count"] == go["declaration_count"]
        else:
            assert result.returncode != 0 and rust["verdict"] == go["verdict"] == "rejected"
            detail = json.loads(rust["error_detail"])
            assert detail["code"] == "CORE_TYPE_MISMATCH" and go["error_kind"] == "core_check"
            if detail["details"]["kind"] == "check_type_mismatch":
                inferred, expected = [detail["details"][key] for key in ["inferred_kind", "expected_kind"]]
                assert re.fullmatch(r"term [0-9]+ inferred " + re.escape(inferred) + " but expected " + re.escape(expected), go["error_detail"])
            else:
                assert detail["details"]["kind"] == "lambda_domain_mismatch" and go["error_detail"] == "lambda domain mismatch"
        print(json.dumps({"name": name, "verdict": go["verdict"], "seconds": row["seconds"]}), flush=True)
    assert len(record["cases"]) == 117
    record.update(status="passed", finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  same_byte_streams=117, accepted_zero_axiom_positives=67, genuine_core_type_mismatches=50)
    save()
    print(json.dumps({"status": "passed", "streams": 117, "accepted": 67, "genuine_core_type_mismatches": 50}), flush=True)
except BaseException:
    record["status"] = "failed"
    save()
    raise
