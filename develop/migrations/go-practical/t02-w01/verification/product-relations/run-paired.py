"""Verify every whole original product-relation stream in the independent Go core."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import time


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
attempt = sys.argv[1]
assert attempt.replace("-", "").isalnum()
execution = root / attempt / "tests/result.json"
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
positives = {"product-relations", "word-product-equality", "honest-different-relations", "honest-different-word-pairs"}
members = ["RelatedMake", "RelatedLeft", "RelatedRight", "Reflexive", "Symmetric", "Transitive",
           "SetLeftRelated", "SetRightRelated", "MapRelated", "AndTrueLeft", "AndTrueRight", "AndTrueIntro",
           "EqualByReflect", "EqualByComplete"]
negatives = {f"false-{name}" for name in members}
for name, indices in [
    ("RelatedMake", [6, 7]), ("RelatedLeft", [6]), ("RelatedRight", [6]), ("Reflexive", [5, 6]),
    ("Symmetric", [6, 7, 8]), ("Transitive", [6, 8, 9, 10]), ("SetLeftRelated", [6, 9]),
    ("SetRightRelated", [6, 9]), ("MapRelated", [6, 13, 14]), ("AndTrueLeft", [2]),
    ("AndTrueRight", [2]), ("AndTrueIntro", [2, 3]), ("EqualByReflect", [8, 9, 10]),
    ("EqualByComplete", [8, 9, 10]),
]:
    negatives.update(f"erased-{name}-{index}" for index in indices)
negatives.update({"drop-right-comparator", "related-requires-right-witness", "wrong-related-right-projection",
                  "false-integer-pair-representation-shift"})
negatives.update(f"false-W{left}W{right}-{member}" for left in [8, 16, 32, 64]
                 for right in [8, 16, 32, 64] for member in ["EquivalentReflect", "EquivalentComplete"])
assert len(negatives) == 80
certificates = sorted((execution.parent / "certificates").glob("*.mpcert"))
assert {p.stem for p in certificates} == positives | negatives
assert ex["certificates"] == [seal(p) for p in certificates]
out = root / attempt / "paired"
out.mkdir(exist_ok=False)
record = {"status": "running", "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "actual_terminal_Rust_execution": seal(execution), "runner": seal(__file__),
          "go_binary": binary, "go_production_sources": prior["go_production_sources"],
          "prior_actual_binary_build_evidence": seal(prior_path), "cases": [],
          "scope": "Complete original generic relation product and all16 mixed-width word comparison/Int equivalence bridges, arbitrary balanced Int representation shifts, exact ownership. Every new law false-claim control, real relation/preservation/comparator premise and actual comparator/relation/projection corruption must genuinely type-reject in both unchanged cores. No missing Go axiom_count default, prefix/limit/fuel substitute, new axiom/import/kernel rule or widened cap. Full56 source/nominal/type/operation/storage/lifetime/state/program foundation/private descriptor/registry/context/immutable original application replay/all169 demands/T02-T11 remain due; no public/source/application/Linux release claim or CSharp tests."}


def save():
    (out / "result.json").write_text(json.dumps(record, indent=2) + "\n")


save()
try:
    for certificate in certificates:
        name = certificate.stem
        rust_path = certificate.with_suffix(".rust.json")
        rust = json.loads(rust_path.read_text())
        before, rust_before = seal(certificate), seal(rust_path)
        command = [binary["path"], "verify", str(certificate)]
        started = time.monotonic()
        completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        stdout, stderr = out / (name + ".stdout.log"), out / (name + ".stderr.log")
        stdout.write_bytes(completed.stdout)
        stderr.write_bytes(completed.stderr)
        go = json.loads(completed.stdout)
        row = {"name": name, "certificate": before, "rust_report": rust_before,
               "command": command, "exit_code": completed.returncode, "seconds": time.monotonic() - started,
               "stdout": seal(stdout), "stderr": seal(stderr), "rust_observation": rust,
               "go_observation": go, "entire_original_bytes_sealed_before_and_after_actual_Go_execution": True}
        record["cases"].append(row)
        save()
        assert seal(certificate) == before and seal(rust_path) == rust_before and not completed.stderr
        if name in positives:
            assert completed.returncode == 0 and rust["verdict"] == go["verdict"] == "accepted"
            assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
            assert rust["hashes"] == go["hashes"] and rust["declaration_count"] == go["declaration_count"]
            assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
        else:
            assert completed.returncode != 0 and rust["verdict"] == go["verdict"] == "rejected"
            assert rust["error_code"] == "KERNEL_CORE_CHECK" and go["error_kind"] == "core_check"
            detail = json.loads(rust["error_detail"])
            assert detail["code"] == "CORE_TYPE_MISMATCH"
            if detail["details"]["kind"] == "check_type_mismatch":
                inferred, expected = [detail["details"][key] for key in ["inferred_kind", "expected_kind"]]
                assert re.fullmatch(r"term [0-9]+ inferred " + re.escape(inferred) + " but expected " + re.escape(expected), go["error_detail"])
            else:
                assert detail["details"]["kind"] == "lambda_domain_mismatch" and go["error_detail"] == "lambda domain mismatch"
        print(json.dumps({"name": name, "verdict": go["verdict"], "seconds": row["seconds"]}), flush=True)
    assert seal(binary["path"]) == binary
    for source in prior["go_production_sources"]:
        assert seal(source["path"]) == source
    for source in ex["sources"]:
        assert seal(source["path"]) == source
    record.update(status="passed", same_byte_streams=84, zero_axiom_positives=4, genuine_core_type_mismatches=80,
                  finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
    print(json.dumps({"status": "passed", "streams": 84, "positives": 4, "type_rejections": 80}), flush=True)
except BaseException:
    record["status"] = "failed"
    save()
    raise
