"""Verify actual pinned offline Go module discovery and original snapshot/replacement binding."""

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
pattern = "^(TestPracticalResolution.*|TestLoaderEnvironmentIsClosedAndSorted|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$"
commands = {
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_resolution.go", "practical_resolution_test.go", "practical_resolution_capture_test.go"],
}
commands["legacy"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^(TestPracticalResolution.*|TestPracticalRootUnavailableIsAnError|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$", "-count=1", "-json", "."]
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
if attempt == "reviewed" and mode == "selected":
    commands[mode] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalResolution(UsesActualGoVersionSelectionAndOriginalSnapshots|BindsActualLocalAndVersionedReplacements|RejectsAlteredGoObservationsAgainstOriginalInputs|RejectsChecksumsOwnedByAnotherModuleInput)$", "-count=1", "-json", "."]
directory = root / mode if attempt == "initial" else root / attempt / mode
directory.mkdir(parents=True, exist_ok=False)
record = {"status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
          "environment": environment, "compiler_version": version,
          "compiler": seal(goroot / "bin/go"),
          "sources": [seal(Path("go-tools/go2vir") / name) for name in ["practical_resolution.go", "practical_resolution_test.go", "practical_resolution_capture_test.go"]],
          "frozen_contract": seal("develop/migrations/go-practical/t01-w05/source-kind/catalog.json"),
          "selection_reason": "New actual pinned offline Go list module discovery, original versioned/local/replacement input binding, transitive version upgrade, direct replacement precedence, strict/failed output rejection, output cancellation and exact compiler/target before source access. Select closed/sorted loader environment and inactive public profile/argument boundaries. Legacy executes four pure resolver tests and three closed/unavailable-root checks. Original source/ZIP/metadata implementations are unchanged and keep preceding evidence. Discovery is preparation only; production immutable replay, complete context/compiler/all-consulted-input closure and package typing remain due. No CSharp tests or intermediate full gate.",
          "full_T02_gate": "deferred to T02-W04; use check-fast.sh --skip-csharp",
          "T02_W01_complete": False,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
if attempt == "reviewed":
    record["selection_reason"] = "After reproducing foreign checksum-owner substitution, retest all four binder consumers: actual transitive selected modules, exact local/versioned replacements, altered observations and the new original-main checksum-owner rejection. Original output parser/offline-failure/environment/public-boundary code is unchanged and retains initial passing results. Legacy recompiles the common code and retains its seven closed/pure/root-unavailable checks. Vet and format cover the three changed files. No CSharp tests or intermediate full gate."
record["input_vectors"] = [seal("develop/migrations/go-practical/t01-w02/module-probes.json"), seal("develop/probes/go-practical/module-cases.json")]

result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    code = subprocess.run(commands[mode], cwd="go-tools/go2vir", env=env, stdout=out, stderr=err, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code,
              wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"))
if mode in ("selected", "legacy") and code == 0:
    events = [json.loads(line) for line in (directory / "stdout.log").read_text().splitlines()]
    passed = [event["Test"] for event in events if event["Action"] == "pass" and "Test" in event]
    record["root_tests_passed"] = [name for name in passed if "/" not in name]
    record["subtests_passed"] = [name for name in passed if "/" in name]
    assert len(record["root_tests_passed"]) == (7 if mode == "legacy" else 4 if attempt == "reviewed" else 12 if attempt == "final" else 11)
if mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
    record["format_error"] = "gofmt listed unformatted files"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
