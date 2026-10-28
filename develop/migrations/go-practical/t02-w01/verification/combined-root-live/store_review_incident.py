"""Losslessly retain the compressed-byte finding and its original review."""
from pathlib import Path
import hashlib
import json
import os

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
review = json.loads((ROOT / 'review-final.json').read_bytes())
assert review['status'] == 'failed' and len(review['active_findings']) == 1 and review['archived_findings']['count'] == 0
finding = review['active_findings'][0]
assert finding['path'] == 'develop/migrations/go-practical/t02-w01/verification/combined-root-live/preflight-control/inputs.tar.gz'
archive = ROOT / 'preflight-control/inputs.tar.gz'
sealed = json.loads((ROOT / 'preflight-control/inputs.json').read_bytes())['archive']
assert str(archive.resolve()) == sealed['path']
files = [
    (archive, ROOT / 'preflight-control/inputs.tar.gz.hex'),
    (ROOT / 'review-final.json', ROOT / 'review-initial.json.hex'),
    (ROOT / 'review-final-archived-findings.json.hex', ROOT / 'review-initial-findings.raw.hex'),
]
rows = []
for original, encoded in files:
    assert original.is_file() and not encoded.exists()
    raw = original.read_bytes()
    mode = original.stat().st_mode & 0o777
    if original == archive:
        assert len(raw) == sealed['bytes'] and hashlib.sha256(raw).hexdigest() == sealed['sha256'] and mode == sealed['mode']
    encoded.write_bytes(raw.hex().encode('ascii'))
    assert bytes.fromhex(encoded.read_text()) == raw
    rows.append({'original_path': str(original.relative_to(REPO)), 'original_bytes': len(raw), 'original_sha256': hashlib.sha256(raw).hexdigest(), 'original_mode': mode, 'encoded_path': str(encoded.relative_to(REPO)), 'encoded_bytes': encoded.stat().st_size, 'encoded_sha256': hashlib.sha256(encoded.read_bytes()).hexdigest()})
out = ROOT / 'review-initial-storage.json'
assert not out.exists()
out.write_text(json.dumps({'status': 'passed', 'selection_reason': 'Preserve the exact original compressed input archive and the complete failed review without normalizing machine bytes or changing scanner rules.', 'originals': rows, 'original_source_inputs_modified': False, 'original_storage_paths_losslessly_encoded': True}, indent=2) + '\n')
for row, (original, encoded) in zip(rows, files):
    raw = original.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == row['original_sha256'] and bytes.fromhex(encoded.read_text()) == raw
    os.unlink(original)
print(json.dumps({'status': 'passed', 'losslessly_stored_originals': len(rows), 'review_finding': finding['matcher_id']}), flush=True)
