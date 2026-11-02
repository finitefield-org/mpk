"""Inspect every exact full64 word-index input in the independent unchanged Go core."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import time

OUT = Path(__file__).resolve().parent
PRIOR = OUT.parent / "core-interfaces/paired/result.json"


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


prior = json.loads(PRIOR.read_bytes())
binary = prior["go_binary"]
assert seal(binary["path"]) == binary
assert all(seal(row["path"]) == row for row in prior["go_production_sources"])
execution = OUT / ("epochs/" + sys.argv[1] + "/selected/result.json")
record = json.loads(execution.read_bytes())
assert record["status"] == "passed" and record["whole_source_inputs_unchanged"]
assert all(hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest() == row["sha256"] for row in record["source_inputs"])
certificates = sorted((execution.parent / "candidates").glob("*.mpcert"))
assert len(certificates) == 346
assert sum(p.stem.startswith("false-") for p in certificates) == 180
assert sum(p.stem.startswith("premise-") for p in certificates) == 129
assert sum(p.stem.startswith("body-") for p in certificates) == 10
assert sum(p.stem.startswith("guard-") for p in certificates) == 25
assert (execution.parent / "candidates/word-index-foundation.mpcert").read_bytes() == (OUT / "epochs/length/selected/candidates/word-index-foundation.mpcert").read_bytes()
out = OUT / "paired"
out.mkdir(exist_ok=False)
summary = {"status": "running", "Rust_execution": seal(execution), "go_binary": binary,
           "go_production_sources": prior["go_production_sources"], "unchanged_Go_build_receipt": seal(PRIOR),
           "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "cases": [],
           "native_release_acceptance": False, "source_program_or_registered_context_proof": False,
           "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh"}


def save():
    (out / "result.json").write_text(json.dumps(summary, indent=2) + "\n")


save()
try:
    for path in certificates:
        rust = json.loads(path.with_suffix(".rust.json").read_bytes())
        command = [binary["path"], "verify", str(path)]
        started = time.monotonic()
        result = subprocess.run(command, capture_output=True)
        stdout, stderr = out / (path.stem + ".stdout.log"), out / (path.stem + ".stderr.log")
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        go = json.loads(result.stdout)
        row = {"name": path.stem, "input": seal(path), "Rust_report": seal(path.with_suffix(".rust.json")),
               "command": command, "exit_code": result.returncode, "seconds": time.monotonic() - started,
               "stdout": seal(stdout), "stderr": seal(stderr), "Rust": rust, "Go": go}
        summary["cases"].append(row)
        save()
        assert not result.stderr
        positive = path.stem in {"word-index-foundation", "word-index-honest-extra"} or (path.stem.startswith("guard-") and path.stem.endswith(".maximum"))
        if positive:
            assert result.returncode == 0 and rust["verdict"] == go["verdict"] == "accepted"
            assert rust["axiom_count"] == 0 and not rust["axiom_report"]["entries"]
            assert rust["hashes"] == go["hashes"] and rust["declaration_count"] == go["declaration_count"]
            assert go["hashes"]["axiom_report"] == "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
        else:
            assert result.returncode != 0 and rust["verdict"] == go["verdict"] == "rejected"
            detail = json.loads(rust["error_detail"])
            assert detail["code"] == "CORE_TYPE_MISMATCH" and go["error_kind"] == "core_check"
            if detail["details"]["kind"] == "check_type_mismatch":
                inferred, expected = [detail["details"][key] for key in ["inferred_kind", "expected_kind"]]
                assert re.fullmatch(r"term [0-9]+ inferred " + re.escape(inferred) + " but expected " + re.escape(expected), go["error_detail"])
            else:
                assert detail["details"]["kind"] == "lambda_domain_mismatch" and go["error_detail"] == "lambda domain mismatch"
    assert seal(binary["path"]) == binary
    assert all(seal(row["path"]) == row for row in prior["go_production_sources"])
    summary.update(status="passed", same_byte_inputs=346, zero_axiom_acceptances=11, genuine_core_type_mismatches=335,
                   finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
    print(json.dumps({k: v for k, v in summary.items() if k not in {"cases", "go_production_sources"}}))
except BaseException:
    summary["status"] = "failed"
    save()
    raise
