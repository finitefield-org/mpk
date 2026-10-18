"""Run both unchanged development checkers on every distinct generated proof."""

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


part, destination = sys.argv[1:]
assert part in {"foundation-identity", "default"}
stages = ["foundation", "identity"] if part == "foundation-identity" else ["default"]
root = Path("develop/migrations/go-practical/t01-w05/retained-csharp/current-proof-inputs")
directory = Path(destination)
directory.mkdir(parents=True, exist_ok=False)
provenance_path = Path(__file__).parent / "checker-provenance/result.json"
provenance = json.loads(provenance_path.read_bytes())
checkers = provenance["checkers"]
for record in checkers.values():
    assert seal(Path(record["path"])) == record
groups = {}
members = []
for stage in stages:
    paths = sorted((root / stage).glob("*.hex"))
    assert len(paths) == {"foundation": 46, "identity": 2, "default": 6}[stage]
    for path in paths:
        raw = bytes.fromhex(path.read_text())
        digest = hashlib.sha256(raw).hexdigest()
        negative = path.stem.endswith("-wrong")
        key = (negative, digest)
        if key in groups:
            assert groups[key]["raw"] == raw
        else:
            groups[key] = {"path": path, "raw": raw}
        members.append({**seal(path), "raw_certificate_sha256": digest, "certificate_bytes": len(raw), "expected": "core_check_rejection" if negative else "zero_axiom_acceptance"})
environment = os.environ.copy()
for name in ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS"]:
    environment.pop(name, None)
record = {
    "status": "running", "started_at_utc": stamp(), "part": part,
    "scope": "Current source-free component proof certificates; positive zero-axiom and matching-hash agreement, negative CoreCheck agreement. No original application proof, profile activation or native Linux release claim.",
    "provenance": seal(provenance_path), "checkers": checkers,
    "members": members, "distinct_certificates": len(groups),
    "deduplication": "Only exact raw certificate byte equality permits reuse of one actual check. Every input member remains individually pinned.",
    "resource_overrides": {}, "application_proofs_discharged": False, "whole_gate_passed": False,
    "observations": [],
}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
start = time.monotonic()
raw_log = directory / "raw-checker-observations.jsonl"
try:
    with raw_log.open("x") as log:
        for number, ((negative, digest), group) in enumerate(groups.items()):
            pair = {}
            runs = []
            for engine in ["go", "rust"]:
                command = [checkers[engine]["path"], "verify" if engine == "go" else "check", str(group["path"].resolve())]
                began = stamp()
                tick = time.monotonic()
                completed = subprocess.run(command, env=environment, capture_output=True)
                run = {
                    "index": number, "engine": engine, "command": command,
                    "started_at_utc": began, "finished_at_utc": stamp(), "wall_seconds": time.monotonic() - tick,
                    "exit_code": completed.returncode,
                    "stdout_raw_utf8": completed.stdout.decode(), "stderr_raw_utf8": completed.stderr.decode(),
                    "stdout_sha256": hashlib.sha256(completed.stdout).hexdigest(),
                    "stderr_sha256": hashlib.sha256(completed.stderr).hexdigest(),
                }
                log.write(json.dumps(run, ensure_ascii=False) + "\n")
                log.flush()
                value = json.loads(completed.stdout)
                if negative:
                    error_field, error_value = ("error_kind", "core_check") if engine == "go" else ("error_code", "KERNEL_CORE_CHECK")
                    assert completed.returncode == 1 and value["verdict"] == "rejected" and value[error_field] == error_value, value
                else:
                    assert completed.returncode == 0 and value["verdict"] == "accepted" and value.get("axiom_count", 0) == 0, value
                pair[engine] = value
                runs.append({key: run[key] for key in ["engine", "command", "exit_code", "wall_seconds", "stdout_sha256", "stderr_sha256"]})
            if not negative:
                assert all(pair["go"][key] == pair["rust"][key] for key in ["module", "declaration_count", "hashes"]), pair
            record["observations"].append({
                "raw_certificate_sha256": digest, "expected": "core_check_rejection" if negative else "zero_axiom_acceptance",
                "agreed": True, "hashes": None if negative else pair["go"]["hashes"], "runs": runs,
            })
            result.write_text(json.dumps(record, indent=2) + "\n")
            print(json.dumps({"completed": number + 1, "total": len(groups), "member": str(group["path"]), "negative": negative}), flush=True)
    record["status"] = "passed"
except BaseException as error:
    record["status"] = "failed"
    record["failure"] = repr(error)
    raise
finally:
    record.update({"finished_at_utc": stamp(), "wall_seconds": time.monotonic() - start, "raw_observations": seal(raw_log)})
    result.write_text(json.dumps(record, indent=2) + "\n")
