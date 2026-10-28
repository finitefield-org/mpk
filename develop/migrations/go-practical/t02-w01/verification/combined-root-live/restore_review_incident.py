"""Restore exact original archive and initial review at their relative paths."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
destination = Path(sys.argv[1])
assert destination.is_absolute() and not destination.exists()
receipt = json.loads((ROOT / 'review-initial-storage.json').read_bytes())
assert receipt['status'] == 'passed' and not receipt['original_source_inputs_modified'] and receipt['original_storage_paths_losslessly_encoded']
destination.mkdir(mode=0o700)
for row in receipt['originals']:
    relative = Path(row['original_path'])
    assert not relative.is_absolute() and all(part not in ('', '.', '..') for part in relative.parts)
    encoded = (REPO / row['encoded_path']).read_bytes()
    assert len(encoded) == row['encoded_bytes'] and hashlib.sha256(encoded).hexdigest() == row['encoded_sha256']
    raw = bytes.fromhex(encoded.decode('ascii'))
    assert len(raw) == row['original_bytes'] and hashlib.sha256(raw).hexdigest() == row['original_sha256']
    target = destination / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open('xb') as stream:
        stream.write(raw)
    target.chmod(row['original_mode'])
    assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == row['original_mode']
result = {'status': 'passed', 'restored_files': len(receipt['originals']), 'all_original_bytes_paths_and_modes_match': True, 'destination': str(destination)}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
