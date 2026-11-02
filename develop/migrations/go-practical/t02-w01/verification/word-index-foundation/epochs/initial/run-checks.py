"""Scoped ordinary word_index checks; outside sandbox for original Git inputs."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

BASE = "a19a97c4d601460b3ac73062b429bd196b9b9333"
OUT = Path(__file__).resolve().parent
ROOT = Path.cwd()
epoch, mode = sys.argv[1:]
assert epoch.replace("-", "").isalnum()
assert mode in {"selected", "diagnostic", "typed", "lint", "format"}
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
                  Path("proofs/std/bool/std-bool.hex"), Path("proofs/std/nat/std-nat.hex"), Path("develop/migrations/go-practical/t02-w01/verification/word-shift-count-bounds/final/rust-tests/certificates/word-shift-counts.mpcert")})
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
        target = archive / "inputs" / path
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
commands = {
    "diagnostic": ["cargo", "test", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_word_index_foundation", "complete_full64_index_component_preserves_original_inputs", "--", "--exact", "--nocapture"],
    "selected": ["cargo", "test", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_word_index_foundation", "--", "--nocapture"],
    "typed": ["cargo", "test", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_word_index_foundation", "word_index_laws_reject_forged_claims_erased_bounds_and_wrong_operations", "--", "--exact", "--nocapture"],
    "lint": ["cargo", "clippy", "--offline", "--locked", "-p", "mpk-vc", "--test", "go_practical_word_index_foundation", "--", "-D", "warnings"],
    "format": ["rustfmt", "--check", "--edition", "2021", "--config", "skip_children=true", "crates/mpk-vc/src/go_practical_ordinary.rs", "crates/mpk-vc/src/go_practical_ordinary/composition.rs", "crates/mpk-vc/src/go_practical_ordinary/word_numeric.rs", "crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts.rs", "crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/counts.rs", "crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/count_bounds.rs", "crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/sequence_index.rs", "crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/sequence_index/natural.rs", "crates/mpk-vc/tests/go_practical_word_index_foundation.rs"],
}
env = os.environ.copy()
env["CARGO_NET_OFFLINE"] = "true"
env["CARGO_INCREMENTAL"] = "0"
if mode in {"selected", "diagnostic", "typed"}:
    env["MPK_GO_WORD_INDEX_FOUNDATION_OUTPUT"] = str(directory / "candidates")
record = {"status": "running", "command": commands[mode], "source_inputs": inputs,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "selection_reason": "Bind all64 word bits to actual full Nat bounds and finite-data observations for every depth0..8. Preserve all413 finite-data declarations and every retained complete original count dependency; verify the original full shift certificate remains byte-exact after factoring shared generation helpers. Exercise the affected producer/consumers and original whole semantic controls. Lint the affected producer/consumer target and format only changed Rust files. Core/ref checker implementations, frozen Go carriers and Go frontend/source capture are unchanged; retain their passing checks without rerunning unrelated suites.",
          "native_release_acceptance": False, "W01_complete": False,
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
