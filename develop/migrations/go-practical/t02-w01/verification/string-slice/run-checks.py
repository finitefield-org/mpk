"""Affected ordinary string checks; outside sandbox for original Git inputs."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

BASE = "20d4afb7c752a8adcb1393876e4e6a32df2fe965"
OUT = Path(__file__).resolve().parent
ROOT = Path.cwd()
epoch, mode = sys.argv[1:]
assert epoch.replace("-", "").isalnum()
assert mode in {"foundation", "alphabet", "selected", "observations", "lint", "format"}
git = lambda *args: subprocess.check_output(["/usr/bin/git", *args])
assert git("rev-parse", "HEAD").decode().strip() == BASE
original = {}
for row in git("ls-files", "--stage", "-z").split(b"\0"):
    if row:
        metadata, name = row.split(b"\t", 1)
        _, oid, stage = metadata.decode().split()
        assert stage == "0"
        original[name.decode()] = oid
sources = sorted({*Path("crates").rglob("*.rs"), *Path("crates").rglob("Cargo.toml"),
                  Path("Cargo.toml"), Path("Cargo.lock"), Path("proofs/std/eq/std-eq.hex"),
                  Path("proofs/std/bool/std-bool.hex"), Path("proofs/std/nat/std-nat.hex"),
                  Path("develop/migrations/go-practical/t02-w01/verification/word-index-foundation/epochs/explicit-controls/selected/candidates/word-index-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/string-foundation/epochs/native-controls/selected/candidates/string-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/string-validity/epochs/native-controls/selected/candidates/validity-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/string-normalization/epochs/typed-controls/selected/candidates/normalization-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/string-equality/epochs/semantic-controls/selected/candidates/equality-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/word-order/corrected/rust-tests/certificates/word-order-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/string-order/epochs/semantic-controls/selected/candidates/order-foundation.mpcert"),
                  Path("develop/migrations/go-practical/t02-w01/verification/string-order-bounded/epochs/direction-corrected/selected/candidates/bounded-order-foundation.mpcert")})
archive = OUT / "epochs" / epoch
archive.mkdir(parents=True, exist_ok=True)
driver = archive / "run-checks.py"
if driver.exists():
    assert driver.read_bytes() == Path(__file__).read_bytes()
else:
    driver.write_bytes(Path(__file__).read_bytes())
inputs = []
for path in sources:
    raw = path.read_bytes()
    row = {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}
    oid = original.get(str(path))
    if oid and hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest() == oid:
        row.update(original_commit=BASE, original_git_blob=oid)
    else:
        relative = Path("external") / (hashlib.sha256(str(path).encode()).hexdigest()+path.suffix) if path.is_absolute() else path
        target = archive / "inputs" / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.read_bytes() == raw
        else:
            target.write_bytes(raw)
        row["complete_changed_input"] = str(target.relative_to(ROOT))
    inputs.append(row)
receipt = {"basis_commit": BASE, "sources": inputs, "runner_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
manifest = archive / "inputs.json"
if manifest.exists():
    assert json.loads(manifest.read_bytes()) == receipt
else:
    manifest.write_text(json.dumps(receipt, indent=2) + "\n")
directory = archive / mode
directory.mkdir(exist_ok=False)
target = ["cargo", "test", "--release", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_string_slice"]
commands = {
    "foundation": target + ["slice_preserves_all1729_prior_identities", "--", "--exact", "--nocapture"],
    "alphabet": target + ["exact_ownership_is_distinct_from_core_accepted_extensions", "--", "--exact", "--nocapture"],
    "selected": target + ["--", "--nocapture"],
    "observations": target + ["observations", "--", "--nocapture"],
    "lint": ["cargo", "clippy", "--release", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_string_slice", "--", "-D", "warnings"],
    "format": ["rustfmt", "--check", "--edition", "2021", "--config", "skip_children=true", "crates/mpk-vc/src/go_practical_ordinary.rs", "crates/mpk-vc/src/go_practical_ordinary/string.rs", "crates/mpk-vc/src/go_practical_ordinary/composition.rs", "crates/mpk-vc/tests/go_practical_string_slice.rs", "crates/mpk-vc/src/go_practical_ordinary/string/slice.rs", "crates/mpk-vc/src/go_practical_ordinary/string/slice/guards.rs", "crates/mpk-vc/src/go_practical_ordinary/string/slice/carrier.rs", "crates/mpk-vc/src/go_practical_ordinary/string/slice/signed.rs", "crates/mpk-vc/src/go_practical_ordinary/string/slice/preservation.rs", "crates/mpk-vc/tests/support/go_string_slice_observations.rs"],
}
env = os.environ.copy()
env["CARGO_NET_OFFLINE"] = "true"
env["CARGO_INCREMENTAL"] = "0"
if mode in {"selected", "foundation", "alphabet", "observations"}:
    env["MPK_GO_STRING_SLICE_OUTPUT"] = str(directory / "candidates")
record = {"status": "running", "command": commands[mode], "source_inputs": inputs,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "selection_reason": "New complete frozen substring carrier uses actual64-bit arithmetic and signed guards, proving length/content/padding/normalization and real operand bound contracts. Select new producer and affected consumers, all1729 original identities/full old producer bytes, exact ownership and actual full candidate controls. No cap/core/observer exception; unchanged Go/frontend/native/byte/arithmetic and unrelated suites are retained without repeat. Complete source epochs/driver/candidate bytes are saved before verification; full T02 gate stays W04.",
          "native_release_acceptance": False, "R01_complete": False, "R06_complete": False, "W01_complete": False,
          "full_T_gate": "deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh"}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
start = time.monotonic()
with (directory / "stdout.log").open("wb") as stdout, (directory / "stderr.log").open("wb") as stderr:
    code = subprocess.run(commands[mode], env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status="passed" if code == 0 else "failed", exit_code=code, seconds=time.monotonic() - start,
              whole_source_inputs_unchanged=all(hashlib.sha256(Path(r["path"]).read_bytes()).hexdigest() == r["sha256"] for r in inputs),
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: v for k, v in record.items() if k != "source_inputs"}))
raise SystemExit(code)
