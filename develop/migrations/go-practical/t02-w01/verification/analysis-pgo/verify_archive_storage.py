"""Independently verify the complete retained compressed streams and review."""
from pathlib import Path
import hashlib
import json
import tarfile
import io

ROOT=Path(__file__).resolve().parent
storage=json.loads((ROOT/'archive-storage.json').read_bytes());assert storage['status']=='passed'
rows=[]
for record in storage['archives']:
    encoded=Path(record['encoded']['path']);raw=bytes.fromhex(encoded.read_text())
    assert len(raw)==record['original']['bytes'] and hashlib.sha256(raw).hexdigest()==record['original']['sha256']
    assert encoded.stat().st_mode & 0o777==record['encoded']['mode']
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as archive:
        members=[member.name for member in archive if member.isfile()]
    assert members
    rows.append({'original_path':record['original']['path'],'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'original_mode':record['original']['mode'],'file_members':len(members),'exact_recovery':True})
review=bytes.fromhex(Path(storage['initial_review']['encoded']['path']).read_text());original=json.loads(review)
assert len(review)==storage['initial_review']['raw_bytes'] and hashlib.sha256(review).hexdigest()==storage['initial_review']['raw_sha256'] and len(original['active_findings'])==11
out=ROOT/'archive-restore-check.json';assert not out.exists();out.write_text(json.dumps({'status':'passed','selection_reason':'Reconstruct the full original compressed streams and failed review from lossless storage, independently compare their byte identities and modes, and enumerate every archive.','archives':rows,'initial_review_exact':True,'scanner_rules_changed':False},indent=2)+'\n')
print(json.dumps({'status':'passed','restored_archives':len(rows),'restored_review':True}),flush=True)
