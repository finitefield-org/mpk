"""Run only the changed certificate checks deferred by the 55-second agent limit."""
import argparse
import hashlib
import json
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
    if not cases:
        raise SystemExit("No pending checks")
    for row in cases:
        path = repo / row["certificate_path"]
        if digest(path) != row["certificate_file_sha256"]:
            raise SystemExit(f"Certificate changed since handoff: {path}")
    for path, expected in manifest["checker_source_sha256"].items():
        if digest(repo / path) != expected:
            raise SystemExit(f"Checker source changed since handoff: {path}")
    if args.list:
        for row in cases:
            print(row["family"], row["file"], row["test_filter"])
        return
    if args.output:
        args.output.mkdir(parents=True, exist_ok=False)
        output = args.output.resolve()
    else:
        output = Path(tempfile.mkdtemp(prefix="mpk-w09-domain-refresh-long-", dir="/tmp"))
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
    print(f"DONE {len(results)} passed; LOGS {output}", flush=True)


if __name__ == "__main__":
    main()
