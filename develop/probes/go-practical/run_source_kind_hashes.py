"""Observe the amended source identity in independent existing hash engines."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from source_kind_freeze import build as source_kind, RECORD, ROOT, DOMAIN
from proof_canonical import canonical
from release_native_build import build
from run_proof_pilot import HERE, DEFAULT_GOROOT, verify_host_toolchain


def main():
    catalog = source_kind(); node = catalog["vectors"][0]["node"]
    raw = canonical({k: v for k, v in node.items() if k != "id"})
    payload = canonical([{"mode": "hash", "id": "final-go-file-source-object", "chunks": [raw.hex()], "domain": DOMAIN}])
    command, audit = build(); outputs = {}; runs = []
    def execute(name, command, env):
        observed = []
        for index in range(2):
            r = subprocess.run(command, input=payload, cwd=ROOT, env=env, capture_output=True, timeout=60)
            assert r.returncode == 0 and not r.stderr, r.stderr.decode(errors="replace")
            value = json.loads(r.stdout); assert value[0]["accepted"] is True and value[0]["digest"] == node["id"]
            assert value[0]["canonical_sha256"] == hashlib.sha256(raw).hexdigest()
            observed.append(value); runs.append({"engine": name, "run": index + 1, "exit_code": 0, "stdout_sha256": hashlib.sha256(r.stdout).hexdigest()})
        assert observed[0] == observed[1]; outputs[name] = observed[0]
    execute("rust", command, os.environ.copy())
    host = verify_host_toolchain(DEFAULT_GOROOT, json.loads((HERE / "host-toolchain.json").read_text()))
    env = {**os.environ, "GOENV": "off", "GOWORK": "off", "GOTOOLCHAIN": "local", "GOPROXY": "off", "GOTELEMETRY": "off", "CGO_ENABLED": "0", "GOFLAGS": "", "GOEXPERIMENT": "", "GOCACHE": "/private/tmp/mpk-go-practical-checker-cache"}
    with tempfile.TemporaryDirectory(prefix="mpk-source-kind-", dir="/private/tmp") as td:
        binary = Path(td) / "hashes"
        compiled = subprocess.run([str(DEFAULT_GOROOT / "bin/go"), "build", "-trimpath", "-buildvcs=false", "-o", str(binary), str(HERE / "release_hashes.go")], cwd=ROOT, env=env, capture_output=True, timeout=300)
        assert compiled.returncode == 0 and not compiled.stderr, compiled.stderr.decode(errors="replace")
        execute("go", [str(binary)], env)
    assert outputs["rust"] == outputs["go"]
    report = {"schema": "mpk.go_practical.final_source_kind_hash_observation.v0", "domain": DOMAIN,
              "canonical_preimage_hex": raw.hex(), "expected_source_id": node["id"], "outputs": outputs,
              "runs": runs, "native_observer_build": audit, "host_toolchain": host,
              "scope": "independent metadata canonical/domain hashes, not source admission, Go program proof or native installed-release acceptance",
              "source_proofs_checked": False, "profile_activated": False}
    (RECORD / "hash-observations.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print("PASS: final source-object identity matches both independent engines in two runs each; preliminary graph remains historical")


if __name__ == "__main__": main()
