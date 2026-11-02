"""Fresh development builds; no installation or immutable identity rebinding."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
attempt = sys.argv[1]
assert attempt in {"first", "second"}
out = OUT / "build" / attempt
out.mkdir(parents=True, exist_ok=False)
driver = out / "build-frontend.py"
driver.write_bytes(Path(__file__).read_bytes())


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


basis = Path("develop/migrations/go-rust-only/verification/build-go/result.json")
prior = json.loads(basis.read_bytes())
command = list(prior["commands"][0 if attempt == "first" else 1]["command"])
old = "/private/tmp/mpk-go-rust-scope/build-go-" + attempt
destination = Path("/private/tmp/mpk-go-practical-string-constants-build-" + attempt)
destination.mkdir(exist_ok=False)
command = [arg.replace("src=" + old + ",dst=/out", "src=" + str(destination) + ",dst=/out") for arg in command]
assert sum(arg.endswith("type=bind,src=" + str(destination) + ",dst=/out") for arg in command) == 1
input_path = OUT / "alias-display/inputs.json"
inputs = json.loads(input_path.read_bytes())["sources"]
assert all(seal(row["path"])["sha256"] == row["sha256"] for row in inputs)
record = {"status": "running", "command": command, "source_receipt": seal(input_path), "sources": inputs,
          "command_basis": seal(basis), "actual_driver_prelaunch": seal(driver),
          "native_release_acceptance": False, "registered_binary_installed": False,
          "selection_reason": "One new Go production input changes the complete frontend build closure. Reproduce the exact prior pinned, network-disabled Linux/amd64 development command in two new empty-cache containers; retain separate binary identities without activating practical admission.",
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
result = out / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (out / "stdout.log").open("wb") as stdout, (out / "stderr.log").open("wb") as stderr:
    code = subprocess.run(command, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code, seconds=time.monotonic() - started,
              stdout=seal(out / "stdout.log"), stderr=seal(out / "stderr.log"),
              whole_source_inputs_unchanged=all(seal(row["path"])["sha256"] == row["sha256"] for row in inputs),
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
if code == 0:
    record["output"] = seal(destination / "go2vir")
if not record["whole_source_inputs_unchanged"] or seal(__file__)["sha256"] != seal(driver)["sha256"]:
    record["status"] = "failed"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: v for k, v in record.items() if k not in {"sources", "command"}}))
raise SystemExit(record["status"] != "passed")
