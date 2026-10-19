"""Build the exact integration target for the additive source-step observer repair."""

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
corrected = len(sys.argv) == 2 and sys.argv[1] == "corrected"
assert len(sys.argv) == 1 or corrected
suffix = "-corrected" if corrected else ""
audit_path = root / f"source-preservation-audit{suffix}.json"
audit = json.loads(audit_path.read_bytes())
analysis = json.loads((root / "failure-analysis.json").read_bytes())
assert audit["status"] == "passed"
assert seal(audit["source"]["path"]) == audit["source"]
for item in audit["production_sources"] + analysis["inputs"]:
    assert all(seal(item["path"])[key] == item[key] for key in ("path", "sha256", "bytes"))
sources = [audit["source"]]
prior = json.loads((root.parent / "input-echo-followup-repair/build/result.json").read_bytes())
for item in prior["test_sources"]:
    assert seal(item["path"]) == item
    sources.append(item)
directory = root / f"build{suffix}"
directory.mkdir(exist_ok=False)
command = ["cargo", "test", "--offline", "-p", "mpk-vc", "--test", "csharp_practical_vc", "--no-run", "--message-format=json"]
environment = {
    "CARGO_NET_OFFLINE": "true",
    "CARGO_TARGET_DIR": "/private/tmp/mpk-t01-w05-json-repair-target",
    "CARGO_PROFILE_TEST_OPT_LEVEL": "1",
    "CARGO_PROFILE_TEST_DEBUG_ASSERTIONS": "true",
    "CARGO_PROFILE_TEST_OVERFLOW_CHECKS": "true",
}
env = os.environ.copy()
env.update(environment)
unset = ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS", "MPK_W09_CONTROL_EDGES_OUT"]
previous = {name: env.pop(name, None) for name in unset}
record = {
    "status": "running", "source_base_commit": "145675b6d362035ae717af82006d3d622f8aa1a0",
    "command": command, "environment": environment,
    "unset_environment": unset, "prior_environment": previous,
    "test_sources": sources,
    "source_preservation_audit": seal(audit_path),
    "failure_analysis": seal(root / "failure-analysis.json"),
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "selection_reason": "Only the private run_steps observer changes: include the existing memory-effect and constructor components and check their exact bindings. Compile its integration target; all112 original assertions and all18 original contexts remain, with seven additional assertions and the additive count RHS.",
    "full_gate_passed": False,
}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.jsonl").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    child = subprocess.Popen(command, env=env, stdout=out, stderr=err)
    record["pid"] = child.pid
    result.write_text(json.dumps(record, indent=2) + "\n")
    code = child.wait()
record.update(
    status="passed" if code == 0 else "failed", exit_code=code,
    wall_seconds=time.monotonic() - started,
    finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    stdout=seal(directory / "stdout.jsonl"), stderr=seal(directory / "stderr.log"),
)
if code == 0:
    entries = [json.loads(line) for line in (directory / "stdout.jsonl").read_text().splitlines()]
    binaries = [entry for entry in entries if entry.get("reason") == "compiler-artifact" and entry.get("target", {}).get("name") == "csharp_practical_vc" and entry.get("executable")]
    assert len(binaries) == 1
    record["executable"] = seal(binaries[0]["executable"])
    record["profile"] = binaries[0]["profile"]
    assert record["profile"]["opt_level"] == "1"
    assert record["profile"]["debug_assertions"] is True
    assert record["profile"]["overflow_checks"] is True
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record.get(key) for key in ("status", "exit_code", "wall_seconds", "executable")}))
raise SystemExit(0 if code == 0 else 1)
