"""Measure both temporary filesystems with every original registered SDK file."""
from pathlib import Path
import hashlib
import json
import os
import sys
import time

repo = Path('/workspace')
sdk = Path('/sdk')
roots = [Path('/host/sdk-original'), Path('/volume/sdk-original')]
release = json.loads((repo / 'release/bundles/candidates/go.json').read_bytes())['toolchain_bundles'][0]
files = release['inventory']['files']
assert len(files) == 5577 and all(not root.exists() for root in roots)
results = []
for root in roots:
    root.mkdir(mode=0o700)
    started = time.monotonic()
    count = 0
    total = 0
    for entry in files:
        relative = Path(entry['path'])
        assert not relative.is_absolute() and all(part not in ('', '.', '..') for part in relative.parts)
        source = sdk / relative
        raw = source.read_bytes()
        mode = source.stat().st_mode & 0o777
        assert len(raw) == entry['size_bytes'] and hashlib.sha256(raw).hexdigest() == entry['sha256']
        assert bool(mode & 0o111) == entry['executable']
        target = root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as stream:
            stream.write(raw)
        target.chmod(mode)
        copied = target.read_bytes()
        assert copied == raw and target.stat().st_mode & 0o777 == mode
        count += 1
        total += len(raw)
    results.append({'target': str(root), 'original_files': count, 'original_bytes': total, 'original_file_contents_and_modes_match': True, 'elapsed_seconds': time.monotonic() - started})
    print(json.dumps(results[-1]), flush=True)
record = {'status': 'passed', 'selection_reason': 'Compare two Linux-container temporary filesystems using every actual registered SDK file with verified bytes, original paths and modes; both complete copies are retained.', 'results': results, 'native_Linux_release_acceptance': False}
out = Path('/out/worker-result.json')
assert not out.exists()
out.write_text(json.dumps(record, indent=2) + '\n')
