"""Local, offline x86-64-emulated Linux compiler replay, never a release gate."""
from pathlib import Path
import base64
import csv
import datetime
import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import uuid

repo = Path.cwd().resolve()
root = Path(__file__).resolve().parent.relative_to(repo)
attempt, = sys.argv[1:]
mode = "special-permission"
assert mode == "special-permission"
assert attempt.replace("-", "").isalnum()
docker = "/usr/local/bin/docker"
image = "sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a"
directory = root / "linux-special-permission" / attempt
directory.mkdir(parents=True, exist_ok=False)
scratch = Path(tempfile.mkdtemp(prefix="mpk-go-practical-permissions-")).resolve()
name = "mpk-go-practical-permissions-" + uuid.uuid4().hex[:12]


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def mount(source, target, readonly=True):
    output = io.StringIO()
    csv.writer(output, lineterminator="").writerow(
        ["type=bind", "source=" + str(source), "target=" + str(target)] + (["readonly"] if readonly else []))
    return ["--mount", output.getvalue()]


def execute(label, command):
    command = list(command)
    command[command.index("--name") + 1] = name + "-" + label
    print(json.dumps({"phase": label, "container": name}), flush=True)
    started = time.monotonic()
    with (directory / (label + ".stdout.log")).open("wb") as out, (directory / (label + ".stderr.log")).open("wb") as err:
        code = subprocess.run(command, stdout=out, stderr=err, check=False).returncode
    result = {"label": label, "command": command, "exit_code": code, "seconds": time.monotonic() - started,
              "stdout": seal(directory / (label + ".stdout.log")), "stderr": seal(directory / (label + ".stderr.log"))}
    checks.append(result)
    record["checks"] = checks
    (directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
    return code


source_paths = sorted(Path("go-tools/go2vir").glob("*.go"))
source_paths += [Path("go-tools/go2vir/go.mod"), Path("go-tools/go2vir/go.sum"), Path("go-tools/go2vir/vendor/modules.txt")]
sources = [seal(path) for path in source_paths]
for source in sources:
    path = directory / "inputs" / source["path"]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(Path(source["path"]).read_bytes())
(directory / "runner.py").write_bytes(Path(__file__).read_bytes())
checks = []
record = {"status": "running", "mode": mode, "container": name, "scratch": str(scratch), "image": image,
          "sources": sources, "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "network": "none", "read_only_root": True, "uid": 501, "capabilities": "ALL dropped",
          "no_new_privileges": True, "compiler": "Go1.25.0 Linux/amd64", "server": "local OrbStack Linux/aarch64",
          "x86_64_emulated": True, "native_Linux_release_acceptance_claimed": False,
          "CSharp_tests_executed": False, "agents_used": False, "GitHub_Actions_used": False,
          "T02_W01_complete": False, "R06_status": "pending", "registered_context_or_application_proof": False,
          "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
          "selection_reason": "Execute all six actual setuid/setgid/sticky file and directory controls on Linux tmpfs. Darwin backing may clear requested setuid/setgid bits, so its unmaterialized controls cannot stand in for Linux runtime observations. Complete original source epochs and terminal native results are retained. CSharp tests and intermediate full gates stay skipped."}
(directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
common = [docker, "run", "--rm", "--name", name, "--platform", "linux/amd64", "--pull=never",
          "--network=none", "--read-only", "--user", "501:501", "--cap-drop=ALL",
          "--security-opt=no-new-privileges", "--memory=4g", "--cpus=2", "--pids-limit=256",
          "--tmpfs", "/tmp:rw,exec,nosuid,nodev,mode=1777,size=2g"]
cache = Path("/private/tmp/mpk-go-practical-replay-linux-build-cache")
cache.mkdir(exist_ok=True)
environment = {
    "GOTOOLCHAIN": "local", "GOENV": "off", "GOPROXY": "off", "GOSUMDB": "off", "GOVCS": "*:off",
    "GOTELEMETRY": "off", "GOWORK": "off", "CGO_ENABLED": "0", "GOMAXPROCS": "1", "GOFLAGS": "-buildvcs=false",
    "GOCACHE": "/mpk-build-cache", "GOPATH": "/tmp/empty-gopath", "GOMODCACHE": "/tmp/empty-module-cache",
    "HOME": "/tmp/empty-home", "MPK_GO_PRACTICAL_REPLAY_OUTPUT": "/replay-output",
    "MPK_GO_PRACTICAL_SYNTAX_EVIDENCE": "/replay-output/evidence",
}
(scratch / "evidence").mkdir()
command = common + mount(repo / "go-tools/go2vir", "/mpk-frontend") + mount(cache, "/mpk-build-cache", False) + mount(scratch, "/replay-output", False)
for key, value in environment.items():
    command += ["-e", key + "=" + value]
command += ["--workdir", "/mpk-frontend", image, "/usr/local/go/bin/go", "test", "-mod=vendor", "-run",
            "^TestPracticalReplayViewRejectsSpecialPermissionBits$", "-count=1", "-timeout=30m", "-json", "."]
code = execute("controls", command)
for source in (scratch / "evidence").iterdir():
    destination = directory / source.name
    if source.is_dir():
        shutil.copytree(source, destination)
    else:
        shutil.copyfile(source, destination)
record["status"] = "passed" if code == 0 else "failed"
record["sources_unchanged_during_execution"] = all(seal(row["path"]) == row for row in sources)
assert record["sources_unchanged_during_execution"]
record["finished_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
(directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "actual_exit_code": code, "container": name}))
raise SystemExit(0 if code == 0 else 1)
