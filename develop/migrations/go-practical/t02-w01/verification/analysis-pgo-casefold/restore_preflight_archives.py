"""Restore every preflight-encoded archive and its exact review report."""
from pathlib import Path
import hashlib
import json
import os
import sys

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
storage=json.loads((ROOT/'archive-preflight-storage.json').read_bytes());assert storage['status']=='passed'
dest=Path(sys.argv[1]);assert dest.is_absolute() and not dest.exists();dest.mkdir(parents=True)
for item in storage['archives']:
    parts=[]
    for row in item['chunks']:
        path=Path(row['path']);raw=path.read_bytes()
        assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and path.stat().st_mode & 0o777==row['mode']
        parts.append(raw)
    raw=bytes.fromhex(b''.join(parts).decode('ascii'));original=item['original']
    assert len(raw)==original['bytes'] and hashlib.sha256(raw).hexdigest()==original['sha256']
    target=dest/Path(original['path']).relative_to(REPO);target.parent.mkdir(parents=True,exist_ok=True)
    with target.open('xb') as out:out.write(raw)
    os.chmod(target,original['mode']);assert target.read_bytes()==raw and target.stat().st_mode & 0o777==original['mode']
report=storage['initial_report'];raw=bytes.fromhex(Path(report['path']).read_text());assert len(raw)==report['bytes'] and hashlib.sha256(raw).hexdigest()==report['sha256']
target=dest/ROOT.relative_to(REPO)/'archive-preflight.json';target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
print(json.dumps({'status':'passed','restored_archives':len(storage['archives']),'review_bytes':len(raw),'destination':str(dest)}),flush=True)
