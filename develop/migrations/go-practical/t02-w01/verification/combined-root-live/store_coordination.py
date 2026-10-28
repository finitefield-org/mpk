"""Retain exact live coordination paths, bytes and original file modes."""
from pathlib import Path
import hashlib
import json
import shutil
import sys

ROOT = Path(__file__).resolve().parent
kind, epoch = sys.argv[1:]
assert kind in ('combined', 'prior-export') and epoch.replace('-', '').isalnum()
run = (ROOT if kind == 'combined' else ROOT.parent / 'export-root-live') / epoch
result = json.loads((run / 'result.json').read_bytes())
assert result['status'] in ('passed', 'failed')
inputs = json.loads((run / 'inputs.json').read_bytes())
work = Path(inputs['temporary_root'])
names = ['source-coord', 'export-coord', 'combined-coord'] if kind == 'combined' else ['coord']
label = kind + '-' + epoch + '-coord-preservation'
out = ROOT / label
assert work.is_absolute() and work.is_dir() and not out.exists()
out.mkdir(mode=0o700)
rows, directories = [], []
for name in names:
    source = work / name
    assert source.is_dir() and not source.is_symlink()
    all_paths = [source] + sorted(source.rglob('*'))
    assert not any(path.is_symlink() for path in all_paths)
    for path in all_paths:
        relative = Path(name) / path.relative_to(source)
        target = out / relative
        mode = path.stat().st_mode & 0o777
        if path.is_dir():
            target.mkdir(parents=True, exist_ok=True)
            target.chmod(mode)
            directories.append({'path': str(relative), 'mode': mode})
            continue
        assert path.is_file()
        raw = path.read_bytes()
        prior = run / relative
        assert prior.is_file() and prior.read_bytes() == raw
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
        target.chmod(mode)
        assert target.read_bytes() == raw and target.stat().st_mode & 0o777 == mode
        rows.append({'path': str(relative), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': mode})
receipt = {'status': 'passed', 'selection_reason': 'Preserve all actual live coordination output bytes, relative paths and original modes; verify the already-recorded copies have identical bytes.', 'original_run': str(run), 'original_work': str(work), 'directories': directories, 'files': rows, 'originals_modified': False}
(out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'status': receipt['status'], 'files': len(rows), 'directories': len(directories)}), flush=True)
