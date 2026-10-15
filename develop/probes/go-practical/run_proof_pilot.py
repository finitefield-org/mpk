#!/usr/bin/env python3
"""Build/check the private W03 pilot; it cannot mark W03 or a profile complete.

Every candidate is checked from the same canonical bytes by independent Rust
and Go executables. Resource failures are not semantic rejections. The report
explicitly leaves the full source bindings/arithmetic/operation scope pending.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile
import time

from run_module_probes import DEFAULT_GOROOT, HERE, ROOT, verify_host_toolchain


TARGET = Path("/private/tmp/mpk-go-practical-proof-target")
RECORD = ROOT / "develop/migrations/go-practical/t01-w03/pilot-corrected"
NATURAL_RECORD = ROOT / "develop/migrations/go-practical/t01-w03/natural-conservation"
PILOT_SCOPE = "ordinary universal representation/control/carry lemmas; no full Go function proof or public-profile acceptance"
NATURAL_SCOPE = "exact one-bit natural/step value conservation; no full signed 64-bit mathematical-sum bridge or Go function proof"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def checked(command, *, cwd=ROOT, env=None):
    result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True, timeout=240)
    if result.returncode:
        raise RuntimeError(f"build/generation failed: {command}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def sources():
    paths = set()
    for crate in ("mpk-core", "mpk-cert", "mpk-kernel", "mpk-theory"):
        paths.add(ROOT / "crates" / crate / "Cargo.toml")
        paths.update((ROOT / "crates" / crate / "src").rglob("*.rs"))
    paths.add(ROOT / "Cargo.toml")
    paths.add(ROOT / "go-tools/mpk-checker-ref/go.mod")
    paths.update(p for p in (ROOT / "go-tools/mpk-checker-ref").rglob("*.go") if not p.name.endswith("_test.go"))
    paths.update((HERE / "certificates").rglob("*.rs"))
    paths.update(HERE / "certificates" / p for p in ("Cargo.toml", "Cargo.lock"))
    paths.update([Path(__file__), HERE / "host-toolchain.json",
                  HERE / "run_module_probes.py", HERE / "provision_host_toolchain.py",
                  HERE / "check_proof_report.py", HERE / "entry_audit.py",
                  ROOT / "fixtures/core-bool-cases/right-identity.hex",
                  ROOT / "proofs/std/nat/std-nat.hex",
                  ROOT / "develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs"])
    return [{"path": p.relative_to(ROOT).as_posix(), "sha256": sha(p.read_bytes())} for p in sorted(paths)]


def timed(command):
    start = time.perf_counter()
    # wait4 reads this process's own child usage without the kern.clockrate
    # sysctl required by BSD time, which the workspace sandbox forbids.
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(list(map(str, command)), stdin=subprocess.DEVNULL,
                                   stdout=stdout, stderr=stderr)
        while True:
            pid, status, usage = os.wait4(process.pid, os.WNOHANG)
            if pid:
                process.returncode = os.waitstatus_to_exitcode(status)
                break
            if time.perf_counter() - start > 60:
                process.kill()
                _, status, _ = os.wait4(process.pid, 0)
                process.returncode = os.waitstatus_to_exitcode(status)
                raise RuntimeError(f"resource timeout is pending feasibility, not a semantic rejection: {command}")
            time.sleep(0.002)
        stdout.seek(0)
        stderr.seek(0)
        output, errors = stdout.read().decode(), stderr.read().decode()
    elapsed = time.perf_counter() - start
    return {"exit_code": process.returncode, "stdout": output, "wall_seconds": elapsed,
            "peak_rss_bytes": usage.ru_maxrss, "resource_provider": "Darwin os.wait4 child rusage",
            "cpu_user_seconds": usage.ru_utime, "cpu_system_seconds": usage.ru_stime,
            "stderr": errors}


def observe(executable, path):
    result = timed([executable, "verify", path])
    result["report"] = json.loads(result.pop("stdout"))
    return result


def semantic_expectation(item, observations, *, declarations=19):
    for name, observed in observations.items():
        report = observed["report"]
        if report.get("verdict") != item["expected"]:
            raise RuntimeError(f"{item['id']}: unexpected {name} verdict: {report}")
        if item["expected"] == "accepted":
            # The Go writer's uint64 axiom_count has omitempty; its absent
            # accepted value therefore denotes zero. Preserve raw JSON too.
            count = report.get("axiom_count", 0 if name == "go" else None)
            if observed["exit_code"] != 0 or type(count) is not int or count != 0:
                raise RuntimeError("accepted ordinary pilot is not zero-axiom")
            if report.get("module") != item["theorem"].removesuffix(".Claim") or report.get("declaration_count") != declarations:
                raise RuntimeError("accepted checker module/declaration identity differs")
            hashes = report.get("hashes", {})
            if set(hashes) != {"export", "certificate", "axiom_report"} or any(
                    not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value)
                    for value in hashes.values()):
                raise RuntimeError("accepted checker hashes are incomplete")
        elif observed["exit_code"] != 1 or report.get("error_code", report.get("error_kind")) not in {"KERNEL_CORE_CHECK", "core_check"}:
            raise RuntimeError("negative claim did not fail at core checking")
    if item["expected"] == "accepted" and observations["rust"]["report"]["hashes"] != observations["go"]["report"]["hashes"]:
        raise RuntimeError("checker hashes disagree")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--natural", action="store_true", help="measure the separate exact Nat conservation pilot")
    args = parser.parse_args()
    record = NATURAL_RECORD if args.natural else RECORD
    scope = NATURAL_SCOPE if args.natural else PILOT_SCOPE

    def generation_command(binary, output, case=None):
        command = [str(binary), "natural-generate" if args.natural else "generate", str(ROOT), str(output)]
        if not args.natural:
            command.append("3")
        if case is not None:
            command.append(case)
        return command
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise RuntimeError("pilot resource units/toolchain pin are darwin/arm64-specific")
    pin = json.loads((HERE / "host-toolchain.json").read_text())
    host_inventory = verify_host_toolchain(DEFAULT_GOROOT, pin)
    cargo = shutil.which("cargo")
    rustc = shutil.which("rustc")
    if not cargo or not rustc:
        raise RuntimeError("local Cargo/rustc unavailable")
    env = {**os.environ, "CARGO_TARGET_DIR": str(TARGET)}
    checked([cargo, "build", "--offline", "--locked", "--manifest-path", str(HERE / "certificates/Cargo.toml")], env=env)
    rust = TARGET / "debug/go-practical-proof-probe"
    source_before = sources()
    with tempfile.TemporaryDirectory(prefix="mpk-go-practical-proof-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        reference = temp / "mpk-checker-ref"
        go_env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local",
                  "GOPROXY": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0",
                  "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
        checked([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-o", str(reference), "./cmd/mpk-checker-ref"],
                cwd=ROOT / "go-tools/mpk-checker-ref", env=go_env)
        first, second = temp / "first", temp / "second"
        construction_start = time.perf_counter()
        checked(generation_command(rust, first))
        construction_seconds = time.perf_counter() - construction_start
        checked(generation_command(rust, second))
        names = sorted(p.name for p in first.iterdir())
        if names != sorted(p.name for p in second.iterdir()) or any((first / n).read_bytes() != (second / n).read_bytes() for n in names):
            raise RuntimeError("candidate generation is nondeterministic")
        index = json.loads((first / "index.json").read_text())
        rows = []
        for item in index["cases"]:
            path = first / (item["id"] + ".mpcert")
            generated = temp / "individual" / item["id"]
            construction = timed(generation_command(rust, generated, item["id"]))
            if construction["exit_code"] or (generated / path.name).read_bytes() != path.read_bytes():
                raise RuntimeError("individual candidate construction differs from the complete run")
            observations = {"rust": observe(rust, path), "go": observe(reference, path)}
            semantic_expectation(item, observations, declarations=23 if args.natural else 19)
            data = path.read_bytes()
            if args.check and (record / path.name).read_bytes() != data:
                raise RuntimeError(f"retained certificate differs: {path.name}")
            rows.append({"case": item, "raw_sha256": sha(data), "construction": construction, "observations": observations})
            print(f"{item['id']}: {item['expected']} by both; Rust {observations['rust']['wall_seconds']:.3f}s, Go {observations['go']['wall_seconds']:.3f}s", flush=True)
        if source_before != sources():
            raise RuntimeError("source changed during construction/checking")
        report = {"schema": "mpk.go_practical.proof_pilot_report.v0", "work_item": "GO-PRAC-T01-W03",
                  "status": "partial feasibility; W03 is not complete",
                  "scope": scope,
                  "pending": ["complete source/model bindings and validity invariants", "index/length/capacity and operation law closure",
                              "full signed-integer and exact mathematical-sum bridge", "profile-wide resource/boundary freeze"],
                  "inputs": source_before, "host_toolchain_inventory": host_inventory,
                  "rustc": checked([rustc, "--version"]).strip(), "cargo": checked([cargo, "--version"]).strip(),
                  "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine(),
                           "model": None, "physical_memory_bytes": None, "cpu_count": os.cpu_count(),
                           "hardware_scope": "local development host; model/RAM sysctl unavailable in workspace sandbox"},
                  "checker_binaries": {"rust_sha256": sha(rust.read_bytes()), "go_sha256": sha(reference.read_bytes())},
                  "deterministic_generations": 2, "construction_wall_seconds": construction_seconds,
                  "cases": rows}
        if args.write:
            record.mkdir(parents=True, exist_ok=True)
            for name in names:
                shutil.copyfile(first / name, record / name)
            (record / "observations.json").write_bytes(json_bytes(report))
        else:
            retained = json.loads((record / "observations.json").read_text())
            if retained["inputs"] != source_before:
                raise RuntimeError("retained pilot source identity differs")
            from check_proof_report import check
            check(retained, natural=args.natural)
            if (record / "index.json").read_bytes() != (first / "index.json").read_bytes():
                raise RuntimeError("retained theorem index differs")
        print(f"PASS: {len(rows)} candidates; each checked by Rust and Go; W03 remains in progress")


if __name__ == "__main__":
    main()
