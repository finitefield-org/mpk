"""Recover exact initial and final scoped-review script versions."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
source = ROOT / 'review.py'
current = source.read_text()
field = ", 'recovered_incident_objects': 0"
assert current.count(field) == 1
initial = current.replace(field, '')
start = initial.index("    incident = json.loads((ROOT / 'review-initial-storage.json')")
end = initial.index("    encoded = (json.dumps(archived", start)
initial = initial[:start] + initial[end:]
reviews = [
    ('initial', initial, json.loads(bytes.fromhex((ROOT / 'review-initial.json.hex').read_text()))),
    ('final', current, json.loads((ROOT / 'review-final.json').read_bytes())),
]
out = ROOT / 'review-script-history'
assert not out.exists()
out.mkdir(mode=0o700)
rows = []
for epoch, text, review in reviews:
    expected = next(row for row in review['inputs'] if row['path'] == str(source.resolve()))
    raw = text.encode()
    assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256']
    path = out / (epoch + '.py')
    path.write_bytes(raw)
    path.chmod(expected['mode'])
    assert path.read_bytes() == raw and path.stat().st_mode & 0o777 == expected['mode']
    rows.append({'epoch': epoch, 'stored_file': path.name, 'original': expected})
result = {'status': 'passed', 'selection_reason': 'Recover both exact review scripts against the hashes recorded in their original review inputs.', 'scripts': rows, 'original_inputs_modified': False}
(out / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'exact_scripts': len(rows)}), flush=True)
