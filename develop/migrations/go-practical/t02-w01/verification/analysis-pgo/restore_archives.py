"""Restore the original reviewed compressed archives to a fresh directory."""
from pathlib import Path
import hashlib
import json
import os
import sys

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
storage=json.loads((ROOT/'archive-storage.json').read_bytes());chunks=json.loads((ROOT/'archive-chunk-storage.json').read_bytes())
chunked={item['encoded_original']['path']:item for item in chunks['records']}
dest=Path(sys.argv[1]);assert dest.is_absolute() and not dest.exists();dest.mkdir(parents=True)
for record in storage['archives']:
    encoded=Path(record['encoded']['path'])
    if encoded.exists():data=encoded.read_bytes()
    else:
        item=chunked[str(encoded)];parts=[]
        for row in item['chunks']:
            path=Path(row['path']);raw=path.read_bytes()
            assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and path.stat().st_mode & 0o777==row['mode']
            parts.append(raw)
        data=b''.join(parts)
    assert len(data)==record['encoded']['bytes'] and hashlib.sha256(data).hexdigest()==record['encoded']['sha256']
    raw=bytes.fromhex(data.decode('ascii'));original=record['original']
    assert len(raw)==original['bytes'] and hashlib.sha256(raw).hexdigest()==original['sha256']
    target=dest/Path(original['path']).relative_to(REPO);target.parent.mkdir(parents=True,exist_ok=True)
    with target.open('xb') as out:out.write(raw)
    os.chmod(target,original['mode'])
    assert target.read_bytes()==raw and target.stat().st_mode & 0o777==original['mode']
review=bytes.fromhex(Path(storage['initial_review']['encoded']['path']).read_text());assert len(review)==storage['initial_review']['raw_bytes'] and hashlib.sha256(review).hexdigest()==storage['initial_review']['raw_sha256']
target=dest/ROOT.relative_to(REPO)/'review-first.json';target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(review)
print(json.dumps({'status':'passed','restored_archives':len(storage['archives']),'original_review_bytes':len(review),'destination':str(dest)}),flush=True)
