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
import re

repo = Path.cwd().resolve()
root = Path(__file__).resolve().parent.relative_to(repo)
attempt, = sys.argv[1:]
mode = "owned"
assert mode == "owned"
assert attempt.replace("-", "").isalnum()
docker = "/usr/local/bin/docker"
image = "sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a"
directory = root / attempt / ("linux-" + mode)
directory.mkdir(parents=True, exist_ok=False)
scratch = Path(tempfile.mkdtemp(prefix="mpk-go-observed-replay-")).resolve()
name = "mpk-go-observed-replay-" + uuid.uuid4().hex[:12]


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
          "selection_reason": "Keep the original Go preparation owner/baseline alive while two independent offline kernel-readonly compiler runs finish. The Go consumer itself compares every original metadata field and complete archive bytes. This does not register a context or attest a production kernel profile. Local Linux/amd64 is emulated on Linux/aarch64; CSharp/agents/Actions/full gate remain skipped."}
(directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
common = [docker, "run", "--rm", "--name", name, "--platform", "linux/amd64", "--pull=never",
          "--network=none", "--read-only", "--user", "501:501", "--cap-drop=ALL",
          "--security-opt=no-new-privileges", "--memory=4g", "--cpus=2", "--pids-limit=256",
          "--tmpfs", "/tmp:rw,exec,nosuid,nodev,mode=1777,size=2g"]
cache = Path("/private/tmp/mpk-go-practical-replay-linux-build-cache")
cache.mkdir(exist_ok=True)
(scratch / "evidence").mkdir()
environment = {
    "GOTOOLCHAIN": "local", "GOENV": "off", "GOPROXY": "off", "GOSUMDB": "off", "GOVCS": "*:off",
    "GOTELEMETRY": "off", "GOWORK": "off", "CGO_ENABLED": "0", "GOMAXPROCS": "1", "GOFLAGS": "-buildvcs=false",
    "GOCACHE": "/mpk-build-cache", "GOPATH": "/tmp/empty-gopath", "GOMODCACHE": "/tmp/empty-module-cache",
    "HOME": "/tmp/empty-home", "MPK_GO_PRACTICAL_REPLAY_OUTPUT": "/replay-output",
    "MPK_GO_PRACTICAL_REPLAY_EXCHANGE": "1", "MPK_GO_PRACTICAL_SYNTAX_EVIDENCE": "/replay-output/evidence",
}
producer_command = common + mount(repo / "go-tools/go2vir", "/mpk-frontend") + mount(cache, "/mpk-build-cache", False) + mount(scratch, "/replay-output", False)
producer_command[producer_command.index("--name") + 1] = name + "-producer"
for key, value in environment.items():
    producer_command += ["-e", key + "=" + value]
producer_command += ["--workdir", "/mpk-frontend", image, "/usr/local/go/bin/go", "test", "-mod=vendor", "-run",
                     "^TestPracticalReplayPrepareWholeOriginalModuleAndCompilerImage$", "-count=1", "-timeout=30m", "-json", "."]
producer_started = time.monotonic()
producer_out = (directory / "producer.stdout.log").open("wb")
producer_err = (directory / "producer.stderr.log").open("wb")
producer = subprocess.Popen(producer_command, stdout=producer_out, stderr=producer_err)
print(json.dumps({"phase": "producer", "container": name + "-producer", "pid": producer.pid}), flush=True)
record["producer_command"] = producer_command
record["producer_driver_pid"] = producer.pid
record["owned_comparisons"] = []
(directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")


def await_owner_file(path):
    next_report = time.monotonic()
    while not path.exists():
        status = producer.poll()
        if status is not None:
            raise RuntimeError("original observation owner exited before " + path.name + ": " + str(status))
        now = time.monotonic()
        if now >= next_report:
            print(json.dumps({"phase": "waiting-for-owner", "file": path.name, "producer_running": True}), flush=True)
            next_report = now + 30
        time.sleep(0.1)


def host_path(private):
    result = scratch / Path(private).relative_to("/replay-output")
    assert result.resolve().is_relative_to(scratch)
    return result


error = None
try:
    await_owner_file(scratch / "baseline-ready")
    bundle = json.loads((scratch / "replay-image.json").read_text())
    assert (scratch / "baseline-ready").read_text().strip() == bundle["image_digest"]
    shutil.copyfile(scratch / "replay-image.json", directory / "replay-image.json")
    record["image_manifest"] = seal(directory / "replay-image.json")
    assert bundle["original_roots_retired_before_consumer"] and bundle["original_donor_buffers_cleared_before_consumer"]
    bindings = {}
    for item in bundle["mounts"]:
        original, private = item["original"], host_path(item["private"])
        assert original.startswith("/") and private.is_dir() and not private.is_symlink()
        assert original not in bindings or bindings[original] == private
        bindings[original] = private
    root_sdk = bundle["compiler_original_root"]
    replay_environment = {
        "CGO_ENABLED": "0", "GO111MODULE": "on", "GOAMD64": "v1", "GOARCH": "amd64",
        "GOCACHE": "/mpk-work/build", "GODEBUG": "", "GOENV": "off", "GOEXPERIMENT": "", "GOFLAGS": "",
        "GOMAXPROCS": "1", "GOMODCACHE": bundle["cache_original_directory"], "GONOPROXY": "", "GONOSUMDB": "",
        "GOOS": "linux", "GOPACKAGESDRIVER": "off", "GOPATH": "/mpk-work/empty", "GOPRIVATE": "", "GOPROXY": "off",
        "GOROOT": root_sdk, "GOSUMDB": "off", "GOTELEMETRY": "off", "GOTOOLCHAIN": "local", "GOVCS": "*:off", "GOWORK": "off",
        "HOME": "/mpk-work/home", "LANG": "C", "LC_ALL": "C", "PATH": root_sdk + "/bin",
        "TMPDIR": "/mpk-work/tmp", "TZ": "UTC",
    }
    for round_name in ["replay-1", "replay-2"]:
        handoff = scratch / "handoff" / round_name
        handoff.mkdir(parents=True)
        workspace = handoff / "work"
        workspace.mkdir()
        for child in ["build", "tmp", "home", "empty"]:
            (workspace / child).mkdir()
        prefix = common + ["--tmpfs", "/usr/local/go:ro,noexec,nosuid,nodev,mode=000,size=4096"]
        for original, private in sorted(bindings.items()):
            prefix += mount(private, original)
        prefix += mount(workspace, "/mpk-work", False)
        for key, value in replay_environment.items():
            prefix += ["-e", key + "=" + value]
        prefix += ["--workdir", bundle["main_original_directory"], image]
        executable = root_sdk + "/bin/go"
        module_code = execute(round_name + "-modules", prefix + [executable, "list", "-mod=readonly", "-m", "-json", "all"])
        package_code = execute(round_name + "-packages", prefix + [executable, "list", "-mod=readonly", "-deps", "-export", "-compiled", "-json", bundle["requested_package"]])
        for label in ["modules", "packages"]:
            for stream in ["stdout", "stderr"]:
                shutil.copyfile(directory / (round_name + "-" + label + "." + stream + ".log"), handoff / (label + "." + stream + ".log"))
        status = {"module_exit_code": module_code, "package_exit_code": package_code}
        temporary = handoff / "status.tmp"
        temporary.write_text(json.dumps(status) + "\n")
        temporary.replace(handoff / "ready.json")
        await_owner_file(handoff / "go-comparison.json")
        comparison = json.loads((handoff / "go-comparison.json").read_text())
        shutil.copyfile(handoff / "go-comparison.json", directory / (round_name + ".go-comparison.json"))
        record["owned_comparisons"].append(seal(directory / (round_name + ".go-comparison.json")))
        (directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        assert module_code == package_code == 0 and comparison["error"] == "<nil>"
    while producer.poll() is None:
        time.sleep(0.1)
except Exception as exc:
    error = repr(exc)
    # A supervisor observation failure is not a completed producer. Keep the
    # same actual handle and wait for its own completion/30-minute test bound.
    while producer.poll() is None:
        print(json.dumps({"phase": "waiting-for-terminal-owner", "error": error}), flush=True)
        time.sleep(30)
finally:
    producer_code = producer.wait()
    producer_out.close()
    producer_err.close()
    checks.append({"label": "producer", "command": producer_command, "exit_code": producer_code,
                   "seconds": time.monotonic() - producer_started,
                   "stdout": seal(directory / "producer.stdout.log"), "stderr": seal(directory / "producer.stderr.log")})
    for source in (scratch / "evidence").iterdir():
        destination = directory / source.name
        if source.is_dir():
            shutil.copytree(source, destination)
        else:
            shutil.copyfile(source, destination)
    record["status"] = "passed" if error is None and producer_code == 0 else "failed"
    record["driver_error"] = error
    record["sources_unchanged_during_execution"] = all(seal(row["path"]) == row for row in sources)
    if not record["sources_unchanged_during_execution"]:
        record["status"] = "failed"
    record["finished_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    record["checks"] = checks
    (directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "producer_exit_code": producer_code, "error": error}))
raise SystemExit(0 if record["status"] == "passed" else 1)
