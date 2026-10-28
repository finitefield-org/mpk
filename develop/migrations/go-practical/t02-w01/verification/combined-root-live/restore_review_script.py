"""Restore one exact historical review script with its original mode."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
epoch, destination_text = sys.argv[1:]
destination = Path(destination_text)
assert epoch in ('initial', 'final') and destination.is_absolute() and not destination.exists()
receipt = json.loads((ROOT / 'review-script-history/receipt.json').read_bytes())
rows = [row for row in receipt['scripts'] if row['epoch'] == epoch]
assert receipt['status'] == 'passed' and len(rows) == 1
row = rows[0]
raw = (ROOT / 'review-script-history' / row['stored_file']).read_bytes()
original = row['original']
assert len(raw) == original['bytes'] and hashlib.sha256(raw).hexdigest() == original['sha256']
destination.mkdir(mode=0o700)
path = destination / Path(original['path']).name
path.write_bytes(raw)
path.chmod(original['mode'])
assert path.read_bytes() == raw and path.stat().st_mode & 0o777 == original['mode']
result = {'status': 'passed', 'epoch': epoch, 'original_path': original['path'], 'restored_path': str(path), 'bytes': len(raw), 'sha256': original['sha256'], 'mode': original['mode']}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
