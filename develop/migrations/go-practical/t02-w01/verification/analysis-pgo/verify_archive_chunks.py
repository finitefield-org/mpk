"""Independently reconstruct the bounded encoded and raw archive streams."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent
storage=json.loads((ROOT/'archive-chunk-storage.json').read_bytes());assert storage['status']=='passed'
results=[]
for item in storage['records']:
    parts=[]
    for row in item['chunks']:
        path=Path(row['path']);raw=path.read_bytes()
        assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and path.stat().st_mode & 0o777==row['mode']
        parts.append(raw)
    encoded=b''.join(parts);archive=bytes.fromhex(encoded.decode('ascii'))
    assert len(encoded)==item['encoded_original']['bytes'] and hashlib.sha256(encoded).hexdigest()==item['encoded_original']['sha256']
    assert len(archive)==item['archive_original']['bytes'] and hashlib.sha256(archive).hexdigest()==item['archive_original']['sha256']
    results.append({'encoded_path':item['encoded_original']['path'],'encoded_bytes':len(encoded),'encoded_sha256':hashlib.sha256(encoded).hexdigest(),'encoded_mode':item['encoded_original']['mode'],'archive_path':item['archive_original']['path'],'archive_bytes':len(archive),'archive_sha256':hashlib.sha256(archive).hexdigest(),'archive_mode':item['archive_original']['mode'],'chunks':len(parts),'exact_recovery':True})
out=ROOT/'archive-chunk-restore-check.json';assert not out.exists();out.write_text(json.dumps({'status':'passed','selection_reason':'Reconstruct both complete encoded and compressed streams from bounded chunks and verify their original identities and modes.','records':results,'scanner_rules_changed':False},indent=2)+'\n');print(json.dumps({'status':'passed','recovered_streams':len(results)}),flush=True)
