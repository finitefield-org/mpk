"""Check the original root-test fixture against the earlier complete receipt."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
prior = ROOT.parent / 'source-root-replay/original/inputs.json'
receipt = json.loads(prior.read_bytes())
paths = []
for row in receipt['inputs']:
    path = Path(row['path'])
    name = str(path)
    if '/analysis-namespace/native/native.json' in name or '/analysis-namespace/native/original-bytes/' in name or '/analysis-build-list/native/native.json' in name or '/analysis-build-list/native/inputs.' in name:
        data = path.read_bytes()
        observed = {'path': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}
        assert observed == row, name
        paths.append(observed)
assert len(paths) == 213, len(paths)
assert sum("/original-bytes/" in row["path"] for row in paths) == 209
result = {'status': 'passed', 'selection_reason': 'Independently recheck the exact root-test packet, every original source buffer, build-list packet, complete original frontend archive and its receipt against the earlier complete replay input receipt.', 'prior_receipt': str(prior), 'prior_receipt_sha256': hashlib.sha256(prior.read_bytes()).hexdigest(), 'all_fixture_inputs_equal': True, 'inputs': paths, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT / 'root-fixture-audit.json'; assert not out.exists(); out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'fixture_inputs': len(paths)}))
