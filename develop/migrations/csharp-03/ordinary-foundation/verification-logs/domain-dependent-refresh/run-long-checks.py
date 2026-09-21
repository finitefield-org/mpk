"""Run only the changed certificate checks deferred by the 55-second agent limit."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--list", action="store_true", help="Validate and list without running tests")
    parser.add_argument("--output", type=Path, help="New directory for this run's logs")
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    repo = here.parents[5]
    manifest = json.loads((here / "pending-long-checks.json").read_text())
    cases = manifest["cases"]
    runtime_cases = manifest.get("runtime_cases", [])
    if not cases and not runtime_cases:
        raise SystemExit("No pending checks")
    for row in cases:
        path = repo / row["certificate_path"]
        if digest(path) != row["certificate_file_sha256"]:
            raise SystemExit(f"Certificate changed since handoff: {path}")
    for path, expected in manifest["checker_source_sha256"].items():
        if digest(repo / path) != expected:
            raise SystemExit(f"Checker source changed since handoff: {path}")
    for path, expected in manifest.get("runtime_source_sha256", {}).items():
        if digest(repo / path) != expected:
            raise SystemExit(f"Runtime source changed since handoff: {path}")
    if args.list:
        for row in cases:
            print(row["family"], row["file"], row["test_filter"])
        for row in runtime_cases:
            print("runtime", row["label"], row["test"])
        return
    if args.output:
        args.output.mkdir(parents=True, exist_ok=False)
        output = args.output.resolve()
    else:
        output = Path(tempfile.mkdtemp(prefix="mpk-w09-domain-dependent-long-", dir="/tmp"))
    print(f"LOGS {output}", flush=True)
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    results = []
    for row in cases:
        label = row["family"] + "__" + Path(row["file"]).stem
        log = output / (label + ".log")
        command = ["go", "test", "-v", "-count=1", "-timeout", "25m",
                   "-tags", "checkeragreement", "-run", row["test_filter"], "."]
        print(f"START {label}", flush=True)
        start = time.monotonic()
        with log.open("w") as stream:
            result = subprocess.run(command, cwd=repo / "go-tools/mpk-checker-ref",
                                    stdout=stream, stderr=subprocess.STDOUT, check=False)
        text = log.read_text()
        passed = result.returncode == 0 and ("/" + row["file"] + " (") in text and "\nPASS\n" in text
        receipt = {**row, "command": command, "exit_code": result.returncode,
                   "passed": passed, "wall_seconds": time.monotonic() - start,
                   "log": log.name, "log_sha256": digest(log)}
        results.append(receipt)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        print(f"END {label} {'PASS' if passed else 'FAIL'} {receipt['wall_seconds']:.2f}s", flush=True)
        if not passed:
            print(text[-12000:], flush=True)
            raise SystemExit(result.returncode or 1)
    for row in runtime_cases:
        label = row["label"]
        log = output / (label + ".log")
        command = ["cargo", "test", "--release", "-p", "mpk-vc", *row["target"],
                   row["test"], "--", "--nocapture", "--test-threads=1"]
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("MPK_W09_", "MPK_CORE_"))}
        env["MPK_CORE_MAX_SECONDS"] = str(row["evaluator_seconds"])
        print(f"START {label}", flush=True)
        start = time.monotonic()
        with log.open("w") as stream:
            result = subprocess.run(command, cwd=repo, env=env,
                                    stdout=stream, stderr=subprocess.STDOUT, check=False)
        text = log.read_text()
        passed = (result.returncode == 0 and row["test"] + " ..." in text
                  and "test result: ok. 1 passed; 0 failed" in text)
        receipt = {**row, "command": command, "exit_code": result.returncode,
                   "passed": passed, "wall_seconds": time.monotonic() - start,
                   "environment": {"MPK_CORE_MAX_SECONDS": row["evaluator_seconds"],
                                   "MPK_CORE_MAX_STEPS": None, "MPK_W09_variables": "unset"},
                   "log": log.name, "log_sha256": digest(log)}
        results.append(receipt)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        print(f"END {label} {'PASS' if passed else 'FAIL'} {receipt['wall_seconds']:.2f}s", flush=True)
        if not passed:
            print(text[-12000:], flush=True)
            raise SystemExit(result.returncode or 1)
    print(f"DONE {len(results)} passed; LOGS {output}", flush=True)


if __name__ == "__main__":
    main()
