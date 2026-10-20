"""Two isolated builds with the previously verified exact release command."""
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
out.mkdir(parents=True, exist_ok=True)
assert not any(out.iterdir())
basis = Path("develop/migrations/go-rust-only/verification/build-go/result.json")
prior = json.loads(basis.read_bytes())
command = list(prior["commands"][0 if attempt == "first" else 1]["command"])
old = "/private/tmp/mpk-go-rust-scope/build-go-" + attempt
destination = Path("/private/tmp/mpk-go-practical-original-packages-build-" + attempt)
destination.mkdir(exist_ok=True)
assert not any(destination.iterdir())
command = [arg.replace("src=" + old + ",dst=/out", "src=" + str(destination) + ",dst=/out") for arg in command]
assert sum(arg.endswith("type=bind,src=" + str(destination) + ",dst=/out") for arg in command) == 1
inputs = json.loads((OUT / "integrated/inputs.json").read_bytes())["sources"]
started = time.monotonic()
with (out / "stdout.log").open("wb") as stdout, (out / "stderr.log").open("wb") as stderr:
    code = subprocess.run(command, stdout=stdout, stderr=stderr, check=False).returncode
record = {"status": "passed" if code == 0 else "failed", "command": command,
          "command_basis": str(basis), "command_basis_sha256": hashlib.sha256(basis.read_bytes()).hexdigest(),
          "exit_code": code, "seconds": time.monotonic() - started,
          "whole_source_inputs_unchanged": all(hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest() == row["sha256"] for row in inputs),
          "native_release_acceptance": False,
          "finished_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
if code == 0:
    binary = destination / "go2vir"
    raw = binary.read_bytes()
    record["output"] = {"path": str(binary), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}
if not record["whole_source_inputs_unchanged"]:
    record["status"] = "failed"
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record))
raise SystemExit(record["status"] != "passed")
