"""Losslessly store the compressed-byte finding and its complete precheck."""
from pathlib import Path
import hashlib
import json
import os

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
precheck = json.loads((ROOT / 'archive-precheck.json').read_bytes())
assert precheck['status'] == 'completed' and not precheck['scanner_rules_changed']
flagged = [row for row in precheck['archives'] if row['findings']]
assert len(flagged) == 1 and flagged[0]['path'] == str((ROOT / 'scoped-final/inputs.tar.gz').relative_to(REPO))
files = [
    (ROOT / 'scoped-final/inputs.tar.gz', ROOT / 'scoped-final/inputs.tar.gz.hex'),
    (ROOT / 'archive-precheck.json', ROOT / 'archive-precheck.json.hex'),
]
rows = []
for original, encoded in files:
    assert original.is_file() and not encoded.exists()
    raw = original.read_bytes()
    mode = original.stat().st_mode & 0o777
    if original == files[0][0]:
        assert len(raw) == flagged[0]['bytes'] and hashlib.sha256(raw).hexdigest() == flagged[0]['sha256'] and mode == flagged[0]['mode']
    encoded.write_bytes(raw.hex().encode('ascii'))
    assert bytes.fromhex(encoded.read_text()) == raw
    data = encoded.read_bytes()
    rows.append({'original_path': str(original.relative_to(REPO)), 'original_bytes': len(raw), 'original_sha256': hashlib.sha256(raw).hexdigest(), 'original_mode': mode, 'encoded_path': str(encoded.relative_to(REPO)), 'encoded_bytes': len(data), 'encoded_sha256': hashlib.sha256(data).hexdigest()})
receipt = {'status': 'passed', 'selection_reason': 'Retain the exact compressed input archive and complete precheck without changing scanner rules or original source inputs.', 'originals': rows, 'original_source_inputs_modified': False, 'original_storage_paths_losslessly_encoded': True}
out = ROOT / 'archive-precheck-storage.json'
assert not out.exists()
out.write_text(json.dumps(receipt, indent=2) + '\n')
for row, (original, encoded) in zip(rows, files):
    raw = original.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == row['original_sha256'] and bytes.fromhex(encoded.read_text()) == raw
    os.unlink(original)
print(json.dumps({'status': receipt['status'], 'losslessly_stored_originals': len(rows)}), flush=True)
