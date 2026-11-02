"""Restore the exact test-only observer binary from its stored chunks."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent
destination = Path(sys.argv[1])
assert destination.is_absolute() and not destination.exists()
receipt = json.loads((ROOT / 'binary-original/receipt.json').read_bytes())
assert receipt['status'] == 'passed' and receipt['original_binary_unchanged']
destination.mkdir(mode=0o700)
path = destination / Path(receipt['original_path']).name
digest = hashlib.sha256()
total = 0
with path.open('xb') as output:
    for chunk in receipt['chunks']:
        encoded = (ROOT / chunk['path']).read_bytes()
        assert len(encoded) == chunk['encoded_bytes'] and hashlib.sha256(encoded).hexdigest() == chunk['encoded_sha256']
        raw = bytes.fromhex(encoded.decode('ascii'))
        assert len(raw) == chunk['raw_bytes'] and hashlib.sha256(raw).hexdigest() == chunk['raw_sha256']
        output.write(raw)
        digest.update(raw)
        total += len(raw)
path.chmod(receipt['mode'])
assert total == receipt['bytes'] and digest.hexdigest() == receipt['sha256'] and path.stat().st_mode & 0o777 == receipt['mode']
result = {'status': 'passed', 'original_path': receipt['original_path'], 'restored_path': str(path), 'bytes': total, 'sha256': digest.hexdigest(), 'mode': receipt['mode']}
(destination / 'restore-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result), flush=True)
