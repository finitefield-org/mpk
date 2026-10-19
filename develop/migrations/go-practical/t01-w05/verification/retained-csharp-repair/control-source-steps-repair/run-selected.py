"""Run the complete original source-step owner or its affected static checks."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time


def seal(path):
    data = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode = sys.argv[1]
assert mode in ("normal", "format", "clippy")
build = json.loads((root / "build-corrected/result.json").read_bytes())
analysis = json.loads((root / "failure-analysis.json").read_bytes())
assert build["status"] == "passed"
assert seal(build["executable"]["path"]) == build["executable"]
for item in build["test_sources"]:
    assert seal(item["path"]) == item
for item in analysis["inputs"] + analysis["production_sources"]:
    current = seal(item["path"])
    assert all(current[key] == item[key] for key in ("path", "sha256", "bytes"))
environment = build["environment"]
env = os.environ.copy()
env.update(environment)
unset = ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS", "MPK_W09_CONTROL_EDGES_OUT"]
previous = {key: env.pop(key, None) for key in unset}
test = "ordinary_carriers::public_domain_tests::contract_expression_tests::control_edge_tests::steps::csharp_03_t06_w09_control_source_steps_original_sources"
commands = {
    "normal": [build["executable"]["path"], "--exact", test, "--nocapture"],
    "format": ["cargo", "fmt", "--package", "mpk-vc", "--", "--check"],
    "clippy": ["cargo", "clippy", "--offline", "-p", "mpk-vc", "--test", "csharp_practical_vc", "--", "-D", "warnings"],
}
reasons = {
    "normal": "The only affected consumer invokes private run_steps. Execute all18 original contexts, full original NativeRuntime::Steps examples, every original argument/bit mutation, independent SSA origins, all component conjunctions, strict import mutations and all36 exact current JSON/certificate goldens. No context filters, output flags, stack overrides or core resource ceilings. Include the existing two memory effects and source constructor without removing any old component checks.",
    "format": "Only the source-step test support file changed in Rust; check its containing package formatting.",
    "clippy": "The private source-step helper is compiled through this integration target. Check it and its callers with warnings denied; completed unrelated owners are not redundantly re-run.",
}
directory = root / mode
directory.mkdir(exist_ok=False)
record = {
    "status": "running", "mode": mode, "command": commands[mode],
    "environment": environment, "unset_environment": unset, "prior_environment": previous,
    "test_sources": build["test_sources"], "binary": build["executable"],
    "build": seal(root / "build-corrected/result.json"),
    "source_preservation_audit": seal(root / "source-preservation-audit-corrected.json"),
    "failure_analysis": seal(root / "failure-analysis.json"),
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "selection_reason": reasons[mode], "original_contexts": [row["case"] for row in analysis["cases"]],
    "full_gate": "The entire repaired T01 check-fast gate remains mandatory. This selected check does not complete W05/T01 or permit T02.",
}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    child = subprocess.Popen(commands[mode], env=env, stdout=out, stderr=err)
    record["pid"] = child.pid
    result.write_text(json.dumps(record, indent=2) + "\n")
    code = child.wait()
record.update(
    status="passed" if code == 0 else "failed", exit_code=code,
    wall_seconds=time.monotonic() - started,
    finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"),
)
if mode == "normal":
    text = (directory / "stdout.log").read_text()
    errors = (directory / "stderr.log").read_text()
    record["expected_one_test_passed"] = "test result: ok. 1 passed; 0 failed;" in text
    record["completed_original_contexts"] = [case for case in record["original_contexts"] if f"control source steps {case}: " in errors]
    record["all18_original_contexts_completed"] = record["completed_original_contexts"] == record["original_contexts"]
    if code == 0 and not (record["expected_one_test_passed"] and record["all18_original_contexts_completed"]):
        record["status"] = "failed"
        record["harness_error"] = "Expected exactly one passing owner and all18 completed original contexts."
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("mode", "status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
