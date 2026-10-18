"""Record the affected complete source-owner test and targeted static checks."""

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


root = Path(__file__).parent
mode = sys.argv[1]
assert mode in ["normal", "runtime-normal", "format", "clippy"]
directory = root / mode
directory.mkdir(exist_ok=False)
source = Path("crates/mpk-vc/tests/support/csharp_practical_ordinary_decimal_clause_tests.rs")
build = json.loads((root / "build-repaired-path/result.json").read_bytes())
assert build["status"] == "passed" and seal(source) == build["test_source"]
assert seal(build["executable"]["path"]) == build["executable"]
environment = build["environment"]
env = os.environ.copy()
env.update(environment)
unset = [
    "RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS",
    "MPK_W09_DECIMAL_CLAUSES_OUT", "MPK_W09_DECIMAL_CLAUSE_RESPONSES",
    "MPK_W09_DECIMAL_CLAUSES_RUNTIME_CONTEXTS", "MPK_W09_DECIMAL_CLAUSE_REQUESTS_OUT",
]
prior = {key: env.pop(key, None) for key in unset}
test = "ordinary_carriers::public_domain_tests::contract_expression_tests::decimal_clause_tests::csharp_03_t06_w09_decimal_clauses_candidates"
commands = {
    "normal": [build["executable"]["path"], "--exact", test, "--nocapture"],
    "runtime-normal": [build["executable"]["path"], "--exact", test.replace("decimal_clauses_candidates", "decimal_clauses_original_source"), "--nocapture"],
    "format": ["cargo", "fmt", "--package", "mpk-vc", "--", "--check"],
    "clippy": ["cargo", "clippy", "--offline", "-p", "mpk-vc", "--test", "csharp_practical_vc", "--", "-D", "warnings"],
}
reasons = {
    "normal": "All nine contexts, 44 operation closures, 49 attachments, full component JSON/certificate byte goldens and existing strict import/mutation checks share this final input echo helper. Runtime cases and their counts are byte unchanged; the mandatory complete repaired gate executes the original-source runtime owner. Do not repeat those expensive unchanged probes as a separate intermediate suite.",
    "runtime-normal": "The still-running original source320 whole diagnostic now also reports the original-source runtime owner as failed. Execute that entire affected owner normally, all nine contexts, every original selected runtime condition/failure case and full component goldens, with no runtime context filter, output flag or kernel ceilings. This new concrete finding justifies running the full runtime owner in addition to the completed candidate.",
    "format": "Check formatting of the affected mpk-vc source after qualifying the added helper's return type.",
    "clippy": "The changed support source is compiled in this integration target; deny warnings for that target and its dependencies.",
}
record = {
    "status": "running", "mode": mode,
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "command": commands[mode], "environment": environment,
    "unset_environment": unset, "prior_environment": prior,
    "test_source": seal(source), "build": seal(root / "build-repaired-path/result.json"),
    "binary": build["executable"], "selection_reason": reasons[mode],
    "full_gate": "Deferred to the mandatory entire repaired T01 gate. This selected run does not complete W05/T01 or permit T02.",
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
if mode in ["normal", "runtime-normal"]:
    record["expected_one_test_passed"] = "test result: ok. 1 passed; 0 failed;" in (directory / "stdout.log").read_text()
    assert code != 0 or record["expected_one_test_passed"]
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["mode", "status", "exit_code", "wall_seconds"]}))
