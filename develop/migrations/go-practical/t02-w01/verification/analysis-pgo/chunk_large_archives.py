"""Keep oversized encoded archive streams in recoverable bounded chunks."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent
storage=json.loads((ROOT/'archive-storage.json').read_bytes());assert storage['status']=='passed'
records=[]
for item in storage['archives']:
    path=Path(item['encoded']['path'])
    if item['encoded']['bytes']<=95*1024*1024:continue
    raw=path.read_bytes();assert len(raw)==item['encoded']['bytes'] and hashlib.sha256(raw).hexdigest()==item['encoded']['sha256']
    chunks=[]
    for i,offset in enumerate(range(0,len(raw),32*1024*1024)):
        data=raw[offset:offset+32*1024*1024];p=path.with_name(path.name+f'.chunk-{i:03d}');assert not p.exists();p.write_bytes(data)
        chunks.append({'path':str(p.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'mode':p.stat().st_mode & 0o777})
    recovered=b''.join(Path(x['path']).read_bytes() for x in chunks);assert recovered==raw
    decoded=bytes.fromhex(recovered.decode('ascii'));assert len(decoded)==item['original']['bytes'] and hashlib.sha256(decoded).hexdigest()==item['original']['sha256']
    records.append({'encoded_original':item['encoded'],'archive_original':item['original'],'chunks':chunks,'encoded_and_archive_exactly_recoverable':True})
assert len(records)==2
out=ROOT/'archive-chunk-storage.json';assert not out.exists();out.write_text(json.dumps({'status':'passed','selection_reason':'Keep both original encoded streams and compressed archives exactly recoverable in bounded chunks, preserving paths and modes in their receipts.','records':records,'scanner_rules_changed':False},indent=2)+'\n')
for item in records:Path(item['encoded_original']['path']).unlink()
print(json.dumps({'status':'passed','chunked_streams':len(records),'original_bytes_exactly_recoverable':True}),flush=True)
