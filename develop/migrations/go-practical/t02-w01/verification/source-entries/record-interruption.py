"""Record verified missing original workers without inventing their result."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


command = ["/bin/ps", "-Ao", "pid,ppid,etime,time,%cpu,rss,command"]
result = subprocess.run(command, capture_output=True, check=True, text=True)
needles = ["source-entries/run.py", "source-entries/check-pair.py", "source-entries/inspect-entry.py", "execution-histories/reference-build/mpk-checker-ref verify"]
workers = [line for line in result.stdout.splitlines() if any(needle in line for needle in needles) and "record-interruption.py" not in line]
assert not workers, workers
record = {"status": "verified-interrupted", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "worker_command": command, "exit_code": result.returncode, "matching_workers": workers,
          "missing_tool_handles": [53302, 12359], "reason": "The calling turn was intentionally interrupted; both original tool handles are missing and no original worker remains. Original executions are not claimed completed."}
pair = ROOT / "paired-all"
path = pair / "result.json"
row = json.loads(path.read_bytes())
assert row["status"] == "running"
with (pair / "result.before-interruption.json").open("x") as out:
    json.dump(row, out, indent=2)
    out.write("\n")
receipt = json.loads((pair / "inputs.json").read_bytes())
unchanged = all(seal(Path(source["path"])) == source for source in receipt["sources"])
row.update(status="failed", exit_code=None, termination="verified-user-turn-interruption", consumed_inputs_unchanged=unchanged,
           stdout=seal(pair / "stdout.log"), stderr=seal(pair / "stderr.log"), interruption=record,
           finished_at_utc=record["observed_at_utc"])
path.write_text(json.dumps(row, indent=2) + "\n")
targets = []
for target in sorted((ROOT / "paired").glob("*/*/prelaunch.json")):
    if target.with_name("result.json").exists():
        continue
    evidence = {"interruption": record, "prelaunch": seal(target),
                "retained_outputs": [seal(p) for p in sorted(target.parent.glob("*.log"))], "completed_checker_result": False}
    with target.with_name("interrupted-attempt.json").open("x") as out:
        json.dump(evidence, out, indent=2)
        out.write("\n")
    targets.append(seal(target.with_name("interrupted-attempt.json")))
partial = ROOT / "complete-parts"
assert not (partial / "inputs.json").exists() and not (partial / "result.json").exists()
record.update(partial_capture={"directory": str(partial), "complete_prelaunch_receipt": False,
                              "retained_incomplete_container": seal(partial / "inputs.tar.gz"), "inspection_started": False},
              interrupted_checker_attempts=targets, pair_result=seal(path),
              driver=seal(Path(__file__).resolve()))
with (ROOT / "interruption.json").open("x") as out:
    json.dump(record, out, indent=2)
    out.write("\n")
print(json.dumps({"status": record["status"], "interrupted_checker_attempts": len(targets), "original_pair_inputs_unchanged": unchanged}))
