"""Local affected checks for the owned original Go package universe.

Run outside the sandbox because this recorder reads the original Git index.
"""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = "ed67a4c388defe8babdbc55f29851552562121d9"
epoch, mode = sys.argv[1:]
assert epoch.replace("-", "").isalnum()
assert mode in {"selected", "consumers", "vet", "format", "legacy"}


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


assert git("rev-parse", "HEAD").decode().strip() == BASE
original = {}
for row in git("ls-files", "--stage", "-z", "--", "go-tools/go2vir").split(b"\0"):
    if not row:
        continue
    metadata, name = row.split(b"\t", 1)
    mode_bits, oid, stage = metadata.decode().split()
    assert stage == "0"
    original[name.decode()] = oid
sources = sorted(Path("go-tools/go2vir").glob("*.go"))
sources += sorted(p for p in Path("go-tools/go2vir/vendor").rglob("*") if p.is_file())
sources += [Path("go-tools/go2vir/go.mod"), Path("go-tools/go2vir/go.sum")]
sources = sorted(set(sources))
archive = OUT / epoch
archive.mkdir(exist_ok=True)
inputs = []
for path in sources:
    raw = path.read_bytes()
    row = seal(path)
    oid = original.get(str(path))
    if oid and hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest() == oid:
        row.update(original_commit=BASE, original_git_blob=oid)
    else:
        target = archive / "inputs" / path
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.read_bytes() == raw
        else:
            target.write_bytes(raw)
        row["complete_changed_input"] = str(target.relative_to(ROOT))
    inputs.append(row)
receipt = {"base_commit": BASE, "sources": inputs, "runner": seal(__file__)}
input_path = archive / "inputs.json"
if input_path.exists():
    assert json.loads(input_path.read_bytes()) == receipt
else:
    input_path.write_text(json.dumps(receipt, indent=2) + "\n")

goroot = Path("/usr/local/go") if mode == "legacy" else Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")
go = str(goroot / "bin/go")
environment = {
    "GOROOT": str(goroot), "GOTOOLCHAIN": "local", "GOENV": "off",
    "GOWORK": "off", "GOFLAGS": "-mod=vendor", "GOPROXY": "off",
    "GOSUMDB": "off", "CGO_ENABLED": "0", "GOMAXPROCS": "1",
    "GOCACHE": "/private/tmp/mpk-go-practical-t02-go-cache",
    "GOMODCACHE": "/private/tmp/mpk-go-practical-t02-go-mod",
}
env = os.environ.copy()
env.update(environment)
version = subprocess.check_output([go, "version"], env=env, text=True).strip()
assert version == ("go version go1.23.2 darwin/arm64" if mode == "legacy" else "go version go1.25.0 darwin/arm64")
directory = archive / mode
directory.mkdir(exist_ok=False)
if mode == "selected":
    environment["MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"] = str(directory)
    env.update(environment)
commands = {
    "selected": [go, "test", "-json", "-count=1", "-timeout=20m", "-run", "^TestPracticalOriginalPackages"],
    "consumers": [go, "test", "-json", "-count=1", "-timeout=20m", "-run", "^TestPractical(ImageInputs|Observation|ReplayBaseline|ModuleChecksumReturnedViews|ExportMetadata|ExportRoot)"],
    "vet": [go, "vet", "./..."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_original_packages.go", "practical_original_packages_test.go"],
    "legacy": [go, "test", "-json", "-run", "^$", "./..."],
}
record = {
    "status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
    "environment": environment, "compiler_version": version, "compiler": seal(go),
    "source_receipt": seal(input_path), "sources": inputs,
    "selection_reason": "The new original-package owner reconstructs source/module/export inputs through the verified image, retypes complete original packages and binds actual command observations. Exercise that owner, crossing/membership/alias/lifetime controls and the directly reused image/module/checksum/observation/export/baseline consumers. Include current vet and changed-file format; legacy compilation executes zero tests. No unrelated arithmetic, kernel, protocol or whole-workspace suites are rerun.",
    "native_release_acceptance": False, "R06_complete": False, "W01_complete": False,
    "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh",
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as stdout, (directory / "stderr.log").open("wb") as stderr:
    code = subprocess.run(commands[mode], cwd="go-tools/go2vir", env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code,
              wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"),
              whole_source_inputs_unchanged=all(seal(row["path"])["sha256"] == row["sha256"] for row in inputs))
if mode in {"selected", "consumers", "legacy"}:
    events = [json.loads(line) for line in (directory / "stdout.log").read_text().splitlines()]
    record["passed_tests"] = [e["Test"] for e in events if e["Action"] == "pass" and "Test" in e]
    record["failed_tests"] = [e["Test"] for e in events if e["Action"] == "fail" and "Test" in e]
    record["skipped_tests"] = [e["Test"] for e in events if e["Action"] == "skip" and "Test" in e]
if not record["whole_source_inputs_unchanged"] or mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: v for k, v in record.items() if k in {"status", "exit_code", "wall_seconds", "whole_source_inputs_unchanged", "passed_tests", "failed_tests", "skipped_tests"}}))
raise SystemExit(record["status"] != "passed")
