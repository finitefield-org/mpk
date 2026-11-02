"""Retain the exact driver used by the completed original live run."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
source = ROOT / 'run_live.py'
expected = json.loads((ROOT / 'live-original/inputs.json').read_bytes())['driver']
raw = source.read_bytes()
mode = source.stat().st_mode & 0o777
assert str(source.resolve()) == expected['path'] and len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256'] and mode == expected['mode']
out = ROOT / 'driver-history'
assert not out.exists()
out.mkdir(mode=0o700)
target = out / source.name
target.write_bytes(raw)
target.chmod(mode)
assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == mode
result = {'status': 'passed', 'selection_reason': 'Retain the exact tested live driver and its original path, bytes and mode.', 'original': expected, 'stored_file': target.name, 'original_source_modified': False}
(out / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'bytes': len(raw), 'sha256': expected['sha256']}), flush=True)
