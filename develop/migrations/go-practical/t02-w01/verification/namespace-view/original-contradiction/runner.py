"""Scoped original namespace consistency and owned execution views."""
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
assert mode in {"diagnostic", "ownership", "regression", "selected", "legacy", "vet", "format"}
assert attempt.replace("-", "").isalnum()
goroot = Path("/usr/local/go") if mode == "legacy" else Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")
pattern = "^(TestPracticalReplayImage.*|TestPracticalReplayNamespace.*|TestPracticalReplayView.*|TestPracticalExport.*|TestPracticalStandardLibrary.*|TestPracticalSnapshot.*|TestPracticalPackageBuild.*|TestPracticalSourceTypes.*|TestPracticalResolutionBindsActualLocalAndVersionedReplacements|TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend|TestPracticalAnalysisCaptureRejectsAlteredAndUnclosedInputs)$"
commands = {
    "diagnostic": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalSourceTypesCheckOriginalBodiesAndNominalIdentity$", "-count=1", "-json", "."],
    "ownership": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalSyntaxReturnedSourceCannotMutateCachedOriginal$", "-count=1", "-json", "."],
    "regression": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalReplayImageRejectsContradictoryOriginalViews$", "-count=1", "-json", "."],
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "legacy": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^$", "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_package.go", "practical_typecheck.go", "practical_package_test.go", "practical_resolution.go", "practical_capture.go", "practical_stdlib.go", "practical_stdlib_test.go", "practical_exports.go", "practical_exports_test.go", "practical_replay.go", "practical_replay_test.go", "practical_namespace_test.go"],
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
source_files += [Path("develop/migrations/go-practical/t01-w04/source-admission/source-vectors.json"), Path("develop/migrations/go-practical/t01-w04/source-admission/native-facts.jsonl"), Path("go-tools/go2vir/go.mod"), Path("go-tools/go2vir/go.sum"), Path("go-tools/go2vir/vendor/modules.txt")]
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
    "selection_reason": "Reject contradictory original namespace views across shared/nested root aliases, bind original namespaces and exact materialized image closure to owned execution views before private context consumption. Preserve complete initial failing epochs. Affected namespace/snapshot/image/native package/analysis paths only. Materialize original captured byte images at original launcher-authorized absolute paths, preserve full native namespace and not-captured distinctions, compose all snapshots under shared frozen budgets, and clone full original compiler inventory without source/mod rewriting. Native export preparation now derives the original offline module cache from bound versioned archive identities. Affected byte-image/export/source/package/module/analysis boundaries only. Capture complete original prepared Go1.25.0 compiler src inventory with final source roles derived atomically from the same original bytes, fixed Linux/amd64/v1 build rules and no relabeling/racing second read. Original native -deps -export -compiled output, complete original compiler .a bytes with own build IDs, exact original source files/import maps/selected root and package closure are bound as preparation observations. Whole original standard source dependencies/SDK vendor/internal namespaces and full source type checking must match native Go package/file/import maps without host fallback or typed-package adapters. Preserve all original SDK raw source objects, original application/native list outputs and every failed epoch. Affected source/capture/package/module/analysis consumers, current176 frontend-input empty-cache builds, legacy compile-only, vet/format; no CSharp/agents/Actions/intermediate whole gate. Complete export/native module/package immutable authority, full56-all6-19 foundation/private context/all169/T02-T11 remain due.",
    "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp",
    "T02_W01_complete": False,
    "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
if mode in {"selected", "regression", "ownership", "diagnostic"}:
    environment["MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"] = str(directory.resolve())
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
if mode in {"ownership", "regression", "selected", "legacy", "diagnostic"}:
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
