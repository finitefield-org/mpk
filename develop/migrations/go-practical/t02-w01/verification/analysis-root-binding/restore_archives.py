"""Restore and verify the complete historical compressed streams elsewhere."""
from pathlib import Path
import hashlib
import json
import sys

ROOT=Path(__file__).resolve().parent

def main():
    destination=Path(sys.argv[1]);assert destination.is_absolute() and not destination.exists();destination.mkdir(mode=0o700)
    record=json.loads((ROOT/'archive-storage.json').read_bytes());assert record['status']=='passed'
    rows=[]
    for item in record['archives']:
        original=Path(item['original']['path']);relative=original.relative_to(ROOT);encoded=Path(item['encoded']['path']);stored=encoded.read_bytes()
        assert len(stored)==item['encoded']['bytes'] and hashlib.sha256(stored).hexdigest()==item['encoded']['sha256']
        raw=bytes.fromhex(stored.decode('ascii'));assert len(raw)==item['original']['bytes'] and hashlib.sha256(raw).hexdigest()==item['original']['sha256']
        target=destination/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw);target.chmod(item['original']['mode'])
        assert target.read_bytes()==raw and target.stat().st_mode & 0o777==item['original']['mode'];rows.append({'path':str(target),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
    review=record['initial_review'];encoded=Path(review['encoded']['path']).read_bytes();assert hashlib.sha256(encoded).hexdigest()==review['encoded']['sha256'];raw=bytes.fromhex(encoded.decode('ascii'));assert len(raw)==review['raw_bytes'] and hashlib.sha256(raw).hexdigest()==review['raw_sha256']
    (destination/'original-review.json').write_bytes(raw)
    print(json.dumps({'status':'passed','restored_archives':len(rows),'original_review_bytes':len(raw),'original_review_sha256':review['raw_sha256'],'all_exact':True}),flush=True)


if __name__=='__main__':main()
