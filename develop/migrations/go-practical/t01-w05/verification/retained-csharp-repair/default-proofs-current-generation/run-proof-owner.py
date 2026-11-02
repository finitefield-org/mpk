"""Record one complete proof-owner invocation without changing its resource limits."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time


def seal(path):
    data = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


stage, mode, build_path, log_path = sys.argv[1:]
assert stage in {"foundation", "identity", "default"}
assert mode in {"generate", "normal"}
build = json.loads(Path(build_path).read_text())
assert build["status"] == "passed"
binary = Path(build["executable"]["path"])
assert seal(binary) == build["executable"]
directory = Path(log_path)
directory.mkdir(parents=True, exist_ok=True)
result = directory / (stage + "-result.json")
assert not result.exists()
inputs = Path("develop/migrations/go-practical/t01-w05/retained-csharp/current-proof-inputs")
output = inputs / stage
if mode == "generate":
    assert not output.exists(), "Never replace an existing proof generation."
owners = {
    "foundation": ("foundation_proof_tests", "csharp_03_t06_w09_foundation_proof_assembly_original_source", "MPK_W09_FOUNDATION_PROOFS_OUT"),
    "identity": ("identity_proof_tests", "csharp_03_t06_w09_identity_projection_proofs_original_source", "MPK_W09_IDENTITY_PROOFS_OUT"),
    "default": ("default_proof_tests", "csharp_03_t06_w09_actual_default_proofs_original_source", "MPK_W09_ACTUAL_DEFAULT_PROOFS_OUT"),
}
module, case, variable = owners[stage]
command = [str(binary), "--exact", "ordinary_carriers::projection_tests::binding_relation_tests::" + module + "::" + case, "--nocapture"]
environment = os.environ.copy()
unset = [
    "RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS",
    "MPK_W09_FOUNDATION_PROOFS_OUT", "MPK_W09_FOUNDATION_SOURCE_FILTER",
    "MPK_W09_IDENTITY_PROOFS_OUT", "MPK_W09_IDENTITY_SOURCE_FILTER",
    "MPK_W09_ACTUAL_DEFAULT_PROOFS_OUT", "MPK_W09_ACTUAL_DEFAULT_SOURCE_FILTER",
]
for name in unset:
    environment.pop(name, None)
if mode == "generate":
    environment[variable] = str(output.resolve())
sources = [Path("crates/mpk-vc/tests/support/csharp_practical_ordinary_" + name + "_proof_tests.rs") for name in ("foundation", "identity", "default")]
record = {
    "status": "running", "stage": stage, "mode": mode,
    "source_base_commit": subprocess.check_output(["/usr/bin/git", "rev-parse", "HEAD"]).decode().strip(),
    "started_at_utc": stamp(), "command": command, "binary": seal(binary),
    "build_record": seal(Path(build_path)), "test_sources": [seal(path) for path in sources],
    "preexisting_current_inputs": [seal(path) for path in sorted(inputs.rglob("*")) if path.is_file()],
    "environment": {variable: environment[variable]} if mode == "generate" else {},
    "unset_environment": [name for name in unset if name not in environment],
    "selection_reason": "All 45 source contexts of the affected proof owner; preserve complete original sequents, prefix, strict import, zero-axiom and negative kernel checks, and all 987 pending application proof IDs. Generation and fixed-golden normal mode are separate runs.",
    "whole_gate_passed": False, "application_proofs_discharged": False,
}
result.write_text(json.dumps(record, indent=2) + "\n")
start = time.monotonic()
stdout = directory / (stage + ".stdout.log")
stderr = directory / (stage + ".stderr.log")
with stdout.open("wb") as out, stderr.open("wb") as err:
    completed = subprocess.run(command, env=environment, stdout=out, stderr=err)
one_passed = "test result: ok. 1 passed; 0 failed; 0 ignored;" in stdout.read_text()
record.update({
    "status": "passed" if completed.returncode == 0 and one_passed else "failed",
    "exit_code": completed.returncode, "expected_one_test_passed": one_passed,
    "wall_seconds": time.monotonic() - start, "finished_at_utc": stamp(),
    "stdout": seal(stdout), "stderr": seal(stderr),
})
if mode == "generate":
    record["generated_members"] = [seal(path) for path in sorted(output.iterdir()) if path.is_file()]
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("stage", "mode", "status", "exit_code", "wall_seconds")}), flush=True)
if record["status"] != "passed":
    print(stderr.read_text()[-2500:])
    raise SystemExit(completed.returncode or 1)
