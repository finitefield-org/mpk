"""Affected new native observer vet/format on the verified original Go image."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import stat
import subprocess
import time


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
native = json.loads((root / "native/result.json").read_text())
assert native["status"] == "passed"
for key in ["original_source", "owned_source", "complete_original_prepared_image", "compiler"]:
    assert seal(native[key]["path"]) == native[key]
out = root / "native-checks"
out.mkdir(exist_ok=False)
(out / "runner.py").write_bytes(Path(__file__).read_bytes())
image = json.loads(Path(native["complete_original_prepared_image"]["path"]).read_text())
goroot = Path(native["environment"]["GOROOT"])
expected = {row["path"] for row in image["inventory"]["files"]}
assert len(expected) == 5577


def validate(label):
    start = time.monotonic()
    actual = set()
    for directory, dirs, files in os.walk(goroot, followlinks=False):
        for name in dirs:
            assert not (Path(directory) / name).is_symlink()
        for name in files:
            actual.add(str((Path(directory) / name).relative_to(goroot.parent)))
    assert actual == expected
    for row in image["inventory"]["files"]:
        path = goroot.parent / row["path"]
        info = path.lstat()
        assert stat.S_ISREG(info.st_mode) and not path.is_symlink()
        raw = path.read_bytes()
        assert len(raw) == row["size_bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
        assert bool(info.st_mode & 0o111) == row["executable"]
    return {"label": label, "status": "passed", "files": 5577, "seconds": time.monotonic() - start}


images = [validate("before affected observer vet/format")]
env = os.environ.copy()
env.update(native["environment"])
source = Path(native["owned_source"]["path"]).resolve()
commands = []
for name, command in [("vet", [native["compiler"]["path"], "vet", str(source)]),
                      ("format", [str(goroot / "bin/gofmt"), "-l", str(source)])]:
    started = time.monotonic()
    result = subprocess.run(command, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    stdout, stderr = out / (name + ".stdout.log"), out / (name + ".stderr.log")
    stdout.write_bytes(result.stdout)
    stderr.write_bytes(result.stderr)
    row = {"name": name, "command": command, "exit_code": result.returncode,
           "seconds": time.monotonic() - started, "stdout": seal(stdout), "stderr": seal(stderr)}
    commands.append(row)
    assert result.returncode == 0 and not result.stdout and not result.stderr
images.append(validate("after affected observer vet/format"))
assert seal(native["original_source"]["path"]) == native["original_source"]
assert seal(native["owned_source"]["path"]) == native["owned_source"]
record = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "native_execution": seal(root / "native/result.json"), "runner": seal(__file__),
          "original_source": native["original_source"], "owned_source": native["owned_source"],
          "compiler": native["compiler"], "complete_original_image_checks": images,
          "environment": native["environment"], "commands": commands, "CSharp_tests_executed": False,
          "selection_reason": "Only the newly added typed equality observer is affected; vet and gofmt check the complete original source on the same verified Go1.25 image. No unrelated observer/go2vir/full-gate suites are repeated."}
(out / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": "passed", "checks": [c["name"] for c in commands]}))
