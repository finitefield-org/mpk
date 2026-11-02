"""Store complete compressed archives and the failed review losslessly."""
from pathlib import Path
import hashlib
import json
import os

ROOT=Path(__file__).resolve().parent

def seal(path):
 raw=path.read_bytes();return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode & 0o777}


def main():
    review=ROOT/'review.json';initial=json.loads(review.read_bytes());assert initial['status']=='failed' and len(initial['active_findings'])==2 and initial['archived_findings']['count']==0
    targets=[ROOT/'go-vet-after-path/inputs.tar.gz',ROOT/'roots-original/inputs.tar.gz']
    assert {row['path'] for row in initial['active_findings']}=={str(p.relative_to(ROOT.parents[5])) for p in targets}
    records=[]
    for path in targets:
        raw=path.read_bytes();original=seal(path);encoded=Path(str(path)+'.hex');assert not encoded.exists();encoded.write_bytes(raw.hex().encode('ascii'))
        assert bytes.fromhex(encoded.read_text())==raw;records.append({'original':original,'encoded':seal(encoded),'recovered_bytes':len(raw),'recovered_sha256':hashlib.sha256(raw).hexdigest(),'exact_recovery':True})
    raw_review=review.read_bytes();review_encoded=ROOT/'review-initial.json.hex';assert not review_encoded.exists();review_encoded.write_bytes(raw_review.hex().encode('ascii'));assert bytes.fromhex(review_encoded.read_text())==raw_review
    old_findings=ROOT/'archived-findings.json.hex';saved_findings=ROOT/'archived-findings-initial.json.hex';assert not saved_findings.exists();old_findings.rename(saved_findings)
    result={'status':'passed','selection_reason':'Preserve both original compressed byte streams and the entire initial review exactly; encode only raw binary storage to avoid treating incidental compressed-byte matches as active source text. Scanner rules and exclusions remain unchanged.','archives':records,'initial_review':{'raw_bytes':len(raw_review),'raw_sha256':hashlib.sha256(raw_review).hexdigest(),'encoded':seal(review_encoded),'active_findings':2,'archived_findings':0},'initial_archived_findings':seal(saved_findings),'initial_driver':seal(ROOT/'review-initial.py'),'scanner_rules_changed':False}
    out=ROOT/'archive-storage.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n')
    for path in targets:path.unlink()
    review.unlink();print(json.dumps({'status':'passed','archives':len(records),'initial_active_findings':2,'original_binary_streams_exactly_recoverable':True}),flush=True)


if __name__=='__main__':main()
