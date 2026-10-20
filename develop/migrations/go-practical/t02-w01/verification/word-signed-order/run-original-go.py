"""Retain the unchanged Go core's complete original failed-recipe observation."""
from pathlib import Path
import hashlib
import json
import subprocess
import time


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
prior = json.loads((root.parent / "core-interfaces/paired/result.json").read_text())
binary = prior["go_binary"]
assert seal(binary["path"]) == binary
for source in prior["go_production_sources"]:
    assert seal(source["path"]) == source
certificate = root / "initial/construction/certificates/word-signed-order.mpcert"
assert seal(certificate)["sha256"] == "d33c2efb0fa90673aa114831d3671d11ecb1d3d3ba4bcb2415d35ca765859fbf"
out = root / "original-Go"
out.mkdir(exist_ok=False)
command = [binary["path"], "verify", str(certificate)]
started = time.monotonic()
result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
stdout, stderr = out / "stdout.log", out / "stderr.log"
stdout.write_bytes(result.stdout)
stderr.write_bytes(result.stderr)
observed = json.loads(result.stdout)
record = {"status": "complete_original_observation", "command": command, "exit_code": result.returncode,
          "seconds": time.monotonic() - started, "certificate": seal(certificate), "go_binary": binary,
          "go_production_sources": prior["go_production_sources"], "stdout": seal(stdout), "stderr": seal(stderr),
          "observation": observed, "scope": "Complete original insufficient proof recipe, not a semantic control, Go source counterexample, fuel/resource failure or prefix acceptance. No unchanged core/axiom/fuel/cap changes."}
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
assert result.returncode != 0 and not result.stderr and observed["verdict"] == "rejected"
assert json.loads(observed["error_detail"])["code"] == "CORE_TYPE_MISMATCH"
print(json.dumps({"status": "original_complete_type_rejection_preserved", "seconds": record["seconds"]}))
