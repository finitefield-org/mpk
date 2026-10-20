"""Observe actual driver/container state after every selected result is terminal."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
phases = ["initial/regression", "restored/regression", "independent/regression",
          "checksum-source/ownership", "checksum-both/ownership", "owned-checksums/ownership"]
phases += [attempt + "/" + mode for attempt in ["closed-consumer", "owned-checksums"]
           for mode in ["selected", "legacy", "vet", "format", "linux-prepare", "linux-replay-1", "linux-replay-2"]]
results, containers = [], []
for phase in phases:
    path = root / phase / "result.json"
    raw = path.read_bytes()
    result = json.loads(raw)
    assert result["status"] in {"passed", "failed"} and result.get("finished_at_utc")
    results.append({"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(),
                    "bytes": len(raw), "status": result["status"]})
    for check in result.get("checks", []):
        command = check["command"]
        name = command[command.index("--name") + 1]
        observed = subprocess.run(["/usr/local/bin/docker", "inspect", "--format",
                                   "{{.State.Running}} {{.State.ExitCode}}", name],
                                  capture_output=True, text=True, check=False)
        if observed.returncode == 0:
            assert observed.stdout.strip().startswith("false ")
        else:
            assert observed.returncode == 1 and "no such object" in observed.stderr
        containers.append({"name": name, "recorded_exit_code": check["exit_code"],
                           "inspect_exit_code": observed.returncode,
                           "stdout": observed.stdout, "stderr": observed.stderr})
observed = subprocess.run(["/bin/ps", "-axo", "pid=,command="],
                          capture_output=True, text=True, check=True)
pattern = re.compile(r"python[^\n]*image-consumer/run-(?:selected|linux|special-permission-linux)\.py(?:\s|$)")
active = [line for line in observed.stdout.splitlines() if pattern.search(line)]
assert not active, active
record = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "terminal_results": results, "containers": containers, "live_matching_drivers": active,
          "all_actual_validation_handles_terminal": True,
          "observation_timeout_treated_as_completion": False, "drivers_restarted": False}
(root / "terminal-handles.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "terminal_results": len(results),
                  "terminal_containers": len(containers), "live_drivers": len(active)}))
