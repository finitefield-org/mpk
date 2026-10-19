"""Record shared metadata preflight and the unabridged original runtime owner."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
import time


def seal(path):
    data = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode = sys.argv[1]
assert mode in ("normal", "metadata", "constructor-metadata", "format", "clippy")
build = json.loads((root / "build/result.json").read_bytes())
analysis = json.loads((root / "binding-origin-analysis.json").read_bytes())
census = json.loads((root.parent / "failure-analysis.json").read_bytes())
assert build["status"] == "passed"
assert seal(build["executable"]["path"]) == build["executable"]
assert [seal(item["path"]) for item in build["test_sources"]] == build["test_sources"]
for item in analysis["inputs"] + analysis["production_sources"]:
    actual = seal(item["path"])
    assert all(actual[key] == item[key] for key in ("path", "sha256", "bytes"))
prefix = "ordinary_carriers::public_domain_tests::contract_expression_tests::control_edge_tests::steps::"
tests = {
    "normal": prefix + "csharp_03_t06_w09_control_source_steps_original_sources",
    "metadata": prefix + "csharp_03_t06_w09_control_source_steps_completeness_metadata",
    "constructor-metadata": prefix + "csharp_03_t06_w09_control_source_constructor_step_metadata",
}
commands = {key: [build["executable"]["path"], "--exact", test, "--nocapture"] for key, test in tests.items()}
commands.update({
    "format": ["cargo", "fmt", "--package", "mpk-vc", "--", "--check"],
    "clippy": ["cargo", "clippy", "--offline", "-p", "mpk-vc", "--test", "csharp_practical_vc", "--", "-D", "warnings"],
})
cases = [row["case"] for row in census["cases"]]
if mode == "constructor-metadata":
    cases = ["type"]
reasons = {
    "normal": "Execute the unchanged original18-context runtime owner through run_steps, whose shared observer flag is hardcoded true. Every original example, literal forgery, component conjunction, SSA origin, argument/bit mutation, positive/negative check, strict import and all36 exact current goldens remain. No context filter, output flag, stack override or core ceiling.",
    "metadata": "The existing full18-context StepMetadata owner now also executes the exact same component/SSA binding observer before the runtime-only section. Verify its complete current metadata, shared SSA origins and all36 byte goldens quickly. This is not a runtime pass and cannot replace the complete normal owner.",
    "constructor-metadata": "The existing dedicated constructor StepMetadata consumer also reaches the new shared observer; check its full original type context and exact byte goldens. This remains metadata-only evidence.",
    "format": "Both edited Rust support sources belong to mpk-vc; check package formatting.",
    "clippy": "Check the changed shared step observer and both metadata consumers through their integration target with warnings denied; do not rerun unrelated completed consumers.",
}
env = os.environ.copy()
env.update(build["environment"])
unset = ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS", "MPK_W09_CONTROL_EDGES_OUT"]
previous = {key: env.pop(key, None) for key in unset}
directory = root / mode
directory.mkdir(exist_ok=False)
record = {
    "status": "running", "mode": mode, "command": commands[mode],
    "environment": build["environment"], "unset_environment": unset, "prior_environment": previous,
    "binary": build["executable"], "test_sources": build["test_sources"],
    "build": seal(root / "build/result.json"),
    "source_preservation_audit": seal(root / "source-preservation-audit.json"),
    "binding_origin_analysis": seal(root / "binding-origin-analysis.json"),
    "original_component_census": seal(root.parent / "failure-analysis.json"),
    "original_contexts": cases, "selection_reason": reasons[mode],
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "full_gate": "The complete original runtime owner and the entire committed repaired T01 check-fast remain mandatory. Metadata/static checks do not complete W05/T01 or permit T02.",
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
if mode in tests:
    out = (directory / "stdout.log").read_text()
    err = (directory / "stderr.log").read_text()
    record["expected_one_test_passed"] = "test result: ok. 1 passed; 0 failed;" in out
    if mode == "normal":
        completed = [case for case in cases if f"control source steps {case}: " in err]
        record["runtime_all_original_contexts_completed"] = completed == cases
        complete = record["runtime_all_original_contexts_completed"]
    else:
        counts = dict(re.findall(r"^control source step component metadata (\S+): (\d+) complete local components$", err, re.M))
        expected = {row["case"]: row["complete_steps"] for row in census["cases"] if row["case"] in cases}
        record["metadata_complete_step_counts"] = {key: int(value) for key, value in counts.items()}
        record["metadata_all_original_contexts_completed"] = record["metadata_complete_step_counts"] == expected
        record["runtime_validation_claimed"] = False
        complete = record["metadata_all_original_contexts_completed"]
    if code == 0 and not (record["expected_one_test_passed"] and complete):
        record["status"] = "failed"
        record["harness_error"] = "Expected one passing original owner and every selected original context completed."
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("mode", "status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
