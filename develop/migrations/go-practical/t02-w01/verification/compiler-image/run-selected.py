"""Verify complete prepared Go compiler images, original source witnesses and executable build identities."""

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
pattern = "^(TestPracticalCompiler.*|TestLoaderEnvironmentIsClosedAndSorted|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$"
commands = {
    "selected": [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."],
    "vet": [str(goroot / "bin/go"), "vet", "-mod=vendor", "."],
    "format": [str(goroot / "bin/gofmt"), "-l", "practical_compiler.go", "practical_compiler_test.go", "practical_compiler_capture_test.go"],
}
commands["legacy"] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^(TestPracticalCompiler.*|TestPracticalRootUnavailableIsAnError|TestPracticalSourceDoesNotActivateInstalledProfile|TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads)$", "-count=1", "-json", "."]
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
assert attempt in ("initial", "final", "reviewed", "captured")
if attempt in ("final", "captured") and mode == "selected":
    commands[mode] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", "^TestPracticalCompilerValidatesActualPreparedImageAndRejectsReauthorizedChanges$", "-count=1", "-json", "."]
if attempt == "reviewed" and mode in ("selected", "legacy"):
    pattern = "^TestPracticalCompilerBuildInfoRejectsMixedAndUnpinnedToolConfigurations$" if mode == "legacy" else "^TestPracticalCompiler(BuildInfoRejectsMixedAndUnpinnedToolConfigurations|ValidatesActualPreparedImageAndRejectsReauthorizedChanges)$"
    commands[mode] = [str(goroot / "bin/go"), "test", "-mod=vendor", "-run", pattern, "-count=1", "-json", "."]
directory = root / mode if attempt == "initial" else root / attempt / mode
directory.mkdir(parents=True, exist_ok=False)
if attempt == "captured" and mode == "selected":
    environment["MPK_GO_PRACTICAL_COMPILER_EVIDENCE"] = str((directory / "prepared-image.json").resolve())
    env.update(environment)
record = {"status": "running", "command": commands[mode], "cwd": "go-tools/go2vir",
          "environment": environment, "compiler_version": version,
          "compiler": seal(goroot / "bin/go"),
          "sources": [seal(Path("go-tools/go2vir") / name) for name in ["practical_compiler.go", "practical_compiler_test.go", "practical_compiler_capture_test.go"]],
          "frozen_contract": seal("develop/migrations/go-practical/t01-w01/entry-audit.json"),
          "selection_reason": "New prepared-image validator covers actual complete original Go1.25.0 implementation/support content and all nine command/tool binaries, exact source witnesses, host/release/tool build metadata, full existing inventory/partition/hash validation and owned revalidation. Reauthorized altered witness/tool/release/mixed-host and old partial-image controls reject. Select unchanged closed environment and inactive public-profile/argument boundaries. Legacy runs two pure compiler metadata tests plus root-unavailable/public boundaries. No application package typing or registered context/native Linux receipt is claimed. CSharp tests and the intermediate full gate are deferred.",
          "full_T02_gate": "deferred to T02-W04; use check-fast.sh --skip-csharp",
          "T02_W01_complete": False,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
if attempt == "final":
    record["selection_reason"] = "Fix the full-SDK test fixture to seal files as well as directories; production compiler validation code is unchanged. Retest the entire native full-image root and all controls, plus current vet/format. The five unaffected initial root tests and five legacy roots remain passing and are not repeated. No CSharp tests, application proof, context/native release or intermediate full gate."
if attempt == "reviewed":
    record["selection_reason"] = "Review requires Linux-host AMD64-v1 tool binaries and isolates the build-info check for both host-independent negative validation and actual nine-tool validation. Retest the complete native full-image root and all controls plus the new eleven-case build-configuration matrix, legacy compilation of that matrix and current vet/format. Unaffected original pins/missing-inventory/environment/public checks retain initial results. No CSharp tests or intermediate full gate."
if attempt == "captured":
    record["selection_reason"] = "Add test-only exclusive evidence output before caller-mutation controls, retaining the complete freshly verified compiler/component inventories. Retest that native root and all six controls, plus vet/format after the capture-test change. Production validation and the reviewed eleven-case metadata matrix are unchanged; previous pure/legacy/public-boundary results are retained. No CSharp tests or intermediate full gate."
record["input_vectors"] = [seal("develop/specs/GO_PRACTICAL_PROFILE_V1.md"), seal("develop/migrations/go-practical/t01-w01/entry-audit.json")]

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
    assert len(record["root_tests_passed"]) == ((1 if mode == "legacy" else 2) if attempt == "reviewed" else (5 if mode == "legacy" else 1 if attempt in ("final", "captured") else 6))
if mode == "format" and (directory / "stdout.log").stat().st_size:
    record["status"] = "failed"
    record["format_error"] = "gofmt listed unformatted files"
if attempt == "captured" and mode == "selected" and record["status"] == "passed":
    record["prepared_image"] = seal(directory / "prepared-image.json")
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("status", "exit_code", "wall_seconds")}))
raise SystemExit(0 if record["status"] == "passed" else 1)
