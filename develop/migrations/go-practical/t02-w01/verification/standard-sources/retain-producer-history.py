"""Keep exact earlier producer files as inert byte records with original paths.

Current source is corrected separately. No historical source is rewritten into
a new observation. Every old preimage is recoverable by original path and hash.
"""
from pathlib import Path
import base64
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
review = root / "input-review/changed-inputs-review.json"
observation = json.loads(review.read_text())
assert observation["status"] == "failed"
paths = {r["path"] for r in observation["findings"]}
paths.remove("go-tools/go2vir/practical_stdlib_test.go")
paths.add(str(review))
out = root / "historical-producer-bytes"
out.mkdir(exist_ok=True)
assert not (out / "manifest.json").exists()
rows = []
for name in sorted(paths):
    path = Path(name)
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    target = out / (digest + ".bytes.json")
    record = {"schema": "mpk.original_bytes.v1", "historical_producer_byte_buffer": True,
              "raw_sha256": digest, "bytes": len(raw), "base64": base64.b64encode(raw).decode()}
    encoded = json.dumps(record, indent=2) + "\n"
    if target.exists():
        assert base64.b64decode(json.loads(target.read_text())["base64"], validate=True) == raw
    target.write_text(encoded)
    assert base64.b64decode(json.loads(target.read_text())["base64"], validate=True) == raw
    rows.append({"original_path": name, "raw_sha256": digest, "bytes": len(raw),
                 "retained_path": str(target), "retained_sha256": hashlib.sha256(target.read_bytes()).hexdigest()})
(out / "manifest.json").write_text(json.dumps({"status": "byte_exact_transfer", "objects": rows,
                                             "old_producer_bytes_rewritten": False,
                                             "fresh_observation_substituted": False}, indent=2) + "\n")
for name in paths:
    Path(name).unlink()
print(json.dumps({"original_producer_files_retained": len(rows), "original_bytes": sum(r["bytes"] for r in rows)}))
