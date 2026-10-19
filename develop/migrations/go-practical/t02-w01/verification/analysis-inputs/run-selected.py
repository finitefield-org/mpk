"""Verify original analysis-library source closure and two independent offline owned-source frontend builds."""

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
pattern = "^(TestPracticalAnalysis.*|TestPracticalCompilerBuildInfoRejectsMixedAndUnpinnedToolConfigurations|TestPracticalCompilerValidatesActualPreparedImageAndRejectsReauthorizedChanges|TestLoaderEnvironmentIsClosedAndSorted|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$"
commands = {
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_analysis.go", "practical_analysis_test.go", "practical_analysis_capture_test.go", "practical_compiler.go"],
}
commands["legacy"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^(TestPracticalAnalysis.*|TestPracticalRootUnavailableIsAnError|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$", "-count=1", "-json", "."]
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
assert attempt in ("initial", "final", "reviewed", "focused")
if attempt == "final" and mode == "selected":
    commands[mode] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend$", "-count=1", "-json", "."]
directory = root / mode if attempt == "initial" else root / attempt / mode
directory.mkdir(parents=True, exist_ok=False)
if mode == "selected":
    environment["MPK_GO_PRACTICAL_ANALYSIS_EVIDENCE"] = str(directory.resolve())
    env.update(environment)
record = {"status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
          "environment": environment, "compiler_version": version,
          "compiler": seal(goroot / "bin/go"),
          "sources": [seal(Path("go-tools/go2vir") / name) for name in ["practical_analysis.go", "practical_analysis_test.go", "practical_analysis_capture_test.go", "practical_compiler.go"]],
          "frozen_contract": seal("develop/migrations/go-practical/t01-w01/entry-audit.json"),
          "selection_reason": "New exact pinned analysis manifests/sums and vendor package ownership; complete original module snapshots, original compiled Go/LICENSE/PATENTS bytes and explicitly retained uncompiled TODO difference; complete sealed frontend/vendor build inputs and binary build metadata. Two actual offline Go1.25.0 frontend builds use only owned original bytes, distinct paths and independent empty caches; compare complete binaries. Retest both affected compiler binary/build-info consumers after the shared rooted-reader refactor and unchanged environment/public boundaries. Legacy runs pure analysis contracts plus root-unavailable/public boundaries. No CSharp tests, application proof, practical context/native release or intermediate full gate.",
          "full_T02_gate": "deferred to T02-W04; use check-fast.sh --skip-csharp",
          "T02_W01_complete": False,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
if attempt == "final":
    record["selection_reason"] = "Fix only the native integration fixture selected output root: explicitly supply its physical normalized path and retain rejection of the original temporary path alias. The first original owned-source offline build succeeded before output-root validation stopped. Retest the complete two-fresh-cache binary reproduction root, plus current vet/format. The other ten current Go roots and seven legacy roots passed and are unchanged; retain their original evidence. No CSharp tests or intermediate full gate."
record["input_vectors"] = [seal("go-tools/go2vir/go.mod"), seal("go-tools/go2vir/go.sum"), seal("go-tools/go2vir/vendor/modules.txt"), seal("go-tools/go2vir/testdata/practical-analysis-cache/PROVENANCE.json")]
record["fixture_inputs"] = [seal(p) for p in sorted(Path("go-tools/go2vir/testdata/practical-analysis-cache").rglob("*")) if p.is_file()]

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
    assert len(record["root_tests_passed"]) == (7 if mode == "legacy" else 1 if attempt == "final" else 11)
if mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
    record["format_error"] = "gofmt listed unformatted files"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
