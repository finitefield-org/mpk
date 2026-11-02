#!/usr/bin/env python3
"""Build/replay private Go evaluation observations under the retained toolchain."""
import argparse
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

from check_evaluation_report import check, json_bytes, sha, sources, strict_json
from evaluation_cases import HERE, PENDING, RECORD, SCOPE, VECTORS, inputs, vectors
from run_module_probes import DEFAULT_GOROOT, verify_host_toolchain


def run(argv, env, stdin=None):
    r = subprocess.run(list(map(str, argv)), input=stdin, capture_output=True, env=env, timeout=60)
    if r.returncode or r.stderr:
        raise RuntimeError(f"evaluation observer failed ({r.returncode}): {r.stderr.decode()}")
    return r.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    g = parser.add_mutually_exclusive_group(required=True)
    g.add_argument("--write", action="store_true")
    g.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ("Darwin", "arm64"):
        raise RuntimeError("retained observer requires Darwin arm64; no host fallback")
    if args.write:
        VECTORS.write_bytes(json_bytes(vectors()))
    before = sources()
    inventory = verify_host_toolchain(DEFAULT_GOROOT, strict_json((HERE/"host-toolchain.json").read_bytes()))
    env = {"PATH": str(DEFAULT_GOROOT/"bin")+":/usr/bin:/bin", "GOROOT": str(DEFAULT_GOROOT),
           "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off",
           "GOSUMDB": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOOS": "darwin",
           "GOARCH": "arm64", "GOFLAGS": "", "GOEXPERIMENT": "",
           "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
    with tempfile.TemporaryDirectory(prefix="mpk-w04-eval-", dir="/private/tmp") as temporary:
        temp = Path(temporary)
        exe = temp/"observer"
        run([DEFAULT_GOROOT/"bin/go", "build", "-trimpath", "-buildvcs=false", "-o", exe,
             HERE/"evaluation_reference.go"], env)
        binary = sha(exe.read_bytes())
        raw = json_bytes(inputs())
        output = [run([exe], env, raw) for _ in range(2)]
        if output[0] != output[1]:
            raise RuntimeError("evaluation native output is not deterministic")
        artifacts = {"native-input.json": raw, "native-output.json": output[0]}
        report = {
            "schema": "mpk.go_practical.w04_evaluation_observations.v0", "scope": SCOPE,
            "status": "native facts retained; W04 in progress", "pending": PENDING, "inputs": before,
            "toolchain_inventory": inventory, "observer_binary_sha256": binary,
            "build": {"release": "go1.25.0", "host": "darwin/arm64", "trimpath": True, "buildvcs": False,
                      "specification_sha256": "42a4ff1c01efe8dc23eca4378b8fb03d76e9f2f079dab01298ffd74d8b0e3149",
                      "runtime_scope": "host facts, not target release or verified application"},
            "artifacts": {n: {"bytes": len(b), "sha256": sha(b)} for n, b in artifacts.items()},
            "cases": len(inputs()), "source_forms": len({r["id"] for r in inputs()}),
        }
        if before != sources() or binary != sha(exe.read_bytes()):
            raise RuntimeError("evaluation source/binary changed during measurement")
        for name, raw in artifacts.items():
            (temp/name).write_bytes(raw)
        check(report, temp)
        if args.write:
            RECORD.mkdir(parents=True, exist_ok=True)
            for name in artifacts:
                shutil.copyfile(temp/name, RECORD/name)
            (RECORD/"observations.json").write_bytes(json_bytes(report))
        else:
            retained = strict_json((RECORD/"observations.json").read_bytes())
            check(retained)
            if retained != report or any((RECORD/n).read_bytes() != b for n, b in artifacts.items()):
                raise RuntimeError("evaluation fresh-build replay differs")
    print(f"PASS: {report['cases']} native cases/{report['source_forms']} source forms; two identical executions; no profile admission or theorem")


if __name__ == "__main__":
    main()
