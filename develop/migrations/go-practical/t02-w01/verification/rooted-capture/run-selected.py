"""Verify the initial practical source-buffer executor and affected old boundaries."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time


def seal(path):
    data = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode = sys.argv[1]
goroot = Path("/usr/local/go") if mode == "legacy" else Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")
pattern = "^(TestPractical(Capture|Source).*|TestSnapshot.*|TestCanonicalJSONAndDomainHashesMatchSharedVectors|TestStrictJSONRejectsDuplicateAndForbiddenContractOrProtocolInput|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$"
commands = {
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_source.go", "practical_source_test.go", "practical_capture.go", "practical_capture_test.go", "practical_root_go125.go", "practical_root_unavailable.go", "practical_root_unavailable_test.go"],
}
commands["rooted"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalCapture", "-count=1", "-json", "."]
commands["legacy"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^(TestPracticalRootUnavailableIsAnError|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$", "-count=1", "-json", "."]
assert mode in commands
environment = {"GOROOT": str(goroot), "GOTOOLCHAIN": "local", "GOENV": "off",
               "GOPROXY": "off", "GOSUMDB": "off", "GOTELEMETRY": "off", "GOWORK": "off",
               "GOCACHE": "/private/tmp/mpk-go-practical-t02-build-cache",
               "GOPATH": "/private/tmp/mpk-go-practical-t02-gopath",
               "GOMODCACHE": "/private/tmp/mpk-go-practical-t02-go-mod"}
if mode == "legacy":
    environment["GOCACHE"] = "/private/tmp/mpk-go-practical-t02-legacy-build-cache"
env = os.environ.copy()
env.update(environment)
version = subprocess.check_output([str(goroot / "bin/go"), "version"], env=env, text=True).strip()
assert version == ("go version go1.23.2 darwin/arm64" if mode == "legacy" else "go version go1.25.0 darwin/arm64")
attempt = sys.argv[2] if len(sys.argv) == 3 else "initial"
assert attempt in ("initial", "final", "reviewed")
directory = root / mode if attempt == "initial" else root / attempt / mode
directory.mkdir(parents=True, exist_ok=False)
record = {"status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
          "environment": environment, "compiler_version": version,
          "compiler": seal(goroot / "bin/go"),
          "sources": [seal(Path("go-tools/go2vir") / name) for name in ["practical_source.go", "practical_source_test.go", "practical_capture.go", "practical_capture_test.go", "practical_root_go125.go", "practical_root_unavailable.go", "practical_root_unavailable_test.go"]],
          "frozen_contract": seal("develop/migrations/go-practical/t01-w05/source-kind/catalog.json"),
          "selection_reason": "New multi-capability rooted filesystem capture and owned snapshot, exact namespace spelling, observed byte/namespace mutations, deterministic leaf swaps including FIFO/outside links, shared budgets and sticky errors. Retain all practical source-object tests, existing snapshot consumers, shared JSON/hash and public-profile argument boundaries. Vet/format cover all changed/new Go files. No CSharp tests; no full stage gate during intermediate W01.",
          "full_T02_gate": "deferred to T02-W04; use check-fast.sh --skip-csharp",
          "T02_W01_complete": False,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
if mode == "rooted":
    record["selection_reason"] = "After source review, retain native names of unrelated assets instead of rejecting them as source paths. Rerun every affected rooted-capture consumer, including the strengthened complete original-module inventory check. Source-object/shared JSON and original snapshot implementations are unchanged after their passing20-test selection; do not rerun those unaffected tests. No CSharp tests or intermediate full gate."
if mode == "legacy":
    record["selection_reason"] = "The new rooted backend is guarded by Go1.25/host build constraints. Verify the existing Go1.23 module build and closed public-profile arguments still compile, while unsupported rooted capture returns an error. This does not add a supported compiler/target to the practical profile. No CSharp tests or whole-stage gate."
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    code = subprocess.run(commands[mode], cwd="go-tools/go2vir", env=env, stdout=out, stderr=err, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code,
              wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"))
if mode in ("selected", "rooted", "legacy") and code == 0:
    events = [json.loads(line) for line in (directory / "stdout.log").read_text().splitlines()]
    passed = [event["Test"] for event in events if event["Action"] == "pass" and "Test" in event]
    record["root_tests_passed"] = [name for name in passed if "/" not in name]
    record["subtests_passed"] = [name for name in passed if "/" in name]
    assert len(record["root_tests_passed"]) == (3 if mode == "legacy" else 8 if mode == "rooted" else 20)
if mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
    record["format_error"] = "gofmt listed unformatted files"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
