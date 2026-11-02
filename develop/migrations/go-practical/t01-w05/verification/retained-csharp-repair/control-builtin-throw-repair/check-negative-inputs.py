"""Exercise unchanged historical assertions with isolated forged input records."""

from pathlib import Path
import datetime
import hashlib
import json
import os
import re
import subprocess
import tempfile
import time


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


directory = Path(__file__).parent / "mutation-controls"
directory.mkdir(exist_ok=False)
build_path = Path(__file__).parent / "build/build-result.json"
build = json.loads(build_path.read_text())
assert build["status"] == "passed"
binary = Path(build["executable"]["path"])
assert seal(binary) == build["executable"]
source = Path("crates/mpk-vc/tests/support/csharp_practical_ordinary_control_edge_tests.rs")
text = source.read_text()
start = text.index("fn preserves_existing_declarations(")
end = text.index("\nfn reference(", start)
first_line, last_line = [text[:index].count("\n") + 1 for index in [start, end]]
baseline = Path("develop/migrations/go-practical/t01-w05/retained-csharp/control-preservation-epochs/after-complete-source-executions")
pre_steps = Path("develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-edges/before-source-steps/control-edges")
metadata_source = baseline / "index_update.json"
base = json.loads(metadata_source.read_text())
prefix = "ordinary_carriers::public_domain_tests::contract_expression_tests::control_edge_tests::"
tests = {
    "builtin": prefix + "csharp_03_t06_w09_control_builtin_throw_preserves_existing_declarations",
    "steps": prefix + "steps::csharp_03_t06_w09_control_source_steps_preserve_existing_declarations",
}
environment = os.environ.copy()
for name in ["RUST_MIN_STACK", "MPK_CORE_MAX_STEPS", "MPK_CORE_MAX_SECONDS", "MPK_W09_CONTROL_EDGES_OUT"]:
    environment.pop(name, None)
record = {
    "status": "running", "started_at_utc": stamp(), "binary": seal(binary),
    "build": seal(build_path), "test_source": seal(source),
    "scope": "Five isolated input mutations must reach and fail an original declaration/metadata assertion inside the unchanged preservation body. Epoch-pin, setup, decoding and runtime failures do not satisfy the control. Positive thirteen-owner and current producer runs are separate.",
    "original_preservation_body_line_bounds": [first_line, last_line],
    "observations": [], "whole_gate_passed": False, "application_proofs_discharged": False,
}
result = directory / "result.json"
result.write_text(json.dumps(record, indent=2) + "\n")
for kind in ["context_hash", "source_function", "public_projection", "alias_source_anchor", "cross_context_certificate"]:
    case_dir = directory / kind
    case_dir.mkdir()
    candidate = Path(tempfile.mkdtemp(prefix="mpk-t01-w05-control-" + kind + "-", dir="/private/tmp"))
    links = []
    for path in sorted(baseline.iterdir()):
        (candidate / path.name).symlink_to(path.resolve())
        links.append(seal(path))
    assert len(links) == 36
    changed = json.loads(metadata_source.read_text())
    selected = tests["builtin"]
    if kind == "context_hash":
        changed["source_ir_sha256"] = "0" * 64
        changed_path = "/source_ir_sha256"
    elif kind == "source_function":
        changed["functions"][0]["source"]["function_id"] = "altered-source-function"
        changed_path = "/functions/0/source/function_id"
    elif kind == "public_projection":
        binding = changed["functions"][0]["memory_bindings"][0]
        assert isinstance(binding["public_projection_definedness"], str)
        binding["public_projection_definedness"] = "altered-public-projection"
        changed_path = "/functions/0/memory_bindings/0/public_projection_definedness"
    elif kind == "alias_source_anchor":
        selected = tests["steps"]
        before = json.loads((pre_steps / "index_update.json").read_text())
        pending = [frame for frame in before["functions"][0]["source_frames"] if frame["pending_reason"] == "memory_alias_frames_pending"]
        assert len(pending) == 1
        frame_id = pending[0]["source_node_id"]
        position = next(index for index, frame in enumerate(changed["functions"][0]["source_frames"]) if frame["source_node_id"] == frame_id)
        changed["functions"][0]["source_frames"][position]["source_node_id"] = "altered-alias-source-anchor"
        changed_path = "/functions/0/source_frames/" + str(position) + "/source_node_id"
    if kind == "cross_context_certificate":
        altered = candidate / "index_update.hex"
        altered.unlink()
        altered.write_bytes((baseline / "for.hex").read_bytes())
        changed_path = "index_update.hex <- complete same-epoch for.hex (already canonically decoded by positive owners)"
    else:
        altered = candidate / "index_update.json"
        altered.unlink()
        altered.write_text(json.dumps(changed, separators=(",", ":")))
    retained = case_dir / ("mutated-input" + altered.suffix)
    retained.write_bytes(altered.read_bytes())
    command = [str(binary), "--exact", selected, "--nocapture"]
    current_env = environment.copy()
    current_env["MPK_W09_CONTROL_EDGES_OUT"] = str(candidate)
    stdout, stderr = case_dir / "stdout.log", case_dir / "stderr.log"
    start = time.monotonic()
    with stdout.open("wb") as out, stderr.open("wb") as err:
        completed = subprocess.run(command, env=current_env, stdout=out, stderr=err)
    error = stderr.read_text()
    panic = re.search(r"panicked at crates/mpk-vc/tests/support/csharp_practical_ordinary_control_edge_tests.rs:(\d+):\d+:", error)
    rejected = (completed.returncode == 101 and "test result: FAILED. 0 passed; 1 failed;" in stdout.read_text() and panic is not None and first_line <= int(panic[1]) < last_line and "assertion" in error and "failed" in error and "has overflowed its stack" not in error and "fatal runtime error" not in error)
    record["observations"].append({
        "kind": kind, "changed_path": changed_path, "candidate_directory": str(candidate),
        "unchanged_candidate_members": links, "retained_mutated_input": seal(retained),
        "command": command, "environment": {"MPK_W09_CONTROL_EDGES_OUT": str(candidate)},
        "exit_code": completed.returncode, "wall_seconds": time.monotonic() - start,
        "stdout": seal(stdout), "stderr": seal(stderr),
        "original_assertion_line": int(panic[1]) if panic else None,
        "rejected_by_original_assertion": rejected,
    })
    result.write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({"kind": kind, "rejected_by_original_assertion": rejected, "seconds": time.monotonic() - start}), flush=True)
    if not rejected:
        record["status"] = "failed"
        result.write_text(json.dumps(record, indent=2) + "\n")
        raise SystemExit(1)
record["status"] = "passed"
record["finished_at_utc"] = stamp()
result.write_text(json.dumps(record, indent=2) + "\n")
