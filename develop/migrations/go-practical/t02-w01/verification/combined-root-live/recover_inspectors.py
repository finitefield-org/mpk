"""Retain the inspection scripts used for both completed live epochs."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
current_path = ROOT / 'inspect_live.py'
current = current_path.read_text()
block = "    checked_before_binding = owner.get('export_cache_retirement_checked_before_binding') is True\n    if checked_before_binding:\n        start = json.loads((run / 'export-coord/roots-ready.json').read_bytes())\n        assert ready['export_cache_directory'] == start['roots'][0]['Directory']\n"
field = "'cache_retirement_checked_before_binding': checked_before_binding, "
assert current.count(block) == current.count(field) == 1
previous = current.replace(block, '').replace(field, '')
out = ROOT / 'inspector-history'
assert not out.exists()
out.mkdir(mode=0o700)
rows = []
for epoch, source in [('live-volume', previous), ('live-retirement', current)]:
    raw = source.encode()
    path = out / (epoch + '.py')
    path.write_bytes(raw)
    path.chmod(current_path.stat().st_mode & 0o777)
    assert path.read_bytes() == raw
    compile(raw, str(path), 'exec')
    rows.append({'epoch': epoch, 'stored_file': path.name, 'original_path': str(current_path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777, 'original_hash_previously_recorded': False})
result = {'status': 'passed', 'selection_reason': 'Retain the current inspector and reconstruct its earlier version by reversing the recorded later patch; the earlier execution did not seal a source hash.', 'inspectors': rows, 'original_inputs_modified': False}
(out / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'inspectors': len(rows), 'earlier_hash_previously_recorded': False}), flush=True)
