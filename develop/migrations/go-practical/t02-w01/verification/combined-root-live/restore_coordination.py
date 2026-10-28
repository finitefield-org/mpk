"""Restore preserved coordination files and their original modes."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
label, destination_text = sys.argv[1:]
assert label.replace('-', '').isalnum()
source = ROOT / label
destination = Path(destination_text)
assert destination.is_absolute() and not destination.exists()
receipt = json.loads((source / 'receipt.json').read_bytes())
assert receipt['status'] == 'passed' and not receipt['originals_modified']
destination.mkdir(mode=0o700)
for row in receipt['directories']:
    relative = Path(row['path'])
    assert not relative.is_absolute() and all(part not in ('', '.', '..') for part in relative.parts)
    (destination / relative).mkdir(parents=True, exist_ok=True)
for row in receipt['files']:
    relative = Path(row['path'])
    assert not relative.is_absolute() and all(part not in ('', '.', '..') for part in relative.parts)
    raw = (source / relative).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
    target = destination / relative
    with target.open('xb') as stream:
        stream.write(raw)
    target.chmod(row['mode'])
    assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == row['mode']
for row in sorted(receipt['directories'], key=lambda item: len(Path(item['path']).parts), reverse=True):
    (destination / row['path']).chmod(row['mode'])
    assert (destination / row['path']).stat().st_mode & 0o777 == row['mode']
result = {'status': 'passed', 'restored_files': len(receipt['files']), 'restored_directories': len(receipt['directories']), 'all_original_bytes_paths_and_modes_match': True, 'destination': str(destination)}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
