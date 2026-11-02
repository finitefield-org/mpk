"""Affected ordinary string checks; outside sandbox for original Git inputs."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

BASE = "a49b1c39da16058e7b7c131fba1547c8efd3e0f1"
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
                  Path("Cargo.toml"), Path("Cargo.lock"), Path("develop/migrations/go-practical/t02-w01/verification/array-foundation/epochs/harness-context/foundation/candidates/array-foundation.mpcert"), Path("develop/migrations/go-practical/t02-w01/verification/string-affix/epochs/checked-read/foundation/candidates/affix-foundation.mpcert"), Path("proofs/std/eq/std-eq.hex"), Path("develop/migrations/go-practical/t02-w01/verification/string-concat/epochs/harness-import/foundation/candidates/concat-foundation.mpcert"), Path("develop/migrations/go-practical/t02-w01/verification/string-slice/epochs/final-controls/selected/candidates/slice-foundation.mpcert"),
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
target = ["cargo", "test", "--release", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_array_equality"]
commands = {
    "foundation": target + ["array_equality_preserves_all2688_prior_identities", "--", "--exact", "--nocapture"],
    "alphabet": target + ["exact_ownership_is_distinct_from_core_accepted_extensions", "--", "--exact", "--nocapture"],
    "selected": target + ["--", "--nocapture", "--skip", "array_equality_preserves_all2688_prior_identities", "--skip", "observations_cover_every_frozen_shape_and_full_index_boundaries"],
    "observations": target + ["observations", "--", "--nocapture"],
    "lint": ["cargo", "clippy", "--release", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_array_equality", "--", "-D", "warnings"],
    "format": ['rustfmt', '--check', '--edition', '2021', '--config', 'skip_children=true', 'crates/mpk-vc/src/go_practical_ordinary.rs', 'crates/mpk-vc/src/go_practical_ordinary/array.rs', 'crates/mpk-vc/src/go_practical_ordinary/array/equality.rs', 'crates/mpk-vc/src/go_practical_ordinary/array/equality/reflection.rs', 'crates/mpk-vc/src/go_practical_ordinary/array/equality/preservation.rs', 'crates/mpk-vc/tests/go_practical_array_equality.rs', 'crates/mpk-vc/src/go_practical_ordinary/array/equality/negative.rs', 'crates/mpk-vc/src/go_practical_ordinary/array/equality/stability.rs', 'crates/mpk-vc/src/go_practical_ordinary/array/equality/algebra.rs'],
}
env = os.environ.copy()
env["CARGO_NET_OFFLINE"] = "true"
env["CARGO_INCREMENTAL"] = "0"
if mode in {"selected", "foundation", "alphabet", "observations"}:
    env["MPK_GO_ARRAY_EQUALITY_OUTPUT"] = str(directory / "candidates")
record = {"status": "running", "command": commands[mode], "source_inputs": inputs,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "selection_reason": "All frozen array comparison depths0..8 use actual element-comparator folds over the logical prefix. Actual validity/element reflection and completeness witnesses are mandatory; full-array relation and pointwise comparator congruence preserve actual all-bit values without function extensionality. Retain all2688 old identities and exact1033685-byte complete Array producer. Select only changed producer/consumers and exact-ownership/semantic controls; unchanged arithmetic/frontend/native/core/protocol suites retained without repeat. Complete source/driver/candidate inputs precede checks; no limits/exclusions change; cumulative T02 gate only at W04.",
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
