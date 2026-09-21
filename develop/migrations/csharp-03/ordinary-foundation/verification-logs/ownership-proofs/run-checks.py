"""Run selected ownership proof checks locally, retaining exact input identities."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cases", nargs="*", help="Certificate stems, or mutations; default: all")
    parser.add_argument("--family", choices=["ownership-proofs", "construction-data"], default="ownership-proofs")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--timeout-seconds", type=int, default=1800)
    args = parser.parse_args()
    if args.timeout_seconds <= 0:
        parser.error("timeout must be positive")
    here = Path(__file__).resolve().parent
    repo = here.parents[5]
    fixtures = here.parents[1] / args.family
    available = {p.stem: p for p in fixtures.glob("*.hex")}
    mutation_cases = ["mutations"] if args.family == "ownership-proofs" else []
    selected = args.cases or [*sorted(available), *mutation_cases]
    if len(set(selected)) != len(selected) or any(c not in available and c not in mutation_cases for c in selected):
        parser.error("unknown or repeated case")
    output = args.output or Path(tempfile.mkdtemp(prefix="mpk-ownership-checks-", dir="/tmp"))
    if args.output:
        output.mkdir(parents=True, exist_ok=False)
    output = output.resolve()
    print(f"LOGS {output}", flush=True)
    checker_dir = repo / "go-tools/mpk-checker-ref"
    binary = output / "checker-tests"
    build = ["go", "test", "-tags", "checkeragreement", "-c", "-o", str(binary)]
    with (output / "build.log").open("w") as stream:
        subprocess.run(build, cwd=checker_dir, stdout=stream, stderr=subprocess.STDOUT, check=True)
    sources = [*checker_dir.glob("*.go"), checker_dir / "go.mod", repo / "Cargo.toml", repo / "Cargo.lock"]
    for crate in ["mpk-core", "mpk-cert", "mpk-kernel", "mpk-theory", "mpk-cli"]:
        sources.extend((repo / "crates" / crate / "src").rglob("*.rs"))
        sources.append(repo / "crates" / crate / "Cargo.toml")
    source_hashes = {str(p.relative_to(repo)): digest(p) for p in sorted(sources)}
    (output / "checker-sources.json").write_text(json.dumps(source_hashes, indent=2) + "\n")
    results = []
    for case in selected:
        paths = sorted((fixtures / "mutations").glob("*.hex")) if case == "mutations" else [available[case]]
        before = {str(p.relative_to(repo)): digest(p) for p in paths}
        if not before:
            raise SystemExit("empty certificate selection")
        test = "TestCheckerAgreementWithRustCLIOwnershipProofs" if args.family == "ownership-proofs" else "TestCheckerAgreementWithRustCLIConstructionData"
        pattern = "^TestCheckerAgreementWithRustCLIOwnershipProofMutations$" if case == "mutations" else (
            "^" + test + "$/^" + re.escape(case + ".hex") + "$")
        command = [str(binary), "-test.run", pattern, "-test.v", "-test.timeout", f"{args.timeout_seconds}s"]
        log = output / (case + ".log")
        print(f"START {case}", flush=True)
        start = time.monotonic()
        timed_out = False
        with log.open("w") as stream:
            process = subprocess.Popen(command, cwd=checker_dir, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                code = process.wait(timeout=args.timeout_seconds + 30)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(process.pid, signal.SIGKILL)
                code = process.wait()
            finally:
                # A Go test timeout can otherwise leave its Rust child alive.
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        text = log.read_text()
        timed_out |= "panic: test timed out" in text
        unchanged = all(digest(repo / p) == h for p, h in before.items())
        sources_unchanged = all(digest(repo / p) == h for p, h in source_hashes.items())
        subtests_passed = all("/" + p.name + " (" in text for p in paths)
        passed = code == 0 and not timed_out and unchanged and sources_unchanged and subtests_passed and "\nPASS\n" in text
        receipt = {"family": args.family, "case": case, "command": command, "certificate_file_sha256": before,
                   "checker_sources": "checker-sources.json", "checker_sources_sha256": digest(output / "checker-sources.json"),
                   "exit_code": code, "timed_out": timed_out, "inputs_unchanged": unchanged,
                   "checker_sources_unchanged": sources_unchanged, "passed": passed,
                   "wall_seconds": round(time.monotonic() - start, 3), "log": log.name, "log_sha256": digest(log)}
        results.append(receipt)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        print(f"END {case} {'PASS' if passed else 'FAIL'} {receipt['wall_seconds']:.3f}s", flush=True)
        if not passed:
            print(text[-8000:], flush=True)
            raise SystemExit(1)


if __name__ == "__main__":
    main()
