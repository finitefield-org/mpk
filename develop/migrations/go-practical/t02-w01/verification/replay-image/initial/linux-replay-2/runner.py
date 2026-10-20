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
mode, attempt = sys.argv[1:]
assert mode in {"prepare", "replay-1", "replay-2", "controls"}
assert attempt.replace("-", "").isalnum()
docker = "/usr/local/bin/docker"
image = "sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a"
directory = root / attempt / ("linux-" + mode)
directory.mkdir(parents=True, exist_ok=False)
scratch = Path(tempfile.mkdtemp(prefix="mpk-go-practical-replay-")).resolve()
name = "mpk-go-practical-replay-" + uuid.uuid4().hex[:12]


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
          "selection_reason": "Actual fixed Go compiler replay over owned byte images, retaining original absolute/relative/versioned replacements and complete unused HTTP handler bodies. Original source/mod bytes remain unchanged. Each replay has a fresh derived cache, no original source/code mount, a hidden ambient SDK and kernel read-only input mounts. Compare full original module/package metadata and all complete compiler archives. This is local development evidence on x86-64 emulation, not a native Linux release receipt."}
(directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
common = [docker, "run", "--rm", "--name", name, "--platform", "linux/amd64", "--pull=never",
          "--network=none", "--read-only", "--user", "501:501", "--cap-drop=ALL",
          "--security-opt=no-new-privileges", "--memory=4g", "--cpus=2", "--pids-limit=256",
          "--tmpfs", "/tmp:rw,exec,nosuid,nodev,mode=1777,size=2g"]
if mode == "prepare":
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
                "^TestPracticalReplayPrepareWholeOriginalModuleAndCompilerImage$", "-count=1", "-timeout=30m", "-json", "."]
    code = execute("producer", command)
    for source in (scratch / "evidence").iterdir():
        destination = directory / source.name
        if source.is_dir():
            shutil.copytree(source, destination)
        else:
            shutil.copyfile(source, destination)
    image_path = scratch / "replay-image.json"
    if image_path.exists():
        shutil.copyfile(image_path, directory / "replay-image.json")
        record["image_manifest"] = seal(directory / "replay-image.json")
    record["status"] = "passed" if code == 0 and image_path.exists() else "failed"
else:
    producer_dir = root / attempt / "linux-prepare"
    producer = json.loads((producer_dir / "result.json").read_text())
    assert producer["status"] == "passed"
    bundle = json.loads((producer_dir / "replay-image.json").read_text())
    original_scratch = Path(producer["scratch"])

    def host_path(private):
        result = original_scratch / Path(private).relative_to("/replay-output")
        assert result.resolve().is_relative_to(original_scratch)
        return result

    for file in bundle["inputs"]:
        path = host_path(file["private"])
        assert path.is_file() and not path.is_symlink()
        raw = path.read_bytes()
        for obj in file["objects"]:
            assert len(raw) == obj["bytes"] and hashlib.sha256(raw).hexdigest() == obj["raw_sha256"]
            encoded = json.dumps({k: v for k, v in obj.items() if k != "id"}, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
            assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + encoded).hexdigest() == obj["id"]
    for child in ["build", "tmp", "home", "empty"]:
        (scratch / child).mkdir()
    bindings = {}
    for item in bundle["mounts"]:
        original, private = item["original"], host_path(item["private"])
        assert original.startswith("/") and private.is_dir() and not private.is_symlink()
        assert original not in bindings or bindings[original] == private
        bindings[original] = private
    root_sdk = bundle["compiler_original_root"]
    environment = {
        "CGO_ENABLED": "0", "GO111MODULE": "on", "GOAMD64": "v1", "GOARCH": "amd64",
        "GOCACHE": "/mpk-work/build", "GODEBUG": "", "GOENV": "off", "GOEXPERIMENT": "", "GOFLAGS": "",
        "GOMAXPROCS": "1", "GOMODCACHE": bundle["cache_original_directory"], "GONOPROXY": "", "GONOSUMDB": "",
        "GOOS": "linux", "GOPACKAGESDRIVER": "off", "GOPATH": "/mpk-work/empty", "GOPRIVATE": "", "GOPROXY": "off",
        "GOROOT": root_sdk, "GOSUMDB": "off", "GOTELEMETRY": "off", "GOTOOLCHAIN": "local", "GOVCS": "*:off", "GOWORK": "off",
        "HOME": "/mpk-work/home", "LANG": "C", "LC_ALL": "C", "PATH": root_sdk + "/bin",
        "TMPDIR": "/mpk-work/tmp", "TZ": "UTC",
    }
    def invocation(overrides=None):
        actual = dict(bindings)
        actual.update(overrides or {})
        result = common + ["--tmpfs", "/usr/local/go:ro,noexec,nosuid,nodev,mode=000,size=4096"]
        for original, private in sorted(actual.items()):
            result += mount(private, original)
        result += mount(scratch, "/mpk-work", False)
        for key, value in environment.items():
            result += ["-e", key + "=" + value]
        return result + ["--workdir", bundle["main_original_directory"], image]
    prefix = invocation()
    executable = root_sdk + "/bin/go"
    if mode == "controls":
        readonly = execute("readonly-input", prefix + ["/usr/bin/chmod", "u+w", bundle["main_original_directory"] + "/go.mod"])
        ambient = execute("ambient-sdk", prefix + ["/usr/bin/test", "-x", "/usr/local/go/bin/go"])
        cache_original = bundle["cache_original_directory"]
        alternate = scratch / "missing-original-dependency"
        shutil.copytree(bindings[cache_original], alternate)
        missing = alternate / "example.com/dep@v1.1.0/dep.go"
        original_missing = seal(missing)
        missing.parent.chmod(0o755)
        missing.unlink()
        missing.parent.chmod(0o555)
        failure = execute("missing-original-source", invocation({cache_original: alternate}) + [executable, "list", "-mod=readonly", "-deps", "-export", "-compiled", "-json", bundle["requested_package"]])
        record["missing_original_file"] = original_missing
        record["semantic_source_counterexample_claimed"] = False
        code = 0 if readonly != 0 and ambient == 1 and failure != 0 else 1
    else:
        code = execute("modules", prefix + [executable, "list", "-mod=readonly", "-m", "-json", "all"])
        if code == 0:
            code = execute("packages", prefix + [executable, "list", "-mod=readonly", "-deps", "-export", "-compiled", "-json", bundle["requested_package"]])
    record["status"] = "passed" if code == 0 else "failed"
    record["producer"] = seal(producer_dir / "result.json")
    record["image_manifest"] = seal(producer_dir / "replay-image.json")
record["sources_unchanged_during_execution"] = all(seal(row["path"]) == row for row in sources)
if not record["sources_unchanged_during_execution"]:
    record["status"] = "failed"
record["finished_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
record["checks"] = checks
(directory / "result.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({"status": record["status"], "mode": mode, "scratch": str(scratch),
                  "sources_unchanged_during_execution": record["sources_unchanged_during_execution"]}))
raise SystemExit(0 if record["status"] == "passed" else 1)
