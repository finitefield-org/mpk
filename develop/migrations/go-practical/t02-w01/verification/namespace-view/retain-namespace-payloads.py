"""Expose every complete namespace/view fixture payload to the strict scanner.

Original producer records remain byte-exact. These additional lossless views
also make nested JSON byte buffers independently inspectable and reviewable.
"""
from pathlib import Path
import base64
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
directory = root / "original-namespace-payloads"
directory.mkdir(exist_ok=True)
rows = []


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def retain(path, field, encoded, obj=None):
    raw = base64.b64decode(encoded, validate=True)
    digest = hashlib.sha256(raw).hexdigest()
    record = {"schema": "mpk.original_bytes.v1", "raw_sha256": digest,
              "bytes": len(raw), "base64": base64.b64encode(raw).decode()}
    if obj is not None:
        assert obj["bytes"] == len(raw) and obj["raw_sha256"] == digest
        canonical = json.dumps({k: v for k, v in obj.items() if k != "id"},
                               sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
        assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + canonical).hexdigest() == obj["id"]
        record["source_id"] = obj["id"]
    key = hashlib.sha256((str(path) + "\0" + field).encode()).hexdigest()
    target = directory / (key + ".bytes.json")
    text = json.dumps(record, indent=2) + "\n"
    if target.exists():
        assert target.read_text() == text
    else:
        target.write_text(text)
    assert base64.b64decode(json.loads(target.read_text())["base64"], validate=True) == raw
    rows.append({"producer_record": seal(path), "field": field, "retained": seal(target),
                 "source_object": obj, "bytes": len(raw), "raw_sha256": digest})


for phase in ["original-contradiction/regression", "reconciled/regression", "owned-views/selected", "special-permission/permission", "sealed-modes/selected",
              "linux-special-permission/before", "linux-special-permission/after", "final-format/permission"]:
    for path in sorted((root / phase).glob("*.json")):
        if "namespace" not in path.name and "view" not in path.name:
            continue
        record = json.loads(path.read_text())
        objects = {obj["id"]: obj for obj in record.get("objects", [])}
        for source_id, encoded in sorted(record.get("raw", {}).items()):
            retain(path, "raw/" + source_id, encoded, objects[source_id])
        for field in ["policy_bytes", "later_bytes", "original_bytes", "original", "changed"]:
            if field not in record:
                continue
            matches = [obj for obj in objects.values()
                       if obj["path"] == {"policy_bytes": "policy.go", "later_bytes": "later.go"}.get(field)]
            assert len(matches) <= 1
            retain(path, field, record[field], matches[0] if matches else None)
receipt = {"status": "byte_exact_transfer", "objects": rows, "payloads": len(rows),
           "original_records_rewritten": False, "source_bytes_normalized": False,
           "decoded_payload_exclusions": False}
(root / "original-namespace-payloads.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps({"status": receipt["status"], "complete_payloads": len(rows)}))
