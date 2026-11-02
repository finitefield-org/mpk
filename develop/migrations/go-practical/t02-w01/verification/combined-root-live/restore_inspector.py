"""Restore a retained inspection script with its recorded mode."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
epoch, destination_text = sys.argv[1:]
destination = Path(destination_text)
assert epoch.replace('-', '').isalnum() and destination.is_absolute() and not destination.exists()
receipt = json.loads((ROOT / 'inspector-history/receipt.json').read_bytes())
rows = [row for row in receipt['inspectors'] if row['epoch'] == epoch]
assert receipt['status'] == 'passed' and len(rows) == 1
row = rows[0]
raw = (ROOT / 'inspector-history' / row['stored_file']).read_bytes()
assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
destination.mkdir(mode=0o700)
path = destination / Path(row['original_path']).name
path.write_bytes(raw)
path.chmod(row['mode'])
assert path.read_bytes() == raw and path.stat().st_mode & 0o777 == row['mode']
result = {'status': 'passed', 'epoch': epoch, 'original_path': row['original_path'], 'restored_path': str(path), 'bytes': len(raw), 'sha256': row['sha256'], 'mode': row['mode'], 'original_hash_previously_recorded': row['original_hash_previously_recorded']}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
