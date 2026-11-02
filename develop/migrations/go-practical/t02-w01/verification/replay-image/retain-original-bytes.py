"""Serialize complete binary/source buffers after actual producers terminate.

Keep an explicit byte-exact transfer receipt from each original raw path.
Original whitespace is data and is never normalized. Independent review scans
every decoded payload, as well as every stored JSON, without exclusions.
"""
from pathlib import Path
import base64
import hashlib
import json

root = Path(__file__).resolve().parent
for directory in sorted(root.glob("*/*/original-source-objects")):
    receipt = directory.parent / "original-byte-retention.json"
    if receipt.exists():
        for row in json.loads(receipt.read_text())["objects"]:
            record = json.loads(Path(row["retained_path"]).read_text())
            raw = base64.b64decode(record["base64"], validate=True)
            assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["raw_sha256"]
            assert record["source_id"] == row["source_id"]
        assert not list(directory.glob("*.raw"))
        continue
    rows = []
    originals = sorted(directory.glob("*.raw"))
    for path in originals:
        source = json.loads(path.with_suffix(".json").read_text())
        raw = path.read_bytes()
        assert source["id"] == path.stem
        assert source["bytes"] == len(raw) and source["raw_sha256"] == hashlib.sha256(raw).hexdigest()
        target = path.with_suffix(".bytes.json")
        assert not target.exists()
        target.write_text(json.dumps({"schema": "mpk.original_bytes.v1", "source_id": source["id"],
                                     "raw_sha256": source["raw_sha256"], "bytes": len(raw),
                                     "base64": base64.b64encode(raw).decode()}, indent=2) + "\n")
        assert base64.b64decode(json.loads(target.read_text())["base64"], validate=True) == raw
        rows.append({"original_producer_path": str(path.relative_to(Path.cwd())),
                     "retained_path": str(target.relative_to(Path.cwd())), "source_id": source["id"],
                     "raw_sha256": source["raw_sha256"], "bytes": len(raw),
                     "original_text_trailing_whitespace": b"\0" not in raw[:8000] and any(line.endswith((b" ", b"\t")) for line in raw.splitlines()),
                     "original_text_blank_EOF": b"\0" not in raw[:8000] and raw.endswith(b"\n\n"),
                     "retained_sha256": hashlib.sha256(target.read_bytes()).hexdigest()})
    receipt.write_text(json.dumps({"status": "byte_exact_transfer", "objects": rows,
                                  "source_bytes_normalized": False, "original_whitespace_preserved": True,
                                  "decoded_source_scanner_exclusions": False}, indent=2) + "\n")
    for path in originals:
        path.unlink()
    print(json.dumps({"attempt": str(directory.parent.relative_to(root)), "original_buffers_retained": len(rows)}))
