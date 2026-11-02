"""Retain complete nested fixture and native-output payloads without rewriting.

Mutated negative-control buffers remain actual mutated bytes. A source object
is attached only when its canonical preimage really matches that payload.
"""
from pathlib import Path
import base64
import hashlib
import json

root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
destination = root / "fixture-payloads"
destination.mkdir(exist_ok=True)
rows = []
phases = [str(path.parent.relative_to(root)) for path in sorted(root.glob("*/*/result.json"))]



def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


def emit(path, field, encoded, obj=None):
    parts = field if isinstance(field, list) else field.split("/")
    raw = base64.b64decode(encoded, validate=True)
    digest = hashlib.sha256(raw).hexdigest()
    canonical = obj is not None and obj["bytes"] == len(raw) and obj["raw_sha256"] == digest
    record = {"schema": "mpk.original_bytes.v1", "raw_sha256": digest,
              "bytes": len(raw), "base64": base64.b64encode(raw).decode()}
    if canonical:
        narrowed = json.dumps({k: v for k, v in obj.items() if k != "id"}, sort_keys=True,
                              ensure_ascii=False, separators=(",", ":")).encode()
        assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + narrowed).hexdigest() == obj["id"]
        record["source_id"] = obj["id"]
    key = hashlib.sha256((str(path) + "\0" + json.dumps(parts, separators=(",", ":"))).encode()).hexdigest()
    target = destination / (key + ".bytes.json")
    text = json.dumps(record, indent=2) + "\n"
    if target.exists():
        assert target.read_text() == text
    else:
        target.write_text(text)
    assert base64.b64decode(json.loads(target.read_text())["base64"], validate=True) == raw
    rows.append({"producer_record": seal(path), "field_path": parts, "retained": seal(target),
                 "claimed_source_object": obj, "canonical_source_preimage": canonical,
                 "raw_sha256": digest, "bytes": len(raw)})


for phase in phases:
    for path in sorted((root / phase).glob("*.json")):
        if path.name in {"result.json", "original-byte-retention.json"}:
            continue
        record = json.loads(path.read_text())
        objects = {obj["id"]: obj for obj in record.get("objects", [])}
        for source_id, encoded in sorted(record.get("raw", {}).items()):
            emit(path, "raw/" + source_id, encoded, objects[source_id])
        for field in ["policy_bytes", "later_bytes", "original_bytes", "original", "changed",
                      "stdout", "stderr", "module_output", "original_output", "candidate_output",
                      "original_raw", "returned_raw", "package_output", "diagnostics",
                      "module_diagnostics", "package_diagnostics", "original_module_output",
                      "original_package_output"]:
            encoded = record.get(field)
            if isinstance(encoded, str):
                obj = record.get("original_object") if field in {"original_raw", "returned_raw"} else None
                emit(path, field, encoded, obj)

        def original_files(value, field):
            if isinstance(value, dict):
                for key, item in sorted(value.items()):
                    original_files(item, field + [key])
            else:
                assert isinstance(value, str)
                emit(path, field, value)

        for field in ["original_files", "original_sources"]:
            if field in record:
                original_files(record[field], [field])
receipt = {"status": "byte_exact_transfer", "payloads": len(rows), "objects": rows,
           "original_records_rewritten": False, "source_bytes_normalized": False,
           "negative_mutated_buffers_relabelled_as_original_preimages": False,
           "decoded_payload_exclusions": False}
(root / "fixture-payloads.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps({"status": receipt["status"], "full_payloads": len(rows),
                  "canonical_source_preimages": sum(row["canonical_source_preimage"] for row in rows)}))
