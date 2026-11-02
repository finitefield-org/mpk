"""Preserve complete prelaunch sources and run only affected history checks."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = Path.cwd()
BASIS = "1285838991ff33b83f02576fe1c4b2de348f70fb"
CHANGED = ["crates/mpk-vc/src/go_practical_ordinary.rs",
           "crates/mpk-vc/src/go_practical_ordinary/execution_history.rs",
           "crates/mpk-vc/tests/go_practical_execution_history.rs"]
SDK = Path("/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go")


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
            "mode": path.stat().st_mode & 0o777}


def main():
    epoch, mode = sys.argv[1:]
    assert epoch.replace("-", "").isalnum()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    inputs = {REPO / "Cargo.toml", REPO / "Cargo.lock", Path(__file__).resolve()}
    inputs.update(ROOT.glob("*.py"))
    for name in ["crates", "proofs/std", "fixtures", "develop/specs/vectors", "release/bundles", "go-tools/mpk-checker-ref"]:
        inputs.update(p for p in (REPO / name).rglob("*") if p.is_file())
    for name in [".cargo", "rust-toolchain", "rust-toolchain.toml"]:
        p = REPO / name
        if p.is_dir():
            inputs.update(child for child in p.rglob("*") if child.is_file())
        elif p.is_file():
            inputs.add(p)
    if mode in ["statements", "pair"]:
        inputs.update(ROOT.glob("rust-*/certificates/*"))
        inputs.add(ROOT / "reference-build/mpk-checker-ref")
        inputs.update([REPO / "develop/probes/go-practical/carrier_terms.py",
                       ROOT.parent / "source-invocations/structural-oracle.py",
                       ROOT.parent / "source-error-layouts/structural-oracle.py"])
    sources = [seal(p) for p in sorted(inputs)]
    with tarfile.open(out / "inputs.tar.gz", "x:gz", dereference=True) as archive:
        for p in sorted(inputs):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    (out / "inputs.json").write_text(json.dumps({"basis_commit": BASIS, "sources": sources,
                                                "complete_archive": seal(out / "inputs.tar.gz")}, indent=2) + "\n")
    commands = {
        "format-write": ["rustfmt", "--edition=2021", "--config", "skip_children=true", *CHANGED],
        "format": ["rustfmt", "--edition=2021", "--check", "--config", "skip_children=true", *CHANGED],
        "rust": ["cargo", "test", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_execution_history", "--", "--nocapture", "--test-threads=1"],
        "clippy": ["cargo", "clippy", "--offline", "--locked", "-p", "mpk-vc", "--lib", "--test", "go_practical_execution_history", "--", "-D", "warnings"],
        "go-build": [str(SDK / "bin/go"), "build", "-trimpath", "-o", str(out / "mpk-checker-ref"), "./cmd/mpk-checker-ref"],
        "statements": [sys.executable, "-B", str(ROOT / "inspect-statements.py"), str(ROOT / "rust-borrow-fixed/certificates/foundation.mpcert")],
        "pair": [sys.executable, "-B", str(ROOT / "check-pair.py")],
    }
    split = commands["rust"].index("--")
    commands["rust-controls"] = commands["rust"][:split] + ["invalid_edges_initial_safety_and_missing_normal_completion_reject"] + commands["rust"][split:]
    env = os.environ.copy()
    changes = {"CARGO_NET_OFFLINE": "true", "CARGO_PROFILE_TEST_OPT_LEVEL": "1", "CARGO_PROFILE_TEST_DEBUG_ASSERTIONS": "true",
               "CARGO_PROFILE_TEST_OVERFLOW_CHECKS": "true", "CARGO_INCREMENTAL": "0", "RUST_TEST_THREADS": "1"}
    if mode in ["rust", "rust-controls"]:
        changes["MPK_GO_EXECUTION_HISTORY_OUTPUT"] = str(out / "certificates")
    if mode == "go-build":
        changes.update({"GOROOT": str(SDK), "GOTOOLCHAIN": "local", "GOENV": "off", "GOWORK": "off", "GOPROXY": "off",
                        "GOSUMDB": "off", "CGO_ENABLED": "0", "GOMAXPROCS": "1", "GOCACHE": "/private/tmp/mpk-go-practical-t02-go-cache"})
    env.update(changes)
    for key in ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS"]:
        env.pop(key, None)
    report = {"status": "running", "command": commands[mode], "environment": changes,
              "inputs": seal(out / "inputs.json"), "driver": seal(__file__),
              "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "tools": {name: seal(shutil.which(name)) for name in ["cargo", "rustc", "rustfmt"]},
              "selection_reason": "The new ordinary history component and its direct mathematical consumers use unbounded Nat histories, every actual transition edge, safety on all finite prefixes, same-run state/post observations and a separate normal completion witness. Kernels, original source/capture code, unchanged large proof matrices and other language tests are not changed or rerun.",
              "whole_T02_gate": "Deferred to final T02-W04: ./scripts/check-fast.sh"}
    if mode == "go-build":
        report["go_tool"] = seal(SDK / "bin/go")
    result_path = out / "result.json"
    result_path.write_text(json.dumps(report, indent=2) + "\n")
    start = time.monotonic()
    with (out / "stdout.log").open("xb") as stdout, (out / "stderr.log").open("xb") as stderr:
        result = subprocess.run(commands[mode], cwd=REPO / "go-tools/mpk-checker-ref" if mode == "go-build" else REPO,
                                env=env, stdout=stdout, stderr=stderr, check=False)
    unchanged = sources == [seal(p) for p in sorted(inputs)]
    report.update(status="passed" if result.returncode == 0 and (unchanged or mode == "format-write") else "failed",
                  exit_code=result.returncode, seconds=time.monotonic() - start, consumed_inputs_unchanged=unchanged,
                  expected_formatter_mutation=mode == "format-write", stdout=seal(out / "stdout.log"), stderr=seal(out / "stderr.log"),
                  finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    result_path.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in ["status", "exit_code", "seconds", "consumed_inputs_unchanged"]}))
    raise SystemExit(report["status"] != "passed")


if __name__ == "__main__":
    main()
