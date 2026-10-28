"""Retain all original files from the completed failed live attempt."""
from pathlib import Path
import hashlib
import json
import shutil

ROOT = Path(__file__).resolve().parent
run = ROOT / 'live-original'
result = json.loads((run / 'result.json').read_bytes())
manifest = json.loads((run / 'original-evidence-manifest.json').read_bytes())
assert result['status'] == 'failed' and result['original_inputs_unchanged'] and result['registered_sdk_unchanged']
source = Path(manifest['root'])
destination = run / 'failed-native-originals'
assert source.is_absolute() and source.is_dir() and not destination.exists()
destination.mkdir(mode=source.stat().st_mode & 0o777)
directories = sorted(path for path in source.rglob('*') if path.is_dir())
assert not any(path.is_symlink() for path in source.rglob('*'))
for path in directories:
    target = destination / path.relative_to(source)
    target.mkdir(mode=path.stat().st_mode & 0o777, parents=True, exist_ok=True)
    target.chmod(path.stat().st_mode & 0o777)
rows = []
for row in manifest['files']:
    original = Path(row['path'])
    target = destination / original.relative_to(source)
    raw = original.read_bytes()
    assert len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256'] and original.stat().st_mode & 0o777 == row['mode']
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(original, target)
    target.chmod(row['mode'])
    assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == row['mode']
    rows.append({'path': str(target.relative_to(run)), 'bytes': len(raw), 'sha256': row['sha256'], 'mode': row['mode']})
assert len(rows) == manifest['file_count'] and sum(row['bytes'] for row in rows) == manifest['total_bytes']
receipt = {'status': 'passed', 'selection_reason': 'Preserve every original diagnostic and already-produced module observation from the completed failed attempt, with the original bytes, relative paths and modes.', 'source_result': str((run / 'result.json').relative_to(ROOT)), 'original_files': rows, 'original_root_mode': source.stat().st_mode & 0o777, 'original_directory_modes': {str(path.relative_to(source)): path.stat().st_mode & 0o777 for path in directories}, 'original_inputs_modified': False}
out = run / 'failure-storage.json'
assert not out.exists()
out.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'status': receipt['status'], 'files': len(rows), 'bytes': sum(row['bytes'] for row in rows)}), flush=True)
