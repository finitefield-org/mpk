"""Verify original module ZIP provenance, namespace/CRC/size controls and independent offline Go checks."""

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
goroot = Path("/usr/local/go") if mode in ("legacy", "legacy-archive") else Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")
pattern = "^(TestPracticalArchive.*|TestPracticalCaptureSharesBudgetsAcrossCapabilities|TestPracticalModuleSumsBindExactManifestBytes|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$"
commands = {
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_archive.go", "practical_archive_fold.go", "practical_archive_test.go", "practical_archive_capture_test.go", "practical_capture.go", "vendor/golang.org/x/mod/sumdb/dirhash/hash.go"],
}
commands["archive"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalArchive", "-count=1", "-json", "."]
commands["legacy-archive"] = commands["archive"]
commands["legacy"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^(TestPracticalArchive.*|TestPracticalRootUnavailableIsAnError|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$", "-count=1", "-json", "."]
assert mode in commands
environment = {"GOROOT": str(goroot), "GOTOOLCHAIN": "local", "GOENV": "off",
               "GOPROXY": "off", "GOSUMDB": "off", "GOTELEMETRY": "off", "GOWORK": "off",
               "GOCACHE": "/private/tmp/mpk-go-practical-t02-build-cache",
               "GOPATH": "/private/tmp/mpk-go-practical-t02-gopath",
               "GOMODCACHE": "/private/tmp/mpk-go-practical-t02-go-mod"}
if mode in ("legacy", "legacy-archive"):
    environment["GOCACHE"] = "/private/tmp/mpk-go-practical-t02-legacy-build-cache"
env = os.environ.copy()
env.update(environment)
version = subprocess.check_output([str(goroot / "bin/go"), "version"], env=env, text=True).strip()
assert version == ("go version go1.23.2 darwin/arm64" if mode in ("legacy", "legacy-archive") else "go version go1.25.0 darwin/arm64")
attempt = sys.argv[2] if len(sys.argv) == 3 else "initial"
assert attempt in ("initial", "final", "reviewed")
directory = root / mode if attempt == "initial" else root / attempt / mode
directory.mkdir(parents=True, exist_ok=False)
record = {"status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
          "environment": environment, "compiler_version": version,
          "compiler": seal(goroot / "bin/go"),
          "sources": [seal(Path("go-tools/go2vir") / name) for name in ["practical_archive.go", "practical_archive_fold.go", "practical_archive_test.go", "practical_archive_capture_test.go", "practical_capture.go", "vendor/golang.org/x/mod/sumdb/dirhash/hash.go"]],
          "frozen_contract": seal("develop/migrations/go-practical/t01-w05/source-kind/catalog.json"),
          "selection_reason": "New owned module ZIP verification and captured checksum authority, complete path/Unicode collision checks, CRC/size errors, immutable content linkage and aggregate cross-capability namespace budgets. Compare the retained independent Go h1 vector and current pinned offline Go mod verify on original/tampered cache files. Shared old budget, manifest sum and closed public-profile boundaries are selected consumers. Legacy runs six pure archive cases and old closed/unavailable-root boundaries. No CSharp tests or intermediate full gate.",
          "full_T02_gate": "deferred to T02-W04; use check-fast.sh --skip-csharp",
          "T02_W01_complete": False,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
record["input_vectors"] = [seal("develop/migrations/go-practical/t01-w02/module-probes.json"), seal("develop/probes/go-practical/module-cases.json"), seal(root/"vendor-provenance.json")]
record["sources"].append(seal("go-tools/go2vir/vendor/modules.txt"))

if mode in ("archive", "legacy-archive"):
    record["selection_reason"] = "After the shadowed-directory review fix, rerun all affected archive parser/capture/native-Go consumers (nine on pinned Go1.25, six pure archive cases on the legacy development compiler). Initial passing old budget/manifest/profile/argument checks are unchanged and retained, not repeated. No CSharp tests or intermediate full gate."
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    code = subprocess.run(commands[mode], cwd="go-tools/go2vir", env=env, stdout=out, stderr=err, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code,
              wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"))
if mode in ("selected", "legacy", "archive", "legacy-archive") and code == 0:
    events = [json.loads(line) for line in (directory / "stdout.log").read_text().splitlines()]
    passed = [event["Test"] for event in events if event["Action"] == "pass" and "Test" in event]
    record["root_tests_passed"] = [name for name in passed if "/" not in name]
    record["subtests_passed"] = [name for name in passed if "/" in name]
    assert len(record["root_tests_passed"]) == (9 if mode in ("legacy", "archive") else 6 if mode == "legacy-archive" else 13)
if mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
    record["format_error"] = "gofmt listed unformatted files"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
