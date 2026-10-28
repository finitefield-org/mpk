"""Restore exact diagnostic files from the completed failed live attempt."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
run = ROOT / 'live-original'
receipt = json.loads((run / 'failure-storage.json').read_bytes())
destination = Path(sys.argv[1])
assert receipt['status'] == 'passed' and destination.is_absolute() and not destination.exists()
destination.mkdir(mode=receipt['original_root_mode'])
for name in receipt['original_directory_modes']:
    relative = Path(name)
    assert not relative.is_absolute() and all(part not in ('', '.', '..') for part in relative.parts)
    (destination / relative).mkdir(parents=True, exist_ok=True)
for row in receipt['original_files']:
    stored = Path(row['path'])
    assert stored.parts[0] == 'failed-native-originals'
    relative = Path(*stored.parts[1:])
    assert all(part not in ('', '.', '..') for part in relative.parts)
    raw = (run / stored).read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
    target = destination / relative
    with target.open('xb') as stream:
        stream.write(raw)
    target.chmod(row['mode'])
    assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == row['mode']
for name, mode in sorted(receipt['original_directory_modes'].items(), key=lambda item: len(Path(item[0]).parts), reverse=True):
    (destination / name).chmod(mode)
    assert (destination / name).stat().st_mode & 0o777 == mode
destination.chmod(receipt['original_root_mode'])
assert destination.stat().st_mode & 0o777 == receipt['original_root_mode']
result = {'status': 'passed', 'restored_files': len(receipt['original_files']), 'all_original_bytes_paths_and_modes_match': True, 'destination': str(destination)}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
