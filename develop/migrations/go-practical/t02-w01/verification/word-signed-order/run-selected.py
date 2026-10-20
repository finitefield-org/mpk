"""Scoped original-input ordinary signed comparison checks."""
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
assert mode in {"construction", "tests", "diagnostic", "diagnostic-list", "clippy", "format"}
assert attempt.replace("-", "").isalnum()
out = root / attempt / mode
out.mkdir(parents=True, exist_ok=False)
sources = [Path("crates/mpk-vc/src/go_practical_ordinary.rs"),
           *sorted(Path("crates/mpk-vc/src/go_practical_ordinary").rglob("*.rs")),
           *sorted(Path("crates/mpk-vc/tests").glob("go_practical_word_signed_order*.rs")),
           *sorted(Path("crates/mpk-vc/tests/support").glob("go_word_signed_order*.rs")),
           Path("crates/mpk-vc/tests/support/go_word_division_dependencies.rs"),
           Path("develop/migrations/go-practical/t01-w04/word-foundation/queries.json"),
           *[Path("proofs/std") / p / ("std-" + p + ".hex") for p in ["bool", "eq", "nat"]]]
archive = root / attempt / "inputs"
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
target = "go_practical_word_signed_order"
if mode in {"construction", "tests", "diagnostic", "diagnostic-list"}:
    overrides["MPK_GO_WORD_SIGNED_ORDER_TEST_OUTPUT"] = str((out / "certificates").resolve())
    env.update(overrides)
    if mode in {"diagnostic", "diagnostic-list"}:
        target += "_diagnostic"
        overrides["MPK_GO_WORD_SIGNED_ORDER_DIAGNOSTIC_OUTPUT"] = str((out / "diagnostic.json").resolve())
        env.update(overrides)
    command = ["cargo", "test", "--offline", "--locked", "-p", "mpk-vc", "--test", target,
               "--", "--nocapture", "--test-threads=1"]
    if mode == "construction":
        command.append("actual_signed_order_component_and_complete_old_identities")
    elif mode in {"diagnostic", "diagnostic-list"}:
        command += ["--ignored"] + (["--list"] if mode == "diagnostic-list" else [])
elif mode == "clippy":
    command = ["cargo", "clippy", "--offline", "--locked", "-p", "mpk-vc", "--lib", "--test", target, "--test", target + "_diagnostic", "--", "-D", "warnings"]
else:
    selected = [str(p) for p in sources if p.suffix == ".rs"]
    command = ["rustfmt", "--check", "--edition", "2021", "--config", "skip_children=true", *selected]
record = {"status": "running", "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "command": command, "environment": overrides, "sources": records, "source_snapshot": str(archive),
          "runner": seal(__file__), "CSharp_tests_executed": False,
          "selection_reason": "Actual all8/16/32/64 signed Less/LessEqual circuits agree with complete balanced unbounded Int comparisons using real top bits/value bounds/projection laws. Retain all3437 prior order declaration identities; reuse genuine unbounded RightStrictGap with its full proof closure. Both cores still inspect every proof in a closed zero-axiom certificate. Later negative controls change real source comparisons/sign bits, mathematical claims and bound premises. Scope tests/lint/format/review to changed producer/consumer; reuse unchanged word/native observations without rerunning unaffected suites. No imported assumption, core/axiom/fuel/cap change, source/program/context/profile authority or CSharp tests. Full56-family foundation,169 original application demands and allT02-T11 remain open.",
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
