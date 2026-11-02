"""Record the entire affected fixed-codec owner and its targeted static checks."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time


def seal(path):
    path = Path(path)
    data = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode = sys.argv[1]
assert mode in ["normal", "format", "clippy"]
directory = root / mode
directory.mkdir(exist_ok=False)
source = Path("crates/mpk-vc/tests/support/csharp_practical_ordinary_fixed_codec_clause_tests.rs")
build = json.loads((root / "build/result.json").read_bytes())
assert build["status"] == "passed" and seal(source) == build["test_source"]
assert seal(build["executable"]["path"]) == build["executable"]
env = os.environ.copy()
env.update(build["environment"])
unset = [
    "RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS",
    "MPK_W09_FIXED_CODEC_CLAUSES_OUT", "MPK_W09_FIXED_CODEC_RESPONSES",
    "MPK_W09_FIXED_CODEC_REQUESTS_OUT",
]
prior = {key: env.pop(key, None) for key in unset}
test = "ordinary_carriers::public_domain_tests::contract_expression_tests::fixed_codec_clause_tests::csharp_03_t06_w09_fixed_codec_clauses_original_source"
commands = {
    "normal": [build["executable"]["path"], "--exact", test, "--nocapture"],
    "format": ["cargo", "fmt", "--package", "mpk-vc", "--", "--check"],
    "clippy": ["cargo", "clippy", "--offline", "-p", "mpk-vc", "--test", "csharp_practical_vc", "--", "-D", "warnings"],
}
reasons = {
    "normal": "The actual original whole diagnostic fails this complete source owner only at the final input echo after all six codec contexts, 44 source clauses and 21088 format/composition bit observations pass. Re-run the entire unchanged case matrix, parser valid/invalid/oversized cases, all original format and complete parse-format bit loops, strict imports, metadata mutation and current component JSON/certificate byte goldens. No filters or output flags. The mixed-source owner does not echo requests.json and its body is byte unchanged; it already passes in the original whole diagnostic and remains required in the repaired full gate.",
    "format": "Check formatting of the one newly changed private fixed-codec support source.",
    "clippy": "The fixed-codec source is compiled in this integration target; deny warnings for that target and its dependencies.",
}
record = {
    "status": "running", "mode": mode,
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "command": commands[mode], "environment": build["environment"],
    "unset_environment": unset, "prior_environment": prior,
    "test_source": seal(source), "build": seal(root / "build/result.json"),
    "binary": build["executable"], "selection_reason": reasons[mode],
    "full_gate": "The entire repaired T01 gate remains mandatory; selected results do not complete W05/T01 or permit T02.",
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
    record["expected_one_test_passed"] = "test result: ok. 1 passed; 0 failed;" in (directory / "stdout.log").read_text()
    if code == 0 and not record["expected_one_test_passed"]:
        record["status"] = "failed"
        record["harness_error"] = "Expected exactly one complete selected owner to pass."
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["mode", "status", "exit_code", "wall_seconds"]}))
raise SystemExit(0 if record["status"] == "passed" else 1)
