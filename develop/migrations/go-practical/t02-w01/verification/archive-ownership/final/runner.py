"""Scoped local checks for the sealed original namespace and its consumers."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
mode, attempt = sys.argv[1:]
assert mode in {"regression", "selected", "legacy", "vet", "format"}
assert attempt in {"original", "final", "reviewed"}
goroot = Path("/usr/local/go") if mode == "legacy" else Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")
pattern = "^(TestPracticalArchiveReturnedValuesCannotMutateSessionCache|TestPracticalArchiveCaptureSharesNamespaceBudgetAndOwnedInputs|TestPracticalArchiveCaptureRejectsForgedOwnerAndUnavailableSnapshot|TestPracticalModuleSnapshotRetainsExactVersionedDownloadInputs|TestPracticalModuleSnapshotRetainsLegacyArchiveWithoutInsertedManifest|TestPracticalModuleSnapshotRejectsMissingStaleOrInconsistentOriginals|TestPracticalModuleSnapshotUsesOriginalSumsAndRejectsLocationAliases|TestPracticalResolutionUsesActualGoVersionSelectionAndOriginalSnapshots|TestPracticalResolutionBindsActualLocalAndVersionedReplacements|TestPracticalResolutionRejectsAlteredGoObservationsAgainstOriginalInputs|TestPracticalResolutionMissingOfflineInputsNeverAcquireReplacementSources|TestPracticalResolutionRejectsChecksumsOwnedByAnotherModuleInput|TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend|TestPracticalAnalysisCaptureRejectsAlteredAndUnclosedInputs)$"
commands = {
    "regression": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalArchiveReturnedValuesCannotMutateSessionCache$", "-count=1", "-json", "."],
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "legacy": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^$", "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_archive.go", "practical_archive_ownership_test.go"],
}
environment = {
    "GOROOT": str(goroot), "GOTOOLCHAIN": "local", "GOENV": "off",
    "GOPROXY": "off", "GOSUMDB": "off", "GOVCS": "*:off", "GOTELEMETRY": "off", "GOWORK": "off",
    "CGO_ENABLED": "0", "GOMAXPROCS": "1", "GOFLAGS": "-buildvcs=false",
    "GOCACHE": "/private/tmp/mpk-go-practical-t02-legacy-build-cache" if mode == "legacy" else "/private/tmp/mpk-go-practical-t02-build-cache",
    "GOPATH": "/private/tmp/mpk-go-practical-t02-gopath",
    "GOMODCACHE": "/private/tmp/mpk-go-practical-t02-go-mod",
}
env = os.environ.copy()
env.update(environment)
version = subprocess.check_output([str(goroot / "bin/go"), "version"], env=env, text=True).strip()
assert version == ("go version go1.23.2 darwin/arm64" if mode == "legacy" else "go version go1.25.0 darwin/arm64")
directory = root / attempt / mode
directory.mkdir(parents=True, exist_ok=False)
source_files = sorted(Path("go-tools/go2vir").glob("*.go"))
source_files += [Path("go-tools/go2vir/go.mod"), Path("go-tools/go2vir/go.sum"), Path("go-tools/go2vir/vendor/modules.txt")]
sources = [seal(p) for p in source_files]
runner_copy = root / attempt / "runner.py"
if not runner_copy.exists():
    runner_copy.write_bytes(Path(__file__).read_bytes())
else:
    assert runner_copy.read_bytes() == Path(__file__).read_bytes()
archive = root / attempt / "inputs"
if not archive.exists():
    for record in sources:
        source = Path(record["path"])
        target = archive / source
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(source.read_bytes())
else:
    for record in sources:
        actual = seal(archive / record["path"])
        assert (actual["sha256"], actual["bytes"]) == (record["sha256"], record["bytes"])
record = {
    "status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
    "environment": environment, "compiler_version": version, "compiler": seal(goroot / "bin/go"),
    "sources": sources, "source_snapshot": str(archive), "runner": seal(__file__),
    "selection_reason": "Actual first/cache-hit archive return ownership bug was reproduced with complete original ZIP/member/main metadata bytes retained. Deep separate original bytes/member maps protect session checksums and source capture against returned-value mutation without double budget charges. Select all14 affected rooted archive/versioned module/original version-resolution/analysis-capture consumers, including actual two-fresh-cache frontend rebuilds because compiled production input changed. Go1.23 performs compile/link compatibility only(no tests), since the changed helper has no legacy rooted-capture path. Unchanged pure ZIP/hash/native oracle/source/scalar/whole-checker suites are retained without repetition. Vet/format scoped. No CSharp/agents/Actions/intermediate whole gate, public/context/application replay/Linux-release claim.",
    "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "T02_W01_complete": False,
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
if mode == "selected":
    environment["MPK_GO_PRACTICAL_ANALYSIS_EVIDENCE"] = str(directory.resolve())
    env.update(environment)
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    code = subprocess.run(commands[mode], cwd="go-tools/go2vir", env=env, stdout=out, stderr=err, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code,
              wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"))
record["sources_unchanged_during_execution"] = all(seal(r["path"]) == r for r in sources)
if not record["sources_unchanged_during_execution"]:
    record["status"] = "failed"
if mode in {"regression", "selected", "legacy"}:
    events = [json.loads(line) for line in (directory / "stdout.log").read_text().splitlines()]
    passed = [e["Test"] for e in events if e["Action"] == "pass" and "Test" in e]
    failed = [e["Test"] for e in events if e["Action"] == "fail" and "Test" in e]
    record["root_tests_passed"] = [n for n in passed if "/" not in n]
    record["subtests_passed"] = [n for n in passed if "/" in n]
    record["tests_failed"] = failed
if mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
    record["format_error"] = "gofmt listed unformatted files"
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ("status", "exit_code", "wall_seconds", "sources_unchanged_during_execution")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
