"""Record the user-authorized cumulative Go development gate from committed bytes."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import time


def seal(path):
    data = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
subprocess.run(["/usr/bin/git", "diff", "--quiet", "HEAD", "--"], check=True)
commit = subprocess.check_output(["/usr/bin/git", "rev-parse", "HEAD"], text=True).strip()
tree = subprocess.check_output(["/usr/bin/git", "rev-parse", "HEAD^{tree}"], text=True).strip()
directory = root / "run"
directory.mkdir(exist_ok=False)
environment = {"CARGO_NET_OFFLINE": "true", "CARGO_PROFILE_TEST_OPT_LEVEL": "1",
               "CARGO_PROFILE_TEST_DEBUG_ASSERTIONS": "true",
               "CARGO_PROFILE_TEST_OVERFLOW_CHECKS": "true", "RUST_TEST_THREADS": "4"}
env = os.environ.copy()
env.update(environment)
unset = ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS"]
for key in unset:
    env.pop(key, None)
record = {"status": "running", "source_commit": commit, "source_tree": tree,
          "command": ["./scripts/check-fast.sh", "--skip-csharp"],
          "scope": "Cumulative T01 Go development gate; CSharp tests explicitly deferred by user",
          "environment": environment, "unset_environment": unset,
          "amendment": seal(root / "amendment.json"),
          "gate": seal("scripts/check-fast.sh"),
          "test_router": seal("scripts/cargo-test-without-csharp.py"),
          "scanner": seal("scripts/check-no-active-gir.sh"),
          "selection_reason": "Final W of T01. All remaining workspace targets and shared tests, format, strict scanner, Clippy, CLI build and two accepting/two rejecting fixtures. CSharp dedicated targets/unit/doc test names omitted only under the explicit user amendment.",
          "native_Linux_installed_release_acceptance": False,
          "started_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
started = time.monotonic()
with (directory / "stdout.log").open("wb") as out, (directory / "stderr.log").open("wb") as err:
    child = subprocess.Popen(record["command"], env=env, stdout=out, stderr=err)
    record["pid"] = child.pid
    result.write_text(json.dumps(record, indent=2) + "\n")
    code = child.wait()
record.update(status="passed" if code == 0 else "failed", exit_code=code,
              wall_seconds=time.monotonic() - started,
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              stdout=seal(directory / "stdout.log"), stderr=seal(directory / "stderr.log"))
result.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("status", "exit_code", "wall_seconds")}))
raise SystemExit(code)
