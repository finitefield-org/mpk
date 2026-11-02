"""Record the two complete affected input-echo owners and targeted static checks."""

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
assert mode in ["transition", "literal", "format", "clippy"]
directory = root / mode
directory.mkdir(exist_ok=False)
build = json.loads((root / "build/result.json").read_bytes())
assert build["status"] == "passed"
assert [seal(item["path"]) for item in build["test_sources"]] == build["test_sources"]
assert seal(build["executable"]["path"]) == build["executable"]
env = os.environ.copy()
env.update(build["environment"])
unset = [
    "RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS",
    "MPK_W09_TRANSITION_CLAUSES_OUT", "MPK_W09_TRANSITION_CLAUSE_RESPONSES",
    "MPK_W09_TRANSITION_CLAUSE_REQUESTS_OUT", "MPK_W09_LITERAL_CLAUSES_OUT",
    "MPK_W09_LITERAL_CLAUSE_RESPONSES", "MPK_W09_LITERAL_CLAUSE_REQUESTS_OUT",
]
prior = {key: env.pop(key, None) for key in unset}
tests = {
    "transition": "ordinary_carriers::public_domain_tests::contract_expression_tests::transition_clause_tests::csharp_03_t06_w09_transition_clauses_original_source",
    "literal": "ordinary_carriers::public_domain_tests::literal_clause_tests::csharp_03_t06_w09_literal_clauses_original_source",
}
commands = {
    "format": ["cargo", "fmt", "--package", "mpk-vc", "--", "--check"],
    "clippy": ["cargo", "clippy", "--offline", "-p", "mpk-vc", "--test", "csharp_practical_vc", "--", "-D", "warnings"],
}
commands.update({key: [build["executable"]["path"], "--exact", name, "--nocapture"] for key, name in tests.items()})
reasons = {
    "transition": "The actual original owner fails only at its final request echo. Execute every original transition context, seed, state/event/response observation and raw product selector/tail loop, full definition closures, imports, mutation and complete current component byte goldens. No filters or output flags.",
    "literal": "The actual original owner fails only at its final request echo. Execute every original rich-literal case, full/sparse/adjacent/tail packet probe, standalone/source/integrated/public definition closure comparison, strict import and complete current byte golden. No filters or output flags.",
    "format": "The only two new Rust changes are private test input-reference selectors and their strict byte/value guards; check mpk-vc formatting.",
    "clippy": "Both modified support sources are compiled in this integration target; deny warnings. Other six completed repair source bodies remain unchanged and are not re-run as an unrelated intermediate suite.",
}
record = {
    "status": "running", "mode": mode,
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "command": commands[mode], "environment": build["environment"],
    "unset_environment": unset, "prior_environment": prior,
    "test_sources": build["test_sources"], "build": seal(root / "build/result.json"),
    "binary": build["executable"], "selection_reason": reasons[mode],
    "full_gate": "The entire committed repaired T01 gate remains mandatory; selected owner checks do not complete W05/T01 or permit T02.",
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
    record["expected_one_test_passed"] = "test result: ok. 1 passed; 0 failed;" in (directory / "stdout.log").read_text()
    if code == 0 and not record["expected_one_test_passed"]:
        record["status"] = "failed"
        record["harness_error"] = "Expected exactly one complete selected owner to pass."
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["mode", "status", "exit_code", "wall_seconds"]}))
raise SystemExit(0 if record["status"] == "passed" else 1)
