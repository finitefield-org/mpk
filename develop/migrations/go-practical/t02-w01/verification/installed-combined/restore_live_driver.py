"""Restore the exact live driver with its original filename and mode."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
destination = Path(sys.argv[1])
assert destination.is_absolute() and not destination.exists()
receipt = json.loads((ROOT / 'driver-history/receipt.json').read_bytes())
assert receipt['status'] == 'passed' and not receipt['original_source_modified']
expected = receipt['original']
raw = (ROOT / 'driver-history' / receipt['stored_file']).read_bytes()
assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
destination.mkdir(mode=0o700)
path = destination / Path(expected['path']).name
path.write_bytes(raw)
path.chmod(expected['mode'])
assert path.read_bytes() == raw and path.stat().st_mode & 0o777 == expected['mode']
result = {'status': 'passed', 'original_path': expected['path'], 'restored_path': str(path), 'bytes': len(raw), 'sha256': expected['sha256'], 'mode': expected['mode']}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
