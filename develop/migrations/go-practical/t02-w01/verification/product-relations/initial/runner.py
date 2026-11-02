"""Run affected ordinary product-relation checks with complete source snapshots."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


repo = Path.cwd()
root = Path(__file__).resolve().parent.relative_to(repo.resolve())
mode, attempt = sys.argv[1:3]
assert mode in {"construction", "tests", "clippy", "format"}
assert attempt.replace("-", "").isalnum()
out = root / attempt / mode
out.mkdir(parents=True, exist_ok=False)
sources = [Path("crates/mpk-vc/src/go_practical_ordinary.rs"),
           *sorted(Path("crates/mpk-vc/src/go_practical_ordinary").rglob("*.rs")),
           Path("crates/mpk-vc/tests/go_practical_product_relations.rs"),
           Path("crates/mpk-vc/tests/support/go_product_relation_controls.rs"),
           Path("crates/mpk-vc/tests/support/go_word_division_dependencies.rs"),
           Path("Cargo.toml"), Path("Cargo.lock"), Path("crates/mpk-vc/Cargo.toml"),
           *[Path("proofs/std") / p / ("std-" + p + ".hex") for p in ["bool", "eq", "nat"]]]
archive = root / attempt / "inputs"
runner_copy = root / attempt / "runner.py"
if not runner_copy.exists():
    runner_copy.write_bytes(Path(__file__).read_bytes())
else:
    assert runner_copy.read_bytes() == Path(__file__).read_bytes()
records = [seal(path) for path in sources]
if not archive.exists():
    for record in records:
        target = archive / record["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(Path(record["path"]).read_bytes())
else:
    for record in records:
        stored = seal(archive / record["path"])
        assert (stored["sha256"], stored["bytes"]) == (record["sha256"], record["bytes"])
overrides = {"CARGO_NET_OFFLINE": "true", "CARGO_PROFILE_TEST_OPT_LEVEL": "1",
             "CARGO_PROFILE_TEST_DEBUG_ASSERTIONS": "true", "CARGO_PROFILE_TEST_OVERFLOW_CHECKS": "true",
             "CARGO_INCREMENTAL": "0", "RUST_TEST_THREADS": "1"}
env = os.environ.copy()
env.update(overrides)
for name in ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS"]:
    env.pop(name, None)
target = "go_practical_product_relations"
if mode in {"construction", "tests"}:
    overrides["MPK_GO_PRODUCT_RELATIONS_TEST_OUTPUT"] = str((out / "certificates").resolve())
    env.update(overrides)
    command = ["cargo", "test", "--offline", "--locked", "-p", "mpk-vc", "--test", target,
               "--", "--nocapture", "--test-threads=1"]
    if mode == "construction":
        command.append("actual_")
elif mode == "clippy":
    command = ["cargo", "clippy", "--offline", "--locked", "-p", "mpk-vc", "--lib", "--test", target,
               "--", "-D", "warnings"]
else:
    command = ["rustfmt", "--check", "--edition", "2021", "--config", "skip_children=true",
               *[str(p) for p in sources if p.suffix == ".rs"]]
record = {"status": "running", "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "command": command, "environment": overrides, "sources": records, "source_snapshot": str(archive),
          "runner": seal(__file__), "CSharp_tests_executed": False,
          "selection_reason": "New typed relation-witness products, lifted equivalence/update/map laws and actual Bool comparator reflection/completeness. Complete original78 product and3489 signed-equality named identities/raw generators retained; all16 arbitrary mixed-width word pairs and arbitrary balanced-Int representative shifts. No source operand enumeration, function extensionality, new axiom/import/core rule or widened cap/fuel. Affected producer/consumer tests/lint/format/review only. Full56 source/nominal/type/operation/storage/lifetime/state/program foundation/private descriptor/registry/context and immutable original package/export/selected-closure replay/all169 application demands/T02-T11 remain due; R06 pending. No public/profile/source Go struct equality/application/Linux release claim, CSharp/agents/Actions or intermediate full gate.",
          "T02_W01_complete": False, "full_T02_gate": "deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp"}
result = out / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (out / "stdout.log").open("wb") as stdout, (out / "stderr.log").open("wb") as stderr:
    code = subprocess.run(command, cwd=repo, env=env, stdout=stdout, stderr=stderr, check=False).returncode
changed = [r for r in records if seal(r["path"]) != r]
record.update(status="passed" if code == 0 and not changed else "failed", exit_code=code,
              changed_inputs_during_execution=changed, wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(out / "stdout.log"), stderr=seal(out / "stderr.log"),
              certificates=[seal(p) for p in sorted((out / "certificates").glob("*.mpcert"))])
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "exit_code", "wall_seconds"]}))
raise SystemExit(0 if record["status"] == "passed" else 1)
