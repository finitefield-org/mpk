#!/usr/bin/env python3
"""Private W08 runtime differential. --update is explicit generated-record output."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import platform
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

import foundation_runtime_model as model

ROOT = Path(__file__).resolve().parents[3]
MODULE = importlib.util.spec_from_file_location("w07_probe", Path(__file__).with_name("run-primitive-string-numeric-codec-probe.py"))
assert MODULE is not None and MODULE.loader is not None
w07 = importlib.util.module_from_spec(MODULE)
MODULE.loader.exec_module(w07)
PROFILES = ("hostile-comma", "hostile-swap")
RECORD = ROOT / model.RUNTIME_PATH
FROZEN_W07 = "0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769"


def canonical(value: object) -> bytes:
    return w07.canonical(value) + b"\n"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def inputs() -> list[dict]:
    result = []
    for path in (model.SOURCE_PATH, "develop/probes/csharp-03/foundation_runtime_model.py",
                 "develop/probes/csharp-03/run-foundation-data-probe.py",
                 "develop/migrations/csharp-03/build-inputs/build-inputs.json",
                 "develop/migrations/csharp-03/build-inputs/candidate-inventory.json"):
        data = w07.read_regular(ROOT / path, 4 * 1024 * 1024)
        result.append({"path": path, "sha256": sha(data), "size_bytes": len(data)})
    return sorted(result, key=lambda row: row["path"])


def input_rows() -> list[dict]:
    return [{k: row[k] for k in ("id", "operation", "inputs")} for row in model.cases()]


def validate_runs(runs: list[dict]) -> None:
    if len(runs) != len(PROFILES):
        raise ValueError("culture count")
    cases = model.cases()
    for profile, run in zip(PROFILES, runs):
        if set(run) != {"runtime", "culture", "vectors"} or run["runtime"] != "10.0.11" or run["culture"] != profile:
            raise ValueError("runtime identity")
        expected_rows = [{"id": row["id"], "operation": row["operation"], "inputs": row["inputs"], "observed": row["expected"]} for row in cases]
        if run["vectors"] != expected_rows:
            for actual, expected in zip(run["vectors"], expected_rows):
                if actual != expected:
                    raise ValueError("independent differential: " + repr({"actual": actual, "expected": expected}))
            raise ValueError("runtime vector count/order")


def document(runs: list[dict], binary_hash: str) -> dict:
    validate_runs(runs)
    return {"schema": model.SCHEMA, "work_item": "CSHARP-03-T01-W08",
            "source_commit": "b0ff7daec663b95b1f88ecc1d98f0b7c1f6fdf00",
            "w07_runtime_sha256": FROZEN_W07, "inputs": inputs(),
            "compiler_arguments": list(w07.COMPILER_ARGUMENTS),
            "toolchain_inputs_sha256": w07.active.TOOLCHAIN_HASH,
            "reference_projection_sha256": w07.active.REFERENCE_HASH,
            "binary_sha256": binary_hash, "clean_builds": 2,
            "executions_per_build": 4, "environment_mutations": ["clean", "hostile"],
            "case_inputs_sha256": sha(canonical(input_rows())),
            "observations_sha256": sha(canonical(runs)),
            "operation_ids": sorted({r["operation"] for r in model.cases()}),
            "vector_count": len(model.cases()), "observations": runs}


def check_record() -> dict:
    w07.validate_frozen_inputs()
    if sha(w07.read_regular(w07.RESULT_PATH, w07.MAX_RESULT_BYTES)) != FROZEN_W07:
        raise ValueError("W07 record drift")
    data = w07.read_regular(RECORD, 8 * 1024 * 1024)
    record = w07.strict_json(data, 8 * 1024 * 1024)
    if data != canonical(record) or record != document(record["observations"], record["binary_sha256"]):
        raise ValueError("record reconstruction")
    w07.require_sha256(record["binary_sha256"])
    return record


def run_once(descriptor: dict, roots: dict, root: Path) -> tuple[list[dict], str]:
    root.mkdir(mode=0o700)
    sdk, runtime = roots["dotnet-sdk-linux-x64"], roots["dotnet-runtime-linux-x64"]
    binary = root / "FoundationDataProbe.dll"
    arguments = [*w07.COMPILER_ARGUMENTS, "/out:" + str(binary), "/main:FoundationDataProbe",
                 "/pathmap:" + str(ROOT) + "=/_/mpk"]
    for reference in descriptor["toolchain_inputs"]["reference_projection"]["inventory"]:
        arguments.append("/reference:" + str(roots["microsoft-netcore-app-ref"] / reference["path"]))
    arguments.append(str(ROOT / model.SOURCE_PATH))
    compiled = w07.active.execute_isolated(
        [str(sdk / "dotnet"), "exec", str(sdk / "sdk/10.0.400/Roslyn/bincore/csc.dll"), *arguments],
        cwd=root, environment=w07.active.closed_dotnet_environment(sdk, root / "compiler-environment"))
    if compiled.returncode or compiled.stdout or compiled.stderr:
        raise ValueError("compiler: " + repr(compiled.stdout + compiled.stderr))
    case_path = root / "cases.json"
    case_path.write_bytes(canonical(input_rows()))
    runs = []
    for profile in PROFILES:
        outputs = []
        for mutation in ("clean", "hostile"):
            environment = w07.active.closed_dotnet_environment(runtime, root / (profile + "-" + mutation))
            environment["MPK_CSHARP_PRACTICAL_UNLISTED_RUNTIME"] = mutation
            executed = w07.active.execute_isolated(
                [str(runtime / "dotnet"), "exec", "--runtimeconfig", str(ROOT / "csharp-tools/csharp2vir/csharp2vir.runtimeconfig.json"),
                 "--fx-version", "10.0.11", str(binary), str(case_path), profile], cwd=root, environment=environment)
            if executed.returncode or executed.stderr or not executed.stdout or len(executed.stdout) > 8 * 1024 * 1024:
                raise ValueError("runtime: " + repr(executed.stderr))
            outputs.append(executed.stdout)
        if outputs[0] != outputs[1]:
            raise ValueError("unlisted environment dependence")
        runs.append(w07.strict_json(outputs[0], 8 * 1024 * 1024))
    validate_runs(runs)
    return runs, sha(binary.read_bytes())


def run_twice() -> bytes:
    if platform.system() != "Linux" or platform.machine() not in {"x86_64", "amd64"}:
        raise ValueError("requires fixed Linux x64 runtime")
    descriptor = w07.validate_frozen_inputs()
    toolchain = descriptor["toolchain_inputs"]
    archives = w07.practical.checked_archives(toolchain)
    with tempfile.TemporaryDirectory(prefix="mpk-csharp-w08-") as temporary:
        root = Path(temporary)
        roots = w07.active.materialize_closure(toolchain, archives, root / "closure")
        first = run_once(descriptor, roots, root / "first")
        second = run_once(descriptor, roots, root / "second")
        if first != second:
            raise ValueError("nondeterministic clean build")
        return canonical(document(*first))


def main() -> None:
    if sys.argv[1:] == ["--check-record"]:
        record = check_record()
        print(f"W08 runtime record: {record['vector_count']} independent vectors, {len(record['operation_ids'])} operations")
    elif sys.argv[1:] in (["--check"], ["--update"]):
        data = run_twice()
        if sys.argv[1:] == ["--update"]:
            RECORD.parent.mkdir(parents=True, exist_ok=True)
            descriptor, path = tempfile.mkstemp(prefix=".w08-runtime-", dir=RECORD.parent)
            try:
                with os.fdopen(descriptor, "wb") as stream:
                    stream.write(data)
                    stream.flush()
                    os.fsync(stream.fileno())
                os.chmod(path, 0o644)
                os.replace(path, RECORD)
            finally:
                Path(path).unlink(missing_ok=True)
        elif data != RECORD.read_bytes():
            raise ValueError("runtime record mismatch")
        check_record()
        print("W08 two-build/eight-execution differential passed")
    else:
        raise ValueError("usage: --check-record | --check | --update")


if __name__ == "__main__":
    main()
