"""Inspect the same complete ordinary normalization inputs in the unchanged Go core."""
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
epoch = sys.argv[1]


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


prior = json.loads(PRIOR.read_bytes())
binary = prior["go_binary"]
assert seal(binary["path"]) == binary
assert all(seal(row["path"]) == row for row in prior["go_production_sources"])
execution = OUT / "epochs" / epoch / "selected/result.json"
record = json.loads(execution.read_bytes())
assert record["status"] == "failed" and record["exit_code"] == 101 and record["whole_source_inputs_unchanged"]
stdout = (execution.parent/"stdout.log").read_text()
core_roots = ["generic_normalization_preserves_all1109_original_identities",
              "exact_ownership_is_distinct_from_core_accepted_extensions",
              "universal_normalization_laws_require_real_conclusions_and_witnesses",
              "wrong_normalizers_are_type_correct_before_their_real_laws_reject"]
assert all("test "+name+" ... ok" in stdout for name in core_roots)
assert "test actual_definitions_preserve_all_bits_and_clear_arbitrary_nonzero_tails ... FAILED" in stdout
assert "finite observation ceiling; universal core proof is checked separately" in (execution.parent/"stderr.log").read_text()
observer_execution = OUT/"epochs/trimmed-observer/observations/result.json"
observer = json.loads(observer_execution.read_bytes())
assert observer["status"] == "passed" and observer["exit_code"] == 0 and observer["whole_source_inputs_unchanged"]
assert all(hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest() == row["sha256"] for row in observer["source_inputs"])
source_delta = [{"path":row["path"], "original_sha256":row["sha256"], "current_sha256":hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest()}
                for row in record["source_inputs"] if hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest() != row["sha256"]]
assert len(source_delta) == 1 and source_delta[0]["path"] == "crates/mpk-vc/tests/support/go_string_normalization_eval.rs"
certificates = sorted((execution.parent / "candidates").glob("*.mpcert"))
assert certificates and {"normalization-foundation", "normalization-honest-extra"} <= {path.stem for path in certificates}
assert len(certificates) == 65
assert sum(path.stem.startswith("false-") for path in certificates) == 28
assert sum(path.stem.startswith("premise-") for path in certificates) == 28
assert sum(path.stem.startswith("operation-") for path in certificates) == 7
out = OUT / "paired"
out.mkdir(exist_ok=False)
driver = out / "check-paired.py"
driver.write_bytes(Path(__file__).read_bytes())
summary = {"status": "running", "Rust_execution": seal(execution), "actual_driver_prelaunch": seal(driver),
           "go_binary": binary, "go_production_sources": prior["go_production_sources"], "unchanged_Go_build_receipt": seal(PRIOR),
           "Rust_complete_core_test_roots_passed": core_roots, "Rust_execution_failed_only_in_finite_observer": True,
           "separate_corrected_observer_execution": seal(observer_execution), "observer_only_source_delta": source_delta,
           "production_and_core_consumer_sources_unchanged": True,
           "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "cases": [],
           "native_release_acceptance": False, "source_program_or_registered_context_proof": False,
           "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh"}


def save():
    (out / "result.json").write_text(json.dumps(summary, indent=2) + "\n")


save()
try:
    for path in certificates:
        rust = json.loads(path.with_suffix(".rust.json").read_bytes())
        positive = path.stem in {"normalization-foundation", "normalization-honest-extra"}
        assert positive or path.stem.startswith(("false-", "premise-", "operation-")), path
        command = [binary["path"], "verify", str(path)]
        started = time.monotonic()
        result = subprocess.run(command, capture_output=True)
        stdout, stderr = out / (path.stem + ".stdout.log"), out / (path.stem + ".stderr.log")
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        go = json.loads(result.stdout)
        row = {"name": path.stem, "expected_acceptance": positive, "input": seal(path), "Rust_report": seal(path.with_suffix(".rust.json")),
               "command": command, "exit_code": result.returncode, "seconds": time.monotonic() - started,
               "stdout": seal(stdout), "stderr": seal(stderr), "Rust": rust, "Go": go}
        summary["cases"].append(row)
        save()
        assert not result.stderr
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
    positives = sum(row["expected_acceptance"] for row in summary["cases"])
    summary.update(status="passed", same_byte_inputs=len(certificates), zero_axiom_acceptances=positives,
                   genuine_core_type_mismatches=len(certificates)-positives,
                   finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
    print(json.dumps({k: v for k, v in summary.items() if k not in {"cases", "go_production_sources"}}))
except BaseException:
    summary["status"] = "failed"
    save()
    raise
